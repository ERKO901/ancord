import { exports } from "cloudflare:workers";
import { DOMAIN, nodeIdOf, hostnameOf, sha256, signingInput } from "../../src/crypto";
import { b64uDecodeLen, b64uEncode, concat, randomBytes, utf8 } from "../../src/encoding";
import { requestPayload } from "../../src/auth";
import { canonicalBytes, type Json } from "../../src/canonical";
import { solveHashcash } from "../../src/hashcash";
import { computeCheckpoints } from "../../src/challenge/cpu";
import { buildTable, buildTree, makeOpening, LEAF_BYTES, spaceT0 } from "../../src/challenge/space";

const PKCS8_PREFIX = new Uint8Array([0x30, 0x2e, 0x02, 0x01, 0x00, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x04, 0x22, 0x04, 0x20]);

export const ZONE = "melonet.cz";

/** Posun času pro testy; Worker, Registry i simulované nody ho respektují. */
export const testClock = { offsetMs: 0 };
export const nowS = () => Math.floor((Date.now() + testClock.offsetMs) / 1000);
export const BASE = "https://ancord.test";

export interface Claimed {
  cpu_cores: number;
  ram_mb: number;
  storage_gb: number;
  bandwidth_mbps: number;
}

export const GOOD_SPECS: Claimed = { cpu_cores: 4, ram_mb: 8192, storage_gb: 200, bandwidth_mbps: 500 };

export interface Behavior {
  /** zpoždění odpovědi na CPU výzvu (simuluje pomalejší procesor) */
  cpuDelayMs?: number;
  /** nepočítá nic, vrací vymyšlené checkpointy */
  cpuFake?: boolean;
  /** část tabulky, kterou skutečně drží (0–1); zbytek vymýšlí */
  spaceFraction?: number;
  /** zpoždění otevření (latence disku) */
  spaceDelayMs?: number;
  /** vynechá bandwidth */
  skipBandwidth?: boolean;
  /** vrací špatný hash blobu */
  bandwidthWrongHash?: boolean;
  /** neodpovídá na výzvy vůbec */
  silent?: boolean;
  /** doba přenosu hlášená jako peer witness */
  witnessMs?: number;
}

export class TestNode {
  readonly nodeId: string;
  readonly hostname: string;
  behavior: Behavior = {};
  seq = 0;
  ws: WebSocket | null = null;
  inbox: Record<string, any>[] = [];
  waiters: { pred: (m: Record<string, any>) => boolean; resolve: (m: Record<string, any>) => void }[] = [];
  tables = new Map<string, { seed: Uint8Array; table: Uint8Array; levels: Uint8Array[][]; leaves: number }>();
  challengesSeen: string[] = [];
  autoRespond = true;
  /** Odpovídá `GET /health` přes simulovaný tunel (ověření dosažitelnosti, spec §6.1). */
  reachable = true;

  private constructor(
    readonly seed: Uint8Array,
    readonly pub: Uint8Array,
    private readonly key: CryptoKey,
    readonly specs: Claimed,
    readonly asn: number | null,
  ) {
    this.nodeId = nodeIdOf(pub);
    this.hostname = hostnameOf(this.nodeId, ZONE);
  }

  static async create(opts: { seed?: Uint8Array; specs?: Claimed; asn?: number | null } = {}): Promise<TestNode> {
    const seed = opts.seed ?? randomBytes(32);
    const key = await crypto.subtle.importKey("pkcs8", concat(PKCS8_PREFIX, seed), { name: "Ed25519" }, true, ["sign"]);
    const jwk = (await crypto.subtle.exportKey("jwk", key)) as JsonWebKey;
    const pub = b64uDecodeLen(jwk.x!, 32)!;
    return new TestNode(seed, pub, key, opts.specs ?? GOOD_SPECS, opts.asn === undefined ? null : opts.asn);
  }

  get pubB64(): string {
    return b64uEncode(this.pub);
  }

  async sign(domain: (typeof DOMAIN)[keyof typeof DOMAIN], payload: Uint8Array): Promise<Uint8Array> {
    return new Uint8Array(await crypto.subtle.sign({ name: "Ed25519" }, this.key, signingInput(domain, payload)));
  }

