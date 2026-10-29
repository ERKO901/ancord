import { beforeEach, describe, expect, it } from "vitest";
import { env } from "cloudflare:workers";
import { canonicalBytes } from "../src/canonical";
import { DOMAIN } from "../src/crypto";
import { anchorsFor } from "../src/hrw";
import { b64uDecodeLen, b64uEncode, randomBytes } from "../src/encoding";
import { pushDeps, pushIdOf } from "../src/push/handler";
import { processPushBatch } from "../src/push/queue";
import { RealPushSender, type PushSender, type SendResult } from "../src/push/senders";
import { readToken, getStored } from "../src/push/store";
import type { SignedDirectory } from "../src/registry/directory";
import { FAST, alignToEpoch, exports, newNode, seedActive, setTimeOffset, setup, sqlOf, tick, withRegistry } from "./support/helpers";
import { BASE, TestNode, nowS } from "./support/node";

async function pushKey(): Promise<{ key: TestNode; pushId: string }> {
  const key = await TestNode.create();
  return { key, pushId: pushIdOf(key.pub) };
}

async function registerPush(key: TestNode, pushId: string, over: Record<string, unknown> = {}, tamper = false): Promise<Response> {
  const fields = { op: "register", platform: "fcm", push_id: pushId, token: "device-token-1", ts: nowS(), ...over };
  const sig = await key.sign(DOMAIN.pushReg, canonicalBytes(fields as never));
  const body = {
    push_pubkey: key.pubB64,
    platform: fields.platform,
    token: tamper ? "other-token" : fields.token,
    ts: fields.ts,
    sig: b64uEncode(sig),
  };
  return exports.default.fetch(new Request(`${BASE}/push/register`, { method: "POST", body: JSON.stringify(body), headers: { "content-type": "application/json" } }));
}

async function deletePush(key: TestNode, pushId: string, ts = nowS()): Promise<Response> {
  const sig = await key.sign(DOMAIN.pushReg, canonicalBytes({ op: "delete", push_id: pushId, ts }));
  return exports.default.fetch(
    new Request(`${BASE}/push/register`, { method: "DELETE", body: JSON.stringify({ push_pubkey: key.pubB64, ts, sig: b64uEncode(sig) }) }),
  );
}

const EPOCH_S = 60; // dev profil
let enqueued: string[] = [];

beforeEach(async () => {
  await setup(FAST);
  enqueued = [];
  pushDeps.enqueue = async (_env, job) => {
    enqueued.push(job.push_id);
  };
});

describe("registrace push tokenu (A5)", () => {
  it("uloží token šifrovaně pod blinded push_id", async () => {
    const { key, pushId } = await pushKey();
    const res = await registerPush(key, pushId);
    expect(res.status).toBe(200);
    expect(((await res.json()) as any).push_id).toBe(pushId);
    const stored = await getStored(env, pushId);
    expect(stored?.platform).toBe("fcm");
    expect(JSON.stringify(stored)).not.toContain("device-token-1");
    expect(await readToken(env, pushId)).toEqual({ platform: "fcm", token: "device-token-1" });
  });

  it("token nelze přečíst pod jiným push_id (AAD)", async () => {
    const { key, pushId } = await pushKey();
    await registerPush(key, pushId);
    const stored = await getStored(env, pushId);
    await env.PUSH_KV.put("push:" + "a".repeat(52), JSON.stringify(stored));
    expect(await readToken(env, "a".repeat(52))).toBeNull();
  });

  it("odmítne špatný podpis, jinou platformu a staré ts", async () => {
    const { key, pushId } = await pushKey();
    expect((await registerPush(key, pushId, {}, true)).status).toBe(401);
    expect((await registerPush(key, pushId, { platform: "sms" })).status).toBe(400);
    expect((await registerPush(key, pushId, { ts: nowS() - 3600 })).status).toBe(401);
    expect((await registerPush(key, pushId, { token: "" })).status).toBe(400);
    expect(await getStored(env, pushId)).toBeNull();
  });

  it("cizí klíč nemůže přepsat cizí push_id", async () => {
    const owner = await pushKey();
    await registerPush(owner.key, owner.pushId);
    const other = await TestNode.create();
    // podpis cizím klíčem, ale push_id vlastníka: push_id se odvozuje z push_pubkey, takže nesedí
    const sig = await other.sign(DOMAIN.pushReg, canonicalBytes({ op: "register", platform: "fcm", push_id: owner.pushId, token: "evil", ts: nowS() }));
    const res = await exports.default.fetch(
      new Request(`${BASE}/push/register`, {
        method: "POST",
        body: JSON.stringify({ push_pubkey: other.pubB64, platform: "fcm", token: "evil", ts: nowS(), sig: b64uEncode(sig) }),
      }),
    );
    expect(res.status).toBe(401);
    expect((await readToken(env, owner.pushId))?.token).toBe("device-token-1");
  });

  it("starší registrace nepřepíše novější (replay)", async () => {
    const { key, pushId } = await pushKey();
    expect((await registerPush(key, pushId, { token: "new", ts: nowS() })).status).toBe(200);
    expect((await registerPush(key, pushId, { token: "old", ts: nowS() - 30 })).status).toBe(409);
    expect((await readToken(env, pushId))?.token).toBe("new");
  });

  it("smazání vyžaduje podpis a odstraní token", async () => {
    const { key, pushId } = await pushKey();
    await registerPush(key, pushId, { ts: nowS() - 10 });
    const other = await TestNode.create();
    expect((await deletePush(other, pushId)).status).toBe(401); // cizí klíč nesmaže cizí token
    expect(await getStored(env, pushId)).not.toBeNull();
    expect((await deletePush(key, pushId)).status).toBe(200);
    expect(await getStored(env, pushId)).toBeNull();
  });
});

