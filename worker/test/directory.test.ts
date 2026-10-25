import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { canonicalBytes } from "../src/canonical";
import { DOMAIN, verifySig } from "../src/crypto";
import { b64uDecodeLen } from "../src/encoding";
import { FAST, alignToEpoch, bringUp, exports, newNode, registryStub, setTimeOffset, setup, sqlOf, tick, withRegistry } from "./support/helpers";
import { BASE, TestNode } from "./support/node";

const EPOCH_S = 60; // dev profil
const open: TestNode[] = [];

async function up(connect = true): Promise<TestNode> {
  const n = await newNode();
  open.push(n);
  return bringUp(n, connect);
}

beforeEach(async () => {
  await setup({ ...FAST, challenges: { ...FAST.challenges, enabled: [] } });
});

afterEach(async () => {
  for (const n of open.splice(0)) n.close();
  await setTimeOffset(0);
});

async function workerKey(): Promise<Uint8Array> {
  const res = await exports.default.fetch(`${BASE}/.well-known/ancord-key`);
  return b64uDecodeLen(((await res.json()) as any).pubkey, 32)!;
}

async function directoryValid(d: any): Promise<boolean> {
  return verifySig(await workerKey(), DOMAIN.directory, canonicalBytes(d.directory?.body ?? d.body), b64uDecodeLen((d.directory ?? d).sig, 64)!);
}

describe("WebSocket a heartbeat (A2)", () => {
  it("vyžaduje aktivní node a platný podpis requestu", async () => {
    const n = await newNode();
    await n.register(); // jen pending
    await expect(n.connect()).rejects.toThrow(/403/);

    const plain = await exports.default.fetch(`${BASE}/ws`);
    expect(plain.status).toBe(426);

    const noAuth = await exports.default.fetch(new Request(`${BASE}/ws`, { headers: { upgrade: "websocket" } }));
    expect(noAuth.status).toBe(401);
  });

  it("uvítá node a odpoví ack s epochou", async () => {
    const n = await up(false);
    open.push(n);
    const welcome = await n.connect();
    expect(welcome.t).toBe("welcome");
    expect(welcome.heartbeat_interval_s).toBe(15);
    const ack = await n.heartbeat();
    expect(ack.t).toBe("ack");
    expect(ack.seq).toBe(1);
    expect(ack.epoch).toBe(Math.floor(Date.now() / 1000 / EPOCH_S));
    const ack2 = await n.heartbeat();
    expect(ack2.seq).toBe(2);
  });

  it("odmítne heartbeat se špatným podpisem, starým seq a špatným časem", async () => {
    const n = await up();
    await n.heartbeat();

    // špatný podpis: změna obsahu po podpisu
    const body = { t: "hb", seq: 50, ts: Math.floor(Date.now() / 1000), load: { conns: 1, max_conns: 10, cpu_pct: 1, mem_pct: 1 } };
    n.send({ ...body, sig: "A".repeat(86) });
    const bad = await n.waitFor((m) => m.t === "error");
    expect(bad.reason).toMatch(/signature/);

    // seq se nesmí vracet
    n.seq = 0;
    const replay = await n.heartbeat();
    expect(replay.t).toBe("error");
    expect(replay.reason).toMatch(/sequence/);

    // čas mimo okno
    n.seq = 100;
    const stale = await n.heartbeat(undefined, undefined, { ts: Math.floor(Date.now() / 1000) - 3600 });
    expect(stale.t).toBe("error");
    expect(stale.reason).toMatch(/timestamp/);

    // poškozená zpráva
    n.send({ t: "hb", seq: 200, ts: 0, load: "x", sig: "x" });
    expect((await n.waitFor((m) => m.t === "error")).reason).toMatch(/malformed/);
  });

  it("ignoruje cizí podpis (jiný klíč)", async () => {
    const a = await up();
    const b = await up();
    const body = { t: "hb", seq: 1, ts: Math.floor(Date.now() / 1000), load: { conns: 1, max_conns: 10, cpu_pct: 1, mem_pct: 1 } };
    const sig = await b.sign("ancord/v1/heartbeat", canonicalBytes(body));
    a.send({ ...body, sig: btoa(String.fromCharCode(...sig)).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "") });
    expect((await a.waitFor((m) => m.t === "error")).reason).toMatch(/signature/);
  });

  it("nové spojení nahradí staré", async () => {
    const n = await up();
    const old = n.ws!;
    let closedCode = 0;
    old.addEventListener("close", (e: CloseEvent) => {
      closedCode = e.code;
    });
    await n.connect();
    await new Promise((r) => setTimeout(r, 100));
    expect(closedCode).toBe(4000);
    expect((await n.heartbeat()).t).toBe("ack");
  });

  it("textový ping obslouží runtime bez DO", async () => {
    const n = await up();
    n.ws!.send("ping");
    // odpověď „pong“ není JSON, proto ji ověříme přes message listener
    const got = await new Promise<string>((resolve) => {
      n.ws!.addEventListener("message", (e: MessageEvent) => {
        if (e.data === "pong") resolve("pong");
      });
    });
    expect(got).toBe("pong");
  });

  it("počítá uptime z heartbeatů", async () => {
    const n = await up();
    await alignToEpoch(EPOCH_S);
    await n.heartbeat();
    await setTimeOffset(Date.now() % 1 + 20_000 + (await withRegistry((r) => r.clock())) - Date.now());
    await n.heartbeat();
    const row = await withRegistry((r) => sqlOf(r).exec("SELECT uptime_s FROM nodes WHERE node_id = ?", n.nodeId).one() as any);
    expect(row.uptime_s).toBeGreaterThanOrEqual(19);
  });
});

