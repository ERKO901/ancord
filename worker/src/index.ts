import { verifyNodeRequest } from "./auth";
import { verifyBwToken } from "./challenge/bandwidth";
import { clock } from "./clock";
import { loadConfig, type Config } from "./config";
import { hmacSha256, sha256, workerSigner } from "./crypto";
import { b64uDecodeLen, b64uEncode, timingSafeEqual, utf8 } from "./encoding";
import type { Env, PushJob } from "./env";
import { HttpError, errorResponse, unwrap, type Result } from "./errors";
import {
  clientInfo,
  ipPrefix,
  json,
  parseJsonObject,
  readBody,
  reqInt,
  reqObject,
  reqString,
  optInt,
} from "./http";
import { handlePushRegister, handleWake } from "./push/handler";
import { processPushBatch } from "./push/queue";
import type { SignedDirectory } from "./registry/directory";
import { KV_DIRECTORY, KV_SNAPSHOT } from "./registry/snapshot";
import { selectNodes, type Snapshot } from "./routing";
import { issueTicket } from "./ticket";

export { Registry } from "./registry/registry";
export { RateLimiter } from "./limiter";

const REGISTRY_NAME = "main";

function registry(env: Env) {
  return env.REGISTRY.getByName(REGISTRY_NAME);
}

/** Volání DO přes RPC, které vrací `Result`; chyba se vyhodí jako `HttpError`. */
async function rpc<T>(call: () => Promise<Result<T>>): Promise<T> {
  return unwrap(await call());
}

/* ------------------------------------------------------------------ routing */

let snapshotMemo: { at: number; snap: Snapshot | null } | null = null;
const SNAPSHOT_MEMO_MS = 2000;

async function readSnapshot(env: Env): Promise<Snapshot | null> {
  const now = clock.nowMs();
  if (snapshotMemo && now - snapshotMemo.at < SNAPSHOT_MEMO_MS) return snapshotMemo.snap;
  const snap = await env.ROUTING_KV.get<Snapshot>(KV_SNAPSHOT, "json");
  snapshotMemo = { at: now, snap };
  return snap;
}

/** Pro testy: zahodí paměť snapshotu v izolátu. */
export function resetRouteCache(): void {
  snapshotMemo = null;
}

function numOrNull(v: unknown): number | null {
  const n = typeof v === "string" || typeof v === "number" ? Number(v) : NaN;
  return Number.isFinite(n) ? n : null;
}

async function handleRoute(request: Request, env: Env, cfg: Config): Promise<Response> {
  const ek = new URL(request.url).searchParams.get("ek");
  if (!ek || !b64uDecodeLen(ek, 32)) throw new HttpError("bad_request", "query parameter 'ek' must be a 32 byte key in base64url");

  const snap = await readSnapshot(env);
  if (!snap) throw new HttpError("unavailable", "routing snapshot is not available yet", 5);
  const nowS = clock.nowS();
  if (nowS - snap.generated_at > cfg.snapshotMaxAgeS) throw new HttpError("stale_snapshot", "routing snapshot is stale", 10);
  const live = snap.nodes.filter((n) => n.b > 0);
  if (live.length < cfg.minLiveNodes) {
    throw new HttpError("insufficient_nodes", `fewer than ${cfg.minLiveNodes} live nodes`, 10);
  }

  const cf = (request.cf ?? {}) as Record<string, unknown>;
  const chosen = selectNodes(
    live,
    {
      lat: numOrNull(cf.latitude),
      lon: numOrNull(cf.longitude),
      country: typeof cf.country === "string" ? cf.country : null,
      continent: typeof cf.continent === "string" ? cf.continent : null,
    },
    cfg.routeNodes,
    cfg.routing.nearKm,
  );
  const signer = await workerSigner(env.WORKER_SIGNING_KEY);
  const exp = nowS + cfg.ticketTtlS;
  const nodes = await Promise.all(
    chosen.map(async (n) => ({ node_id: n.id, hostname: n.host, ticket: await issueTicket(signer, n.id, ek, exp) })),
  );
  return json({ epoch: snap.epoch, ttl: cfg.ticketTtlS, nodes });
}

async function handleDirectory(env: Env, cfg: Config): Promise<Response> {
  const epoch = Math.floor(clock.nowS() / cfg.epochSeconds);
  let dir = await env.ROUTING_KV.get<SignedDirectory>(KV_DIRECTORY, "json");
  if (!dir || dir.body.epoch < epoch) dir = await rpc(() => registry(env).getDirectory());
  return json(dir, 200, { "cache-control": "public, max-age=15" });
}

/* -------------------------------------------------------------- registrace */

function parseClaimed(o: Record<string, unknown>) {
  const c = reqObject(o, "claimed_specs");
  return {
    cpu_cores: reqInt(c, "cpu_cores", 1, 1024),
    ram_mb: reqInt(c, "ram_mb", 1, 4_194_304),
    storage_gb: reqInt(c, "storage_gb", 1, 1_048_576),
    bandwidth_mbps: reqInt(c, "bandwidth_mbps", 1, 400_000),
  };
}

