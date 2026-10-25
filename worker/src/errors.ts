export type ErrorCode =
  | "bad_request"
  | "unauthorized"
  | "forbidden"
  | "not_found"
  | "conflict"
  | "too_large"
  | "proof_failed"
  | "rate_limited"
  | "upstream"
  | "insufficient_nodes"
  | "stale_snapshot"
  | "unavailable";

const STATUS: Record<ErrorCode, number> = {
  bad_request: 400,
  unauthorized: 401,
  forbidden: 403,
  not_found: 404,
  conflict: 409,
  too_large: 413,
  proof_failed: 422,
  rate_limited: 429,
  upstream: 502,
  insufficient_nodes: 503,
  stale_snapshot: 503,
  unavailable: 503,
};

export class HttpError extends Error {
  readonly status: number;
  constructor(
    readonly code: ErrorCode,
    message: string,
    readonly retryAfter?: number,
  ) {
    super(message);
    this.name = "HttpError";
    this.status = STATUS[code];
  }
}

export function errorResponse(e: HttpError): Response {
  const headers = new Headers({ "content-type": "application/json", "cache-control": "no-store" });
  if (e.retryAfter !== undefined) headers.set("retry-after", String(Math.max(1, Math.ceil(e.retryAfter))));
  return new Response(JSON.stringify({ error: { code: e.code, message: e.message } }), {
    status: e.status,
    headers,
  });
}

/**
 * Výsledek RPC volání Durable Objectu. Očekávané chyby (4xx) se nevyhazují, protože vyhozená
 * výjimka přes RPC hranici se v runtime loguje jako neodchycená; vracejí se jako hodnota.
 */
export type Result<T> = { ok: true; value: T } | { ok: false; code: ErrorCode; message: string; retryAfter?: number };

export async function toResult<T>(fn: () => Promise<T> | T): Promise<Result<T>> {
  try {
    return { ok: true, value: await fn() };
  } catch (e) {
    if (e instanceof HttpError) {
      const r: Result<T> = { ok: false, code: e.code, message: e.message };
      if (e.retryAfter !== undefined) r.retryAfter = e.retryAfter;
      return r;
    }
    throw e;
  }
}

export function unwrap<T>(r: Result<T>): T {
  if (r.ok) return r.value;
  throw new HttpError(r.code, r.message, r.retryAfter);
}
