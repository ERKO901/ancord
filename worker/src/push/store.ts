import { aesGcmDecrypt, aesGcmEncrypt } from "../crypto";
import { b64uDecode, b64uDecodeLen, b64uEncode, fromUtf8, utf8 } from "../encoding";
import type { Env } from "../env";
import type { Platform } from "./senders";

export interface StoredToken {
  platform: Platform;
  /** b64u(iv ‖ ciphertext ‖ tag) */
  ct: string;
  ts: number;
}

const key = (pushId: string) => `push:${pushId}`;

function encKey(env: Env): Uint8Array {
  const k = b64uDecodeLen(env.PUSH_ENC_KEY, 32);
  if (!k) throw new Error("PUSH_ENC_KEY must be 32 bytes in base64url");
  return k;
}

export async function getStored(env: Env, pushId: string): Promise<StoredToken | null> {
  return env.PUSH_KV.get<StoredToken>(key(pushId), "json");
}

/** Uloží token šifrovaně; starší `ts` než uložený je ignorováno (ochrana proti přehrání). */
export async function putToken(env: Env, pushId: string, platform: Platform, token: string, ts: number): Promise<boolean> {
  const cur = await getStored(env, pushId);
  if (cur && cur.ts >= ts) return false;
  const ct = await aesGcmEncrypt(encKey(env), utf8(token), utf8(pushId));
  const v: StoredToken = { platform, ct: b64uEncode(ct), ts };
  await env.PUSH_KV.put(key(pushId), JSON.stringify(v));
  return true;
}

export async function deleteToken(env: Env, pushId: string, ts: number): Promise<boolean> {
  const cur = await getStored(env, pushId);
  if (cur && cur.ts >= ts) return false;
  await env.PUSH_KV.delete(key(pushId));
  return true;
}

export async function readToken(env: Env, pushId: string): Promise<{ platform: Platform; token: string } | null> {
  const s = await getStored(env, pushId);
  if (!s) return null;
  const blob = b64uDecode(s.ct);
  if (!blob) return null;
  const pt = await aesGcmDecrypt(encKey(env), blob, utf8(pushId));
  return pt ? { platform: s.platform, token: fromUtf8(pt) } : null;
}

/** Bezpodmínečné smazání (neplatný token nahlášený FCM/APNs). */
export async function purgeToken(env: Env, pushId: string): Promise<void> {
  await env.PUSH_KV.delete(key(pushId));
}
