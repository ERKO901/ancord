import type { ChallengeKind } from "../config";
import { DOMAIN, verifySig } from "../crypto";
import { canonicalBytes } from "../canonical";
import { b64uDecode, b64uDecodeLen, b64uEncode, randomBytes, randomFloat } from "../encoding";
import { banNode } from "../registry/registration";
import { all, getNode, one, run, type NodeRow } from "../registry/store";
import { parseClaimed } from "../registry/scoring";
import { log, type RegCtx } from "../registry/context";
import { ensureBlob, signBwToken } from "./bandwidth";
import { verifyCpu } from "./cpu";
import { LEAF_BYTES, leavesFor, pickIndices, verifyOpenings } from "./space";

type Slot = ChallengeKind | "witness";
type WireKind = "cpu" | "space_commit" | "space_open" | "bandwidth" | "witness";
type Dim = "cpu" | "ram" | "storage" | "bw";

interface ChallengeRow {
  id: string;
  node_id: string;
  kind: WireKind;
  state: "ping" | "sent";
  params: string;
  created_at: number;
  sent_at: number | null;
  deadline_at: number;
  rtt_ms: number | null;
  data: string | null;
}

type Outcome =
  | { t: "ok"; apply?: Record<string, number> }
  /** bez odpovědi / chyba nodu: sníží zdraví, kapacitu nechá */
  | { t: "soft"; reason: string }
  /** příliš pomalé: nulová kapacita dané dimenze, bez trestného bodu */
  | { t: "slow"; reason: string; dim: Dim }
  /** neplatný důkaz: nulová dimenze + trestný bod */
  | { t: "cheat"; reason: string; dim: Dim };

const DIM_COL: Record<Dim, string> = { cpu: "m_cpu_hps", ram: "m_ram_mb", storage: "m_storage_gb", bw: "m_bw_mbps" };

function slotsOf(ctx: RegCtx): Slot[] {
  const slots: Slot[] = [...ctx.cfg.challenges.enabled];
  if (ctx.cfg.challenges.witness.enabled) slots.push("witness");
  return slots;
}

function intervalOf(ctx: RegCtx, slot: Slot): number {
  const c = ctx.cfg.challenges;
  switch (slot) {
    case "cpu":
      return c.cpu.intervalS;
    case "space_ram":
      return c.space.ram.openIntervalS;
    case "space_storage":
      return c.space.storage.openIntervalS;
    case "bandwidth":
      return c.bandwidth.intervalS;
    case "witness":
      return c.witness.intervalS;
  }
}

function jittered(ctx: RegCtx, base: number): number {
  const j = ctx.cfg.challenges.jitter;
  return Math.max(1, Math.round(base * (1 - j + 2 * j * randomFloat())));
}

/** Založí plán výzev nodu (první výzvy hned). */
export function ensureSchedule(ctx: RegCtx, nodeId: string): void {
  const now = ctx.nowS();
  for (const slot of slotsOf(ctx)) {
    run(ctx.sql, "INSERT OR IGNORE INTO schedule (node_id, kind, next_due) VALUES (?, ?, ?)", nodeId, slot, now);
  }
}

function send(ctx: RegCtx, nodeId: string, msg: unknown): boolean {
  const ws = ctx.sockets(nodeId)[0];
  if (!ws) return false;
  try {
    ws.send(JSON.stringify(msg));
    return true;
  } catch {
    return false;
  }
}

function inFlight(ctx: RegCtx, nodeId: string): boolean {
  return one<{ c: number }>(ctx.sql, "SELECT COUNT(*) AS c FROM challenges WHERE node_id = ?", nodeId)!.c > 0;
}

/** Spustí nejdříve splatnou výzvu nodu, pokud žádná neběží. */
export function startNext(ctx: RegCtx, node: NodeRow): void {
  if (node.status !== "active" || inFlight(ctx, node.node_id)) return;
  if (ctx.sockets(node.node_id).length === 0) return;
  const now = ctx.nowS();
  const due = all<{ kind: string }>(
    ctx.sql,
    "SELECT kind FROM schedule WHERE node_id = ? AND next_due <= ? ORDER BY next_due ASC",
    node.node_id,
    now,
  );
  for (const d of due) {
    if (startSlot(ctx, node, d.kind as Slot)) return;
  }
}

