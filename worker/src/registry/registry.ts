import { DurableObject } from "cloudflare:workers";
import { createCloudflareClient, type CloudflareClient } from "../cloudflare";
import { loadConfig, type Config } from "../config";
import { workerSigner, type Signer } from "../crypto";
import type { Env } from "../env";
import { HttpError, toResult, type Result } from "../errors";
import { bwAuthorize, bwRecord, challengeTick, type BwRecord } from "../challenge/scheduler";
import {
  activate,
  banNode,
  closeSockets,
  consumeNonce,
  deregister,
  registerBegin,
  registerConfirm,
  sweepRegistrations,
  unbanNode,
  type AuthCtx,
  type BeginInput,
  type BeginResult,
  type ConfirmInput,
  type ConfirmResult,
} from "./registration";
import { broadcastDirectory, ensureDirectory, type SignedDirectory } from "./directory";
import { publishDirectory, publishSnapshot } from "./snapshot";
import { scoreOf, isLive } from "./scoring";
import { all, migrate, one, type NodeRow } from "./store";
import { handleMessage, onConnect, type CfInfo } from "./ws";
import { log, type RegCtx } from "./context";

export interface NodeSummary {
  node_id: string;
  hostname: string;
  status: string;
  genesis: boolean;
  live: boolean;
  asn: number | null;
  country: string | null;
  claimed: unknown;
  measured: { cpu_hps: number | null; ram_mb: number | null; storage_gb: number | null; bandwidth_mbps: number | null };
  health: number;
  strikes: number;
  challenges_ok: number;
  challenges_total: number;
  score: { capacity: number; trust: number; base: number; net: number };
  ban_reason: string | null;
}

/**
 * Registr nodů, heartbeat, adresář, challenge a skóre. Jedna globální instance („main“);
 * shardování podle regionu je otevřený bod 10 v ARCHITECTURE.md.
 */
export class Registry extends DurableObject<Env> {
  cfg: Config;
  clock: () => number = () => Date.now();
  cloudflareClient: CloudflareClient | null = null;
  fetchImpl: typeof fetch | null = null;

  private readonly rc: RegCtx;
  private ticking = false;

  constructor(ctx: DurableObjectState, env: Env) {
    super(ctx, env);
    this.cfg = loadConfig(env);
    ctx.blockConcurrencyWhile(async () => migrate(ctx.storage.sql));
    ctx.setWebSocketAutoResponse(new WebSocketRequestResponsePair("ping", "pong"));
    const self = this;
    this.rc = {
      sql: ctx.storage.sql,
      env,
      get cfg() {
        return self.cfg;
      },
      nowS: () => Math.floor(self.clock() / 1000),
      nowMs: () => self.clock(),
      cloudflare: () => {
        self.cloudflareClient ??= createCloudflareClient(env, self.fetchImpl ?? fetch);
        return self.cloudflareClient;
      },
      fetcher: () => self.fetchImpl ?? fetch,
      signer: (): Promise<Signer> => workerSigner(env.WORKER_SIGNING_KEY),
      sockets: (nodeId) => (nodeId ? ctx.getWebSockets(nodeId) : ctx.getWebSockets()),
    };
  }

  /* -------------------------------------------------------- registrace (RPC) */

  async registerBegin(input: BeginInput): Promise<Result<BeginResult>> {
    return toResult(() => registerBegin(this.rc, input));
  }

  async registerConfirm(input: ConfirmInput): Promise<Result<ConfirmResult>> {
    return toResult(async () => {
      const r = await registerConfirm(this.rc, input);
      await this.ensureAlarm();
      return r;
    });
  }

  async activate(auth: AuthCtx): Promise<Result<{ status: "active" }>> {
    return toResult(async () => {
      const r = await activate(this.rc, auth);
      await this.ensureAlarm();
      return r;
    });
  }

  async deregister(auth: AuthCtx): Promise<Result<{ status: "retired" }>> {
    return toResult(() => deregister(this.rc, auth));
  }

  /* ------------------------------------------------------------ adresář (RPC) */

  async getDirectory(): Promise<Result<SignedDirectory>> {
    return toResult(async () => {
      const { directory, created } = await ensureDirectory(this.rc);
      if (created) await publishDirectory(this.env, directory);
      return directory;
    });
  }

  /* ------------------------------------------------------------ WebSocket */

  override async fetch(request: Request): Promise<Response> {
    const url = new URL(request.url);
    if (url.pathname !== "/ws" || request.headers.get("upgrade") !== "websocket") {
      return new Response("not found", { status: 404 });
    }
    // hlavičky nastavuje výhradně Worker po ověření podpisu; DO není dostupné zvenčí
    const pubkey = request.headers.get("x-ancord-pk");
    const replayKey = request.headers.get("x-ancord-replay");
    if (!pubkey || !replayKey) return new Response("missing identity", { status: 400 });
    let cf: CfInfo = {};
    try {
      cf = JSON.parse(request.headers.get("x-ancord-cf") ?? "{}") as CfInfo;
    } catch {
      /* poloha není povinná */
    }
    try {
      const node = this.authorizedNode(pubkey, replayKey);
      closeSockets(this.rc, node.node_id, 4000, "replaced");
      const pair = new WebSocketPair();
      const [client, server] = [pair[0], pair[1]];
      this.ctx.acceptWebSocket(server, [node.node_id]);
      server.serializeAttachment({ nodeId: node.node_id });
      onConnect(this.rc, node, server, cf);
      await this.ensureAlarm();
      return new Response(null, { status: 101, webSocket: client });
    } catch (e) {
      if (e instanceof HttpError) {
        return new Response(JSON.stringify({ error: { code: e.code, message: e.message } }), {
          status: e.status,
          headers: { "content-type": "application/json" },
        });
      }
      throw e;
    }
  }

