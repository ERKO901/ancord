import { timeoutSignal } from "../http";
import { all, one, run } from "./store";
import { proofOrigin } from "./registration";
import { log, type RegCtx } from "./context";

/**
 * Dosažitelnost nodů (spec §6.1, ARCHITECTURE.md §9 bod 7). Nody v heartbeatu hlásí peery, na které se nedovolaly.
 * Hlášení Worker nevěří (Sybil by jím vyřadil poctivé nody), jen podle něj sám ověří `GET https://<hostname>/health`
 * stejnou cestou, jakou volají nody (Cloudflare → tunel). Node, který neprojde `failuresToExclude` ověřeními po sobě,
 * vypadne z dalšího adresáře i z routingu, dokud neprojde dalším ověřením.
 */

export const MAX_REPORTED = 16;
const HEALTH_PATH = "/health";

/** `unreachable` z heartbeatu; chybějící pole je prázdný seznam, špatný tvar `null`. */
export function parseUnreachable(v: unknown): string[] | null {
  if (v === undefined) return [];
  if (!Array.isArray(v) || v.length > MAX_REPORTED) return null;
  const out = new Set<string>();
  for (const e of v) {
    if (typeof e !== "string" || e.length === 0 || e.length > 64) return null;
    out.add(e);
  }
  return [...out];
}

export function recordReports(ctx: RegCtx, reporterId: string, targets: string[]): void {
  if (!ctx.cfg.reach.enabled) return;
  const now = ctx.nowS();
  for (const t of targets) {
    if (t === reporterId) continue;
    // jen existující aktivní nody; neznámé id se tiše zahodí
    if (!one(ctx.sql, "SELECT 1 AS x FROM nodes WHERE node_id = ? AND status = 'active'", t)) continue;
    run(
      ctx.sql,
      "INSERT INTO reach_reports (target_id, reporter_id, at) VALUES (?, ?, ?) ON CONFLICT(target_id, reporter_id) DO UPDATE SET at = excluded.at",
      t,
      reporterId,
      now,
    );
  }
}

/** Nody vyřazené z adresáře a routingu pro nedosažitelnost. */
export function excludedIds(ctx: RegCtx): Set<string> {
  return new Set(all<{ node_id: string }>(ctx.sql, "SELECT node_id FROM reach WHERE excluded = 1").map((r) => r.node_id));
}

async function probe(ctx: RegCtx, hostname: string): Promise<boolean> {
  try {
    const res = await ctx.fetcher()(`${proofOrigin(ctx, hostname)}${HEALTH_PATH}`, {
      redirect: "manual",
      signal: timeoutSignal(ctx.cfg.reach.probeTimeoutMs),
    });
    await res.body?.cancel();
    return res.status === 200;
  } catch {
    return false;
  }
}

interface Candidate {
  node_id: string;
  hostname: string;
  fails: number | null;
  excluded: number | null;
  [k: string]: SqlStorageValue;
}

/** Krok tiku: úklid hlášení, ověření nahlášených a vyřazených nodů. */
export async function reachTick(ctx: RegCtx): Promise<void> {
  const r = ctx.cfg.reach;
  if (!r.enabled) return;
  const now = ctx.nowS();
  run(ctx.sql, "DELETE FROM reach_reports WHERE at < ?", now - r.reportTtlS);
  run(ctx.sql, "DELETE FROM reach WHERE node_id NOT IN (SELECT node_id FROM nodes WHERE status = 'active')");
  run(ctx.sql, "DELETE FROM reach_reports WHERE target_id NOT IN (SELECT node_id FROM nodes WHERE status = 'active')");

  const live = one<{ c: number }>(
    ctx.sql,
    "SELECT COUNT(*) AS c FROM nodes WHERE status = 'active' AND last_hb_at >= ?",
    now - ctx.cfg.heartbeatTimeoutS,
  )!.c;
  const candidates = all<Candidate>(
    ctx.sql,
    `SELECT n.node_id, n.hostname, r.fails, r.excluded FROM nodes n
       LEFT JOIN reach r ON r.node_id = n.node_id
      WHERE n.status = 'active' AND n.last_hb_at >= ?
        AND (
          (r.excluded = 1 AND r.probed_at <= ?)
          OR (r.excluded = 0 AND r.fails > 0 AND r.probed_at <= ?)
          OR ((SELECT COUNT(*) FROM reach_reports p WHERE p.target_id = n.node_id) >= ?
              AND (r.node_id IS NULL OR r.probed_at <= ?))
        )
      ORDER BY COALESCE(r.probed_at, 0)
      LIMIT ?`,
    now - ctx.cfg.heartbeatTimeoutS,
    now - r.reprobeS,
    now - r.probeIntervalS,
    r.minReporters,
    now - r.probeIntervalS,
    r.maxProbesPerTick,
  );
  if (candidates.length === 0) return;

  const results = await Promise.all(candidates.map((c) => probe(ctx, c.hostname)));
  const maxExcluded = Math.floor(live * r.maxExcludedFraction);
  let excludedNow = one<{ c: number }>(ctx.sql, "SELECT COUNT(*) AS c FROM reach WHERE excluded = 1")!.c;
  candidates.forEach((c, k) => {
    const wasExcluded = c.excluded === 1;
    if (results[k]) {
      run(ctx.sql, "INSERT OR REPLACE INTO reach (node_id, fails, excluded, probed_at) VALUES (?, 0, 0, ?)", c.node_id, now);
      // hlášení se nepotvrdila: zahodit, nové ověření až po dalších
      run(ctx.sql, "DELETE FROM reach_reports WHERE target_id = ?", c.node_id);
      if (wasExcluded) {
        excludedNow--;
        log("reach_restored", { node_id: c.node_id });
      }
      return;
    }
    const fails = (c.fails ?? 0) + 1;
    let excluded = wasExcluded;
    if (!excluded && fails >= r.failuresToExclude) {
      if (excludedNow < maxExcluded) {
        excluded = true;
        excludedNow++;
        log("reach_excluded", { node_id: c.node_id, fails });
      } else {
        log("reach_exclusion_capped", { node_id: c.node_id, fails, excluded: excludedNow, live });
      }
    }
    run(
      ctx.sql,
      "INSERT OR REPLACE INTO reach (node_id, fails, excluded, probed_at) VALUES (?, ?, ?, ?)",
      c.node_id,
      fails,
      excluded ? 1 : 0,
      now,
    );
  });
}
