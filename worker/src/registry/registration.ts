import { genesisSet } from "../config";
import { DOMAIN, hostnameOf, nodeIdOf, verifySig } from "../crypto";
import { b64uDecodeLen, b64uEncode, concat, randomBytes, timingSafeEqual, utf8 } from "../encoding";
import { HttpError } from "../errors";
import { hashcashValid, requiredBits } from "../hashcash";
import { timeoutSignal } from "../http";
import { TUNNEL_NAME_PREFIX } from "../cloudflare";
import { all, getNode, one, run, type Claimed, type NodeRow } from "./store";
import { log, type RegCtx } from "./context";

export interface BeginInput {
  pubkey: string;
  claimed: Claimed;
  asn: number | null;
}

export interface BeginResult {
  node_id: string;
  hostname: string;
  challenge_nonce: string;
  expires_at: number;
  hashcash_bits: number;
}

export interface ConfirmInput {
  pubkey: string;
  nonce: string;
  signature: string;
  counter: number | null;
  asn: number | null;
  ipHash: string | null;
}

export interface ConfirmResult {
  node_id: string;
  hostname: string;
  status: "pending" | "active";
  tunnel_token: string;
  proof_path: string;
  activate_before: number;
}

export const PROOF_PATH = "/.well-known/melo-proof";

function decodePubkey(pubkey: string): Uint8Array {
  const pk = b64uDecodeLen(pubkey, 32);
  if (!pk) throw new HttpError("bad_request", "pubkey must be 32 bytes in base64url");
  return pk;
}

function isGenesis(ctx: RegCtx, nodeId: string): boolean {
  return genesisSet(ctx.env).has(nodeId);
}

export function registerBegin(ctx: RegCtx, input: BeginInput): BeginResult {
  const { cfg } = ctx;
  const now = ctx.nowS();
  const pk = decodePubkey(input.pubkey);
  const nodeId = nodeIdOf(pk);
  const hostname = hostnameOf(nodeId, ctx.env.ZONE_DOMAIN);
  const genesis = isGenesis(ctx, nodeId);

  if (cfg.genesisOnly && !genesis) throw new HttpError("forbidden", "node is not on the genesis allowlist");

  const existing = getNode(ctx.sql, nodeId);
  if (existing?.status === "banned") throw new HttpError("forbidden", "node is banned");

  const clash = one<{ node_id: string }>(ctx.sql, "SELECT node_id FROM nodes WHERE hostname = ? AND pubkey != ?", hostname, input.pubkey);
  if (clash) throw new HttpError("conflict", "hostname collision, use a different key");

  if (input.asn !== null && !genesis) {
    const open = one<{ c: number }>(
      ctx.sql,
      "SELECT COUNT(*) AS c FROM reg_challenges WHERE asn = ? AND expires_at > ?",
      input.asn,
      now,
    );
    if ((open?.c ?? 0) >= cfg.maxOpenChallengesPerAsn) {
      throw new HttpError("rate_limited", "too many open registrations from this network", 60);
    }
    if (cfg.limits.enabled) assertAsnNotPenalized(ctx, input.asn);
  }

  let bits = 0;
  if (!genesis && cfg.hashcash.enabled) {
    const recent = one<{ c: number }>(
      ctx.sql,
      "SELECT COUNT(*) AS c FROM reg_log WHERE at > ?",
      now - cfg.hashcash.surgeWindowS,
    );
    bits = requiredBits(cfg.hashcash, recent?.c ?? 0);
  }

  const open = one<{ c: number }>(ctx.sql, "SELECT COUNT(*) AS c FROM reg_challenges WHERE expires_at > ?", now);
  if (!genesis && (open?.c ?? 0) >= cfg.maxOpenChallenges) {
    throw new HttpError("rate_limited", "registration is temporarily overloaded", 60);
  }

  const nonce = b64uEncode(randomBytes(32));
  const expiresAt = now + cfg.registerChallengeTtlS;
  run(ctx.sql, "DELETE FROM reg_challenges WHERE pubkey = ?", input.pubkey);
  run(
    ctx.sql,
    "INSERT INTO reg_challenges (nonce, pubkey, hostname, claimed, hashcash_bits, asn, expires_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
    nonce,
    input.pubkey,
    hostname,
    JSON.stringify(input.claimed),
    bits,
    input.asn,
    expiresAt,
  );
  return { node_id: nodeId, hostname, challenge_nonce: nonce, expires_at: expiresAt, hashcash_bits: bits };
}

