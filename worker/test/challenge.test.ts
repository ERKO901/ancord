import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { scoreOf } from "../src/registry/scoring";
import { computeCheckpoints, verifyCpu } from "../src/challenge/cpu";
import { LEAF_BYTES, buildTable, buildTree, leavesFor, makeOpening, merklePath, verifyOpenings, verifyPath, leafHash } from "../src/challenge/space";
import { b64uEncode, randomBytes } from "../src/encoding";
import { FAST, bringUp, exports, newNode, registryStub, setTimeOffset, setup, sqlOf, tick, withRegistry } from "./support/helpers";
import { BASE, GOOD_SPECS, testClock, type Behavior, type TestNode } from "./support/node";

const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));
const open: TestNode[] = [];

async function rowOf(nodeId: string): Promise<any> {
  return withRegistry((r) => sqlOf(r).exec("SELECT * FROM nodes WHERE node_id = ?", nodeId).toArray()[0]);
}

async function scoreRow(nodeId: string) {
  return withRegistry((r) => {
    const row = sqlOf(r).exec("SELECT * FROM nodes WHERE node_id = ?", nodeId).toArray()[0] as any;
    return scoreOf(r.cfg, row, Math.floor(r.clock() / 1000));
  });
}

async function until(cond: () => Promise<boolean>, ms = 25_000): Promise<void> {
  const end = Date.now() + ms;
  while (Date.now() < end) {
    if (await cond()) return;
    await tick();
    await sleep(80);
  }
  throw new Error("condition not met in time");
}

async function node(behavior: Behavior = {}, asn?: number, cores?: number): Promise<TestNode> {
  const n = await newNode({ ...(asn === undefined ? {} : { asn }), ...(cores === undefined ? {} : { specs: { ...GOOD_SPECS, cpu_cores: cores } }) });
  n.behavior = behavior;
  open.push(n);
  return bringUp(n);
}

const allMeasured = (id: string) => async () => {
  const r = await rowOf(id);
  return r.m_cpu_hps !== null && r.m_ram_mb !== null && r.m_storage_gb !== null && r.m_bw_mbps !== null;
};

beforeEach(async () => {
  await setup(FAST);
});

afterEach(async () => {
  for (const n of open.splice(0)) n.close();
  await setTimeOffset(0);
});

describe("CPU hash-chain", () => {
  const seed = randomBytes(32);
  const lanes = 2;
  const iters = 1000;
  const every = 100;
  const honest = computeCheckpoints(seed, lanes, iters, every);

  it("poctivá odpověď projde", async () => {
    expect((await verifyCpu(b64uEncode(seed), lanes, iters, every, 4, honest)).ok).toBe(true);
  });

  it("vymyšlené checkpointy neprojdou", async () => {
    const fake = honest.map((l) => l.map(() => b64uEncode(randomBytes(32))));
    expect((await verifyCpu(b64uEncode(seed), lanes, iters, every, 4, fake)).ok).toBe(false);
  });

  it("částečný podvod se odhalí s pravděpodobností rostoucí s počtem segmentů", async () => {
    // podvádí polovina segmentů; kontrola všech segmentů ho odhalí vždy
    const half = honest.map((l) => l.map((c, j) => (j % 2 ? b64uEncode(randomBytes(32)) : c)));
    expect((await verifyCpu(b64uEncode(seed), lanes, iters, every, lanes * (iters / every), half)).ok).toBe(false);
    let caught = 0;
    for (let i = 0; i < 200; i++) if (!(await verifyCpu(b64uEncode(seed), lanes, iters, every, 4, half)).ok) caught++;
    expect(caught).toBeGreaterThan(170); // 1 − 0,5⁴ ≈ 94 %
  });

  it("odmítne špatný tvar", async () => {
    const s = b64uEncode(seed);
    expect((await verifyCpu(s, lanes, iters, every, 4, "x")).ok).toBe(false);
    expect((await verifyCpu(s, lanes, iters, every, 4, honest.slice(0, 1))).ok).toBe(false);
    expect((await verifyCpu(s, lanes, iters, every, 4, honest.map((l) => l.slice(1)))).ok).toBe(false);
    expect((await verifyCpu(s, lanes, iters, every, 4, honest.map((l) => l.map(() => 5)))).ok).toBe(false);
    expect((await verifyCpu("short", lanes, iters, every, 4, honest)).ok).toBe(false);
  });
});

