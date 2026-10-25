import { canonicalBytes } from "../canonical";
import type { Config } from "../config";
import { DOMAIN, blake3, verifySig } from "../crypto";
import { base32Encode, b64uDecodeLen } from "../encoding";
import type { Env, PushJob } from "../env";
import { HttpError } from "../errors";
import { anchorsFor } from "../hrw";
import { json, parseJsonObject, readBody, reqInt, reqString } from "../http";
import type { NodeAuth } from "../auth";
import type { SignedDirectory } from "../registry/directory";
import { KV_DIRECTORY, kvDirectoryKey } from "../registry/snapshot";
import type { AdmitResult } from "../limiter";
import { deleteToken, getStored, putToken } from "./store";

const MAX_TOKEN_LEN = 4096;

/** Odeslání do fronty; přepisovatelné v testech. */
export const pushDeps = {
  enqueue: (env: Env, job: PushJob): Promise<unknown> => env.PUSH_QUEUE.send(job),
};

export function pushIdOf(pushPubkey: Uint8Array): string {
  return base32Encode(blake3(pushPubkey));
}

/** `POST /push/register` a `DELETE /push/register` (spec §10.1). */
export async function handlePushRegister(request: Request, env: Env, cfg: Config, nowS: number): Promise<Response> {
  const isDelete = request.method === "DELETE";
  const body = parseJsonObject(await readBody(request));
  const pk = b64uDecodeLen(reqString(body, "push_pubkey", 64), 32);
  if (!pk) throw new HttpError("bad_request", "push_pubkey must be 32 bytes in base64url");
  const sig = b64uDecodeLen(reqString(body, "sig", 128), 64);
  if (!sig) throw new HttpError("bad_request", "sig must be 64 bytes in base64url");
  const ts = reqInt(body, "ts", 0, Number.MAX_SAFE_INTEGER);
  if (Math.abs(nowS - ts) > cfg.push.regSkewS) throw new HttpError("unauthorized", "timestamp outside allowed window");
  const pushId = pushIdOf(pk);

  if (isDelete) {
    const payload = canonicalBytes({ op: "delete", push_id: pushId, ts });
    if (!(await verifySig(pk, DOMAIN.pushReg, payload, sig))) throw new HttpError("unauthorized", "invalid signature");
    await deleteToken(env, pushId, ts);
    return json({ push_id: pushId });
  }

  const platform = reqString(body, "platform", 8);
  if (platform !== "fcm" && platform !== "apns") throw new HttpError("bad_request", "platform must be 'fcm' or 'apns'");
  const token = reqString(body, "token", MAX_TOKEN_LEN);
  const payload = canonicalBytes({ op: "register", platform, push_id: pushId, token, ts });
  if (!(await verifySig(pk, DOMAIN.pushReg, payload, sig))) throw new HttpError("unauthorized", "invalid signature");
  const stored = await putToken(env, pushId, platform, token, ts);
  if (!stored) throw new HttpError("conflict", "a newer registration exists");
  return json({ push_id: pushId });
}

async function readDirectory(env: Env, epoch: number): Promise<SignedDirectory | null> {
  return env.ROUTING_KV.get<SignedDirectory>(kvDirectoryKey(epoch), "json");
}

/** Aktuální a předchozí adresář (spec §7: wake akceptuje obě epochy). */
export async function recentDirectories(env: Env, cfg: Config, nowS: number): Promise<SignedDirectory[]> {
  const epoch = Math.floor(nowS / cfg.epochSeconds);
  const [cur, prev] = await Promise.all([readDirectory(env, epoch), readDirectory(env, epoch - 1)]);
  const out: SignedDirectory[] = [];
  if (cur) out.push(cur);
  else {
    const latest = await env.ROUTING_KV.get<SignedDirectory>(KV_DIRECTORY, "json");
    if (latest && latest.body.epoch >= epoch - 1) out.push(latest);
  }
  if (prev) out.push(prev);
  return out;
}

/** `POST /wake` (spec §10.2). Autentizace podpisem requestu je hotová, tady se ověřuje anchor a limity. */
export async function handleWake(request: Request, body: Uint8Array, auth: NodeAuth, env: Env, cfg: Config, nowS: number): Promise<Response> {
  const o = parseJsonObject(body);
  const pushId = reqString(o, "push_id", 64);
  const mailboxId = b64uDecodeLen(reqString(o, "mailbox_id", 64), 32);
  if (!mailboxId) throw new HttpError("bad_request", "mailbox_id must be 32 bytes in base64url");
  if (!/^[a-z2-7]{52}$/.test(pushId)) throw new HttpError("bad_request", "push_id has an invalid format");
  void request;

  const dirs = await recentDirectories(env, cfg, nowS);
  let isAnchor = false;
  for (const d of dirs) {
    const member = d.body.nodes.find((n) => n.id === auth.nodeId);
    if (!member || member.pk !== auth.pubkeyB64) continue;
    const anchors = anchorsFor(mailboxId, d.body.nodes.map((n) => n.id), d.body.epoch, d.body.params.r);
    if (anchors.includes(auth.nodeId)) {
      isAnchor = true;
      break;
    }
  }
  if (!isAnchor) throw new HttpError("forbidden", "node is not an anchor of this mailbox");

  const nodeLimiter = env.LIMITER.getByName(`node:${auth.nodeId}`);
  const admitted: AdmitResult = await nodeLimiter.admit({
    nonce: auth.replayKey,
    nonceExpiresMs: (nowS + 2 * cfg.requestSkewS + 5) * 1000,
    capacity: cfg.push.nodeBurst,
    refillPerS: cfg.push.nodeRefillPerS,
  });
  if (admitted.status === "replay") throw new HttpError("conflict", "request replayed");
  if (admitted.status === "limited") throw new HttpError("rate_limited", "wake rate limit exceeded", admitted.retryAfterS);

  if (!(await getStored(env, pushId))) return json({ status: "no_token" }, 202);

  const coalesced: AdmitResult = await env.LIMITER.getByName(`push:${pushId}`).admit({
    capacity: 1,
    refillPerS: 1 / cfg.push.coalesceS,
  });
  if (coalesced.status !== "ok") return json({ status: "coalesced" }, 202);

  await pushDeps.enqueue(env, { push_id: pushId });
  return json({ status: "queued" }, 202);
}