async function ipHashOf(env: Env, ip: string | null): Promise<string | null> {
  const prefix = ipPrefix(ip);
  const key = b64uDecodeLen(env.CHALLENGE_HMAC_KEY, 32);
  if (!prefix || !key) return null;
  return b64uEncode((await hmacSha256(key, utf8(`ip|${prefix}`))).slice(0, 16));
}

async function handleRegister(request: Request, env: Env): Promise<Response> {
  const body = parseJsonObject(await readBody(request));
  const { asn } = clientInfo(request);
  const result = await rpc(() =>
    registry(env).registerBegin({ pubkey: reqString(body, "pubkey", 64), claimed: parseClaimed(body), asn }),
  );
  return json(result);
}

async function handleRegisterConfirm(request: Request, env: Env): Promise<Response> {
  const body = parseJsonObject(await readBody(request));
  const { ip, asn } = clientInfo(request);
  const result = await rpc(async () =>
    registry(env).registerConfirm({
      pubkey: reqString(body, "pubkey", 64),
      nonce: reqString(body, "challenge_nonce", 64),
      signature: reqString(body, "signature", 128),
      counter: optInt(body, "hashcash_counter", 0, Number.MAX_SAFE_INTEGER) ?? null,
      asn,
      ipHash: await ipHashOf(env, ip),
    }),
  );
  return json(result);
}

async function signedAuth(request: Request, cfg: Config, body: Uint8Array) {
  const auth = await verifyNodeRequest(request, body, clock.nowS(), cfg.requestSkewS);
  return { nodeId: auth.nodeId, pubkey: auth.pubkeyB64, replayKey: auth.replayKey };
}

async function handleSigned(request: Request, env: Env, cfg: Config, action: "activate" | "deregister"): Promise<Response> {
  const body = await readBody(request, 1024);
  const auth = await signedAuth(request, cfg, body);
  if (action === "activate") return json(await rpc(() => registry(env).activate(auth)));
  return json(await rpc(() => registry(env).deregister(auth)));
}

async function handleWs(request: Request, env: Env, cfg: Config): Promise<Response> {
  if (request.headers.get("upgrade")?.toLowerCase() !== "websocket") {
    return new Response("expected websocket upgrade", { status: 426 });
  }
  const auth = await verifyNodeRequest(request, new Uint8Array(0), clock.nowS(), cfg.requestSkewS);
  const cf = (request.cf ?? {}) as Record<string, unknown>;
  const headers = new Headers({
    upgrade: "websocket",
    "x-ancord-pk": auth.pubkeyB64,
    "x-ancord-replay": auth.replayKey,
    "x-ancord-cf": JSON.stringify({
      country: cf.country,
      continent: cf.continent,
      colo: cf.colo,
      latitude: cf.latitude,
      longitude: cf.longitude,
    }),
  });
  return registry(env).fetch(new Request("https://registry/ws", { headers }));
}

async function handleWakeRoute(request: Request, env: Env, cfg: Config): Promise<Response> {
  const body = await readBody(request, 2048);
  const nowS = clock.nowS();
  const auth = await verifyNodeRequest(request, body, nowS, cfg.requestSkewS);
  return handleWake(request, body, auth, env, cfg, nowS);
}

/* ---------------------------------------------------------------- challenge */

async function bwToken(env: Env, dir: "dl" | "ul", token: string): Promise<string> {
  const key = b64uDecodeLen(env.CHALLENGE_HMAC_KEY, 32);
  if (!key) throw new Error("CHALLENGE_HMAC_KEY must be 32 bytes in base64url");
  const id = await verifyBwToken(key, dir, token, clock.nowS());
  if (!id) throw new HttpError("unauthorized", "invalid or expired token");
  return id;
}

async function handleBlobDownload(env: Env, ctx: ExecutionContext, token: string): Promise<Response> {
  const id = await bwToken(env, "dl", token);
  const auth = await registry(env).bwAuthorize(id, "dl");
  if (!auth) throw new HttpError("not_found", "no such challenge");
  const obj = await env.BW_BLOBS.get(auth.key);
  if (!obj) throw new HttpError("unavailable", "challenge blob is missing");
  const { readable, writable } = new FixedLengthStream(obj.size);
  const started = clock.nowMs();
  ctx.waitUntil(
    (async () => {
      try {
        await obj.body.pipeTo(writable);
        await registry(env).bwRecord(id, "dl", { bytes: obj.size, ms: Math.max(1, clock.nowMs() - started) });
      } catch {
        /* klient přenos přerušil; výzva vyprší sama */
      }
    })(),
  );
  return new Response(readable, {
    headers: { "content-type": "application/octet-stream", "content-length": String(obj.size), "cache-control": "no-store" },
  });
}