describe("Proof of Space", () => {
  const seed = randomBytes(32);
  const leaves = 64;
  const table = buildTable(seed, leaves);
  const levels = buildTree(table);
  const root = levels[levels.length - 1]![0]!;
  const indices = [1, 2, 17, 40, 63];
  const openings = indices.map((i) => makeOpening(seed, table, levels, i));

  it("poctivá otevření projdou", () => {
    expect(verifyOpenings(seed, root, leaves, indices, openings).ok).toBe(true);
  });

  it("Merkle cesta ověřuje list a odmítne cizí", () => {
    const t = (i: number) => table.subarray(i * LEAF_BYTES, (i + 1) * LEAF_BYTES);
    expect(verifyPath(root, 5, t(5), merklePath(levels, 5), leaves)).toBe(true);
    expect(verifyPath(root, 5, t(6), merklePath(levels, 5), leaves)).toBe(false);
    expect(verifyPath(root, 4, t(5), merklePath(levels, 5), leaves)).toBe(false);
    expect(verifyPath(root, 5, t(5), merklePath(levels, 5).slice(1), leaves)).toBe(false);
    expect(leafHash(t(0))).not.toEqual(t(0));
  });

  it("podvržený list, pozměněná cesta či špatné pořadí neprojdou", () => {
    const forged = openings.map((o) => ({ ...o }));
    forged[2]!.leaf = b64uEncode(randomBytes(32));
    expect(verifyOpenings(seed, root, leaves, indices, forged).ok).toBe(false);

    const badPath = openings.map((o) => ({ ...o, path: [...o.path] }));
    badPath[1]!.path[0] = b64uEncode(randomBytes(32));
    expect(verifyOpenings(seed, root, leaves, indices, badPath).ok).toBe(false);

    expect(verifyOpenings(seed, root, leaves, indices, [...openings].reverse()).ok).toBe(false);
    expect(verifyOpenings(seed, root, leaves, indices, openings.slice(1)).ok).toBe(false);
    expect(verifyOpenings(seed, randomBytes(32), leaves, indices, openings).ok).toBe(false);
  });

  it("tabulka z jiného seedu (jiný commitment) neprojde", () => {
    const other = buildTable(randomBytes(32), leaves);
    const oLevels = buildTree(other);
    const oOpen = indices.map((i) => makeOpening(seed, other, oLevels, i));
    // kořen odpovídá, ale hodnoty nejsou odvozeny ze seedu
    expect(verifyOpenings(seed, oLevels[oLevels.length - 1]![0]!, leaves, indices, oOpen).ok).toBe(false);
  });

  it("velikost tabulky: mocnina dvou v mezích", () => {
    expect(leavesFor(10 * 1024 * 1024, 1 << 10, 1 << 11)).toBe(1 << 11);
    expect(leavesFor(1, 1 << 10, 1 << 11)).toBe(1 << 10);
    expect(leavesFor(40 * 1024, 1 << 10, 1 << 16)).toBe(1 << 10);
    expect(leavesFor(100 * 32 * 1024, 1 << 10, 1 << 20)).toBe(1 << 16);
  });
});

