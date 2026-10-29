import type { Env } from "./env";

export type ProfileName = "dev" | "prod" | "open";
export type ChallengeKind = "cpu" | "space_ram" | "space_storage" | "bandwidth";

export interface SpaceTierConfig {
  minLeaves: number;
  maxLeaves: number;
  /** Limit odpovědi na otevření po odečtení RTT. */
  deadlineMs: number;
  openIntervalS: number;
  commitMaxAgeS: number;
}

/** Parametry sítě podle profilu (ARCHITECTURE.md §10). Nic z toho není v kódu natvrdo. */
export interface Config {
  profile: ProfileName;
  r: number;
  quorum: number;
  routeNodes: number;
  minLiveNodes: number;
  epochSeconds: number;
  heartbeatIntervalS: number;
  heartbeatTimeoutS: number;
  tickIntervalS: number;
  snapshotRefreshS: number;
  snapshotMaxAgeS: number;
  ticketTtlS: number;
  requestSkewS: number;
  maxWsMessageBytes: number;
  keepDirectories: number;

  genesisOnly: boolean;
  registerChallengeTtlS: number;
  pendingTtlS: number;
  provisioningTimeoutS: number;
  maxOpenChallengesPerAsn: number;
  maxOpenChallenges: number;
  directoryMinChallenges: number;

  probation: { enabled: boolean; durationS: number; floor: number; minChallenges: number };
  hashcash: { enabled: boolean; baseBits: number; maxBits: number; surgeWindowS: number; surgeUnit: number };
  limits: { enabled: boolean; maxPerAsn: number; maxPerPrefix: number; asnBanPenalties: number };

  routing: { nearKm: number };

  score: {
    ref: { cpuHps: number; ramMb: number; storageGb: number; bandwidthMbps: number };
    weights: { cpu: number; ram: number; storage: number; bandwidth: number };
    ratioCap: number;
    loadWeight: number;
    healthAlpha: number;
    banStrikes: number;
  };

  challenges: {
    enabled: ChallengeKind[];
    jitter: number;
    cpu: { iters: number; every: number; segments: number; maxLanes: number; intervalS: number; maxHps: number; slackMs: number; timeoutMs: number };
    space: { ram: SpaceTierConfig; storage: SpaceTierConfig; openCount: number; commitTimeoutMs: number };
    bandwidth: { bytes: number; poolSize: number; intervalS: number; timeoutMs: number; tokenTtlS: number };
    witness: { enabled: boolean; minAsns: number; count: number; bytes: number; intervalS: number; timeoutMs: number };
    pingTimeoutMs: number;
    /** Za jak dlouho se zopakuje výzva, která selhala. */
    retryDelayS: number;
  };

  push: { nodeBurst: number; nodeRefillPerS: number; coalesceS: number; regSkewS: number };

  /** Dosažitelnost nodů z ostatních (ARCHITECTURE.md §9 bod 7, PROTOCOL §6.1). */
  reach: {
    enabled: boolean;
    /** Kolik různých nodů musí cíl nahlásit, než ho Worker ověří. */
    minReporters: number;
    /** Jak dlouho platí hlášení z heartbeatu. */
    reportTtlS: number;
    probeTimeoutMs: number;
    /** Nejkratší rozestup ověření téhož nodu. */
    probeIntervalS: number;
    /** Po kolika neúspěšných ověřeních po sobě se node vyřadí z adresáře. */
    failuresToExclude: number;
    /** Jak často se ověřuje vyřazený node. */
    reprobeS: number;
    maxProbesPerTick: number;
    /** Víc než tento podíl živých nodů se nevyřadí: to už spíš nevidí Worker, ne oni. */
    maxExcludedFraction: number;
  };
}

const MiB = 1024 * 1024;

