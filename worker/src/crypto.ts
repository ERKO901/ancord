import { blake3 } from "@noble/hashes/blake3.js";
import { sha256 } from "@noble/hashes/sha2.js";
import { b64uDecodeLen, b64uEncode, base32Encode, concat, utf8 } from "./encoding";

export { blake3, sha256 };

export const PUBKEY_LEN = 32;
export const SIG_LEN = 64;

export const DOMAIN = {
  register: "ancord/v1/register",
  proof: "ancord/v1/proof",
  req: "ancord/v1/req",
  heartbeat: "ancord/v1/heartbeat",
  directory: "ancord/v1/directory",
  ticket: "ancord/v1/ticket",
  report: "ancord/v1/report",
  pushReg: "ancord/v1/push-reg",
  clientAuth: "ancord/v1/client-auth",
} as const;

export type Domain = (typeof DOMAIN)[keyof typeof DOMAIN];

/** `DOMAIN ‖ 0x00 ‖ payload` (spec/PROTOCOL.md §1.3). */
export function signingInput(domain: Domain, payload: Uint8Array): Uint8Array {
  return concat(utf8(domain), new Uint8Array([0]), payload);
}

const ED25519 = { name: "Ed25519" } as const;
const verifyKeyCache = new Map<string, CryptoKey>();
const VERIFY_CACHE_MAX = 512;

async function verifyKey(pubkey: Uint8Array): Promise<CryptoKey> {
  const id = b64uEncode(pubkey);
  const hit = verifyKeyCache.get(id);
  if (hit) return hit;
  const key = await crypto.subtle.importKey("raw", pubkey, ED25519, false, ["verify"]);
  if (verifyKeyCache.size >= VERIFY_CACHE_MAX) {
    const oldest = verifyKeyCache.keys().next().value;
    if (oldest !== undefined) verifyKeyCache.delete(oldest);
  }
  verifyKeyCache.set(id, key);
  return key;
}

/** Ověří Ed25519 podpis; nikdy nevyhazuje, neplatný vstup = false. */
export async function verifySig(
  pubkey: Uint8Array,
  domain: Domain,
  payload: Uint8Array,
  sig: Uint8Array,
): Promise<boolean> {
  if (pubkey.length !== PUBKEY_LEN || sig.length !== SIG_LEN) return false;
  try {
    return await crypto.subtle.verify(ED25519, await verifyKey(pubkey), sig, signingInput(domain, payload));
  } catch {
    return false;
  }
}

export interface Signer {
  readonly publicKey: Uint8Array;
  readonly kid: string;
  sign(domain: Domain, payload: Uint8Array): Promise<Uint8Array>;
}

const PKCS8_ED25519_PREFIX = new Uint8Array([
  0x30, 0x2e, 0x02, 0x01, 0x00, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x04, 0x22, 0x04, 0x20,
]);

export function kidOf(publicKey: Uint8Array): string {
  return b64uEncode(sha256(publicKey).slice(0, 8));
}

/** Vytvoří podepisovatele ze 32B seedu (b64u). */
export async function signerFromSeed(seedB64u: string): Promise<Signer> {
  const seed = b64uDecodeLen(seedB64u, 32);
  if (!seed) throw new Error("signing key must be a 32 byte base64url seed");
  const key = await crypto.subtle.importKey("pkcs8", concat(PKCS8_ED25519_PREFIX, seed), ED25519, true, ["sign"]);
  const jwk = (await crypto.subtle.exportKey("jwk", key)) as JsonWebKey;
  const x = jwk.x ? b64uDecodeLen(jwk.x, 32) : null;
  if (!x) throw new Error("cannot derive public key");
  return {
    publicKey: x,
    kid: kidOf(x),
    async sign(domain, payload) {
      return new Uint8Array(await crypto.subtle.sign(ED25519, key, signingInput(domain, payload)));
    },
  };
}

const signerCache = new Map<string, Promise<Signer>>();
/** Podepisovatel Workeru z `WORKER_SIGNING_KEY`, cachovaný v izolátu. */
export function workerSigner(seed: string | undefined): Promise<Signer> {
  if (!seed) return Promise.reject(new Error("WORKER_SIGNING_KEY is not configured"));
  let p = signerCache.get(seed);
  if (!p) {
    p = signerFromSeed(seed);
    p.catch(() => signerCache.delete(seed));
    signerCache.set(seed, p);
  }
  return p;
}

/** node_id = base32(blake3(pubkey)). */
export function nodeIdOf(pubkey: Uint8Array): string {
  return base32Encode(blake3(pubkey));
}

export const HOSTNAME_ID_CHARS = 16;

export function hostnameOf(nodeId: string, zone: string): string {
  return `node${nodeId.slice(0, HOSTNAME_ID_CHARS)}.${zone}`;
}

/** Počet úvodních nulových bitů. */
export function leadingZeroBits(bytes: Uint8Array): number {
  let n = 0;
  for (const b of bytes) {
    if (b === 0) {
      n += 8;
      continue;
    }
    return n + Math.clz32(b) - 24;
  }
  return n;
}

export async function hmacSha256(key: Uint8Array, data: Uint8Array): Promise<Uint8Array> {
  const k = await crypto.subtle.importKey("raw", key, { name: "HMAC", hash: "SHA-256" }, false, ["sign"]);
  return new Uint8Array(await crypto.subtle.sign("HMAC", k, data));
}

export async function aesGcmEncrypt(key: Uint8Array, plaintext: Uint8Array, aad: Uint8Array): Promise<Uint8Array> {
  const k = await crypto.subtle.importKey("raw", key, "AES-GCM", false, ["encrypt"]);
  const iv = crypto.getRandomValues(new Uint8Array(12));
  const ct = new Uint8Array(await crypto.subtle.encrypt({ name: "AES-GCM", iv, additionalData: aad }, k, plaintext));
  return concat(iv, ct);
}

export async function aesGcmDecrypt(key: Uint8Array, blob: Uint8Array, aad: Uint8Array): Promise<Uint8Array | null> {
  if (blob.length < 12 + 16) return null;
  try {
    const k = await crypto.subtle.importKey("raw", key, "AES-GCM", false, ["decrypt"]);
    const pt = await crypto.subtle.decrypt(
      { name: "AES-GCM", iv: blob.slice(0, 12), additionalData: aad },
      k,
      blob.slice(12),
    );
    return new Uint8Array(pt);
  } catch {
    return null;
  }
}