function startSlot(ctx: RegCtx, node: NodeRow, slot: Slot): boolean {
  let kind: WireKind;
  switch (slot) {
    case "cpu":
      kind = "cpu";
      break;
    case "bandwidth":
      kind = "bandwidth";
      break;
    case "witness":
      if (!witnessFeasible(ctx, node)) {
        run(ctx.sql, "UPDATE schedule SET next_due = ? WHERE node_id = ? AND kind = 'witness'", ctx.nowS() + 60, node.node_id);
        return false;
      }
      kind = "witness";
      break;
    case "space_ram":
    case "space_storage": {
      const tier = slot === "space_ram" ? "ram" : "storage";
      kind = needsCommit(ctx, node, tier) ? "space_commit" : "space_open";
      break;
    }
  }
  const id = b64uEncode(randomBytes(12));
  const nowMs = ctx.nowMs();
  run(
    ctx.sql,
    "INSERT INTO challenges (id, node_id, kind, state, params, created_at, deadline_at, data) VALUES (?, ?, ?, 'ping', '{}', ?, ?, ?)",
    id,
    node.node_id,
    kind,
    nowMs,
    nowMs + ctx.cfg.challenges.pingTimeoutMs,
    JSON.stringify({ slot }),
  );
  if (!send(ctx, node.node_id, { t: "ping", id })) {
    run(ctx.sql, "DELETE FROM challenges WHERE id = ?", id);
    return false;
  }
  return true;
}

function tierOf(slot: string): "ram" | "storage" {
  return slot === "space_ram" ? "ram" : "storage";
}

function tierLeaves(ctx: RegCtx, node: NodeRow, tier: "ram" | "storage"): number {
  const claimed = parseClaimed(node);
  const t = ctx.cfg.challenges.space[tier];
  const bytes = tier === "ram" ? claimed.ram_mb * 1024 * 1024 : claimed.storage_gb * 1024 ** 3;
  return leavesFor(bytes, t.minLeaves, t.maxLeaves);
}

function needsCommit(ctx: RegCtx, node: NodeRow, tier: "ram" | "storage"): boolean {
  const c = one<{ leaves: number; committed_at: number }>(
    ctx.sql,
    "SELECT leaves, committed_at FROM space WHERE node_id = ? AND tier = ?",
    node.node_id,
    tier,
  );
  if (!c) return true;
  if (c.leaves !== tierLeaves(ctx, node, tier)) return true;
  return ctx.nowS() - c.committed_at > ctx.cfg.challenges.space[tier].commitMaxAgeS;
}

export function onPong(ctx: RegCtx, nodeId: string, id: unknown): Promise<void> {
  if (typeof id !== "string") return Promise.resolve();
  const row = one<ChallengeRow & Record<string, SqlStorageValue>>(
    ctx.sql,
    "SELECT * FROM challenges WHERE id = ? AND node_id = ? AND state = 'ping'",
    id,
    nodeId,
  );
  const node = getNode(ctx.sql, nodeId);
  if (!row || !node) return Promise.resolve();
  return dispatch(ctx, node, row, Math.max(0, ctx.nowMs() - row.created_at));
}

function setSent(ctx: RegCtx, row: ChallengeRow, params: unknown, data: unknown, deadlineMs: number, rtt: number): number {
  const sentAt = ctx.nowMs();
  run(
    ctx.sql,
    "UPDATE challenges SET state = 'sent', params = ?, data = ?, sent_at = ?, deadline_at = ?, rtt_ms = ? WHERE id = ?",
    JSON.stringify(params),
    JSON.stringify(data),
    sentAt,
    sentAt + deadlineMs,
    rtt,
    row.id,
  );
  return sentAt;
}