  async authHeader(method: string, pathAndQuery: string, body: Uint8Array = new Uint8Array(0), t = nowS(), n = b64uEncode(randomBytes(16))): Promise<string> {
    const sig = await this.sign(DOMAIN.req, requestPayload(method, pathAndQuery, t, n, body));
    return `Ancord-Ed25519 k=${this.pubB64},t=${t},n=${n},s=${b64uEncode(sig)}`;
  }

  cf(): Record<string, unknown> | undefined {
    return this.asn === null ? undefined : { asn: this.asn };
  }

  async http(method: string, path: string, body?: unknown, extra: Record<string, string> = {}): Promise<Response> {
    const bytes = body === undefined ? new Uint8Array(0) : utf8(JSON.stringify(body));
    return exports.default.fetch(
      new Request(`${BASE}${path}`, {
        method,
        headers: { "content-type": "application/json", ...extra },
        body: method === "GET" ? undefined : bytes,
        cf: this.cf(),
      } as RequestInit),
    );
  }

  async signed(method: string, path: string, body?: unknown): Promise<Response> {
    const bytes = body === undefined ? new Uint8Array(0) : utf8(JSON.stringify(body));
    const auth = await this.authHeader(method, path, bytes);
    return exports.default.fetch(
      new Request(`${BASE}${path}`, { method, headers: { authorization: auth, "content-type": "application/json" }, body: method === "GET" ? undefined : bytes, cf: this.cf() } as RequestInit),
    );
  }

  /** Celá registrace až do stavu `pending`. Vrací odpověď potvrzení. */
  async register(): Promise<{ begin: any; confirm: any; res: Response }> {
    const beginRes = await this.http("POST", "/register", { pubkey: this.pubB64, claimed_specs: this.specs });
    const begin: any = await beginRes.json();
    if (beginRes.status !== 200) return { begin, confirm: begin, res: beginRes };
    const nonce = b64uDecodeLen(begin.challenge_nonce, 32)!;
    const sig = await this.sign(DOMAIN.register, concat(nonce, utf8(begin.hostname), this.pub));
    const body: Record<string, unknown> = { pubkey: this.pubB64, challenge_nonce: begin.challenge_nonce, signature: b64uEncode(sig) };
    if (begin.hashcash_bits > 0) body.hashcash_counter = solveHashcash(nonce, this.pub, begin.hashcash_bits);
    const res = await this.http("POST", "/register/confirm", body);
    return { begin, confirm: await res.json(), res };
  }

  /** Odpověď, kterou by node dal na `GET /.well-known/melo-proof?c=…`. */
  async proofResponse(url: string): Promise<Response> {
    const c = new URL(url).searchParams.get("c");
    const challenge = c ? b64uDecodeLen(c, 32) : null;
    if (!challenge) return new Response("bad", { status: 400 });
    const sig = await this.sign(DOMAIN.proof, concat(challenge, utf8(this.hostname), this.pub));
    return Response.json({ pubkey: this.pubB64, signature: b64uEncode(sig) });
  }

  async activate(): Promise<Response> {
    return this.signed("POST", "/register/activate", {});
  }

  /* --------------------------------------------------------------- WebSocket */

  async connect(): Promise<Record<string, any>> {
    const auth = await this.authHeader("GET", "/ws");
    const res = await exports.default.fetch(
      new Request(`${BASE}/ws`, { headers: { upgrade: "websocket", authorization: auth }, cf: this.cf() } as RequestInit),
    );
    if (res.status !== 101 || !res.webSocket) throw new Error(`ws upgrade failed: ${res.status} ${await res.text()}`);
    const ws = res.webSocket;
    ws.accept();
    ws.addEventListener("message", (ev: MessageEvent) => void this.onMessage(ev.data as string));
    this.ws = ws;
    return this.waitFor((m) => m.t === "welcome");
  }

  close(): void {
    try {
      this.ws?.close(1000, "test done");
    } catch {
      /* už zavřeno */
    }
    this.ws = null;
  }

  send(msg: Record<string, unknown>): void {
    if (!this.ws) return;
    this.ws.send(JSON.stringify(msg));
  }

