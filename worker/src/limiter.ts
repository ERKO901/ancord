import { DurableObject } from "cloudflare:workers";
import type { Env } from "./env";

export interface AdmitOptions {
  /** Nonce proti replayi; vynechat, pokud se nekontroluje. */
  nonce?: string;
  nonceExpiresMs?: number;
  capacity: number;
  refillPerS: number;
  cost?: number;
}

export type AdmitResult = { status: "ok" } | { status: "replay" } | { status: "limited"; retryAfterS: number };

/**
 * Token bucket a detekce replaye pro jeden klíč (node nebo push_id). Jedna instance DO = jeden klíč,
 * takže stav je silně konzistentní a nezávislý na ostatních klíčích.
 */
export class RateLimiter extends DurableObject<Env> {
  constructor(ctx: DurableObjectState, env: Env) {
    super(ctx, env);
    ctx.storage.sql.exec(`
      CREATE TABLE IF NOT EXISTS bucket (id INTEGER PRIMARY KEY CHECK (id = 1), tokens REAL NOT NULL, ts INTEGER NOT NULL);
      CREATE TABLE IF NOT EXISTS nonces (nonce TEXT PRIMARY KEY, expires_ms INTEGER NOT NULL);
    `);
  }

  now(): number {
    return Date.now();
  }

  async admit(opts: AdmitOptions): Promise<AdmitResult> {
    const sql = this.ctx.storage.sql;
    const now = this.now();
    if (opts.nonce !== undefined) {
      sql.exec("DELETE FROM nonces WHERE expires_ms < ?", now);
      const seen = sql.exec("SELECT 1 FROM nonces WHERE nonce = ?", opts.nonce).toArray();
      if (seen.length > 0) return { status: "replay" };
    }
    const cost = opts.cost ?? 1;
    const row = sql.exec<{ tokens: number; ts: number }>("SELECT tokens, ts FROM bucket WHERE id = 1").toArray()[0];
    const tokens = row
      ? Math.min(opts.capacity, row.tokens + ((now - row.ts) / 1000) * opts.refillPerS)
      : opts.capacity;
    if (tokens < cost) {
      sql.exec("INSERT OR REPLACE INTO bucket (id, tokens, ts) VALUES (1, ?, ?)", tokens, now);
      return { status: "limited", retryAfterS: Math.ceil((cost - tokens) / opts.refillPerS) };
    }
    sql.exec("INSERT OR REPLACE INTO bucket (id, tokens, ts) VALUES (1, ?, ?)", tokens - cost, now);
    // nonce se zapisuje až po úspěšném odečtení, aby limitovaný request šlo zopakovat se stejným podpisem
    if (opts.nonce !== undefined) {
      sql.exec(
        "INSERT INTO nonces (nonce, expires_ms) VALUES (?, ?)",
        opts.nonce,
        opts.nonceExpiresMs ?? now + 120_000,
      );
    }
    return { status: "ok" };
  }
}