const base: Config = {
  profile: "dev",
  r: 3,
  quorum: 2,
  routeNodes: 3,
  minLiveNodes: 3,
  epochSeconds: 60,
  heartbeatIntervalS: 15,
  heartbeatTimeoutS: 45,
  tickIntervalS: 5,
  snapshotRefreshS: 30,
  snapshotMaxAgeS: 120,
  ticketTtlS: 120,
  requestSkewS: 60,
  maxWsMessageBytes: 2 * 1024 * 1024,
  keepDirectories: 12,

  genesisOnly: true,
  registerChallengeTtlS: 300,
  pendingTtlS: 900,
  provisioningTimeoutS: 120,
  maxOpenChallengesPerAsn: 50,
  maxOpenChallenges: 10_000,
  directoryMinChallenges: 0,

  probation: { enabled: false, durationS: 7 * 86400, floor: 0.05, minChallenges: 10 },
  hashcash: { enabled: false, baseBits: 16, maxBits: 28, surgeWindowS: 3600, surgeUnit: 10 },
  limits: { enabled: false, maxPerAsn: 3, maxPerPrefix: 2, asnBanPenalties: 5 },

  routing: { nearKm: 3000 },

  score: {
    ref: { cpuHps: 2_000_000, ramMb: 2, storageGb: 0.0078125, bandwidthMbps: 10 },
    weights: { cpu: 0.3, ram: 0.2, storage: 0.2, bandwidth: 0.3 },
    ratioCap: 2,
    loadWeight: 0.8,
    healthAlpha: 0.2,
    banStrikes: 5,
  },

  challenges: {
    enabled: ["cpu", "space_ram", "space_storage", "bandwidth"],
    jitter: 0.5,
    cpu: { iters: 200_000, every: 2_000, segments: 4, maxLanes: 4, intervalS: 120, maxHps: 400_000_000, slackMs: 300, timeoutMs: 30_000 },
    space: {
      ram: { minLeaves: 1 << 10, maxLeaves: 1 << 16, deadlineMs: 3000, openIntervalS: 90, commitMaxAgeS: 900 },
      storage: { minLeaves: 1 << 10, maxLeaves: 1 << 18, deadlineMs: 3000, openIntervalS: 180, commitMaxAgeS: 1800 },
      openCount: 12,
      commitTimeoutMs: 60_000,
    },
    bandwidth: { bytes: 1 * MiB, poolSize: 2, intervalS: 240, timeoutMs: 60_000, tokenTtlS: 120 },
    witness: { enabled: false, minAsns: 3, count: 3, bytes: 1 * MiB, intervalS: 600, timeoutMs: 60_000 },
    pingTimeoutMs: 10_000,
    retryDelayS: 30,
  },

  push: { nodeBurst: 60, nodeRefillPerS: 5, coalesceS: 20, regSkewS: 300 },

  reach: {
    enabled: true,
    minReporters: 1,
    reportTtlS: 120,
    probeTimeoutMs: 5000,
    probeIntervalS: 30,
    failuresToExclude: 2,
    reprobeS: 60,
    maxProbesPerTick: 5,
    maxExcludedFraction: 0.34,
  },
};

type DeepPartial<T> = { [K in keyof T]?: T[K] extends object ? (T[K] extends unknown[] ? T[K] : DeepPartial<T[K]>) : T[K] };

const PROFILES: Record<ProfileName, DeepPartial<Config>> = {
  dev: {},
  prod: {
    profile: "prod",
    epochSeconds: 300,
    heartbeatIntervalS: 45,
    heartbeatTimeoutS: 150,
    tickIntervalS: 10,
    snapshotRefreshS: 60,
    probation: { enabled: false },
    score: { ref: { cpuHps: 10_000_000, ramMb: 128, storageGb: 2, bandwidthMbps: 100 } },
    challenges: {
      cpu: { iters: 20_000_000, every: 5_000, segments: 6, maxLanes: 8, intervalS: 900, slackMs: 500, timeoutMs: 120_000 },
      space: {
        ram: { minLeaves: 1 << 16, maxLeaves: 1 << 22, deadlineMs: 150, openIntervalS: 300, commitMaxAgeS: 21_600 },
        storage: { minLeaves: 1 << 18, maxLeaves: 1 << 26, deadlineMs: 1000, openIntervalS: 900, commitMaxAgeS: 86_400 },
        openCount: 16,
        commitTimeoutMs: 1_800_000,
      },
      bandwidth: { bytes: 8 * MiB, poolSize: 4, intervalS: 1800, timeoutMs: 120_000, tokenTtlS: 300 },
      witness: { enabled: true, bytes: 4 * MiB, intervalS: 1800 },
    },
    push: { nodeBurst: 300, nodeRefillPerS: 20, coalesceS: 30 },
    reach: { reportTtlS: 300, probeIntervalS: 60, reprobeS: 120 },
  },
  open: {
    profile: "open",
    epochSeconds: 300,
    heartbeatIntervalS: 45,
    heartbeatTimeoutS: 150,
    tickIntervalS: 10,
    snapshotRefreshS: 60,
    genesisOnly: false,
    directoryMinChallenges: 3,
    probation: { enabled: true },
    hashcash: { enabled: true },
    limits: { enabled: true },
    score: { ref: { cpuHps: 10_000_000, ramMb: 128, storageGb: 2, bandwidthMbps: 100 } },
    challenges: {
      cpu: { iters: 20_000_000, every: 5_000, segments: 6, maxLanes: 8, intervalS: 900, slackMs: 500, timeoutMs: 120_000 },
      space: {
        ram: { minLeaves: 1 << 16, maxLeaves: 1 << 22, deadlineMs: 150, openIntervalS: 300, commitMaxAgeS: 21_600 },
        storage: { minLeaves: 1 << 18, maxLeaves: 1 << 26, deadlineMs: 1000, openIntervalS: 900, commitMaxAgeS: 86_400 },
        openCount: 16,
        commitTimeoutMs: 1_800_000,
      },
      bandwidth: { bytes: 8 * MiB, poolSize: 4, intervalS: 1800, timeoutMs: 120_000, tokenTtlS: 300 },
      witness: { enabled: true, bytes: 4 * MiB, intervalS: 1800 },
    },
    push: { nodeBurst: 300, nodeRefillPerS: 20, coalesceS: 30 },
    reach: { minReporters: 2, reportTtlS: 300, probeIntervalS: 60, reprobeS: 120 },
  },
};