async function dispatch(ctx: RegCtx, node: NodeRow, row: ChallengeRow, rtt: number): Promise<void> {
  const cfg = ctx.cfg.challenges;
  const slot = (JSON.parse(row.data ?? "{}") as { slot: Slot }).slot;
  switch (row.kind) {
    case "cpu": {
      const lanes = Math.max(1, Math.min(parseClaimed(node).cpu_cores, cfg.cpu.maxLanes));
      const seed = b64uEncode(randomBytes(32));
      const params = { kind: "cpu", seed, iters: cfg.cpu.iters, every: cfg.cpu.every, lanes, deadline_ms: cfg.cpu.timeoutMs };
      setSent(ctx, row, params, { slot, seed, lanes }, cfg.cpu.timeoutMs, rtt);
      send(ctx, node.node_id, { t: "challenge", id: row.id, ...params });
      return;
    }
    case "space_commit": {
      const tier = tierOf(slot);
      const leaves = tierLeaves(ctx, node, tier);
      const seed = b64uEncode(randomBytes(32));
      const params = { kind: "space_commit", tier, seed, leaves, deadline_ms: cfg.space.commitTimeoutMs };
      setSent(ctx, row, params, { slot, tier, seed, leaves }, cfg.space.commitTimeoutMs, rtt);
      send(ctx, node.node_id, { t: "challenge", id: row.id, ...params });
      return;
    }
    case "space_open":
      sendOpen(ctx, node, row, slot, rtt);
      return;
    case "bandwidth": {
      const idx = Math.floor(randomFloat() * cfg.bandwidth.poolSize);
      let blob: { key: string; sha256: string };
      try {
        blob = await ensureBlob(ctx.env.BW_BLOBS, cfg.bandwidth.bytes, idx);
      } catch (e) {
        log("blob_unavailable", { error: String(e) });
        run(ctx.sql, "DELETE FROM challenges WHERE id = ?", row.id);
        run(ctx.sql, "UPDATE schedule SET next_due = ? WHERE node_id = ? AND kind = ?", ctx.nowS() + 60, node.node_id, slot);
        return;
      }
      const hmacKey = b64uDecodeLen(ctx.env.CHALLENGE_HMAC_KEY, 32);
      if (!hmacKey) throw new Error("CHALLENGE_HMAC_KEY must be 32 bytes in base64url");
      const exp = ctx.nowS() + cfg.bandwidth.tokenTtlS;
      const dl = await signBwToken(hmacKey, "dl", row.id, exp);
      const ul = await signBwToken(hmacKey, "ul", row.id, exp);
      const params = {
        kind: "bandwidth",
        bytes: cfg.bandwidth.bytes,
        dl_url: `https://${ctx.env.WORKER_HOST}/challenge/blob/${dl}`,
        ul_url: `https://${ctx.env.WORKER_HOST}/challenge/upload/${ul}`,
        deadline_ms: cfg.bandwidth.timeoutMs,
      };
      setSent(ctx, row, params, { slot, key: blob.key, sha: blob.sha256, bytes: cfg.bandwidth.bytes, dl: null, ul: null, resp: null }, cfg.bandwidth.timeoutMs, rtt);
      send(ctx, node.node_id, { t: "challenge", id: row.id, ...params });
      return;
    }
    case "witness":
      dispatchWitness(ctx, node, row, rtt);
      return;
  }
}

function sendOpen(ctx: RegCtx, node: NodeRow, row: ChallengeRow, slot: Slot, rtt: number): void {
  const cfg = ctx.cfg.challenges.space;
  const tier = tierOf(slot);
  const commit = one<{ leaves: number }>(ctx.sql, "SELECT leaves FROM space WHERE node_id = ? AND tier = ?", node.node_id, tier);
  if (!commit) {
    // commitment zmizel (zrušen po chybě): přepnout na commit při příštím tiku
    run(ctx.sql, "DELETE FROM challenges WHERE id = ?", row.id);
    return;
  }
  const indices = pickIndices(commit.leaves, cfg.openCount);
  const params = { kind: "space_open", tier, indices, deadline_ms: cfg[tier].deadlineMs };
  setSent(ctx, row, params, { slot, tier, indices }, cfg[tier].deadlineMs + 30_000, rtt);
  send(ctx, node.node_id, { t: "challenge", id: row.id, ...params });
}

