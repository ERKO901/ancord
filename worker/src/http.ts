import { HttpError } from "./errors";
import { b64uDecodeLen } from "./encoding";

export const MAX_BODY_BYTES = 16 * 1024;

/** `AbortSignal.timeout` chybí v typech workers-types. */
export function timeoutSignal(ms: number): AbortSignal {
  const c = new AbortController();
  setTimeout(() => c.abort(), ms);
  return c.signal;
}

export function json(data: unknown, status = 200, headers: HeadersInit = {}): Response {
  const h = new Headers(headers);
  h.set("content-type", "application/json");
  h.set("x-content-type-options", "nosniff");
  if (!h.has("cache-control")) h.set("cache-control", "no-store");
  return new Response(JSON.stringify(data), { status, headers: h });
}

/** Přečte tělo s tvrdým limitem velikosti (nevěří Content-Length). */
export async function readBody(request: Request, maxBytes = MAX_BODY_BYTES): Promise<Uint8Array> {
  const declared = request.headers.get("content-length");
  if (declared !== null && Number(declared) > maxBytes) throw new HttpError("too_large", "request body too large");
  if (!request.body) return new Uint8Array(0);
  const reader = request.body.getReader();
  const chunks: Uint8Array[] = [];
  let total = 0;
  for (;;) {
    const { done, value } = await reader.read();
    if (done) break;
    total += value.length;
    if (total > maxBytes) {
      await reader.cancel();
      throw new HttpError("too_large", "request body too large");
    }
    chunks.push(value);
  }
  const out = new Uint8Array(total);
  let o = 0;
  for (const c of chunks) {
    out.set(c, o);
    o += c.length;
  }
  return out;
}

export function parseJsonObject(body: Uint8Array): Record<string, unknown> {
  let v: unknown;
  try {
    v = JSON.parse(new TextDecoder("utf-8", { fatal: true, ignoreBOM: false }).decode(body));
  } catch {
    throw new HttpError("bad_request", "body must be valid JSON");
  }
  if (typeof v !== "object" || v === null || Array.isArray(v)) {
    throw new HttpError("bad_request", "body must be a JSON object");
  }
  return v as Record<string, unknown>;
}

export function reqString(o: Record<string, unknown>, key: string, maxLen = 4096): string {
  const v = o[key];
  if (typeof v !== "string" || v.length === 0 || v.length > maxLen) {
    throw new HttpError("bad_request", `field '${key}' must be a non-empty string`);
  }
  return v;
}

export function reqInt(o: Record<string, unknown>, key: string, min: number, max: number): number {
  const v = o[key];
  if (typeof v !== "number" || !Number.isSafeInteger(v) || v < min || v > max) {
    throw new HttpError("bad_request", `field '${key}' must be an integer in [${min}, ${max}]`);
  }
  return v;
}

export function optInt(o: Record<string, unknown>, key: string, min: number, max: number): number | undefined {
  return o[key] === undefined ? undefined : reqInt(o, key, min, max);
}

export function reqB64(o: Record<string, unknown>, key: string, len: number): Uint8Array {
  const v = o[key];
  const b = typeof v === "string" ? b64uDecodeLen(v, len) : null;
  if (!b) throw new HttpError("bad_request", `field '${key}' must be ${len} bytes in base64url`);
  return b;
}

export function reqObject(o: Record<string, unknown>, key: string): Record<string, unknown> {
  const v = o[key];
  if (typeof v !== "object" || v === null || Array.isArray(v)) {
    throw new HttpError("bad_request", `field '${key}' must be an object`);
  }
  return v as Record<string, unknown>;
}

export function requireMethod(request: Request, method: string): void {
  if (request.method !== method) {
    throw new HttpError("bad_request", `method ${request.method} not allowed`);
  }
}

/** Kdo je klient: IP z Cloudflare a ASN. IP se dál nikde neukládá v čitelné podobě. */
export function clientInfo(request: Request): { ip: string | null; asn: number | null } {
  const cf = request.cf as { asn?: number } | undefined;
  return { ip: request.headers.get("cf-connecting-ip"), asn: typeof cf?.asn === "number" ? cf.asn : null };
}

/** /24 pro IPv4, /48 pro IPv6. Vrací null pro neplatnou adresu. */
export function ipPrefix(ip: string | null): string | null {
  if (!ip) return null;
  if (ip.includes(":")) {
    const groups = expandIpv6(ip);
    return groups ? groups.slice(0, 3).join(":") + "::/48" : null;
  }
  const parts = ip.split(".");
  if (parts.length !== 4 || parts.some((p) => !/^\d{1,3}$/.test(p) || Number(p) > 255)) return null;
  return `${parts[0]}.${parts[1]}.${parts[2]}.0/24`;
}

function expandIpv6(ip: string): string[] | null {
  const dbl = ip.split("::");
  if (dbl.length > 2) return null;
  const head = dbl[0] ? dbl[0].split(":") : [];
  const tail = dbl.length === 2 && dbl[1] ? dbl[1].split(":") : [];
  const missing = 8 - head.length - tail.length;
  if (dbl.length === 1 ? head.length !== 8 : missing < 0) return null;
  const all = [...head, ...Array<string>(dbl.length === 2 ? missing : 0).fill("0"), ...tail];
  if (all.length !== 8 || all.some((g) => !/^[0-9a-fA-F]{1,4}$/.test(g))) return null;
  return all.map((g) => g.toLowerCase().padStart(4, "0"));
}
