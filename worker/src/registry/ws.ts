import { canonicalBytes, type Json } from "../canonical";
import { DOMAIN, verifySig } from "../crypto";
import { b64uDecodeLen } from "../encoding";
import { ensureSchedule, onPong, onResponse, startNext } from "../challenge/scheduler";
import { ensureDirectory, epochOf } from "./directory";
import { getNode, run, type Load, type NodeRow } from "./store";
import { log, type RegCtx } from "./context";

export interface CfInfo {
  country?: string;
  continent?: string;
  colo?: string;
  latitude?: string | number;
  longitude?: string | number;
}

function num(v: string | number | undefined): number | null {
  if (v === undefined) return null;
  const n = typeof v === "number" ? v : Number(v);
  return Number.isFinite(n) ? n : null;
}

/** Zaznamená nové spojení: poloha podle Cloudflare, plán výzev, uvítání. */
export function onConnect(ctx: RegCtx, node: NodeRow, ws: WebSocket, cf: CfInfo): void {
  const now = ctx.nowS();
  run(
    ctx.sql,
    "UPDATE nodes SET last_hb_at = ?, hb_seq = -1, country = ?, continent = ?, colo = ?, lat = ?, lon = ?, updated_at = ? WHERE node_id = ?",
    now,
    cf.country ?? null,
    cf.continent ?? null,
    cf.colo ?? null,
    num(cf.latitude),
    num(cf.longitude),
    now,
    node.node_id,
  );
  ensureSchedule(ctx, node.node_id);
  ws.send(
    JSON.stringify({
      t: "welcome",
      epoch: epochOf(now, ctx.cfg.epochSeconds),
      heartbeat_interval_s: ctx.cfg.heartbeatIntervalS,
    }),
  );
}

function isInt(v: unknown, min: number, max: number): v is number {
  return typeof v === "number" && Number.isSafeInteger(v) && v >= min && v <= max;
}

function parseLoad(v: unknown): Load | null {
  if (typeof v !== "object" || v === null) return null;
  const l = v as Record<string, unknown>;
  if (!isInt(l.conns, 0, 1e9) || !isInt(l.max_conns, 0, 1e9) || !isInt(l.cpu_pct, 0, 100) || !isInt(l.mem_pct, 0, 100)) return null;
  return { conns: l.conns, max_conns: l.max_conns, cpu_pct: l.cpu_pct, mem_pct: l.mem_pct };
}

async function handleHeartbeat(ctx: RegCtx, ws: WebSocket, nodeId: string, msg: Record<string, unknown>): Promise<void> {
  const node = getNode(ctx.sql, nodeId);
  if (!node || node.status !== "active") return;
  const now = ctx.nowS();
  const sig = typeof msg.sig === "string" ? b64uDecodeLen(msg.sig, 64) : null;
  const load = parseLoad(msg.load);
  const { sig: _sig, ...unsigned } = msg;
  void _sig;
  if (!sig || !load || !isInt(msg.seq, 0, Number.MAX_SAFE_INTEGER) || !isInt(msg.ts, 0, Number.MAX_SAFE_INTEGER)) {
    ws.send(JSON.stringify({ t: "error", reason: "malformed heartbeat" }));
    return;
  }
  if (msg.epoch !== undefined && !isInt(msg.epoch, 0, Number.MAX_SAFE_INTEGER)) {
    ws.send(JSON.stringify({ t: "error", reason: "malformed heartbeat" }));
    return;
  }
  if (Math.abs(now - msg.ts) > ctx.cfg.requestSkewS) {
    ws.send(JSON.stringify({ t: "error", reason: "heartbeat timestamp outside allowed window" }));
    return;
  }
  const pk = b64uDecodeLen(node.pubkey, 32)!;
  if (!(await verifySig(pk, DOMAIN.heartbeat, canonicalBytes(unsigned as Json), sig))) {
    ws.send(JSON.stringify({ t: "error", reason: "invalid heartbeat signature" }));
    return;
  }
  // po awaitu načíst znovu: mezitím se mohl změnit stav i seq
  const fresh = getNode(ctx.sql, nodeId);
  if (!fresh || fresh.status !== "active") return;
  if (msg.seq <= fresh.hb_seq) {
    ws.send(JSON.stringify({ t: "error", reason: "heartbeat sequence must increase" }));
    return;
  }
  const gap = fresh.last_hb_at === null ? 0 : Math.max(0, now - fresh.last_hb_at);
  const credited = Math.min(gap, 2 * ctx.cfg.heartbeatIntervalS);
  run(
    ctx.sql,
    "UPDATE nodes SET last_hb_at = ?, hb_seq = ?, load = ?, uptime_s = uptime_s + ?, updated_at = ? WHERE node_id = ?",
    now,
    msg.seq,
    JSON.stringify(load),
    credited,
    now,
    nodeId,
  );
  const epoch = epochOf(now, ctx.cfg.epochSeconds);
  ws.send(JSON.stringify({ t: "ack", seq: msg.seq, epoch }));
  if (typeof msg.epoch === "number" && msg.epoch < epoch) {
    const { directory } = await ensureDirectory(ctx);
    ws.send(JSON.stringify({ t: "directory", directory }));
  }
  startNext(ctx, fresh);
}

export async function handleMessage(ctx: RegCtx, ws: WebSocket, nodeId: string, raw: string | ArrayBuffer): Promise<void> {
  if (typeof raw !== "string") {
    ws.close(1003, "text frames only");
    return;
  }
  if (raw.length > ctx.cfg.maxWsMessageBytes) {
    ws.close(1009, "message too large");
    return;
  }
  let msg: Record<string, unknown>;
  try {
    const v = JSON.parse(raw);
    if (typeof v !== "object" || v === null || Array.isArray(v)) throw new Error("not an object");
    msg = v as Record<string, unknown>;
  } catch {
    ws.send(JSON.stringify({ t: "error", reason: "invalid JSON" }));
    return;
  }
  try {
    switch (msg.t) {
      case "hb":
        await handleHeartbeat(ctx, ws, nodeId, msg);
        break;
      case "pong":
        await onPong(ctx, nodeId, msg.id);
        break;
      case "challenge_response":
        await onResponse(ctx, nodeId, msg);
        break;
      default:
        ws.send(JSON.stringify({ t: "error", reason: "unknown message type" }));
    }
  } catch (e) {
    log("ws_handler_error", { node_id: nodeId, t: String(msg.t), error: String(e) });
  }
}