function assertAsnNotPenalized(ctx: RegCtx, asn: number): void {
  const p = one<{ count: number }>(ctx.sql, "SELECT count FROM asn_penalties WHERE asn = ?", asn);
  if ((p?.count ?? 0) >= ctx.cfg.limits.asnBanPenalties) {
    throw new HttpError("forbidden", "registrations from this network are suspended");
  }
}

function assertLimits(ctx: RegCtx, input: ConfirmInput, nodeId: string): void {
  const { limits } = ctx.cfg;
  if (!limits.enabled) return;
  if (input.asn !== null) {
    assertAsnNotPenalized(ctx, input.asn);
    const c = one<{ c: number }>(
      ctx.sql,
      "SELECT COUNT(*) AS c FROM nodes WHERE asn = ? AND node_id != ? AND status IN ('provisioning','pending','active')",
      input.asn,
      nodeId,
    );
    if ((c?.c ?? 0) >= limits.maxPerAsn) throw new HttpError("rate_limited", "too many nodes in this ASN", 3600);
  }
  if (input.ipHash !== null) {
    const c = one<{ c: number }>(
      ctx.sql,
      "SELECT COUNT(*) AS c FROM nodes WHERE ip_hash = ? AND node_id != ? AND status IN ('provisioning','pending','active')",
      input.ipHash,
      nodeId,
    );
    if ((c?.c ?? 0) >= limits.maxPerPrefix) throw new HttpError("rate_limited", "too many nodes in this network prefix", 3600);
  }
}