describe("challenge nad WebSocketem (A4)", () => {
  it("poctivý node projde všemi výzvami a dostane kladné skóre", async () => {
    const n = await node();
    await until(allMeasured(n.nodeId));
    const row = await rowOf(n.nodeId);
    expect(row.m_cpu_hps).toBeGreaterThan(0);
    expect(row.m_ram_mb).toBeCloseTo((1 << 11) * 32 / 1024 ** 2, 6);
    expect(row.m_storage_gb).toBeCloseTo((1 << 11) * 32 / 1024 ** 3, 9);
    expect(row.m_bw_mbps).toBeGreaterThan(0);
    expect(row.strikes).toBe(0);
    expect(row.ch_ok).toBeGreaterThanOrEqual(4);
    expect(row.health).toBeGreaterThan(0.99);
    const s = await scoreRow(n.nodeId);
    expect(s.base).toBeGreaterThan(0);
    expect(new Set(n.challengesSeen)).toEqual(new Set(["cpu", "space_commit", "space_open", "bandwidth"]));
  });

  it("node, který lže o specs, dostane nižší skóre než poctivý", async () => {
    const honest = await node();
    const liar = await node({ cpuFake: true, spaceFraction: 0.25, bandwidthWrongHash: true });
    await until(async () => (await allMeasured(honest.nodeId)()) && (await rowOf(liar.nodeId)).strikes >= 3);
    const sh = await scoreRow(honest.nodeId);
    const sl = await scoreRow(liar.nodeId);
    expect(sh.base).toBeGreaterThan(0);
    expect(sl.base).toBe(0);
    expect(sl.base).toBeLessThan(sh.base);
    const lr = await rowOf(liar.nodeId);
    expect(lr.strikes).toBeGreaterThan(0);
    expect(lr.health).toBeLessThan(1);
    expect(lr.m_cpu_hps).toBe(0);
  });

  it("pomalejší procesor naměří nižší výkon než rychlý", async () => {
    const hps = async (id: string) => (await rowOf(id)).m_cpu_hps as number | null;
    // po jednom, aby se výpočty nenabíraly o jedno vlákno izolátu
    const fast = await node({}, undefined, 1);
    await until(async () => (await hps(fast.nodeId)) !== null);
    fast.close();
    const slow = await node({ cpuDelayMs: 2500 }, undefined, 1);
    await until(async () => (await hps(slow.nodeId)) !== null);
    expect((await hps(fast.nodeId))!).toBeGreaterThan((await hps(slow.nodeId))! * 2);
  });

  it("pomalá odpověď na otevření (disk místo RAM) nedostane kredit za RAM, ale není trestána", async () => {
    await withRegistry((r) => {
      r.cfg = { ...r.cfg, challenges: { ...r.cfg.challenges, space: { ...r.cfg.challenges.space, ram: { ...r.cfg.challenges.space.ram, deadlineMs: 150 } } } };
    });
    const n = await node({ spaceDelayMs: 600 });
    await until(async () => (await rowOf(n.nodeId)).m_ram_mb === 0);
    const r = await rowOf(n.nodeId);
    expect(r.strikes).toBe(0);
    expect(r.m_ram_mb).toBe(0);
    expect((await scoreRow(n.nodeId)).base).toBe(0);
  });

  it("opakované neplatné důkazy vedou k banu a uvolnění zdrojů", async () => {
    await withRegistry((r) => {
      r.cfg = { ...r.cfg, score: { ...r.cfg.score, banStrikes: 2 } };
    });
    const n = await node({ cpuFake: true });
    await until(async () => (await rowOf(n.nodeId)).status === "banned");
    const r = await rowOf(n.nodeId);
    expect(r.ban_reason).toMatch(/invalid challenge/);
    expect(r.tunnel_id).toBeNull();
  });

  it("mlčící node: výzva propadne, zdraví klesne, kapacita zůstane nezměřená", async () => {
    const n = await node({ silent: true });
    await tick();
    await sleep(150);
    // ping bez pongu: výzva se po timeoutu zahodí a znovu spustí
    await setTimeOffset(60_000);
    await withRegistry((r) => {
      sqlOf(r).exec("UPDATE nodes SET last_hb_at = ? WHERE node_id = ?", Math.floor(r.clock() / 1000), n.nodeId);
    });
    await tick();
    const pending = await withRegistry((r) => sqlOf(r).exec("SELECT COUNT(*) AS c FROM challenges WHERE node_id = ?", n.nodeId).one() as any);
    expect(pending.c).toBeLessThanOrEqual(1);
    expect((await rowOf(n.nodeId)).m_cpu_hps).toBeNull();
    expect((await scoreRow(n.nodeId)).base).toBe(0);
  });

  it("timeout odpovědi po pongu sníží zdraví (soft), nikoli kapacitu", async () => {
    const n = await node();
    await until(async () => (await rowOf(n.nodeId)).m_cpu_hps !== null);
    const before = (await rowOf(n.nodeId)).m_cpu_hps;
    n.behavior = { silent: true };
    await withRegistry((r) => sqlOf(r).exec("UPDATE schedule SET next_due = 0"));
    // ping odpoví runtime přes tick; po pongu nechceme odpovídat na challenge – simulujeme: pong ano, odpověď ne
    n.behavior = {};
    n.autoRespond = false;
    n.ws!.addEventListener("message", (ev: MessageEvent) => {
      const m = JSON.parse(ev.data as string);
      if (m.t === "ping") n.send({ t: "pong", id: m.id });
    });
    await tick();
    await sleep(300);
    await setTimeOffset(10 * 60 * 1000);
    await withRegistry((r) => sqlOf(r).exec("UPDATE nodes SET last_hb_at = ? WHERE node_id = ?", Math.floor(r.clock() / 1000), n.nodeId));
    const h0 = (await rowOf(n.nodeId)).health;
    await tick();
    const after = await rowOf(n.nodeId);
    expect(after.health).toBeLessThan(h0);
    expect(after.m_cpu_hps).toBe(before);
    expect(after.strikes).toBe(0);
  });

  it("commitment se znovu používá: další kolo je jen space_open", async () => {
    const n = await node();
    await until(allMeasured(n.nodeId));
    const commits = n.challengesSeen.filter((k) => k === "space_commit").length;
    expect(commits).toBe(2); // ram + storage
    n.challengesSeen.length = 0;
    await withRegistry((r) => sqlOf(r).exec("UPDATE schedule SET next_due = 0"));
    await until(async () => n.challengesSeen.includes("space_open") && n.challengesSeen.includes("cpu") && n.challengesSeen.includes("bandwidth"));
    expect(n.challengesSeen).not.toContain("space_commit");
  });

  it("po úspěchu se další výzva plánuje s jitterem do budoucna", async () => {
    const n = await node();
    await until(allMeasured(n.nodeId));
    await sleep(200);
    const rows = await withRegistry((r) => sqlOf(r).exec("SELECT kind, next_due FROM schedule WHERE node_id = ?", n.nodeId).toArray() as any[]);
    const now = Math.floor((Date.now() + testClock.offsetMs) / 1000);
    expect(rows).toHaveLength(4);
    for (const r of rows) expect(r.next_due).toBeGreaterThan(now);
  });

  it("jen jedna výzva na node najednou", async () => {
    const n = await node();
    await tick();
    await tick();
    const c = await withRegistry((r) => sqlOf(r).exec("SELECT COUNT(*) AS c FROM challenges WHERE node_id = ?", n.nodeId).one() as any);
    expect(c.c).toBeLessThanOrEqual(1);
  });

  it("neznámá nebo cizí odpověď na výzvu se ignoruje", async () => {
    const a = await node({ silent: true });
    const b = await node({ silent: true });
    await tick();
    const cid = await withRegistry((r) => (sqlOf(r).exec("SELECT id FROM challenges WHERE node_id = ?", a.nodeId).toArray()[0] as any)?.id);
    if (cid) b.send({ t: "challenge_response", id: cid, sha256: "x" });
    b.send({ t: "challenge_response", id: "neexistuje", checkpoints: [] });
    await sleep(150);
    expect((await rowOf(b.nodeId)).ch_total).toBe(0);
    expect((await rowOf(a.nodeId)).ch_total).toBe(0);
  });
});

