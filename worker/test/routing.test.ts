import { beforeEach, describe, expect, it } from "vitest";
import { DOMAIN, verifySig } from "../src/crypto";
import { b64uDecode, b64uDecodeLen, b64uEncode, randomBytes } from "../src/encoding";
import { resetRouteCache } from "../src/index";
import { selectNodes, type SnapshotNode } from "../src/routing";
import { verifyTicket } from "../src/ticket";
import { FAST, exports, newNode, registryStub, seedActive, setTimeOffset, setup, tick, withRegistry } from "./support/helpers";
import { BASE, testClock, type TestNode } from "./support/node";

const EK = b64uEncode(randomBytes(32));

async function route(ek: string | null = EK, cf?: Record<string, unknown>): Promise<Response> {
  resetRouteCache();
  const url = ek === null ? `${BASE}/route` : `${BASE}/route?ek=${ek}`;
  return exports.default.fetch(new Request(url, { cf } as RequestInit));
}

async function workerKey(): Promise<Uint8Array> {
  const res = await exports.default.fetch(`${BASE}/.well-known/ancord-key`);
  return b64uDecodeLen(((await res.json()) as any).pubkey, 32)!;
}

async function fourNodes(): Promise<TestNode[]> {
  const nodes: TestNode[] = [];
  for (let i = 0; i < 4; i++) nodes.push(await seedActive(await newNode()));
  await tick();
  return nodes;
}

beforeEach(async () => {
  await setup(FAST);
});

describe("GET /route (A3)", () => {
  it("vrací 3 různé nody, každý s ticketem vázaným na efemérní klíč klienta", async () => {
    const nodes = await fourNodes();
    const res = await route();
    expect(res.status).toBe(200);
    expect(res.headers.get("cache-control")).toBe("no-store");
    const body = (await res.json()) as any;
    expect(body.nodes).toHaveLength(3);
    expect(new Set(body.nodes.map((n: any) => n.node_id)).size).toBe(3);
    const key = await workerKey();
    const nowS = Math.floor((Date.now() + testClock.offsetMs) / 1000);
    for (const n of body.nodes) {
      expect(nodes.map((x) => x.nodeId)).toContain(n.node_id);
      expect(n.hostname).toBe(nodes.find((x) => x.nodeId === n.node_id)!.hostname);
      const check = await verifyTicket(key, n.ticket, n.node_id, nowS);
      expect(check.ok).toBe(true);
      if (check.ok) {
        expect(check.body.ek).toBe(EK);
        // nowS se počítá až po requestu, na hraně sekundy se může lišit o 1
        expect(Math.abs(check.body.exp - (nowS + body.ttl))).toBeLessThanOrEqual(1);
        expect(check.body.nid).toBe(n.node_id);
      }
      // ticket jednoho nodu neplatí na jiném
      const other = body.nodes.find((x: any) => x.node_id !== n.node_id);
      expect((await verifyTicket(key, n.ticket, other.node_id, nowS)).ok).toBe(false);
      // po expiraci neprojde
      expect((await verifyTicket(key, n.ticket, n.node_id, nowS + body.ttl + 1)).ok).toBe(false);
    }
  });

  it("tickety jsou unikátní (nonce) a podpis nejde přenést na změněný obsah", async () => {
    await fourNodes();
    const a = ((await (await route()).json()) as any).nodes[0].ticket as string;
    const b = ((await (await route()).json()) as any).nodes.map((n: any) => n.ticket);
    expect(b).not.toContain(a);
    const [payload, sig] = a.split(".") as [string, string];
    const tampered = JSON.parse(new TextDecoder().decode(b64uDecode(payload)!));
    tampered.exp += 1000;
    const bytes = new TextEncoder().encode(JSON.stringify(tampered));
    expect(await verifySig(await workerKey(), DOMAIN.ticket, bytes, b64uDecode(sig)!)).toBe(false);
  });

  it("přestane vracet vypnutý node", async () => {
    const nodes = await fourNodes();
    await registryStub().adminBan(nodes[0]!.nodeId, "test");
    await tick();
    const seen = new Set<string>();
    for (let i = 0; i < 15; i++) {
      const res = await route();
      if (res.status === 200) for (const n of ((await res.json()) as any).nodes) seen.add(n.node_id);
      else break;
    }
    expect(seen.has(nodes[0]!.nodeId)).toBe(false);
  });

  it("node se starým heartbeatem vypadne ze snapshotu", async () => {
    const nodes = await fourNodes();
    await withRegistry((r) => {
      r.clock = () => Date.now() + testClock.offsetMs;
    });
    await setTimeOffset(10 * 60 * 1000);
    // tři nody pošlou „heartbeat“ (zápisem), jeden ne
    for (const n of nodes.slice(1)) {
      await withRegistry((r) => (r as any).ctx.storage.sql.exec("UPDATE nodes SET last_hb_at = ? WHERE node_id = ?", Math.floor(r.clock() / 1000), n.nodeId));
    }
    await tick();
    const res = await route();
    expect(res.status).toBe(200);
    const ids = ((await res.json()) as any).nodes.map((n: any) => n.node_id);
    expect(ids).toHaveLength(3);
    expect(ids).not.toContain(nodes[0]!.nodeId);
  });

  it("pod minimem živých nodů vrací 503 insufficient_nodes s Retry-After", async () => {
    await seedActive(await newNode());
    await seedActive(await newNode());
    await tick();
    const res = await route();
    expect(res.status).toBe(503);
    expect(((await res.json()) as any).error.code).toBe("insufficient_nodes");
    expect(res.headers.get("retry-after")).toBeTruthy();
  });

  it("bez snapshotu 503 unavailable, se starým 503 stale_snapshot", async () => {
    expect((await route()).status).toBe(503);
    await fourNodes();
    expect((await route()).status).toBe(200);
    await setTimeOffset(10 * 60 * 1000);
    const stale = await route();
    expect(stale.status).toBe(503);
    expect(((await stale.json()) as any).error.code).toBe("stale_snapshot");
  });

  it("validuje klíč klienta", async () => {
    await fourNodes();
    expect((await route(null)).status).toBe(400);
    expect((await route("short")).status).toBe(400);
    expect((await route(b64uEncode(randomBytes(31)))).status).toBe(400);
  });

  it("nody s nulovou kapacitou (nezměřené) se do snapshotu nedostanou", async () => {
    await fourNodes();
    const unmeasured = await newNode();
    await (await import("./support/helpers")).bringUp(unmeasured, false);
    await withRegistry((r) => (r as any).ctx.storage.sql.exec("UPDATE nodes SET last_hb_at = ? WHERE node_id = ?", Math.floor(r.clock() / 1000), unmeasured.nodeId));
    await tick();
    for (let i = 0; i < 10; i++) {
      const ids = ((await (await route()).json()) as any).nodes.map((n: any) => n.node_id);
      expect(ids).not.toContain(unmeasured.nodeId);
    }
  });
});