/* ---------------------------------------------------------------- odpovědi */

export async function onResponse(ctx: RegCtx, nodeId: string, msg: Record<string, unknown>): Promise<void> {
  const id = msg.id;
  if (typeof id !== "string") return;
  if (id.includes(".")) return onWitnessResponse(ctx, nodeId, id, msg);

  const row = one<ChallengeRow & Record<string, SqlStorageValue>>(
    ctx.sql,
    "SELECT * FROM challenges WHERE id = ? AND node_id = ? AND state = 'sent'",
    id,
    nodeId,
  );
  const node = getNode(ctx.sql, nodeId);
  if (!row || !node || row.sent_at === null) return;
  const data = JSON.parse(row.data ?? "{}") as Record<string, unknown> & { slot: Slot };
  const netMs = Math.max(1, ctx.nowMs() - row.sent_at - (row.rtt_ms ?? 0));

  if (typeof msg.error === "string") {
    if (row.kind === "space_open") run(ctx.sql, "DELETE FROM space WHERE node_id = ? AND tier = ?", nodeId, data.tier as string);
    await finish(ctx, node, row, { t: "soft", reason: `node error: ${msg.error.slice(0, 100)}` });
    return;
  }

  switch (row.kind) {
    case "cpu": {
      const p = JSON.parse(row.params) as { seed: string; iters: number; every: number; lanes: number };
      const v = await verifyCpu(p.seed, p.lanes, p.iters, p.every, ctx.cfg.challenges.cpu.segments, msg.checkpoints);
      if (!v.ok) return void (await finish(ctx, node, row, { t: "cheat", reason: v.reason, dim: "cpu" }));
      const hps = Math.min((p.lanes * p.iters) / (netMs / 1000), ctx.cfg.challenges.cpu.maxHps);
      return void (await finish(ctx, node, row, { t: "ok", apply: { m_cpu_hps: hps } }));
    }
    case "space_commit": {
      const root = typeof msg.root === "string" ? b64uDecodeLen(msg.root, 32) : null;
      if (!root) return void (await finish(ctx, node, row, { t: "cheat", reason: "malformed root", dim: data.tier === "ram" ? "ram" : "storage" }));
      run(
        ctx.sql,
        "INSERT OR REPLACE INTO space (node_id, tier, seed, root, leaves, committed_at) VALUES (?, ?, ?, ?, ?, ?)",
        nodeId,
        data.tier as string,
        data.seed as string,
        msg.root as string,
        data.leaves as number,
        ctx.nowS(),
      );
      // bezprostředně navazuje otevření; samotný commit se do statistik nepočítá
      const next = { ...row, id: row.id, kind: "space_open" as const, state: "ping" as const };
      run(ctx.sql, "UPDATE challenges SET kind = 'space_open', state = 'ping' WHERE id = ?", row.id);
      sendOpen(ctx, node, next, data.slot, row.rtt_ms ?? 0);
      return;
    }
    case "space_open": {
      const tier = data.tier as "ram" | "storage";
      const commit = one<{ seed: string; root: string; leaves: number }>(
        ctx.sql,
        "SELECT seed, root, leaves FROM space WHERE node_id = ? AND tier = ?",
        nodeId,
        tier,
      );
      const dim: Dim = tier === "ram" ? "ram" : "storage";
      if (!commit) return void (await finish(ctx, node, row, { t: "soft", reason: "commitment missing" }));
      const seed = b64uDecodeLen(commit.seed, 32);
      const root = b64uDecodeLen(commit.root, 32);
      if (!seed || !root) return void (await finish(ctx, node, row, { t: "soft", reason: "corrupt commitment" }));
      const v = verifyOpenings(seed, root, commit.leaves, data.indices as number[], msg.openings);
      if (!v.ok) {
        run(ctx.sql, "DELETE FROM space WHERE node_id = ? AND tier = ?", nodeId, tier);
        return void (await finish(ctx, node, row, { t: "cheat", reason: v.reason, dim }));
      }
      const deadline = ctx.cfg.challenges.space[tier].deadlineMs;
      if (netMs > deadline) {
        return void (await finish(ctx, node, row, { t: "slow", reason: `opening took ${Math.round(netMs)} ms > ${deadline} ms`, dim }));
      }
      const bytes = commit.leaves * LEAF_BYTES;
      const apply: Record<string, number> = tier === "ram" ? { m_ram_mb: bytes / 1024 ** 2 } : { m_storage_gb: bytes / 1024 ** 3 };
      return void (await finish(ctx, node, row, { t: "ok", apply }));
    }
    case "bandwidth": {
      if (typeof msg.sha256 !== "string") return void (await finish(ctx, node, row, { t: "cheat", reason: "missing hash", dim: "bw" }));
      data.resp = msg.sha256;
      run(ctx.sql, "UPDATE challenges SET data = ? WHERE id = ?", JSON.stringify(data), row.id);
      await tryFinishBandwidth(ctx, row.id);
      return;
    }
    default:
      return;
  }
}

