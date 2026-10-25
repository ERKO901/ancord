import { beforeEach, describe, expect, it } from "vitest";
import { env } from "cloudflare:workers";
import { MockCloudflareClient, RealCloudflareClient, sharedMockClient } from "../src/cloudflare";
import { b64uEncode, concat, utf8 } from "../src/encoding";
import { DOMAIN, hostnameOf } from "../src/crypto";
import { requiredBits } from "../src/hashcash";
import { loadConfig } from "../src/config";
import { bringUp, configure, newNode, registryStub, setup, sqlOf, tick, withRegistry } from "./support/helpers";
import { GOOD_SPECS, ZONE } from "./support/node";

const mock = sharedMockClient;

beforeEach(async () => {
  mock().tunnels.clear();
  mock().dns.clear();
  mock().foreignNames.clear();
  mock().failOn.clear();
  await setup();
});

async function nodeRow(nodeId: string) {
  return withRegistry((r) => sqlOf(r).exec("SELECT * FROM nodes WHERE node_id = ?", nodeId).toArray()[0] as any);
}

describe("registrace (A1)", () => {
  it("projde: tunel a DNS pro node<id>.zóna, stav pending → active", async () => {
    const n = await newNode();
    const { begin, confirm, res } = await n.register();
    expect(res.status).toBe(200);
    expect(begin.hostname).toBe(hostnameOf(n.nodeId, ZONE));
    expect(begin.hostname).toMatch(/^node[a-z2-7]{16}\.melonet\.cz$/);
    expect(confirm.status).toBe("pending");
    expect(confirm.tunnel_token).toMatch(/^token-for-/);
    expect(confirm.proof_path).toBe("/.well-known/melo-proof");

    expect(mock().tunnels.size).toBe(1);
    const [tunnel] = [...mock().tunnels.values()];
    expect(tunnel!.name).toBe(`ancord-${begin.hostname.split(".")[0]}`);
    expect(tunnel!.hostname).toBe(begin.hostname);
    expect(tunnel!.service).toBe(env.NODE_ORIGIN);
    expect([...mock().dns.values()].map((d) => d.hostname)).toEqual([begin.hostname]);
    expect((await nodeRow(n.nodeId)).status).toBe("pending");

    const act = await n.activate();
    expect(act.status).toBe(200);
    expect(await act.json()).toEqual({ status: "active" });
    expect((await nodeRow(n.nodeId)).status).toBe("active");

    // idempotentní
    const again = await n.activate();
    expect(again.status).toBe(200);
  });

  it("odmítne špatný podpis a nic nevytvoří", async () => {
    const n = await newNode();
    const other = await newNode();
    const beginRes = await n.http("POST", "/register", { pubkey: n.pubB64, claimed_specs: GOOD_SPECS });
    const begin: any = await beginRes.json();
    const nonce = Uint8Array.from(atob(begin.challenge_nonce.replace(/-/g, "+").replace(/_/g, "/") + "="), (c) => c.charCodeAt(0));
    const bad = await other.sign(DOMAIN.register, concat(nonce, utf8(begin.hostname), n.pub));
    const res = await n.http("POST", "/register/confirm", { pubkey: n.pubB64, challenge_nonce: begin.challenge_nonce, signature: b64uEncode(bad) });
    expect(res.status).toBe(422);
    expect(((await res.json()) as any).error.code).toBe("proof_failed");
    expect(mock().tunnels.size).toBe(0);
    expect(mock().dns.size).toBe(0);
  });

  it("výzva je jednorázová (replay potvrzení)", async () => {
    const n = await newNode();
    const beginRes = await n.http("POST", "/register", { pubkey: n.pubB64, claimed_specs: GOOD_SPECS });
    const begin: any = await beginRes.json();
    const raw = Uint8Array.from(atob(begin.challenge_nonce.replace(/-/g, "+").replace(/_/g, "/") + "="), (c) => c.charCodeAt(0));
    const sig = b64uEncode(await n.sign(DOMAIN.register, concat(raw, utf8(begin.hostname), n.pub)));
    const body = { pubkey: n.pubB64, challenge_nonce: begin.challenge_nonce, signature: sig };
    expect((await n.http("POST", "/register/confirm", body)).status).toBe(200);
    expect((await n.http("POST", "/register/confirm", body)).status).toBe(422);
    expect(mock().tunnels.size).toBe(1);
  });

  it("propadlá výzva se odmítne", async () => {
    const n = await newNode();
    const beginRes = await n.http("POST", "/register", { pubkey: n.pubB64, claimed_specs: GOOD_SPECS });
    const begin: any = await beginRes.json();
    const raw = Uint8Array.from(atob(begin.challenge_nonce.replace(/-/g, "+").replace(/_/g, "/") + "="), (c) => c.charCodeAt(0));
    const sig = b64uEncode(await n.sign(DOMAIN.register, concat(raw, utf8(begin.hostname), n.pub)));
    await withRegistry((r) => {
      r.clock = () => Date.now() + 10 * 60 * 1000;
    });
    const res = await n.http("POST", "/register/confirm", { pubkey: n.pubB64, challenge_nonce: begin.challenge_nonce, signature: sig });
    expect(res.status).toBe(422);
    await withRegistry((r) => {
      r.clock = () => Date.now();
    });
  });

  it("kolize jména: existující DNS záznam v zóně → 409 a kompenzace (tunel se smaže)", async () => {
    const n = await newNode();
    mock().foreignNames.add(hostnameOf(n.nodeId, ZONE));
    const { res } = await n.register();
    expect(res.status).toBe(409);
    expect(mock().tunnels.size).toBe(0);
    expect(mock().dns.size).toBe(0);
    expect(await nodeRow(n.nodeId)).toBeUndefined();
    // cizí záznam zůstal nedotčen
    expect(mock().foreignNames.has(hostnameOf(n.nodeId, ZONE))).toBe(true);
  });

  it("kolize hostname s jiným klíčem se odmítne při výzvě", async () => {
    const a = await newNode();
    const b = await newNode();
    await withRegistry((r) => {
      sqlOf(r).exec(
        "INSERT INTO nodes (node_id, pubkey, hostname, status, claimed, created_at, updated_at) VALUES ('x', 'otherkey', ?, 'active', '{}', 0, 0)",
        hostnameOf(b.nodeId, ZONE),
      );
    });
    const res = await b.http("POST", "/register", { pubkey: b.pubB64, claimed_specs: GOOD_SPECS });
    expect(res.status).toBe(409);
    void a;
  });

  it("selhání uprostřed provisioningu uklidí vytvořené zdroje", async () => {
    const n = await newNode();
    mock().failOn.add("createDnsCname");
    const { res } = await n.register();
    expect(res.status).toBe(502);
    expect(mock().tunnels.size).toBe(0);
    expect(await nodeRow(n.nodeId)).toBeUndefined();
    mock().failOn.clear();
    const retry = await n.register();
    expect(retry.res.status).toBe(200);
  });

  it("opakovaná registrace vydá znovu token bez nových zdrojů", async () => {
    const n = await newNode();
    const first = await n.register();
    const second = await n.register();
    expect(second.res.status).toBe(200);
    expect(second.confirm.tunnel_token).toBe(first.confirm.tunnel_token);
    expect(mock().tunnels.size).toBe(1);
    expect(mock().dns.size).toBe(1);
  });

  it("validuje vstup", async () => {
    const n = await newNode();
    const r1 = await n.http("POST", "/register", { pubkey: "short", claimed_specs: GOOD_SPECS });
    expect(r1.status).toBe(400);
    const r2 = await n.http("POST", "/register", { pubkey: n.pubB64, claimed_specs: { ...GOOD_SPECS, ram_mb: -1 } });
    expect(r2.status).toBe(400);
    const r3 = await n.http("POST", "/register", { pubkey: n.pubB64, claimed_specs: { ...GOOD_SPECS, cpu_cores: 1.5 } });
    expect(r3.status).toBe(400);
    const res = await n.http("POST", "/register", undefined);
    expect(res.status).toBe(400);
    const big = await n.http("POST", "/register", { pubkey: n.pubB64, junk: "x".repeat(40_000) });
    expect(big.status).toBe(413);
  });

  it("genesis allowlist: neuvedený node se odmítne, uvedený projde", async () => {
    await configure({ genesisOnly: true });
    const a = await newNode();
    const b = await newNode();
    expect((await a.register()).res.status).toBe(403);
    await withRegistry((r) => {
      (r as any).env.GENESIS_NODE_IDS = `${b.nodeId}`;
    });
    expect((await b.register()).res.status).toBe(200);
    await withRegistry((r) => {
      (r as any).env.GENESIS_NODE_IDS = "";
    });
  });

  it("hashcash: bez něj nebo se špatným odmítne, správný projde", async () => {
    await configure({ hashcash: { enabled: true, baseBits: 8, maxBits: 12 } });
    const n = await newNode();
    const beginRes = await n.http("POST", "/register", { pubkey: n.pubB64, claimed_specs: GOOD_SPECS });
    const begin: any = await beginRes.json();
    expect(begin.hashcash_bits).toBe(8);
    const raw = Uint8Array.from(atob(begin.challenge_nonce.replace(/-/g, "+").replace(/_/g, "/") + "="), (c) => c.charCodeAt(0));
    const sig = b64uEncode(await n.sign(DOMAIN.register, concat(raw, utf8(begin.hostname), n.pub)));
    const base = { pubkey: n.pubB64, challenge_nonce: begin.challenge_nonce, signature: sig };
    expect((await n.http("POST", "/register/confirm", base)).status).toBe(422);
    // špatný counter nespotřebuje výzvu
    const { hashcashValid } = await import("../src/hashcash");
    let bad = 0;
    while (hashcashValid(raw, n.pub, bad, 8)) bad++;
    expect((await n.http("POST", "/register/confirm", { ...base, hashcash_counter: bad })).status).toBe(422);
    const { res } = { res: await n.register() };
    expect(res.res.status).toBe(200);
  });

  it("hashcash obtížnost roste s vlnou registrací", () => {
    const cfg = { enabled: true, baseBits: 16, maxBits: 20, surgeWindowS: 3600, surgeUnit: 10 };
    expect(requiredBits(cfg, 0)).toBe(16);
    expect(requiredBits(cfg, 10)).toBe(17);
    expect(requiredBits(cfg, 70)).toBe(19);
    expect(requiredBits(cfg, 100_000)).toBe(20);
    expect(requiredBits({ ...cfg, enabled: false }, 1000)).toBe(0);
  });

  it("limit nodů na ASN", async () => {
    await configure({ limits: { enabled: true, maxPerAsn: 1, maxPerPrefix: 10 } });
    const a = await newNode({ asn: 64500 });
    const b = await newNode({ asn: 64500 });
    const c = await newNode({ asn: 64501 });
    expect((await a.register()).res.status).toBe(200);
    expect((await b.register()).res.status).toBe(429);
    expect((await c.register()).res.status).toBe(200);
    expect(mock().tunnels.size).toBe(2);
  });

  it("aktivace selže, když node nedrží klíč na hostname", async () => {
    const n = await newNode();
    const impostor = await newNode();
    await n.register();
    await withRegistry((r) => {
      r.fetchImpl = async (input) => impostor.proofResponse(typeof input === "string" ? input : (input as Request).url);
    });
    const act = await n.activate();
    expect(act.status).toBe(422);
    expect((await nodeRow(n.nodeId)).status).toBe("pending");
  });

  it("aktivace: replay stejného podepsaného requestu se odmítne", async () => {
    const n = await newNode();
    await n.register();
    const auth = await n.authHeader("POST", "/register/activate", utf8("{}"));
    const send = () =>
      n.http("POST", "/register/activate", {}, { authorization: auth });
    const first = await send();
    expect(first.status).toBe(200);
    const second = await send();
    expect(second.status).toBe(409);
  });

  it("podepsaný request: zastaralý čas a cizí tělo se odmítnou", async () => {
    const n = await newNode();
    await n.register();
    const old = await n.authHeader("POST", "/register/activate", utf8("{}"), Math.floor(Date.now() / 1000) - 600);
    expect((await n.http("POST", "/register/activate", {}, { authorization: old })).status).toBe(401);
    const forged = await n.authHeader("POST", "/register/activate", utf8('{"x":1}'));
    expect((await n.http("POST", "/register/activate", {}, { authorization: forged })).status).toBe(401);
    expect((await n.http("POST", "/register/activate", {})).status).toBe(401);
  });

  it("neaktivovaný node propadne a Worker smaže jen své zdroje", async () => {
    const n = await newNode();
    mock().foreignNames.add("www.melonet.cz");
    await n.register();
    expect(mock().tunnels.size).toBe(1);
    await withRegistry((r) => {
      r.clock = () => Date.now() + 2 * 3600 * 1000;
    });
    await tick();
    expect((await nodeRow(n.nodeId)).status).toBe("retired");
    expect(mock().tunnels.size).toBe(0);
    expect(mock().dns.size).toBe(0);
    expect(mock().foreignNames.has("www.melonet.cz")).toBe(true);
    await withRegistry((r) => {
      r.clock = () => Date.now();
    });
  });

  it("deregistrace smaže tunel a DNS a dovolí novou registraci", async () => {
    const n = await newNode();
    await bringUp(n, false);
    const res = await n.signed("POST", "/deregister", {});
    expect(res.status).toBe(200);
    expect(mock().tunnels.size).toBe(0);
    expect(mock().dns.size).toBe(0);
    expect((await nodeRow(n.nodeId)).status).toBe("retired");
    expect((await n.register()).res.status).toBe(200);
    expect(mock().tunnels.size).toBe(1);
  });

  it("zabanovaný node se nemůže znovu registrovat", async () => {
    const n = await newNode();
    await bringUp(n, false);
    const res = await registryStub().adminBan(n.nodeId, "test");
    expect(res).toBe(true);
    expect(mock().tunnels.size).toBe(0);
    expect((await n.http("POST", "/register", { pubkey: n.pubB64, claimed_specs: GOOD_SPECS })).status).toBe(403);
  });
});