async function setupAnchors(mailbox: Uint8Array) {
  await alignToEpoch(EPOCH_S, 5);
  const nodes: TestNode[] = [];
  for (let i = 0; i < 4; i++) nodes.push(await seedActive(await newNode()));
  await tick();
  const dir = (await exports.default.fetch(`${BASE}/directory`).then((r) => r.json())) as SignedDirectory;
  const anchorIds = anchorsFor(mailbox, dir.body.nodes.map((n) => n.id), dir.body.epoch, dir.body.params.r);
  const anchor = nodes.find((n) => n.nodeId === anchorIds[0])!;
  const nonAnchor = nodes.find((n) => !anchorIds.includes(n.nodeId))!;
  return { nodes, dir, anchor, nonAnchor, anchorIds };
}

describe("POST /wake (A5)", () => {
  const mailbox = randomBytes(32);

  async function wake(node: TestNode, pushId: string, mb: Uint8Array = mailbox): Promise<Response> {
    return node.signed("POST", "/wake", { push_id: pushId, mailbox_id: b64uEncode(mb) });
  }

  it("wake od anchoru projde a zařadí push do fronty", async () => {
    const { key, pushId } = await pushKey();
    await registerPush(key, pushId);
    const { anchor } = await setupAnchors(mailbox);
    const res = await wake(anchor, pushId);
    expect(res.status).toBe(202);
    expect(((await res.json()) as any).status).toBe("queued");
    expect(enqueued).toEqual([pushId]);
  });

  it("wake od nodu, který není anchorem mailboxu, je odmítnut", async () => {
    const { key, pushId } = await pushKey();
    await registerPush(key, pushId);
    const { nonAnchor } = await setupAnchors(mailbox);
    const res = await wake(nonAnchor, pushId);
    expect(res.status).toBe(403);
    expect(((await res.json()) as any).error.code).toBe("forbidden");
    expect(enqueued).toEqual([]);
  });

  it("node mimo adresář (neregistrovaný) se odmítne; bez podpisu 401", async () => {
    const { key, pushId } = await pushKey();
    await registerPush(key, pushId);
    await setupAnchors(mailbox);
    const stranger = await newNode();
    expect((await wake(stranger, pushId)).status).toBe(403);
    const anon = await exports.default.fetch(new Request(`${BASE}/wake`, { method: "POST", body: JSON.stringify({ push_id: pushId, mailbox_id: b64uEncode(mailbox) }) }));
    expect(anon.status).toBe(401);
    expect(enqueued).toEqual([]);
  });

  it("bez registrovaného tokenu: no_token a nic ve frontě", async () => {
    const { anchor } = await setupAnchors(mailbox);
    const res = await wake(anchor, "b".repeat(52));
    expect(res.status).toBe(202);
    expect(((await res.json()) as any).status).toBe("no_token");
    expect(enqueued).toEqual([]);
  });

  it("koalescence: druhý wake pro stejný push_id v okně se nezařadí", async () => {
    const { key, pushId } = await pushKey();
    await registerPush(key, pushId);
    const { anchor, nodes, anchorIds } = await setupAnchors(mailbox);
    expect(((await (await wake(anchor, pushId)).json()) as any).status).toBe("queued");
    expect(((await (await wake(anchor, pushId)).json()) as any).status).toBe("coalesced");
    const second = nodes.find((n) => n.nodeId === anchorIds[1])!;
    expect(((await (await wake(second, pushId)).json()) as any).status).toBe("coalesced");
    expect(enqueued).toEqual([pushId]);
  });

  it("rate limit na node (token bucket) vrátí 429 s Retry-After", async () => {
    (env as any).CONFIG_OVERRIDES = JSON.stringify({ push: { nodeBurst: 2, nodeRefillPerS: 0.01 } });
    try {
      const keys = await Promise.all([pushKey(), pushKey(), pushKey()]);
      for (const k of keys) await registerPush(k.key, k.pushId);
      const { anchor } = await setupAnchors(mailbox);
      expect((await wake(anchor, keys[0]!.pushId)).status).toBe(202);
      expect((await wake(anchor, keys[1]!.pushId)).status).toBe(202);
      const limited = await wake(anchor, keys[2]!.pushId);
      expect(limited.status).toBe(429);
      expect(Number(limited.headers.get("retry-after"))).toBeGreaterThan(0);
    } finally {
      delete (env as any).CONFIG_OVERRIDES;
    }
  });

  it("přehrání téhož podepsaného requestu se odmítne", async () => {
    const { key, pushId } = await pushKey();
    await registerPush(key, pushId);
    const { anchor } = await setupAnchors(mailbox);
    const body = new TextEncoder().encode(JSON.stringify({ push_id: pushId, mailbox_id: b64uEncode(mailbox) }));
    const auth = await anchor.authHeader("POST", "/wake", body);
    const send = () => exports.default.fetch(new Request(`${BASE}/wake`, { method: "POST", body, headers: { authorization: auth } }));
    expect((await send()).status).toBe(202);
    expect((await send()).status).toBe(409);
  });

  it("validuje push_id a mailbox_id", async () => {
    const { anchor } = await setupAnchors(mailbox);
    expect((await anchor.signed("POST", "/wake", { push_id: "short", mailbox_id: b64uEncode(mailbox) })).status).toBe(400);
    expect((await anchor.signed("POST", "/wake", { push_id: "b".repeat(52), mailbox_id: "x" })).status).toBe(400);
    expect((await anchor.signed("POST", "/wake", {})).status).toBe(400);
  });

  it("akceptuje anchory předchozí epochy, ale ne starší", async () => {
    const { key, pushId } = await pushKey();
    await registerPush(key, pushId);
    const { nodes, anchor, dir } = await setupAnchors(mailbox);
    const epoch = dir.body.epoch;
    // další epocha: adresář bez tohoto anchoru (neposlal heartbeat)
    await setTimeOffset((((epoch + 1) * EPOCH_S + 5) * 1000) - Date.now());
    await withRegistry((r) => {
      for (const n of nodes.filter((x) => x !== anchor)) sqlOf(r).exec("UPDATE nodes SET last_hb_at = ? WHERE node_id = ?", Math.floor(r.clock() / 1000), n.nodeId);
    });
    await tick();
    const next = (await exports.default.fetch(`${BASE}/directory`).then((r) => r.json())) as SignedDirectory;
    expect(next.body.epoch).toBe(epoch + 1);
    expect(next.body.nodes.map((n) => n.id)).not.toContain(anchor.nodeId);
    expect((await wake(anchor, pushId)).status).toBe(202);

    // o dvě epochy později už ne
    await setTimeOffset((((epoch + 3) * EPOCH_S + 5) * 1000) - Date.now());
    await withRegistry((r) => {
      for (const n of nodes.filter((x) => x !== anchor)) sqlOf(r).exec("UPDATE nodes SET last_hb_at = ? WHERE node_id = ?", Math.floor(r.clock() / 1000), n.nodeId);
    });
    await tick();
    expect((await wake(anchor, "c".repeat(52))).status).toBe(403);
  });
});