describe("adresář a epochy (A2)", () => {
  it("nody ve stejné epoše dostanou stejný podepsaný adresář", async () => {
    const epoch = await alignToEpoch(EPOCH_S, 5);
    const nodes = [await up(), await up(), await up()];
    await tick();
    const dirs = await Promise.all(nodes.map((n) => n.waitFor((m) => m.t === "directory")));
    const first = JSON.stringify(dirs[0]!.directory);
    for (const d of dirs) expect(JSON.stringify(d.directory)).toBe(first);

    const body = dirs[0]!.directory.body;
    expect(body.epoch).toBe(epoch);
    expect(body.issued_at).toBe(epoch * EPOCH_S);
    expect(body.params).toEqual({ r: 3, quorum: 2 });
    expect(body.nodes.map((x: any) => x.id)).toEqual(nodes.map((n) => n.nodeId).sort());
    expect(await directoryValid(dirs[0])).toBe(true);

    // veřejný endpoint vrací totéž
    const res = await exports.default.fetch(`${BASE}/directory`);
    expect(res.headers.get("cache-control")).toContain("max-age");
    expect(JSON.stringify(await res.json())).toBe(first);
  });

  it("adresář je v epoše neměnný, odpojení se projeví až v další epoše", async () => {
    const epoch = await alignToEpoch(EPOCH_S, 2);
    const [a, b, c] = [await up(), await up(), await up()];
    await tick();
    const d1 = (await a.waitFor((m) => m.t === "directory")).directory;
    expect(d1.body.nodes).toHaveLength(3);

    // c přestane posílat heartbeaty a čas se posune v rámci téže epochy
    c.close();
    await setTimeOffset(testOffset(epoch, 50));
    await a.heartbeat();
    await b.heartbeat();
    await tick();
    const stillSame = await exports.default.fetch(`${BASE}/directory`).then((r) => r.json());
    expect(stillSame).toEqual(d1);

    // další epocha: c je mimo
    await setTimeOffset(testOffset(epoch + 1, 3));
    await a.heartbeat();
    await b.heartbeat();
    await tick();
    const d2 = (await a.waitFor((m) => m.t === "directory" && m.directory.body.epoch === epoch + 1)).directory;
    expect(d2.body.nodes.map((x: any) => x.id)).toEqual([a.nodeId, b.nodeId].sort());
    expect(d2.body.issued_at).toBe((epoch + 1) * EPOCH_S);
    expect(await directoryValid(d2)).toBe(true);

    // c se vrátí a v následující epoše je zpět
    await c.connect();
    await setTimeOffset(testOffset(epoch + 2, 3));
    for (const n of [a, b, c]) await n.heartbeat();
    await tick();
    const d3 = (await exports.default.fetch(`${BASE}/directory`).then((r) => r.json())) as any;
    expect(d3.body.epoch).toBe(epoch + 2);
    expect(d3.body.nodes).toHaveLength(3);
  });

  it("heartbeat s nižší epochou dostane aktuální adresář", async () => {
    const epoch = await alignToEpoch(EPOCH_S, 5);
    const n = await up();
    await tick();
    await n.waitFor((m) => m.t === "directory");
    await setTimeOffset(testOffset(epoch + 1, 5));
    const ack = await n.heartbeat(undefined, epoch);
    expect(ack.epoch).toBe(epoch + 1);
    const d = await n.waitFor((m) => m.t === "directory");
    expect(d.directory.body.epoch).toBe(epoch + 1);
  });

  it("do adresáře nepatří zabanované ani neaktivní nody; genesis a probation výjimka", async () => {
    const epoch = await alignToEpoch(EPOCH_S, 5);
    const a = await up();
    const b = await up();
    await registryStub().adminBan(b.nodeId, "test");
    await tick();
    const d = (await exports.default.fetch(`${BASE}/directory`).then((r) => r.json())) as any;
    expect(d.body.epoch).toBe(epoch);
    expect(d.body.nodes.map((x: any) => x.id)).toEqual([a.nodeId]);
  });

  it("ban zavře WebSocket zprávou bye", async () => {
    const n = await up();
    await registryStub().adminBan(n.nodeId, "abuse");
    const bye = await n.waitFor((m) => m.t === "bye");
    expect(bye.reason).toBe("banned");
  });

  it("directoryMinChallenges vyřadí nový nenegenesis node", async () => {
    await alignToEpoch(EPOCH_S, 5);
    await withRegistry((r) => {
      r.cfg = { ...r.cfg, directoryMinChallenges: 2 };
    });
    await up();
    await tick();
    const d = (await exports.default.fetch(`${BASE}/directory`).then((r) => r.json())) as any;
    expect(d.body.nodes).toHaveLength(0);
  });
});

function testOffset(epoch: number, intoS: number): number {
  return (epoch * EPOCH_S + intoS) * 1000 - Date.now();
}