async function handleBlobUpload(request: Request, env: Env, token: string): Promise<Response> {
  const id = await bwToken(env, "ul", token);
  const auth = await registry(env).bwAuthorize(id, "ul");
  if (!auth) throw new HttpError("not_found", "no such challenge");
  if (!request.body) throw new HttpError("bad_request", "empty upload");
  let received = 0;
  const limit = auth.bytes;
  const digest = new crypto.DigestStream("SHA-256");
  digest.digest.catch(() => undefined); // po přerušení se odmítne; nesmí zůstat neošetřené
  const writer = digest.getWriter();
  const reader = request.body.getReader();
  const started = clock.nowMs();
  for (;;) {
    const { done, value } = await reader.read();
    if (done) break;
    received += value.byteLength;
    if (received > limit) {
      await reader.cancel();
      await writer.abort();
      throw new HttpError("too_large", "upload exceeds expected size");
    }
    await writer.write(value);
  }
  await writer.close();
  const sha = b64uEncode(new Uint8Array(await digest.digest));
  await registry(env).bwRecord(id, "ul", { bytes: received, ms: Math.max(1, clock.nowMs() - started), sha });
  return json({ bytes: received });
}

/* -------------------------------------------------------------------- admin */

async function adminAllowed(request: Request, env: Env): Promise<boolean> {
  const token = env.ADMIN_TOKEN;
  const header = request.headers.get("authorization");
  if (!token || !header?.startsWith("Bearer ")) return false;
  return timingSafeEqual(sha256(utf8(header.slice(7))), sha256(utf8(token)));
}

async function handleAdmin(request: Request, env: Env, path: string): Promise<Response> {
  if (!(await adminAllowed(request, env))) throw new HttpError("unauthorized", "admin token required");
  if (path === "/admin/nodes" && request.method === "GET") return json(await registry(env).adminNodes());
  const m = /^\/admin\/nodes\/([a-z2-7]{52})\/(ban|unban)$/.exec(path);
  if (m && request.method === "POST") {
    const nodeId = m[1]!;
    if (m[2] === "ban") {
      const body = parseJsonObject(await readBody(request));
      const changed = await registry(env).adminBan(nodeId, reqString(body, "reason", 200));
      return json({ banned: changed });
    }
    return json({ unbanned: await registry(env).adminUnban(nodeId) });
  }
  throw new HttpError("not_found", "unknown admin route");
}

/* ------------------------------------------------------------------- router */

async function route(request: Request, env: Env, ctx: ExecutionContext): Promise<Response> {
  const cfg = loadConfig(env);
  const url = new URL(request.url);
  const path = url.pathname;
  const method = request.method;

  if (path === "/health" && method === "GET") {
    return json({ ok: true, profile: cfg.profile, epoch: Math.floor(clock.nowS() / cfg.epochSeconds) });
  }
  if (path === "/.well-known/ancord-key" && method === "GET") {
    const s = await workerSigner(env.WORKER_SIGNING_KEY);
    return json({ pubkey: b64uEncode(s.publicKey), kid: s.kid }, 200, { "cache-control": "public, max-age=3600" });
  }
  if (path === "/directory" && method === "GET") return handleDirectory(env, cfg);
  if (path === "/route" && method === "GET") return handleRoute(request, env, cfg);
  if (path === "/register" && method === "POST") return handleRegister(request, env);
  if (path === "/register/confirm" && method === "POST") return handleRegisterConfirm(request, env);
  if (path === "/register/activate" && method === "POST") return handleSigned(request, env, cfg, "activate");
  if (path === "/deregister" && method === "POST") return handleSigned(request, env, cfg, "deregister");
  if (path === "/ws" && method === "GET") return handleWs(request, env, cfg);
  if (path === "/wake" && method === "POST") return handleWakeRoute(request, env, cfg);
  if (path === "/push/register" && (method === "POST" || method === "DELETE")) {
    return handlePushRegister(request, env, cfg, clock.nowS());
  }
  if (path.startsWith("/challenge/blob/") && method === "GET") return handleBlobDownload(env, ctx, path.slice("/challenge/blob/".length));
  if (path.startsWith("/challenge/upload/") && method === "PUT") return handleBlobUpload(request, env, path.slice("/challenge/upload/".length));
  if (path.startsWith("/admin/")) return handleAdmin(request, env, path);
  throw new HttpError("not_found", "no such route");
}

export default {
  async fetch(request: Request, env: Env, ctx: ExecutionContext): Promise<Response> {
    try {
      return await route(request, env, ctx);
    } catch (e) {
      if (e instanceof HttpError) return errorResponse(e);
      console.log(JSON.stringify({ event: "unhandled_error", path: new URL(request.url).pathname, error: String(e) }));
      return errorResponse(new HttpError("unavailable", "internal error"));
    }
  },

  async scheduled(_controller: ScheduledController, env: Env, ctx: ExecutionContext): Promise<void> {
    ctx.waitUntil(registry(env).tick());
  },

  async queue(batch: MessageBatch<PushJob>, env: Env): Promise<void> {
    await processPushBatch(batch, env);
  },
} satisfies ExportedHandler<Env, PushJob>;