describe("bandwidth endpointy", () => {
  async function pendingBandwidth(): Promise<{ dl: string; ul: string; node: TestNode }> {
    const n = await newNode();
    n.autoRespond = false;
    open.push(n);
    await withRegistry((r) => {
      r.cfg = { ...r.cfg, challenges: { ...r.cfg.challenges, enabled: ["bandwidth"] } };
    });
    await bringUp(n);
    n.ws!.addEventListener("message", (ev: MessageEvent) => {
      const m = JSON.parse(ev.data as string);
      if (m.t === "ping") n.send({ t: "pong", id: m.id });
    });
    await tick();
    const ch = await n.waitFor((m) => m.t === "challenge" && m.kind === "bandwidth");
    return { dl: new URL(ch.dl_url).pathname, ul: new URL(ch.ul_url).pathname, node: n };
  }

  it("odmítá neplatný token a jednorázově obslouží stažení i nahrání", async () => {
    const { dl, ul } = await pendingBandwidth();
    expect((await exports.default.fetch(`${BASE}/challenge/blob/garbage`)).status).toBe(401);
    const flipped = dl.slice(0, -1) + (dl.endsWith("A") ? "B" : "A"); // vždy jiný poslední znak MAC
    expect((await exports.default.fetch(`${BASE}${flipped}`)).status).toBe(401);
    // token pro stažení nelze použít pro nahrání
    expect((await exports.default.fetch(new Request(`${BASE}${dl.replace("blob", "upload")}`, { method: "PUT", body: new Uint8Array(10) }))).status).toBe(401);

    const first = await exports.default.fetch(`${BASE}${dl}`);
    expect(first.status).toBe(200);
    const bytes = new Uint8Array(await first.arrayBuffer());
    expect(bytes.length).toBe(64 * 1024);
    expect((await exports.default.fetch(`${BASE}${dl}`)).status).toBe(404);

    expect((await exports.default.fetch(new Request(`${BASE}${ul}`, { method: "PUT", body: new Uint8Array(64 * 1024 + 1) }))).status).toBe(413);
  });
});