describe("selectNodes (power-of-two-choices)", () => {
  const mk = (id: string, over: Partial<SnapshotNode> = {}): SnapshotNode => ({
    id,
    host: `${id}.example`,
    b: 1,
    s: 1,
    cc: null,
    ct: null,
    lat: null,
    lon: null,
    ...over,
  });
  const noGeo = { lat: null, lon: null, country: null, continent: null };

  it("při ≤ 3 kandidátech je to jen failover seřazený podle skóre", () => {
    const r = selectNodes([mk("a", { s: 0.2 }), mk("b", { s: 0.9 }), mk("c", { s: 0.5 })], noGeo, 3, 3000);
    expect(r.map((n) => n.id)).toEqual(["b", "c", "a"]);
    expect(selectNodes([mk("a")], noGeo, 3, 3000)).toHaveLength(1);
  });

  it("vrací vždy 3 různé nody a ignoruje b = 0", () => {
    const nodes = ["a", "b", "c", "d", "e", "f"].map((id) => mk(id));
    nodes.push(mk("zero", { b: 0, s: 0 }));
    for (let i = 0; i < 200; i++) {
      const r = selectNodes(nodes, noGeo, 3, 3000);
      expect(new Set(r.map((n) => n.id)).size).toBe(3);
      expect(r.map((n) => n.id)).not.toContain("zero");
    }
  });

  it("nižší zátěž vyhrává souboj dvou kandidátů; přetížený node je první volbou vzácně", () => {
    const nodes = [mk("busy", { s: 0.1 }), ...["a", "b", "c", "d", "e"].map((id) => mk(id, { s: 1 }))];
    let busyFirst = 0;
    const N = 3000;
    for (let i = 0; i < N; i++) if (selectNodes(nodes, noGeo, 3, 3000)[0]!.id === "busy") busyFirst++;
    // 1/6 šance, že je busy v páru, a v něm vždy prohraje → téměř nikdy první
    expect(busyFirst).toBeLessThan(N * 0.02);
  });

  it("váží podle základního skóre", () => {
    const nodes = [mk("big", { b: 10, s: 10 }), ...["a", "b", "c", "d", "e"].map((id) => mk(id, { b: 1, s: 1 }))];
    let included = 0;
    const N = 2000;
    for (let i = 0; i < N; i++) if (selectNodes(nodes, noGeo, 3, 3000).some((n) => n.id === "big")) included++;
    expect(included / N).toBeGreaterThan(0.85);
  });

  it("preferuje nody v dosahu klienta a při nedostatku je doplní nejbližšími", () => {
    const eu = ["eu1", "eu2", "eu3", "eu4"].map((id, i) => mk(id, { lat: 50 + i * 0.1, lon: 14, ct: "EU" }));
    const us = ["us1", "us2", "us3"].map((id) => mk(id, { lat: 40, lon: -74, ct: "NA" }));
    const client = { lat: 50.1, lon: 14.4, country: "CZ", continent: "EU" };
    for (let i = 0; i < 100; i++) {
      const ids = selectNodes([...eu, ...us], client, 3, 3000).map((n) => n.id);
      expect(ids.every((id) => id.startsWith("eu"))).toBe(true);
    }
    // jen dva v dosahu → doplní se nejbližším dalším
    const few = [...eu.slice(0, 2), ...us];
    const ids = selectNodes(few, client, 3, 3000).map((n) => n.id);
    expect(ids).toHaveLength(3);
    expect(ids).toContain("eu1");
    expect(ids).toContain("eu2");
  });

  it("bez souřadnic použije kontinent", () => {
    const nodes = [
      ...["e1", "e2", "e3", "e4"].map((id) => mk(id, { ct: "EU" })),
      ...["n1", "n2", "n3", "n4"].map((id) => mk(id, { ct: "NA" })),
    ];
    for (let i = 0; i < 50; i++) {
      const ids = selectNodes(nodes, { lat: null, lon: null, country: null, continent: "NA" }, 3, 3000).map((n) => n.id);
      expect(ids.every((id) => id.startsWith("n"))).toBe(true);
    }
  });
});