export async function registerConfirm(ctx: RegCtx, input: ConfirmInput): Promise<ConfirmResult> {
  const now = ctx.nowS();
  const pk = decodePubkey(input.pubkey);
  const nonce = b64uDecodeLen(input.nonce, 32);
  const sig = b64uDecodeLen(input.signature, 64);
  if (!nonce || !sig) throw new HttpError("bad_request", "malformed nonce or signature");

  const ch = one<{ nonce: string; pubkey: string; hostname: string; claimed: string; hashcash_bits: number; expires_at: number }>(
    ctx.sql,
    "SELECT * FROM reg_challenges WHERE nonce = ?",
    input.nonce,
  );
  if (!ch || ch.expires_at < now || ch.pubkey !== input.pubkey) {
    throw new HttpError("proof_failed", "unknown or expired registration challenge");
  }
  const payload = concat(nonce, utf8(ch.hostname), pk);
  if (!(await verifySig(pk, DOMAIN.register, payload, sig))) {
    throw new HttpError("proof_failed", "invalid registration signature");
  }
  if (ch.hashcash_bits > 0) {
    if (input.counter === null || !hashcashValid(nonce, pk, input.counter, ch.hashcash_bits)) {
      throw new HttpError("proof_failed", "invalid hashcash");
    }
  }
  // výzva je jednorázová; po této řádce už jiný souběžný confirm stejnou výzvu nepoužije
  const taken = ctx.sql.exec("DELETE FROM reg_challenges WHERE nonce = ?", input.nonce).rowsWritten;
  if (taken === 0) throw new HttpError("proof_failed", "registration challenge already used");

  const nodeId = nodeIdOf(pk);
  const genesis = isGenesis(ctx, nodeId);
  const existing = getNode(ctx.sql, nodeId);

  if (existing?.status === "banned") throw new HttpError("forbidden", "node is banned");
  if (existing?.status === "provisioning" && (existing.expires_at ?? 0) > now) {
    throw new HttpError("conflict", "registration already in progress", 5);
  }

  // znovu vydat token existujícímu nodu (restart), bez nových zdrojů
  if (existing && (existing.status === "pending" || existing.status === "active") && existing.tunnel_id) {
    const token = await ctx.cloudflare().getTunnelToken(existing.tunnel_id);
    const activateBefore = existing.status === "pending" ? now + ctx.cfg.pendingTtlS : 0;
    if (existing.status === "pending") {
      run(ctx.sql, "UPDATE nodes SET expires_at = ?, updated_at = ? WHERE node_id = ?", activateBefore, now, nodeId);
    }
    return {
      node_id: nodeId,
      hostname: existing.hostname,
      status: existing.status,
      tunnel_token: token,
      proof_path: PROOF_PATH,
      activate_before: activateBefore,
    };
  }

  if (!genesis) assertLimits(ctx, input, nodeId);

  // stav „provisioning“ drží místo a zabrání souběžné registraci téhož nodu
  const tunnelName = `${TUNNEL_NAME_PREFIX}${ch.hostname.split(".")[0]}`;
  if (existing) {
    run(
      ctx.sql,
      "UPDATE nodes SET status = 'provisioning', claimed = ?, asn = ?, ip_hash = ?, genesis = ?, tunnel_id = NULL, dns_record_id = NULL, tunnel_name = ?, expires_at = ?, updated_at = ? WHERE node_id = ?",
      ch.claimed,
      input.asn,
      input.ipHash,
      genesis ? 1 : 0,
      tunnelName,
      now + ctx.cfg.provisioningTimeoutS,
      now,
      nodeId,
    );
  } else {
    run(
      ctx.sql,
      "INSERT INTO nodes (node_id, pubkey, hostname, status, genesis, claimed, asn, ip_hash, tunnel_name, created_at, updated_at, expires_at) VALUES (?, ?, ?, 'provisioning', ?, ?, ?, ?, ?, ?, ?, ?)",
      nodeId,
      input.pubkey,
      ch.hostname,
      genesis ? 1 : 0,
      ch.claimed,
      input.asn,
      input.ipHash,
      tunnelName,
      now,
      now,
      now + ctx.cfg.provisioningTimeoutS,
    );
  }

  const cf = ctx.cloudflare();
  let tunnelId: string | null = null;
  let dnsId: string | null = null;
  try {
    const t = await cf.createTunnel(tunnelName);
    tunnelId = t.id;
    run(ctx.sql, "UPDATE nodes SET tunnel_id = ? WHERE node_id = ?", tunnelId, nodeId);
    await cf.configureTunnel(tunnelId, ch.hostname, ctx.env.NODE_ORIGIN);
    dnsId = await cf.createDnsCname(ch.hostname, tunnelId);
    run(ctx.sql, "UPDATE nodes SET dns_record_id = ? WHERE node_id = ?", dnsId, nodeId);

    const activateBefore = ctx.nowS() + ctx.cfg.pendingTtlS;
    run(
      ctx.sql,
      "UPDATE nodes SET status = 'pending', expires_at = ?, updated_at = ? WHERE node_id = ?",
      activateBefore,
      ctx.nowS(),
      nodeId,
    );
    run(ctx.sql, "INSERT INTO reg_log (at) VALUES (?)", now);
    log("node_provisioned", { node_id: nodeId });
    return {
      node_id: nodeId,
      hostname: ch.hostname,
      status: "pending",
      tunnel_token: t.token,
      proof_path: PROOF_PATH,
      activate_before: activateBefore,
    };
  } catch (e) {
    // kompenzace: smazat, co jsme stihli vytvořit
    let cleaned = true;
    try {
      if (dnsId) await cf.deleteDns(dnsId, ch.hostname);
      if (tunnelId) await cf.deleteTunnel(tunnelId, tunnelName);
    } catch (ce) {
      cleaned = false;
      log("provision_cleanup_failed", { node_id: nodeId, error: String(ce) });
    }
    if (cleaned) {
      if (existing) {
        run(ctx.sql, "UPDATE nodes SET status = 'retired', tunnel_id = NULL, dns_record_id = NULL, updated_at = ? WHERE node_id = ?", ctx.nowS(), nodeId);
      } else {
        run(ctx.sql, "DELETE FROM nodes WHERE node_id = ?", nodeId);
      }
    } else {
      // zdroje zůstaly: nechat řádek retired s id, úklid dožene tick
      run(ctx.sql, "UPDATE nodes SET status = 'retired', updated_at = ? WHERE node_id = ?", ctx.nowS(), nodeId);
    }
    if (e instanceof HttpError) throw e;
    log("provision_failed", { node_id: nodeId, error: String(e) });
    throw new HttpError("upstream", "failed to provision tunnel");
  }
}

export interface AuthCtx {
  nodeId: string;
  pubkey: string;
  replayKey: string;
}