/* --------------------------------------------------------------- bandwidth */

export interface BwRecord {
  bytes: number;
  ms: number;
  sha?: string;
}

/** Autorizace koncových bodů blob/upload: výzva musí být odeslaná a nesmí mít záznam daného směru. */
export function bwAuthorize(ctx: RegCtx, id: string, dir: "dl" | "ul"): { key: string; bytes: number } | null {
  const row = one<ChallengeRow & Record<string, SqlStorageValue>>(
    ctx.sql,
    "SELECT * FROM challenges WHERE id = ? AND kind = 'bandwidth' AND state = 'sent'",
    id,
  );
  if (!row) return null;
  const data = JSON.parse(row.data ?? "{}") as { key: string; bytes: number; dl: unknown; ul: unknown; claimed?: Record<string, boolean> };
  if (data[dir] !== null) return null;
  const claimed = data.claimed ?? {};
  if (claimed[dir]) return null; // jedno použití
  claimed[dir] = true;
  data.claimed = claimed;
  run(ctx.sql, "UPDATE challenges SET data = ? WHERE id = ?", JSON.stringify(data), id);
  return { key: data.key, bytes: data.bytes };
}

export async function bwRecord(ctx: RegCtx, id: string, dir: "dl" | "ul", rec: BwRecord): Promise<void> {
  const row = one<ChallengeRow & Record<string, SqlStorageValue>>(
    ctx.sql,
    "SELECT * FROM challenges WHERE id = ? AND kind = 'bandwidth' AND state = 'sent'",
    id,
  );
  if (!row) return;
  const data = JSON.parse(row.data ?? "{}") as Record<string, unknown>;
  data[dir] = rec;
  run(ctx.sql, "UPDATE challenges SET data = ? WHERE id = ?", JSON.stringify(data), id);
  await tryFinishBandwidth(ctx, id);
}

async function tryFinishBandwidth(ctx: RegCtx, id: string): Promise<void> {
  const row = one<ChallengeRow & Record<string, SqlStorageValue>>(ctx.sql, "SELECT * FROM challenges WHERE id = ?", id);
  if (!row) return;
  const node = getNode(ctx.sql, row.node_id);
  if (!node) return;
  const d = JSON.parse(row.data ?? "{}") as {
    sha: string;
    bytes: number;
    dl: BwRecord | null;
    ul: BwRecord | null;
    resp: string | null;
  };
  if (!d.dl || !d.ul || d.resp === null) return; // čeká se na zbylé části
  if (d.resp !== d.sha) return void (await finish(ctx, node, row, { t: "cheat", reason: "wrong blob hash", dim: "bw" }));
  if (d.dl.bytes !== d.bytes || d.ul.bytes !== d.bytes) {
    return void (await finish(ctx, node, row, { t: "slow", reason: "incomplete transfer", dim: "bw" }));
  }
  if (d.ul.sha !== d.sha) return void (await finish(ctx, node, row, { t: "cheat", reason: "uploaded data differs", dim: "bw" }));
  const mbps = (ms: number) => (d.bytes * 8) / 1e6 / (Math.max(1, ms) / 1000);
  const r2 = Math.min(mbps(d.dl.ms), mbps(d.ul.ms));
  await finish(ctx, node, row, { t: "ok", apply: { m_bw_r2: r2 } });
}