function fakeBatch(ids: string[]) {
  const results: Record<string, string> = {};
  const messages = ids.map((id) => ({
    id,
    attempts: 1,
    body: { push_id: id },
    ack: () => void (results[id] = "ack"),
    retry: () => void (results[id] = "retry"),
  }));
  return { batch: { messages } as unknown as MessageBatch<{ push_id: string }>, results };
}

describe("konzument fronty", () => {
  class Sender implements PushSender {
    sent: string[] = [];
    constructor(private result: SendResult) {}
    async send(_platform: string, token: string): Promise<SendResult> {
      this.sent.push(token);
      return this.result;
    }
  }

  it("odešle push, potvrdí zprávu; neplatný token smaže; chyba = retry", async () => {
    const a = await pushKey();
    const b = await pushKey();
    const c = await pushKey();
    for (const k of [a, b, c]) await registerPush(k.key, k.pushId, { token: `tok-${k.pushId.slice(0, 4)}` });

    const ok = fakeBatch([a.pushId]);
    const s1 = new Sender("sent");
    await processPushBatch(ok.batch, env, s1);
    expect(ok.results[a.pushId]).toBe("ack");
    expect(s1.sent).toEqual([`tok-${a.pushId.slice(0, 4)}`]);
    expect(await getStored(env, a.pushId)).not.toBeNull();

    const bad = fakeBatch([b.pushId]);
    await processPushBatch(bad.batch, env, new Sender("invalid_token"));
    expect(bad.results[b.pushId]).toBe("ack");
    expect(await getStored(env, b.pushId)).toBeNull();

    const retry = fakeBatch([c.pushId]);
    await processPushBatch(retry.batch, env, new Sender("retry"));
    expect(retry.results[c.pushId]).toBe("retry");
    expect(await getStored(env, c.pushId)).not.toBeNull();
  });

  it("zpráva bez tokenu nebo s chybným tvarem se potvrdí bez odeslání", async () => {
    const s = new Sender("sent");
    const gone = fakeBatch(["z".repeat(52)]);
    await processPushBatch(gone.batch, env, s);
    expect(gone.results["z".repeat(52)]).toBe("ack");
    const malformed = { messages: [{ id: "m", attempts: 1, body: {}, ack: () => void 0, retry: () => void 0 }] } as unknown as MessageBatch<{ push_id: string }>;
    await processPushBatch(malformed, env, s);
    expect(s.sent).toEqual([]);
  });
});