/** Zapíše nonce požadavku; opakování = replay. */
export function consumeNonce(ctx: RegCtx, replayKey: string): void {
  const now = ctx.nowS();
  const r = ctx.sql.exec(
    "INSERT OR IGNORE INTO nonces (key, expires_at) VALUES (?, ?)",
    replayKey,
    now + 2 * ctx.cfg.requestSkewS + 5,
  );
  if (r.rowsWritten === 0) throw new HttpError("conflict", "request replayed");
}

export function requireKnownNode(ctx: RegCtx, auth: AuthCtx): NodeRow {
  const node = getNode(ctx.sql, auth.nodeId);
  if (!node || node.pubkey !== auth.pubkey) throw new HttpError("forbidden", "unknown node");
  return node;
}

/** Kam Worker pošle žádost o důkaz. `PROOF_ORIGIN` platí jen v mock režimu (lokální vývoj bez tunelu). */
function proofOrigin(ctx: RegCtx, hostname: string): string {
  const dev = ctx.env.CLOUDFLARE_MODE === "mock" ? ctx.env.PROOF_ORIGIN : undefined;
  return dev ? dev.replace("{hostname}", hostname).replace(/\/+$/, "") : `https://${hostname}`;
}

export async function activate(ctx: RegCtx, auth: AuthCtx): Promise<{ status: "active" }> {
  consumeNonce(ctx, auth.replayKey);
  const node = requireKnownNode(ctx, auth);
  if (node.status === "active") return { status: "active" };
  if (node.status !== "pending") throw new HttpError("conflict", `node is ${node.status}`);

  const challenge = randomBytes(32);
  const url = `${proofOrigin(ctx, node.hostname)}${PROOF_PATH}?c=${b64uEncode(challenge)}`;
  let res: Response;
  try {
    res = await ctx.fetcher()(url, {
      redirect: "manual",
      signal: timeoutSignal(5000),
      headers: { accept: "application/json" },
    });
  } catch (e) {
    throw new HttpError("upstream", `cannot reach ${node.hostname}: ${String(e)}`);
  }
  if (res.status !== 200) throw new HttpError("upstream", `${node.hostname} answered ${res.status}`);

  let body: { pubkey?: unknown; signature?: unknown };
  try {
    const text = await res.text();
    if (text.length > 1024) throw new Error("too long");
    body = JSON.parse(text);
  } catch {
    throw new HttpError("proof_failed", "invalid proof response");
  }
  const pk = decodePubkey(node.pubkey);
  const claimedPk = typeof body.pubkey === "string" ? b64uDecodeLen(body.pubkey, 32) : null;
  const sig = typeof body.signature === "string" ? b64uDecodeLen(body.signature, 64) : null;
  if (!claimedPk || !sig || !timingSafeEqual(claimedPk, pk)) throw new HttpError("proof_failed", "proof key mismatch");
  const payload = concat(challenge, utf8(node.hostname), pk);
  if (!(await verifySig(pk, DOMAIN.proof, payload, sig))) throw new HttpError("proof_failed", "invalid proof signature");

  const now = ctx.nowS();
  const changed = ctx.sql.exec(
    "UPDATE nodes SET status = 'active', activated_at = ?, expires_at = NULL, updated_at = ? WHERE node_id = ? AND status = 'pending'",
    now,
    now,
    node.node_id,
  ).rowsWritten;
  if (changed === 0) {
    const cur = getNode(ctx.sql, node.node_id);
    if (cur?.status !== "active") throw new HttpError("conflict", "node changed state during activation");
  }
  log("node_activated", { node_id: node.node_id });
  return { status: "active" };
}

/** Smaže DNS a tunel, které Worker vytvořil. Vrací true, když nic nezbylo. */
export async function releaseResources(ctx: RegCtx, node: NodeRow): Promise<boolean> {
  const cf = ctx.cloudflare();
  try {
    if (node.dns_record_id) {
      await cf.deleteDns(node.dns_record_id, node.hostname);
      run(ctx.sql, "UPDATE nodes SET dns_record_id = NULL WHERE node_id = ?", node.node_id);
    }
    if (node.tunnel_id && node.tunnel_name) {
      await cf.deleteTunnel(node.tunnel_id, node.tunnel_name);
      run(ctx.sql, "UPDATE nodes SET tunnel_id = NULL WHERE node_id = ?", node.node_id);
    }
    return true;
  } catch (e) {
    log("release_failed", { node_id: node.node_id, error: String(e) });
    return false;
  }
}