  waitFor(pred: (m: Record<string, any>) => boolean, timeoutMs = 10_000): Promise<Record<string, any>> {
    const hit = this.inbox.findIndex(pred);
    if (hit >= 0) return Promise.resolve(this.inbox.splice(hit, 1)[0]!);
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => reject(new Error("timeout waiting for message")), timeoutMs);
      this.waiters.push({
        pred,
        resolve: (m) => {
          clearTimeout(timer);
          resolve(m);
        },
      });
    });
  }

  async heartbeat(load = { conns: 1, max_conns: 100, cpu_pct: 5, mem_pct: 10 }, epoch?: number, overrides: Record<string, unknown> = {}): Promise<Record<string, any>> {
    const body: Record<string, unknown> = { t: "hb", seq: ++this.seq, ts: nowS(), load, ...overrides };
    if (epoch !== undefined) body.epoch = epoch;
    const sig = await this.sign(DOMAIN.heartbeat, canonicalBytes(body as Json));
    this.send({ ...body, sig: b64uEncode(sig) });
    return this.waitFor((m) => m.t === "ack" || m.t === "error");
  }

  private async onMessage(data: string): Promise<void> {
    if (typeof data === "string" && data === "pong") return;
    const m = JSON.parse(data) as Record<string, any>;
    if (this.autoRespond && !this.behavior.silent) {
      if (m.t === "ping") {
        this.send({ t: "pong", id: m.id });
        return;
      }
      if (m.t === "challenge") {
        this.challengesSeen.push(m.kind);
        const resp = await this.answer(m);
        if (resp) this.send({ t: "challenge_response", id: m.id, ...resp });
        return;
      }
    }
    const w = this.waiters.findIndex((x) => x.pred(m));
    if (w >= 0) this.waiters.splice(w, 1)[0]!.resolve(m);
    else this.inbox.push(m);
  }

  /* --------------------------------------------------------- challenge (node) */

  private async answer(m: Record<string, any>): Promise<Record<string, unknown> | null> {
    const b = this.behavior;
    switch (m.kind) {
      case "cpu": {
        if (b.cpuDelayMs) await new Promise((r) => setTimeout(r, b.cpuDelayMs));
        if (b.cpuFake) {
          const per = m.iters / m.every;
          return { checkpoints: Array.from({ length: m.lanes }, () => Array.from({ length: per }, () => b64uEncode(randomBytes(32)))) };
        }
        return { checkpoints: computeCheckpoints(b64uDecodeLen(m.seed, 32)!, m.lanes, m.iters, m.every) };
      }
      case "space_commit": {
        const seed = b64uDecodeLen(m.seed, 32)!;
        const table = buildTable(seed, m.leaves);
        const levels = buildTree(table);
        this.tables.set(m.tier, { seed, table, levels, leaves: m.leaves });
        return { root: b64uEncode(levels[levels.length - 1]![0]!) };
      }
      case "space_open": {
        const t = this.tables.get(m.tier);
        if (!t) return { error: "no commitment" };
        if (b.spaceDelayMs) await new Promise((r) => setTimeout(r, b.spaceDelayMs));
        const keep = Math.floor(t.leaves * (b.spaceFraction ?? 1));
        const openings = (m.indices as number[]).map((i) => {
          const o = makeOpening(t.seed, t.table, t.levels, i);
          if (i >= keep) {
            // nemá data: vymyslí list
            return { ...o, leaf: b64uEncode(randomBytes(32)) };
          }
          return o;
        });
        return { openings };
      }
      case "bandwidth": {
        if (b.skipBandwidth) return null;
        const local = (u: string) => `${BASE}${new URL(u).pathname}`;
        const dl = await exports.default.fetch(new Request(local(m.dl_url)));
        const bytes = new Uint8Array(await dl.arrayBuffer());
        await exports.default.fetch(new Request(local(m.ul_url), { method: "PUT", body: bytes }));
        const hash = b.bandwidthWrongHash ? randomBytes(32) : sha256(bytes);
        return { sha256: b64uEncode(hash) };
      }
      case "witness": {
        const report = { target_id: m.target_id, witness_id: this.nodeId, bytes: m.bytes, duration_ms: this.behavior.witnessMs ?? 200, ts: nowS() };
        const sig = await this.sign(DOMAIN.report, canonicalBytes(report as Json));
        return { report, sig: b64uEncode(sig) };
      }
      default:
        return { error: `unsupported ${m.kind}` };
    }
  }
}

export { spaceT0, LEAF_BYTES };