describe("CloudflareClient (bezpečnost sdílené zóny)", () => {
  function fakeApi(handlers: (method: string, url: string, body: any) => { status?: number; result: unknown }) {
    const calls: { method: string; url: string; body: any }[] = [];
    const fetcher = (async (input: RequestInfo | URL, init?: RequestInit) => {
      const url = String(input);
      const body = init?.body ? JSON.parse(String(init.body)) : undefined;
      calls.push({ method: init?.method ?? "GET", url, body });
      const h = handlers(init?.method ?? "GET", url, body);
      return new Response(JSON.stringify({ success: true, result: h.result }), { status: h.status ?? 200 });
    }) as typeof fetch;
    return { fetcher, calls };
  }
  const host = "nodeabcdefghijklmnop.melonet.cz";

  it("odmítne zapisovat a mazat cizí jména", async () => {
    const { fetcher, calls } = fakeApi(() => ({ result: [] }));
    const c = new RealCloudflareClient("tok", "acc", "zone", "melonet.cz", fetcher);
    await expect(c.createDnsCname("www.melonet.cz", "t1")).rejects.toThrow(/foreign/);
    await expect(c.createDnsCname("nodeabcdefghijklmnop.example.com", "t1")).rejects.toThrow(/foreign/);
    await expect(c.createDnsCname("nodeshort.melonet.cz", "t1")).rejects.toThrow(/foreign/);
    await expect(c.deleteDns("r1", "www.melonet.cz")).rejects.toThrow(/foreign/);
    await expect(c.createTunnel("prod-db")).rejects.toThrow(/foreign/);
    expect(calls).toHaveLength(0);
  });

  it("DNS: před zápisem čte existující záznamy a při kolizi nezapíše", async () => {
    const { fetcher, calls } = fakeApi((m) => (m === "GET" ? { result: [{ id: "x" }] } : { result: { id: "new" } }));
    const c = new RealCloudflareClient("tok", "acc", "zone", "melonet.cz", fetcher);
    await expect(c.createDnsCname(host, "tun")).rejects.toMatchObject({ code: "conflict" });
    expect(calls.map((x) => x.method)).toEqual(["GET"]);
  });

  it("DNS: vytvoří CNAME s komentářem ancord-managed, proxied", async () => {
    const { fetcher, calls } = fakeApi((m) => (m === "GET" ? { result: [] } : { result: { id: "rec1" } }));
    const c = new RealCloudflareClient("tok", "acc", "zone", "melonet.cz", fetcher);
    expect(await c.createDnsCname(host, "tun")).toBe("rec1");
    expect(calls[1]!.body).toMatchObject({ type: "CNAME", name: host, content: "tun.cfargotunnel.com", proxied: true, comment: "ancord-managed" });
  });

  it("DNS: nesmaže záznam bez naší značky", async () => {
    const { fetcher, calls } = fakeApi(() => ({ result: { id: "r1", name: host, comment: "someone else" } }));
    const c = new RealCloudflareClient("tok", "acc", "zone", "melonet.cz", fetcher);
    await expect(c.deleteDns("r1", host)).rejects.toThrow(/not managed/);
    expect(calls.some((x) => x.method === "DELETE")).toBe(false);
  });

  it("tunel: nesmaže tunel s cizím jménem", async () => {
    const { fetcher, calls } = fakeApi(() => ({ result: { id: "t1", name: "production-tunnel" } }));
    const c = new RealCloudflareClient("tok", "acc", "zone", "melonet.cz", fetcher);
    await expect(c.deleteTunnel("t1", "ancord-nodeabcdefghijklmnop")).rejects.toThrow(/not managed/);
    expect(calls.some((x) => x.method === "DELETE")).toBe(false);
  });

  it("smaže vlastní DNS i tunel", async () => {
    const { fetcher, calls } = fakeApi((m, url) => {
      if (m === "GET" && url.includes("dns_records")) return { result: { id: "r1", name: host, comment: "ancord-managed" } };
      if (m === "GET") return { result: { id: "t1", name: "ancord-nodeabcdefghijklmnop" } };
      return { result: {} };
    });
    const c = new RealCloudflareClient("tok", "acc", "zone", "melonet.cz", fetcher);
    await c.deleteDns("r1", host);
    await c.deleteTunnel("t1", "ancord-nodeabcdefghijklmnop");
    expect(calls.filter((x) => x.method === "DELETE").map((x) => x.url)).toEqual([
      "https://api.cloudflare.com/client/v4/zones/zone/dns_records/r1",
      "https://api.cloudflare.com/client/v4/accounts/acc/cfd_tunnel/t1/connections",
      "https://api.cloudflare.com/client/v4/accounts/acc/cfd_tunnel/t1",
    ]);
  });

  it("MockCloudflareClient odmítne smazat cizí zdroj", async () => {
    const m = new MockCloudflareClient();
    const t = await m.createTunnel("ancord-nodeaaaaaaaaaaaaaaaa");
    await expect(m.deleteTunnel(t.id, "other")).rejects.toThrow();
  });
});

describe("konfigurace profilů", () => {
  it("načte dev, prod a open a odmítne neznámý profil i překlep v přepisu", () => {
    expect(loadConfig({ PROFILE: "dev" }).genesisOnly).toBe(true);
    expect(loadConfig({ PROFILE: "prod" }).epochSeconds).toBe(300);
    const open = loadConfig({ PROFILE: "open" });
    expect(open.probation.enabled).toBe(true);
    expect(open.hashcash.enabled).toBe(true);
    expect(open.limits.enabled).toBe(true);
    expect(() => loadConfig({ PROFILE: "nope" })).toThrow();
    expect(() => loadConfig({ PROFILE: "dev", CONFIG_OVERRIDES: '{"epocSeconds": 5}' })).toThrow(/unknown key/);
    expect(() => loadConfig({ PROFILE: "dev", CONFIG_OVERRIDES: '{"r": 1, "quorum": 2}' })).toThrow(/quorum/);
    expect(loadConfig({ PROFILE: "dev", CONFIG_OVERRIDES: '{"epochSeconds": 7}' }).epochSeconds).toBe(7);
  });
});
