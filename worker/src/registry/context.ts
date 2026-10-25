import type { CloudflareClient } from "../cloudflare";
import type { Config } from "../config";
import type { Signer } from "../crypto";
import type { Env } from "../env";

/** Společné rozhraní, přes které moduly Registry přistupují ke stavu DO. */
export interface RegCtx {
  readonly sql: SqlStorage;
  readonly env: Env;
  readonly cfg: Config;
  nowS(): number;
  nowMs(): number;
  cloudflare(): CloudflareClient;
  fetcher(): typeof fetch;
  signer(): Promise<Signer>;
  /** Živé WebSockety (volitelně jen pro daný node). */
  sockets(nodeId?: string): WebSocket[];
}

export function log(event: string, fields: Record<string, unknown> = {}): void {
  console.log(JSON.stringify({ event, ...fields }));
}