function isObject(v: unknown): v is Record<string, unknown> {
  return typeof v === "object" && v !== null && !Array.isArray(v);
}

/** Rekurzivní merge; neznámé klíče a hodnoty jiného typu jsou chyba (překlep v konfiguraci se nesmí tiše ignorovat). */
function merge<T>(target: T, patch: unknown, path = ""): T {
  if (patch === undefined) return target;
  if (!isObject(target) || !isObject(patch)) {
    if (Array.isArray(target) && Array.isArray(patch)) return patch as T;
    if (typeof target !== typeof patch) throw new Error(`config ${path || "<root>"}: wrong type`);
    return patch as T;
  }
  const out: Record<string, unknown> = { ...target };
  for (const [k, v] of Object.entries(patch)) {
    if (!(k in target)) throw new Error(`config ${path}${k}: unknown key`);
    out[k] = merge((target as Record<string, unknown>)[k], v, `${path}${k}.`);
  }
  return out as T;
}

function validate(c: Config): void {
  const pos = (n: number, name: string) => {
    if (!Number.isFinite(n) || n <= 0) throw new Error(`config ${name} must be positive`);
  };
  pos(c.epochSeconds, "epochSeconds");
  pos(c.heartbeatIntervalS, "heartbeatIntervalS");
  pos(c.tickIntervalS, "tickIntervalS");
  pos(c.ticketTtlS, "ticketTtlS");
  if (c.heartbeatTimeoutS < c.heartbeatIntervalS) throw new Error("config heartbeatTimeoutS < heartbeatIntervalS");
  if (c.quorum > c.r || c.quorum < 1) throw new Error("config quorum must be in [1, r]");
  if (c.routeNodes < 1) throw new Error("config routeNodes must be >= 1");
  if (c.challenges.cpu.iters % c.challenges.cpu.every !== 0) throw new Error("config cpu.iters must be a multiple of cpu.every");
  for (const t of [c.challenges.space.ram, c.challenges.space.storage]) {
    for (const l of [t.minLeaves, t.maxLeaves]) {
      if (l < 2 || (l & (l - 1)) !== 0) throw new Error("config space leaves must be powers of two");
    }
  }
  if (c.reach.minReporters < 1 || c.reach.failuresToExclude < 1) throw new Error("config reach thresholds must be >= 1");
  if (c.reach.maxExcludedFraction < 0 || c.reach.maxExcludedFraction >= 1) throw new Error("config reach.maxExcludedFraction must be in [0, 1)");
  const w = c.score.weights;
  if (w.cpu + w.ram + w.storage + w.bandwidth <= 0) throw new Error("config score weights must not all be zero");
}

const cache = new Map<string, Config>();

export function loadConfig(env: Pick<Env, "PROFILE" | "CONFIG_OVERRIDES">): Config {
  const key = `${env.PROFILE}|${env.CONFIG_OVERRIDES ?? ""}`;
  const hit = cache.get(key);
  if (hit) return hit;
  const name = env.PROFILE as ProfileName;
  if (!(name in PROFILES)) throw new Error(`unknown PROFILE '${env.PROFILE}'`);
  let cfg = merge(base, PROFILES[name]);
  if (env.CONFIG_OVERRIDES) cfg = merge(cfg, JSON.parse(env.CONFIG_OVERRIDES));
  validate(cfg);
  cache.set(key, cfg);
  return cfg;
}

/** Pro testy a odvozené konfigurace. */
export function withOverrides(cfg: Config, patch: DeepPartial<Config>): Config {
  const c = merge(cfg, patch);
  validate(c);
  return c;
}

export function genesisSet(env: Pick<Env, "GENESIS_NODE_IDS">): Set<string> {
  return new Set(
    (env.GENESIS_NODE_IDS ?? "")
      .split(",")
      .map((s) => s.trim().toLowerCase())
      .filter(Boolean),
  );
}