/* ----------------------------------------------------------------- witness */

interface WitnessData {
  slot: "witness";
  bytes: number;
  witnesses: string[];
  reports: Record<string, number>;
}

function liveConnectedNodes(ctx: RegCtx, exceptId: string): NodeRow[] {
  const now = ctx.nowS();
  return all<NodeRow & Record<string, SqlStorageValue>>(
    ctx.sql,
    "SELECT * FROM nodes WHERE status = 'active' AND last_hb_at >= ? AND node_id != ?",
    now - ctx.cfg.heartbeatTimeoutS,
    exceptId,
  ).filter((n) => ctx.sockets(n.node_id).length > 0);
}

function witnessFeasible(ctx: RegCtx, target: NodeRow): boolean {
  return pickWitnesses(ctx, target).length >= Math.min(2, ctx.cfg.challenges.witness.count);
}

function pickWitnesses(ctx: RegCtx, target: NodeRow): NodeRow[] {
  const w = ctx.cfg.challenges.witness;
  const candidates = liveConnectedNodes(ctx, target.node_id).filter((n) => n.asn !== null && n.asn !== target.asn && !inFlight(ctx, n.node_id));
  const asns = new Set(candidates.map((n) => n.asn));
  asns.add(target.asn);
  if (asns.size < w.minAsns) return [];
  const chosen: NodeRow[] = [];
  const seen = new Set<number | null>();
  for (const c of candidates.sort(() => randomFloat() - 0.5)) {
    if (seen.has(c.asn)) continue;
    seen.add(c.asn);
    chosen.push(c);
    if (chosen.length >= w.count) break;
  }
  return chosen;
}

function dispatchWitness(ctx: RegCtx, target: NodeRow, row: ChallengeRow, rtt: number): void {
  const w = ctx.cfg.challenges.witness;
  const witnesses = pickWitnesses(ctx, target);
  if (witnesses.length < Math.min(2, w.count)) {
    run(ctx.sql, "DELETE FROM challenges WHERE id = ?", row.id);
    return;
  }
  const data: WitnessData = { slot: "witness", bytes: w.bytes, witnesses: witnesses.map((n) => n.node_id), reports: {} };
  setSent(ctx, row, { kind: "witness" }, data, w.timeoutMs, rtt);
  for (const wn of witnesses) {
    send(ctx, wn.node_id, {
      t: "challenge",
      id: `${row.id}.${wn.node_id.slice(0, 8)}`,
      kind: "witness",
      target_id: target.node_id,
      target_host: target.hostname,
      bytes: w.bytes,
      deadline_ms: w.timeoutMs,
    });
  }
}

async function onWitnessResponse(ctx: RegCtx, senderId: string, id: string, msg: Record<string, unknown>): Promise<void> {
  const [parentId] = id.split(".");
  const row = one<ChallengeRow & Record<string, SqlStorageValue>>(
    ctx.sql,
    "SELECT * FROM challenges WHERE id = ? AND kind = 'witness' AND state = 'sent'",
    parentId!,
  );
  if (!row) return;
  const data = JSON.parse(row.data ?? "{}") as WitnessData;
  if (!data.witnesses.includes(senderId) || senderId in data.reports) return;
  const witness = getNode(ctx.sql, senderId);
  const report = msg.report as { target_id?: unknown; witness_id?: unknown; bytes?: unknown; duration_ms?: unknown; ts?: unknown } | undefined;
  const sig = typeof msg.sig === "string" ? b64uDecode(msg.sig) : null;
  if (!witness || !report || !sig) return;
  if (
    report.target_id !== row.node_id ||
    report.witness_id !== senderId ||
    report.bytes !== data.bytes ||
    typeof report.duration_ms !== "number" ||
    !Number.isSafeInteger(report.duration_ms) ||
    report.duration_ms < 1 ||
    typeof report.ts !== "number"
  ) {
    return;
  }
  const pk = b64uDecodeLen(witness.pubkey, 32);
  const ok = pk && (await verifySig(pk, DOMAIN.report, canonicalBytes(report as Record<string, string | number>), sig));
  if (!ok) return;
  data.reports[senderId] = (data.bytes * 8) / 1e6 / (report.duration_ms / 1000);
  run(ctx.sql, "UPDATE challenges SET data = ? WHERE id = ?", JSON.stringify(data), row.id);
  if (Object.keys(data.reports).length >= data.witnesses.length) await finishWitness(ctx, row.id);
}

