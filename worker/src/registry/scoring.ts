import type { Config } from "../config";
import type { Claimed, Load, NodeRow } from "./store";

export interface Score {
  capacity: number;
  health: number;
  trust: number;
  base: number;
  penalty: number;
  /** capacity × health × trust − load_penalty */
  net: number;
  util: number;
}

export function parseClaimed(row: NodeRow): Claimed {
  return JSON.parse(row.claimed) as Claimed;
}

export function parseLoad(row: NodeRow): Load | null {
  return row.load ? (JSON.parse(row.load) as Load) : null;
}

export function isLive(cfg: Config, row: NodeRow, nowS: number): boolean {
  return row.status === "active" && row.last_hb_at !== null && nowS - row.last_hb_at <= cfg.heartbeatTimeoutS;
}

/** Vážený geometrický průměr poměrů min(claimed, measured) / reference přes zapnuté dimenze. */
export function capacityOf(cfg: Config, row: NodeRow): number {
  const claimed = parseClaimed(row);
  const { ref, weights, ratioCap } = cfg.score;
  const enabled = new Set(cfg.challenges.enabled);
  const dims: { w: number; measured: number | null; claimed: number; ref: number }[] = [];
  if (enabled.has("cpu")) dims.push({ w: weights.cpu, measured: row.m_cpu_hps, claimed: Infinity, ref: ref.cpuHps });
  if (enabled.has("space_ram")) dims.push({ w: weights.ram, measured: row.m_ram_mb, claimed: claimed.ram_mb, ref: ref.ramMb });
  if (enabled.has("space_storage")) {
    dims.push({ w: weights.storage, measured: row.m_storage_gb, claimed: claimed.storage_gb, ref: ref.storageGb });
  }
  if (enabled.has("bandwidth")) {
    dims.push({ w: weights.bandwidth, measured: row.m_bw_mbps, claimed: claimed.bandwidth_mbps, ref: ref.bandwidthMbps });
  }
  let wSum = 0;
  let acc = 0;
  for (const d of dims) {
    if (d.w <= 0) continue;
    if (d.measured === null || d.measured <= 0) return 0; // nezměřeno = bez kapacity
    const ratio = Math.min(Math.min(d.claimed, d.measured) / d.ref, ratioCap);
    wSum += d.w;
    acc += d.w * Math.log(ratio);
  }
  return wSum === 0 ? 0 : Math.exp(acc / wSum);
}

export function trustOf(cfg: Config, row: NodeRow): number {
  if (row.status !== "active") return 0;
  if (row.genesis === 1 || !cfg.probation.enabled) return 1;
  const p = cfg.probation;
  const progress =
    Math.min(1, row.uptime_s / p.durationS) * Math.min(1, row.ch_ok / Math.max(1, p.minChallenges));
  return p.floor + (1 - p.floor) * progress;
}

export function utilOf(load: Load | null): number {
  if (!load) return 0;
  const byConns = load.max_conns > 0 ? load.conns / load.max_conns : 0;
  return Math.min(1, Math.max(load.cpu_pct / 100, byConns));
}

export function scoreOf(cfg: Config, row: NodeRow, nowS: number): Score {
  const live = isLive(cfg, row, nowS);
  const capacity = capacityOf(cfg, row);
  const health = live ? Math.max(0, Math.min(1, row.health)) : 0;
  const trust = trustOf(cfg, row);
  const base = capacity * health * trust;
  const util = utilOf(parseLoad(row));
  // kvantizace zátěže po 5 %, aby snapshot nekmital kvůli šumu
  const q = Math.round(util * 20) / 20;
  const penalty = base * cfg.score.loadWeight * q;
  return { capacity, health, trust, base, penalty, net: Math.max(0, base - penalty), util: q };
}
