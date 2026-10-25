import { sha256 } from "../crypto";
import { b64uEncode, utf8 } from "../encoding";
import type { Snapshot, SnapshotNode } from "../routing";
import { all, getMeta, setMeta, type NodeRow } from "./store";
import { epochOf, type SignedDirectory } from "./directory";
import { scoreOf } from "./scoring";
import type { RegCtx } from "./context";

export const KV_SNAPSHOT = "routing:snapshot";
export const KV_DIRECTORY = "directory:current";
export const kvDirectoryKey = (epoch: number) => `directory:e:${epoch}`;

/** Adresář epochy do KV (klíč epochy i `current`), odkud ho čtou `/directory` a `/wake`. */
export async function publishDirectory(env: RegCtx["env"], directory: SignedDirectory): Promise<void> {
  const json = JSON.stringify(directory);
  await Promise.all([
    env.ROUTING_KV.put(KV_DIRECTORY, json, { expirationTtl: 3600 }),
    env.ROUTING_KV.put(kvDirectoryKey(directory.body.epoch), json, { expirationTtl: 3600 }),
  ]);
}

const round3 = (x: number) => Math.round(x * 1000) / 1000;

export function buildSnapshotNodes(ctx: RegCtx): SnapshotNode[] {
  const nowS = ctx.nowS();
  const rows = all<NodeRow & Record<string, SqlStorageValue>>(ctx.sql, "SELECT * FROM nodes WHERE status = 'active' ORDER BY node_id");
  const out: SnapshotNode[] = [];
  for (const r of rows) {
    const sc = scoreOf(ctx.cfg, r, nowS);
    if (sc.base <= 0) continue;
    out.push({
      id: r.node_id,
      host: r.hostname,
      b: round3(sc.base),
      s: round3(sc.net),
      cc: r.country,
      ct: r.continent,
      lat: r.lat === null ? null : Math.round(r.lat * 100) / 100,
      lon: r.lon === null ? null : Math.round(r.lon * 100) / 100,
    });
  }
  return out;
}

/**
 * Zapíše routing snapshot do KV, když se obsah změnil nebo když uplynul `snapshotRefreshS`
 * (čerstvost, kterou kontroluje `/route`). KV má limit 1 zápis/s na klíč, proto hash.
 */
export async function publishSnapshot(ctx: RegCtx, force = false): Promise<boolean> {
  const nowS = ctx.nowS();
  const nodes = buildSnapshotNodes(ctx);
  const hash = b64uEncode(sha256(utf8(JSON.stringify(nodes))));
  const lastHash = getMeta(ctx.sql, "snapshot_hash");
  const lastAt = Number(getMeta(ctx.sql, "snapshot_at") ?? 0);
  if (!force && hash === lastHash && nowS - lastAt < ctx.cfg.snapshotRefreshS) return false;
  const snap: Snapshot = { v: 1, generated_at: nowS, epoch: epochOf(nowS, ctx.cfg.epochSeconds), nodes };
  await ctx.env.ROUTING_KV.put(KV_SNAPSHOT, JSON.stringify(snap), { expirationTtl: 3600 });
  setMeta(ctx.sql, "snapshot_hash", hash);
  setMeta(ctx.sql, "snapshot_at", String(nowS));
  return true;
}