async function finishWitness(ctx: RegCtx, id: string): Promise<void> {
  const row = one<ChallengeRow & Record<string, SqlStorageValue>>(ctx.sql, "SELECT * FROM challenges WHERE id = ?", id);
  if (!row) return;
  const node = getNode(ctx.sql, row.node_id);
  if (!node) return;
  const data = JSON.parse(row.data ?? "{}") as WitnessData;
  const values = Object.values(data.reports).sort((a, b) => a - b);
  if (values.length < 2) return void (await finish(ctx, node, row, { t: "soft", reason: "not enough witness reports" }));
  const median = values.length % 2 ? values[(values.length - 1) / 2]! : (values[values.length / 2 - 1]! + values[values.length / 2]!) / 2;
  await finish(ctx, node, row, { t: "ok", apply: { m_bw_wit: median } });
}

/* ------------------------------------------------------------------ výsledek */

const ALLOWED_COLUMNS = new Set(["m_cpu_hps", "m_ram_mb", "m_storage_gb", "m_bw_r2", "m_bw_wit"]);

async function finish(ctx: RegCtx, node: NodeRow, row: ChallengeRow, outcome: Outcome): Promise<void> {
  const data = JSON.parse(row.data ?? "{}") as { slot?: Slot };
  const slot = data.slot ?? "cpu";
  const alpha = ctx.cfg.score.healthAlpha;
  const ok = outcome.t === "ok";
  const health = (1 - alpha) * node.health + alpha * (ok ? 1 : 0);

  // idempotentní: odpověď a vypršení (nebo dva souběžné tiky) nesmí výzvu započítat dvakrát
  if (ctx.sql.exec("DELETE FROM challenges WHERE id = ?", row.id).rowsWritten === 0) return;
  // trestné body se počítají po slotech: úspěch v jedné výzvě nesmaže podvod v jiné
  if (ok) run(ctx.sql, "UPDATE schedule SET strikes = 0 WHERE node_id = ? AND kind = ?", node.node_id, slot);
  else if (outcome.t === "cheat") run(ctx.sql, "UPDATE schedule SET strikes = strikes + 1 WHERE node_id = ? AND kind = ?", node.node_id, slot);
  const slotStrikes = one<{ strikes: number }>(ctx.sql, "SELECT strikes FROM schedule WHERE node_id = ? AND kind = ?", node.node_id, slot)?.strikes ?? 0;
  const totalStrikes = one<{ s: number }>(ctx.sql, "SELECT COALESCE(SUM(strikes), 0) AS s FROM schedule WHERE node_id = ?", node.node_id)!.s;
  run(
    ctx.sql,
    "UPDATE nodes SET health = ?, ch_total = ch_total + 1, ch_ok = ch_ok + ?, strikes = ?, updated_at = ? WHERE node_id = ?",
    health,
    ok ? 1 : 0,
    totalStrikes,
    ctx.nowS(),
    node.node_id,
  );
  if (ok && row.rtt_ms !== null) {
    run(ctx.sql, "UPDATE nodes SET rtt_ms = COALESCE(0.7 * rtt_ms + 0.3 * ?, ?) WHERE node_id = ?", row.rtt_ms, row.rtt_ms, node.node_id);
  }

  if (outcome.t === "ok") {
    for (const [col, value] of Object.entries(outcome.apply ?? {})) {
      if (!ALLOWED_COLUMNS.has(col)) throw new Error(`unexpected column ${col}`);
      run(ctx.sql, `UPDATE nodes SET ${col} = ? WHERE node_id = ?`, value, node.node_id);
    }
    combineBandwidth(ctx, node.node_id);
  } else if (outcome.t === "slow" || outcome.t === "cheat") {
    if (outcome.dim === "bw") {
      run(ctx.sql, "UPDATE nodes SET m_bw_r2 = 0, m_bw_mbps = 0 WHERE node_id = ?", node.node_id);
    } else {
      run(ctx.sql, `UPDATE nodes SET ${DIM_COL[outcome.dim]} = 0 WHERE node_id = ?`, node.node_id);
    }
  }
  if (outcome.t !== "ok") log("challenge_failed", { node_id: node.node_id, slot, outcome: outcome.t, reason: outcome.reason });

  const next = ok ? jittered(ctx, intervalOf(ctx, slot)) : Math.min(jittered(ctx, intervalOf(ctx, slot)), ctx.cfg.challenges.retryDelayS);
  run(ctx.sql, "UPDATE schedule SET next_due = ? WHERE node_id = ? AND kind = ?", ctx.nowS() + next, node.node_id, slot);

  const fresh = getNode(ctx.sql, node.node_id);
  if (!fresh) return;
  if (outcome.t === "cheat" && slotStrikes >= ctx.cfg.score.banStrikes) {
    await banNode(ctx, node.node_id, `repeated invalid challenge proofs: ${outcome.reason}`);
    return;
  }
  startNext(ctx, fresh);
}

