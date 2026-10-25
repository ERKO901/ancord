import { hmacSha256 } from "../crypto";
import { b64uDecode, b64uEncode, timingSafeEqual, utf8 } from "../encoding";

export type BwDir = "dl" | "ul";

export async function signBwToken(key: Uint8Array, dir: BwDir, id: string, expS: number): Promise<string> {
  const mac = await hmacSha256(key, utf8(`bw|${dir}|${id}|${expS}`));
  return `${id}.${expS}.${b64uEncode(mac)}`;
}

/** Vrací id výzvy, nebo null, pokud token nesedí nebo vypršel. */
export async function verifyBwToken(key: Uint8Array, dir: BwDir, token: string, nowS: number): Promise<string | null> {
  const parts = token.split(".");
  if (parts.length !== 3) return null;
  const [id, expStr, macStr] = parts as [string, string, string];
  const exp = Number(expStr);
  const mac = b64uDecode(macStr);
  if (!Number.isSafeInteger(exp) || exp < nowS || !mac) return null;
  const expected = await hmacSha256(key, utf8(`bw|${dir}|${id}|${exp}`));
  return timingSafeEqual(expected, mac) ? id : null;
}

export function blobKey(bytes: number, idx: number): string {
  return `bw/${bytes}/${idx}`;
}

/**
 * Zajistí, že blob existuje v R2, a vrátí jeho sha256 (b64u). Bloby jsou náhodné a sdílené
 * v poolu; generují se líně, protože CPU stojí jen `getRandomValues` a nativní digest.
 */
export async function ensureBlob(bucket: R2Bucket, bytes: number, idx: number): Promise<{ key: string; sha256: string }> {
  const key = blobKey(bytes, idx);
  const head = await bucket.head(key);
  const known = head?.customMetadata?.sha256;
  if (head && head.size === bytes && known) return { key, sha256: known };

  const data = new Uint8Array(bytes);
  for (let o = 0; o < bytes; o += 65_536) crypto.getRandomValues(data.subarray(o, Math.min(bytes, o + 65_536)));
  const digest = b64uEncode(new Uint8Array(await crypto.subtle.digest("SHA-256", data)));
  await bucket.put(key, data, { customMetadata: { sha256: digest } });
  return { key, sha256: digest };
}
