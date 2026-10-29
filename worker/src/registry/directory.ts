import { canonicalBytes, type Json } from "../canonical";
import { DOMAIN } from "../crypto";
import { b64uEncode } from "../encoding";
import { all, one, run, type NodeRow } from "./store";
import type { RegCtx } from "./context";
import { excludedIds } from "./reach";

export interface DirectoryNode {
  id: string;
  pk: string;
  host: string;
}

export interface DirectoryBody {
  v: 1;
  epoch: number;
  epoch_seconds: number;
  issued_at: number;
  params: { r: number; quorum: number };
  nodes: DirectoryNode[];
}

export interface SignedDirectory {
  body: DirectoryBody;
  sig: string;
  kid: string;
}

export function epochOf(nowS: number, epochSeconds: number): number {
  return Math.floor(nowS / epochSeconds);
}

export function readDirectory(ctx: RegCtx, epoch: number): SignedDirectory | null {
  const row = one<{ body: string; sig: string; kid: string }>(
    ctx.sql,
    "SELECT body, sig, kid FROM directories WHERE epoch = ?",
    epoch,
  );
  return row ? { body: JSON.parse(row.body) as DirectoryBody, sig: row.sig, kid: row.kid } : null;
}

/** Aktivní, živé a dosažitelné nody (spec §6, §6.1). */
export function eligibleForDirectory(ctx: RegCtx, nowS: number): NodeRow[] {
  const { cfg } = ctx;
  const excluded = excludedIds(ctx);
  return all<NodeRow & Record<string, SqlStorageValue>>(
    ctx.sql,
    `SELECT * FROM nodes
      WHERE status = 'active' AND last_hb_at IS NOT NULL AND last_hb_at >= ?
        AND (genesis = 1 OR ch_ok >= ?)
      ORDER BY node_id`,
    nowS - cfg.heartbeatTimeoutS,
    cfg.directoryMinChallenges,
  ).filter((n) => !excluded.has(n.node_id));
}

/**
 * Adresář aktuální epochy: sestaví se jednou při prvním přístupu a je neměnný (spec §6).
 * Při souběhu vyhrává první zapsaný, ostatní vrátí uložený.
 */
export async function ensureDirectory(ctx: RegCtx): Promise<{ directory: SignedDirectory; created: boolean }> {
  const { cfg } = ctx;
  const epoch = epochOf(ctx.nowS(), cfg.epochSeconds);
  const existing = readDirectory(ctx, epoch);
  if (existing) return { directory: existing, created: false };

  const nodes = eligibleForDirectory(ctx, ctx.nowS()).map((n) => ({ id: n.node_id, pk: n.pubkey, host: n.hostname }));
  const body: DirectoryBody = {
    v: 1,
    epoch,
    epoch_seconds: cfg.epochSeconds,
    issued_at: epoch * cfg.epochSeconds,
    params: { r: cfg.r, quorum: cfg.quorum },
    nodes,
  };
  const signer = await ctx.signer();
  const sig = b64uEncode(await signer.sign(DOMAIN.directory, canonicalBytes(body as unknown as Json)));

  // během podepisování mohl zapsat jiný volající
  const raced = readDirectory(ctx, epoch);
  if (raced) return { directory: raced, created: false };

  run(
    ctx.sql,
    "INSERT INTO directories (epoch, body, sig, kid) VALUES (?, ?, ?, ?)",
    epoch,
    JSON.stringify(body),
    sig,
    signer.kid,
  );
  run(ctx.sql, "DELETE FROM directories WHERE epoch <= ?", epoch - cfg.keepDirectories);
  return { directory: { body, sig, kid: signer.kid }, created: true };
}

export function broadcastDirectory(ctx: RegCtx, dir: SignedDirectory): void {
  const msg = JSON.stringify({ t: "directory", directory: dir });
  for (const ws of ctx.sockets()) {
    try {
      ws.send(msg);
    } catch {
      /* zavřený socket */
    }
  }
}