/** m_bw_mbps = min(R2 měření, medián svědků); chybí-li jedno, platí druhé. */
function combineBandwidth(ctx: RegCtx, nodeId: string): void {
  const n = getNode(ctx.sql, nodeId);
  if (!n) return;
  const parts = [n.m_bw_r2, n.m_bw_wit].filter((v): v is number => v !== null);
  if (parts.length === 0) return;
  run(ctx.sql, "UPDATE nodes SET m_bw_mbps = ? WHERE node_id = ?", Math.min(...parts), nodeId);
}

/** Tik: propadlé výzvy → soft neúspěch (ping bez pongu se jen zahodí), pak start splatných. */
export async function challengeTick(ctx: RegCtx): Promise<void> {
  const nowMs = ctx.nowMs();
  const expired = all<ChallengeRow & Record<string, SqlStorageValue>>(ctx.sql, "SELECT * FROM challenges WHERE deadline_at < ?", nowMs);
  for (const row of expired) {
    const node = getNode(ctx.sql, row.node_id);
    if (!node || node.status !== "active") {
      run(ctx.sql, "DELETE FROM challenges WHERE id = ?", row.id);
      continue;
    }
    if (row.state === "ping") {
      // node na ping neodpověděl: zkusit později, ne při každém tiku
      const slot = (JSON.parse(row.data ?? "{}") as { slot?: Slot }).slot;
      run(ctx.sql, "DELETE FROM challenges WHERE id = ?", row.id);
      if (slot) {
        run(ctx.sql, "UPDATE schedule SET next_due = ? WHERE node_id = ? AND kind = ?", ctx.nowS() + ctx.cfg.challenges.retryDelayS, row.node_id, slot);
      }
      continue;
    }
    if (row.kind === "witness") {
      const d = JSON.parse(row.data ?? "{}") as WitnessData;
      if (Object.keys(d.reports).length >= 2) {
        await finishWitness(ctx, row.id);
        continue;
      }
    }
    await finish(ctx, node, row, { t: "soft", reason: "timeout" });
  }

  const nodes = all<NodeRow & Record<string, SqlStorageValue>>(
    ctx.sql,
    "SELECT * FROM nodes WHERE status = 'active' AND last_hb_at >= ?",
    ctx.nowS() - ctx.cfg.heartbeatTimeoutS,
  );
  for (const n of nodes) startNext(ctx, n);
}

export function dropChallengesOf(ctx: RegCtx, nodeId: string): void {
  run(ctx.sql, "DELETE FROM challenges WHERE node_id = ?", nodeId);
}