describe("odesílatelé FCM a APNs", () => {
  function pem(label: string, der: ArrayBuffer): string {
    const b64 = btoa(String.fromCharCode(...new Uint8Array(der))).replace(/(.{64})/g, "$1\n");
    return `-----BEGIN ${label}-----\n${b64}\n-----END ${label}-----`;
  }

  async function senderEnv() {
    const rsa = (await crypto.subtle.generateKey(
      { name: "RSASSA-PKCS1-v1_5", modulusLength: 2048, publicExponent: new Uint8Array([1, 0, 1]), hash: "SHA-256" },
      true,
      ["sign", "verify"],
    )) as CryptoKeyPair;
    const ec = (await crypto.subtle.generateKey({ name: "ECDSA", namedCurve: "P-256" }, true, ["sign", "verify"])) as CryptoKeyPair;
    return {
      env: {
        ...env,
        FCM_SERVICE_ACCOUNT: JSON.stringify({
          project_id: "proj",
          client_email: "svc@proj.iam.gserviceaccount.com",
          private_key: pem("PRIVATE KEY", (await crypto.subtle.exportKey("pkcs8", rsa.privateKey)) as ArrayBuffer),
        }),
        APNS_KEY_P8: pem("PRIVATE KEY", (await crypto.subtle.exportKey("pkcs8", ec.privateKey)) as ArrayBuffer),
        APNS_KEY_ID: "KEY123",
        APNS_TEAM_ID: "TEAM123",
        APNS_TOPIC: "cz.melonet.app",
      } as typeof env,
      ecPub: ec.publicKey,
    };
  }

  function recorder(handler: (url: string, init: RequestInit) => Response) {
    const calls: { url: string; init: RequestInit }[] = [];
    // jako globální fetch ve workerd: volání s cizím `this` (metoda objektu) je „Illegal invocation“
    const fetcher = async function (this: unknown, input: RequestInfo | URL, init?: RequestInit) {
      if (this !== undefined && this !== globalThis) throw new TypeError("Illegal invocation");
      calls.push({ url: String(input), init: init ?? {} });
      return handler(String(input), init ?? {});
    } as typeof fetch;
    return { fetcher, calls };
  }

  it("FCM: získá OAuth token (cache), pošle tichou data zprávu bez obsahu", async () => {
    const { env: e } = await senderEnv();
    const rec = recorder((url) =>
      url.includes("oauth2") ? Response.json({ access_token: "ya29", expires_in: 3600 }) : Response.json({ name: "m" }),
    );
    const s = new RealPushSender(e, rec.fetcher);
    expect(await s.send("fcm", "dev-token")).toBe("sent");
    expect(await s.send("fcm", "dev-token-2")).toBe("sent");
    expect(rec.calls.filter((c) => c.url.includes("oauth2"))).toHaveLength(1);
    const send = rec.calls.filter((c) => c.url.includes("messages:send"));
    expect(send).toHaveLength(2);
    expect((send[0]!.init.headers as Record<string, string>).authorization).toBe("Bearer ya29");
    const msg = JSON.parse(String(send[0]!.init.body)).message;
    expect(msg.token).toBe("dev-token");
    expect(msg.data).toEqual({ t: "wake" });
    expect(msg.notification).toBeUndefined();
  });

  it("FCM: UNREGISTERED = neplatný token, 5xx = opakovat", async () => {
    const { env: e } = await senderEnv();
    let mode: "gone" | "down" = "gone";
    const rec = recorder((url) => {
      if (url.includes("oauth2")) return Response.json({ access_token: "t", expires_in: 3600 });
      return mode === "gone" ? new Response('{"error":{"status":"NOT_FOUND","details":[{"errorCode":"UNREGISTERED"}]}}', { status: 404 }) : new Response("x", { status: 503 });
    });
    const s = new RealPushSender(e, rec.fetcher);
    expect(await s.send("fcm", "t")).toBe("invalid_token");
    mode = "down";
    expect(await s.send("fcm", "t")).toBe("retry");
  });

  it("APNs: JWT ES256 s kid/iss, background push bez obsahu", async () => {
    const { env: e, ecPub } = await senderEnv();
    const rec = recorder(() => new Response("", { status: 200 }));
    const s = new RealPushSender(e, rec.fetcher);
    const token = "ab".repeat(32);
    expect(await s.send("apns", token)).toBe("sent");
    const call = rec.calls[0]!;
    expect(call.url).toBe(`https://api.push.apple.com/3/device/${token}`);
    const h = call.init.headers as Record<string, string>;
    expect(h["apns-push-type"]).toBe("background");
    expect(h["apns-priority"]).toBe("5");
    expect(h["apns-topic"]).toBe("cz.melonet.app");
    expect(JSON.parse(String(call.init.body))).toEqual({ aps: { "content-available": 1 } });
    const jwt = h.authorization!.replace("bearer ", "");
    const [hd, pl, sg] = jwt.split(".") as [string, string, string];
    expect(JSON.parse(atob(hd.replace(/-/g, "+").replace(/_/g, "/")))).toEqual({ alg: "ES256", kid: "KEY123" });
    expect(JSON.parse(atob(pl.replace(/-/g, "+").replace(/_/g, "/"))).iss).toBe("TEAM123");
    const sig = b64uDecodeLen(sg, 64)!;
    expect(await crypto.subtle.verify({ name: "ECDSA", hash: "SHA-256" }, ecPub, sig, new TextEncoder().encode(`${hd}.${pl}`))).toBe(true);
  });

  it("APNs: 410 a BadDeviceToken = neplatný, špatný formát tokenu = neplatný, 500 = opakovat", async () => {
    const { env: e } = await senderEnv();
    let status = 410;
    let body = "";
    const rec = recorder(() => new Response(body, { status }));
    const s = new RealPushSender(e, rec.fetcher);
    const token = "cd".repeat(32);
    expect(await s.send("apns", token)).toBe("invalid_token");
    status = 400;
    body = '{"reason":"BadDeviceToken"}';
    expect(await s.send("apns", token)).toBe("invalid_token");
    status = 500;
    body = "";
    expect(await s.send("apns", token)).toBe("retry");
    expect(await s.send("apns", "not hex!")).toBe("invalid_token");
  });
});
