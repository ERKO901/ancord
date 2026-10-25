import { env, exports } from "cloudflare:workers";
import { runInDurableObject } from "cloudflare:test";
import { sharedMockClient } from "../../src/cloudflare";
import { withOverrides } from "../../src/config";
import type { Registry } from "../../src/registry/registry";
import { clock } from "../../src/clock";
import { resetRouteCache } from "../../src/index";
import { TestNode, testClock } from "./node";

export function registryStub() {
  return env.REGISTRY.getByName("main");
}

/** SQL storage Registry DO pro přímé nahlédnutí/seed v testech. */
export function sqlOf(r: Registry): SqlStorage {
  return (r as unknown as { ctx: DurableObjectState }).ctx.storage.sql;
}

export async function withRegistry<T>(fn: (r: Registry) => T | Promise<T>): Promise<T> {
  return runInDurableObject(registryStub(), (instance) => fn(instance as Registry));
}

export async function configure(patch: Parameters<typeof withOverrides>[1]): Promise<void> {
  await withRegistry((r) => {
    r.cfg = withOverrides(r.cfg, patch);
  });
}

/** Otevřená síť bez genesis allowlistu, rychlé challenge parametry. */
export const FAST: Parameters<typeof withOverrides>[1] = {
  genesisOnly: false,
  score: { ref: { cpuHps: 50_000, ramMb: 0.0625, storageGb: 0.00006103515625, bandwidthMbps: 1 } },
  challenges: {
    cpu: { iters: 20_000, every: 1_000, segments: 4, maxLanes: 4 },
    space: {
      ram: { minLeaves: 1 << 10, maxLeaves: 1 << 11, deadlineMs: 5000 },
      storage: { minLeaves: 1 << 10, maxLeaves: 1 << 11, deadlineMs: 5000 },
      openCount: 8,
    },
    bandwidth: { bytes: 64 * 1024, poolSize: 1 },
    pingTimeoutMs: 5000,
    retryDelayS: 1,
  },
};

/** Nastaví společný posunutý čas Workeru, Registry a simulovaných nodů. */
export async function setTimeOffset(ms: number): Promise<void> {
  testClock.offsetMs = ms;
  clock.nowMs = () => Date.now() + testClock.offsetMs;
  await withRegistry((r) => {
    r.clock = () => Date.now() + testClock.offsetMs;
  });
  resetRouteCache();
}

/** Posune čas tak, aby „teď“ bylo v zadaném místě epochy (sekundy od začátku epochy). */
export async function alignToEpoch(epochSeconds: number, intoEpochS = 5, epochsAhead = 0): Promise<number> {
  const nowS = Math.floor(Date.now() / 1000);
  const targetStart = (Math.floor(nowS / epochSeconds) + 1 + epochsAhead) * epochSeconds;
  await setTimeOffset((targetStart + intoEpochS - nowS) * 1000);
  return targetStart / epochSeconds;
}

const TABLES = ["nodes", "reg_challenges", "reg_log", "nonces", "directories", "schedule", "challenges", "space", "asn_penalties", "meta"];

/** Vyčistí stav mezi testy (úložiště DO ani KV se mezi testy spolehlivě neodvrací, když zůstanou otevřené WebSockety). */
export async function resetState(): Promise<void> {
  await withRegistry((r) => {
    for (const w of (r as unknown as { ctx: DurableObjectState }).ctx.getWebSockets()) {
      try {
        w.close(1000, "reset");
      } catch {
        /* už zavřeno */
      }
    }
    for (const t of TABLES) sqlOf(r).exec(`DELETE FROM ${t}`);
  });
  for (const kv of [env.ROUTING_KV, env.PUSH_KV]) {
    for (const k of (await kv.list()).keys) await kv.delete(k.name);
  }
  sharedMockClient().tunnels.clear();
  sharedMockClient().dns.clear();
  sharedMockClient().foreignNames.clear();
  sharedMockClient().failOn.clear();
}

export async function setup(patch: Parameters<typeof withOverrides>[1] = FAST): Promise<void> {
  sharedMockClient().calls.length = 0;
  await setTimeOffset(0);
  await resetState();
  await configure(patch);
  await withRegistry((r) => {
    r.fetchImpl = async (input: RequestInfo | URL) => {
      const url = typeof input === "string" ? input : input instanceof URL ? input.href : input.url;
      const host = new URL(url).hostname;
      const node = knownNodes.get(host);
      if (node && url.includes("/.well-known/melo-proof")) return node.proofResponse(url);
      return new Response("no such host", { status: 502 });
    };
  });
}

const knownNodes = new Map<string, TestNode>();

export async function newNode(opts: ConstructorParameters<typeof Object>[0] & Parameters<typeof TestNode.create>[0] = {}): Promise<TestNode> {
  const n = await TestNode.create(opts);
  knownNodes.set(n.hostname, n);
  return n;
}

/** Zaregistruje, aktivuje a připojí node; vrací připojený node. */
export async function bringUp(node: TestNode, connect = true): Promise<TestNode> {
  const { res } = await node.register();
  if (res.status !== 200) throw new Error(`register failed: ${res.status}`);
  const act = await node.activate();
  if (act.status !== 200) throw new Error(`activate failed: ${act.status} ${await act.text()}`);
  if (connect) await node.connect();
  return node;
}

export async function tick(): Promise<void> {
  await registryStub().tick();
}

export { exports };

export interface Seed {
  lat?: number | null;
  lon?: number | null;
  country?: string | null;
  continent?: string | null;
  load?: { conns: number; max_conns: number; cpu_pct: number; mem_pct: number } | null;
  /** poměr naměřené kapacity k referenci (1 = přesně reference) */
  quality?: number;
  lastHbS?: number;
}

/** Aktivuje node (bez WebSocketu) a zapíše naměřené hodnoty přímo do registru. */
export async function seedActive(node: TestNode, seed: Seed = {}): Promise<TestNode> {
  await bringUp(node, false);
  await seedMeasured(node.nodeId, seed);
  return node;
}

export async function seedMeasured(nodeId: string, seed: Seed = {}): Promise<void> {
  const q = seed.quality ?? 1;
  await withRegistry((r) => {
    const ref = r.cfg.score.ref;
    sqlOf(r).exec(
      `UPDATE nodes SET m_cpu_hps = ?, m_ram_mb = ?, m_storage_gb = ?, m_bw_mbps = ?, m_bw_r2 = ?, health = 1,
         last_hb_at = ?, lat = ?, lon = ?, country = ?, continent = ?, load = ? WHERE node_id = ?`,
      ref.cpuHps * q,
      ref.ramMb * q,
      ref.storageGb * q,
      ref.bandwidthMbps * q,
      ref.bandwidthMbps * q,
      seed.lastHbS ?? r.clock() / 1000,
      seed.lat ?? null,
      seed.lon ?? null,
      seed.country ?? null,
      seed.continent ?? null,
      seed.load ? JSON.stringify(seed.load) : null,
      nodeId,
    );
  });
}