describe("peer witnessy", () => {
  it("mediánem svědků doplní měření šířky pásma cíle", async () => {
    await withRegistry((r) => {
      r.cfg = {
        ...r.cfg,
        challenges: { ...r.cfg.challenges, enabled: ["bandwidth"], witness: { ...r.cfg.challenges.witness, enabled: true, minAsns: 3, count: 3, bytes: 1_000_000, intervalS: 600 } },
      };
    });
    const nodes: TestNode[] = [];
    for (let i = 0; i < 4; i++) nodes.push(await node({ witnessMs: 1000 + i * 1000 }, 64500 + i));
    const asns = await withRegistry((r) => sqlOf(r).exec("SELECT asn FROM nodes").toArray().map((x: any) => x.asn));
    if (asns.every((a: number | null) => a === null)) {
      // runtime nepropaguje request.cf do testovacího requestu; ASN nastavíme přímo
      await withRegistry((r) => nodes.forEach((n, i) => sqlOf(r).exec("UPDATE nodes SET asn = ? WHERE node_id = ?", 64500 + i, n.nodeId)));
    }
    await until(async () => {
      const rows = await withRegistry((r) => sqlOf(r).exec("SELECT m_bw_wit FROM nodes").toArray() as any[]);
      return rows.some((x) => x.m_bw_wit !== null);
    });
    const rows = await withRegistry((r) => sqlOf(r).exec("SELECT m_bw_wit, m_bw_r2, m_bw_mbps FROM nodes WHERE m_bw_wit IS NOT NULL").toArray() as any[]);
    const x = rows[0]!;
    // 1 000 000 B za 1–4 s = 8 / 4 … 8 Mbit/s; kombinace je minimum z R2 a svědků
    expect(x.m_bw_wit).toBeGreaterThan(1);
    expect(x.m_bw_wit).toBeLessThan(9);
    expect(x.m_bw_mbps).toBeLessThanOrEqual(Math.min(x.m_bw_wit, x.m_bw_r2 ?? Infinity) + 1e-9);
  });

  it("bez dostatku různých ASN se svědci nepoužijí", async () => {
    await withRegistry((r) => {
      r.cfg = { ...r.cfg, challenges: { ...r.cfg.challenges, enabled: ["bandwidth"], witness: { ...r.cfg.challenges.witness, enabled: true, minAsns: 3 } } };
    });
    const a = await node();
    const b = await node();
    await until(async () => (await rowOf(a.nodeId)).m_bw_r2 !== null && (await rowOf(b.nodeId)).m_bw_r2 !== null);
    await tick();
    expect((await rowOf(a.nodeId)).m_bw_wit).toBeNull();
    void registryStub;
  });
});