  private authorizedNode(pubkey: string, replayKey: string): NodeRow {
    const row = one<NodeRow & Record<string, SqlStorageValue>>(this.rc.sql, "SELECT * FROM nodes WHERE pubkey = ?", pubkey);
    if (!row) throw new HttpError("forbidden", "unknown node");
    consumeNonce(this.rc, replayKey);
    if (row.status !== "active") throw new HttpError("forbidden", `node is ${row.status}`);
    return row;
  }

  override async webSocketMessage(ws: WebSocket, message: string | ArrayBuffer): Promise<void> {
    const att = ws.deserializeAttachment() as { nodeId?: string } | null;
    if (!att?.nodeId) {
      ws.close(1008, "unidentified");
      return;
    }
    await handleMessage(this.rc, ws, att.nodeId, message);
  }

  override async webSocketClose(ws: WebSocket, code: number, reason: string): Promise<void> {
    try {
      ws.close(code === 1005 || code === 1006 ? 1000 : code, reason.slice(0, 100));
    } catch {
      /* už zavřeno */
    }
  }

  override async webSocketError(ws: WebSocket, error: unknown): Promise<void> {
    log("ws_error", { error: String(error) });
    try {
      ws.close(1011, "error");
    } catch {
      /* už zavřeno */
    }
  }

  /* ------------------------------------------------------ bandwidth (RPC) */

  async bwAuthorize(id: string, dir: "dl" | "ul"): Promise<{ key: string; bytes: number } | null> {
    return bwAuthorize(this.rc, id, dir);
  }

  async bwRecord(id: string, dir: "dl" | "ul", rec: BwRecord): Promise<void> {
    await bwRecord(this.rc, id, dir, rec);
  }

  /* ------------------------------------------------------------ správa (RPC) */

  async adminNodes(): Promise<NodeSummary[]> {
    const nowS = this.rc.nowS();
    return all<NodeRow & Record<string, SqlStorageValue>>(this.rc.sql, "SELECT * FROM nodes ORDER BY node_id").map((n) => {
      const sc = scoreOf(this.cfg, n, nowS);
      return {
        node_id: n.node_id,
        hostname: n.hostname,
        status: n.status,
        genesis: n.genesis === 1,
        live: isLive(this.cfg, n, nowS),
        asn: n.asn,
        country: n.country,
        claimed: JSON.parse(n.claimed),
        measured: { cpu_hps: n.m_cpu_hps, ram_mb: n.m_ram_mb, storage_gb: n.m_storage_gb, bandwidth_mbps: n.m_bw_mbps },
        health: n.health,
        strikes: n.strikes,
        challenges_ok: n.ch_ok,
        challenges_total: n.ch_total,
        score: { capacity: sc.capacity, trust: sc.trust, base: sc.base, net: sc.net },
        ban_reason: n.ban_reason,
      };
    });
  }

  async adminBan(nodeId: string, reason: string): Promise<boolean> {
    return banNode(this.rc, nodeId, reason);
  }

  async adminUnban(nodeId: string): Promise<boolean> {
    return unbanNode(this.rc, nodeId);
  }

  /* ----------------------------------------------------------------- tik */

  /** Volá cron i alarm. Každý krok je izolovaný, aby jedna chyba nezastavila ostatní. */
  async tick(): Promise<void> {
    if (this.ticking) return; // cron a alarm se nesmí překrývat
    this.ticking = true;
    try {
      await this.runTick();
    } finally {
      this.ticking = false;
    }
  }

  private async runTick(): Promise<void> {
    const steps: [string, () => Promise<unknown>][] = [
      ["sweep", () => sweepRegistrations(this.rc)],
      [
        "directory",
        async () => {
          const { directory, created } = await ensureDirectory(this.rc);
          if (created) {
            broadcastDirectory(this.rc, directory);
            await publishDirectory(this.env, directory);
          }
        },
      ],
      ["challenges", () => challengeTick(this.rc)],
      ["snapshot", () => publishSnapshot(this.rc)],
    ];
    for (const [name, step] of steps) {
      try {
        await step();
      } catch (e) {
        log("tick_step_failed", { step: name, error: String(e) });
      }
    }
    await this.rescheduleAlarm();
  }

  override async alarm(): Promise<void> {
    await this.tick();
  }

  private async ensureAlarm(): Promise<void> {
    if ((await this.ctx.storage.getAlarm()) === null) {
      await this.ctx.storage.setAlarm(this.clock() + this.cfg.tickIntervalS * 1000);
    }
  }

  /** Alarm běží, dokud existuje node, který se může objevit nebo žít; jinak ho znovu spustí cron. */
  private async rescheduleAlarm(): Promise<void> {
    const busy = one<{ c: number }>(
      this.rc.sql,
      "SELECT COUNT(*) AS c FROM nodes WHERE status IN ('provisioning','pending','active')",
    )!.c;
    if (busy > 0) await this.ctx.storage.setAlarm(this.clock() + this.cfg.tickIntervalS * 1000);
    else await this.ctx.storage.deleteAlarm();
  }
}

