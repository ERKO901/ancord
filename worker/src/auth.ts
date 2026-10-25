import { DOMAIN, nodeIdOf, sha256, verifySig } from "./crypto";
import { b64uDecodeLen, b64uEncode, utf8 } from "./encoding";
import { HttpError } from "./errors";

export interface NodeAuth {
  pubkey: Uint8Array;
  pubkeyB64: string;
  nodeId: string;
  /** Klíč pro detekci replaye: `(k, n)`. */
  replayKey: string;
  t: number;
}

const HEADER = /^Ancord-Ed25519 k=([A-Za-z0-9_-]+),t=(\d{1,12}),n=([A-Za-z0-9_-]+),s=([A-Za-z0-9_-]+)$/;

export function requestPayload(method: string, pathAndQuery: string, t: number, n: string, body: Uint8Array): Uint8Array {
  return utf8(`${method}\n${pathAndQuery}\n${t}\n${n}\n${b64uEncode(sha256(body))}`);
}

/**
 * Ověří podepsaný request (spec §3): formát hlavičky, časové okno a podpis.
 * Replay (`replayKey`) a příslušnost klíče k nodu ověřuje volající.
 */
export async function verifyNodeRequest(
  request: Request,
  body: Uint8Array,
  nowS: number,
  skewS: number,
): Promise<NodeAuth> {
  const header = request.headers.get("authorization");
  const m = header ? HEADER.exec(header) : null;
  if (!m) throw new HttpError("unauthorized", "missing or malformed Authorization header");
  const [, k, tStr, n, s] = m as unknown as [string, string, string, string, string];
  const pubkey = b64uDecodeLen(k, 32);
  const nonce = b64uDecodeLen(n, 16);
  const sig = b64uDecodeLen(s, 64);
  const t = Number(tStr);
  if (!pubkey || !nonce || !sig) throw new HttpError("unauthorized", "malformed credentials");
  if (Math.abs(nowS - t) > skewS) throw new HttpError("unauthorized", "request timestamp outside allowed window");
  const url = new URL(request.url);
  const payload = requestPayload(request.method, url.pathname + url.search, t, n, body);
  if (!(await verifySig(pubkey, DOMAIN.req, payload, sig))) throw new HttpError("unauthorized", "invalid signature");
  return { pubkey, pubkeyB64: k, nodeId: nodeIdOf(pubkey), replayKey: `${k}.${n}`, t };
}