export function closeSockets(ctx: RegCtx, nodeId: string, code: number, reason: string): void {
  for (const ws of ctx.sockets(nodeId)) {
    try {
      ws.send(JSON.stringify({ t: "bye", reason }));
      ws.close(code, reason);
    } catch {
      /* už zavřeno */
    }
  }
}

export async function deregister(ctx: RegCtx, auth: AuthCtx): Promise<{ status: "retired" }> {
  consumeNonce(ctx, auth.replayKey);
  const node = requireKnownNode(ctx, auth);
  if (node.status === "banned") throw new HttpError("forbidden", "node is banned");
  if (node.status === "provisioning") throw new HttpError("conflict", "registration in progress");
  run(ctx.sql, "UPDATE nodes SET status = 'retired', expires_at = NULL, updated_at = ? WHERE node_id = ?", ctx.nowS(), node.node_id);
  run(ctx.sql, "DELETE FROM challenges WHERE node_id = ?", node.node_id);
  closeSockets(ctx, node.node_id, 1000, "deregistered");
  const fresh = getNode(ctx.sql, node.node_id);
  if (fresh) await releaseResources(ctx, fresh);
  log("node_deregistered", { node_id: node.node_id });
  return { status: "retired" };
}

export async function banNode(ctx: RegCtx, nodeId: string, reason: string): Promise<boolean> {
  const node = getNode(ctx.sql, nodeId);
  if (!node || node.status === "banned") return false;
  run(
    ctx.sql,
    "UPDATE nodes SET status = 'banned', ban_reason = ?, expires_at = NULL, updated_at = ? WHERE node_id = ?",
    reason.slice(0, 200),
    ctx.nowS(),
    nodeId,
  );
  run(ctx.sql, "DELETE FROM challenges WHERE node_id = ?", nodeId);
  if (node.asn !== null) {
    run(
      ctx.sql,
      "INSERT INTO asn_penalties (asn, count) VALUES (?, 1) ON CONFLICT(asn) DO UPDATE SET count = count + 1",
      node.asn,
    );
  }
  closeSockets(ctx, nodeId, 4003, "banned");
  const fresh = getNode(ctx.sql, nodeId);
  if (fresh) await releaseResources(ctx, fresh);
  log("node_banned", { node_id: nodeId, reason });
  return true;
}

export async function unbanNode(ctx: RegCtx, nodeId: string): Promise<boolean> {
  const r = ctx.sql.exec(
    "UPDATE nodes SET status = 'retired', ban_reason = NULL, strikes = 0, updated_at = ? WHERE node_id = ? AND status = 'banned'",
    ctx.nowS(),
    nodeId,
  );
  return r.rowsWritten > 0;
}

/** Úklid: propadlé výzvy a nonce, zaseklé provisioning a nevyřízené pending, nedokončené mazání. */
export async function sweepRegistrations(ctx: RegCtx): Promise<void> {
  const now = ctx.nowS();
  run(ctx.sql, "DELETE FROM reg_challenges WHERE expires_at < ?", now);
  run(ctx.sql, "DELETE FROM nonces WHERE expires_at < ?", now);
  run(ctx.sql, "DELETE FROM reg_log WHERE at < ?", now - Math.max(ctx.cfg.hashcash.surgeWindowS, 3600));

  const stale = all<NodeRow & Record<string, SqlStorageValue>>(
    ctx.sql,
    "SELECT * FROM nodes WHERE status IN ('provisioning','pending') AND expires_at IS NOT NULL AND expires_at < ?",
    now,
  );
  for (const n of stale) {
    run(ctx.sql, "UPDATE nodes SET status = 'retired', expires_at = NULL, updated_at = ? WHERE node_id = ?", now, n.node_id);
  }
  const leftovers = all<NodeRow & Record<string, SqlStorageValue>>(
    ctx.sql,
    "SELECT * FROM nodes WHERE status IN ('retired','banned') AND (tunnel_id IS NOT NULL OR dns_record_id IS NOT NULL) LIMIT 10",
  );
  for (const n of leftovers) await releaseResources(ctx, n);
}
