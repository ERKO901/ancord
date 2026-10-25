import { HttpError } from "./errors";
import { timeoutSignal } from "./http";
import type { Env } from "./env";

/** Značka, kterou Worker píše do DNS komentáře; maže se jen to, co ji nese. */
export const MANAGED_COMMENT = "ancord-managed";
export const TUNNEL_NAME_PREFIX = "ancord-";

export interface CloudflareClient {
  createTunnel(name: string): Promise<{ id: string; token: string }>;
  configureTunnel(tunnelId: string, hostname: string, service: string): Promise<void>;
  getTunnelToken(tunnelId: string): Promise<string>;
  /** Vrací id DNS záznamu. Selže (`conflict`), pokud jméno už existuje. */
  createDnsCname(hostname: string, tunnelId: string): Promise<string>;
  deleteDns(recordId: string, hostname: string): Promise<void>;
  deleteTunnel(tunnelId: string, name: string): Promise<void>;
}

/** Hostname, který smí Worker spravovat: `node<16 znaků base32>.<zóna>`. */
export function isManagedHostname(hostname: string, zone: string): boolean {
  const suffix = `.${zone}`;
  if (!hostname.endsWith(suffix)) return false;
  return /^node[a-z2-7]{16}$/.test(hostname.slice(0, -suffix.length));
}

interface ApiEnvelope<T> {
  success: boolean;
  errors?: { code: number; message: string }[];
  result: T;
}

export class RealCloudflareClient implements CloudflareClient {
  private static readonly API = "https://api.cloudflare.com/client/v4";

  constructor(
    private readonly token: string,
    private readonly accountId: string,
    private readonly zoneId: string,
    private readonly zone: string,
    private readonly fetcher: typeof fetch = fetch,
  ) {
    if (!token || !accountId || !zoneId) throw new Error("Cloudflare client is not configured");
  }

  private async call<T>(method: string, path: string, body?: unknown, okStatuses: number[] = []): Promise<T | null> {
    const init: RequestInit = {
      method,
      headers: { authorization: `Bearer ${this.token}`, "content-type": "application/json" },
      signal: timeoutSignal(15_000),
    };
    if (body !== undefined) init.body = JSON.stringify(body);
    let lastErr: unknown;
    for (let attempt = 0; attempt < 3; attempt++) {
      try {
        const res = await this.fetcher(`${RealCloudflareClient.API}${path}`, init);
        if (okStatuses.includes(res.status)) return null;
        if (res.status === 429 || res.status >= 500) {
          lastErr = new Error(`Cloudflare API ${res.status}`);
          await new Promise((r) => setTimeout(r, 250 * 2 ** attempt));
          continue;
        }
        const data = (await res.json()) as ApiEnvelope<T>;
        if (!res.ok || !data.success) {
          const msg = data.errors?.map((e) => `${e.code}: ${e.message}`).join("; ") ?? `HTTP ${res.status}`;
          throw new HttpError("upstream", `Cloudflare API error: ${msg}`);
        }
        return data.result;
      } catch (e) {
        if (e instanceof HttpError) throw e;
        lastErr = e;
        await new Promise((r) => setTimeout(r, 250 * 2 ** attempt));
      }
    }
    throw new HttpError("upstream", `Cloudflare API unreachable: ${String(lastErr)}`);
  }

  async createTunnel(name: string): Promise<{ id: string; token: string }> {
    if (!name.startsWith(TUNNEL_NAME_PREFIX)) throw new Error("refusing to create tunnel with foreign name");
    const r = await this.call<{ id: string; token?: string }>("POST", `/accounts/${this.accountId}/cfd_tunnel`, {
      name,
      config_src: "cloudflare",
    });
    if (!r) throw new HttpError("upstream", "empty tunnel response");
    const token = r.token ?? (await this.getTunnelToken(r.id));
    return { id: r.id, token };
  }

  async configureTunnel(tunnelId: string, hostname: string, service: string): Promise<void> {
    if (!isManagedHostname(hostname, this.zone)) throw new Error("refusing to configure foreign hostname");
    await this.call("PUT", `/accounts/${this.accountId}/cfd_tunnel/${tunnelId}/configurations`, {
      config: { ingress: [{ hostname, service }, { service: "http_status:404" }] },
    });
  }

  async getTunnelToken(tunnelId: string): Promise<string> {
    const t = await this.call<string>("GET", `/accounts/${this.accountId}/cfd_tunnel/${tunnelId}/token`);
    if (typeof t !== "string" || !t) throw new HttpError("upstream", "empty tunnel token");
    return t;
  }

  async createDnsCname(hostname: string, tunnelId: string): Promise<string> {
    if (!isManagedHostname(hostname, this.zone)) throw new Error("refusing to write foreign DNS name");
    // před zápisem přečíst existující záznamy (sdílená zóna)
    const existing = await this.call<{ id: string }[]>(
      "GET",
      `/zones/${this.zoneId}/dns_records?name=${encodeURIComponent(hostname)}`,
    );
    if (existing && existing.length > 0) throw new HttpError("conflict", `DNS name ${hostname} already exists`);
    const r = await this.call<{ id: string }>("POST", `/zones/${this.zoneId}/dns_records`, {
      type: "CNAME",
      name: hostname,
      content: `${tunnelId}.cfargotunnel.com`,
      proxied: true,
      ttl: 1,
      comment: MANAGED_COMMENT,
    });
    if (!r) throw new HttpError("upstream", "empty DNS response");
    return r.id;
  }

  async deleteDns(recordId: string, hostname: string): Promise<void> {
    if (!isManagedHostname(hostname, this.zone)) throw new Error("refusing to delete foreign DNS name");
    const rec = await this.call<{ id: string; name: string; comment?: string | null }>(
      "GET",
      `/zones/${this.zoneId}/dns_records/${recordId}`,
      undefined,
      [404],
    );
    if (!rec) return; // už neexistuje
    if (rec.name !== hostname || rec.comment !== MANAGED_COMMENT) {
      throw new Error(`refusing to delete DNS record ${recordId}: not managed by ancord`);
    }
    await this.call("DELETE", `/zones/${this.zoneId}/dns_records/${recordId}`, undefined, [404]);
  }

  async deleteTunnel(tunnelId: string, name: string): Promise<void> {
    const t = await this.call<{ id: string; name: string }>(
      "GET",
      `/accounts/${this.accountId}/cfd_tunnel/${tunnelId}`,
      undefined,
      [404],
    );
    if (!t) return;
    if (t.name !== name || !t.name.startsWith(TUNNEL_NAME_PREFIX)) {
      throw new Error(`refusing to delete tunnel ${tunnelId}: not managed by ancord`);
    }
    await this.call("DELETE", `/accounts/${this.accountId}/cfd_tunnel/${tunnelId}/connections`, undefined, [404]);
    await this.call("DELETE", `/accounts/${this.accountId}/cfd_tunnel/${tunnelId}`, undefined, [404]);
  }
}

export interface MockCall {
  op: string;
  args: unknown[];
}

/** Paměťová implementace pro testy a lokální vývoj (`CLOUDFLARE_MODE=mock`). */
export class MockCloudflareClient implements CloudflareClient {
  readonly tunnels = new Map<string, { name: string; hostname?: string; service?: string }>();
  readonly dns = new Map<string, { hostname: string; tunnelId: string }>();
  /** Jména, která v „zóně“ existují bez našeho přičinění (cizí záznamy). */
  readonly foreignNames = new Set<string>();
  readonly calls: MockCall[] = [];
  /** Operace, které mají selhat (pro testy kompenzace). */
  readonly failOn = new Set<string>();
  private seq = 0;

  private enter(op: string, ...args: unknown[]): void {
    this.calls.push({ op, args });
    if (this.failOn.has(op)) throw new HttpError("upstream", `mock failure in ${op}`);
  }

  async createTunnel(name: string): Promise<{ id: string; token: string }> {
    this.enter("createTunnel", name);
    const id = `tun-${++this.seq}`;
    this.tunnels.set(id, { name });
    return { id, token: `token-for-${id}` };
  }

  async configureTunnel(tunnelId: string, hostname: string, service: string): Promise<void> {
    this.enter("configureTunnel", tunnelId, hostname, service);
    const t = this.tunnels.get(tunnelId);
    if (!t) throw new HttpError("upstream", "unknown tunnel");
    t.hostname = hostname;
    t.service = service;
  }

  async getTunnelToken(tunnelId: string): Promise<string> {
    this.enter("getTunnelToken", tunnelId);
    if (!this.tunnels.has(tunnelId)) throw new HttpError("upstream", "unknown tunnel");
    return `token-for-${tunnelId}`;
  }

  async createDnsCname(hostname: string, tunnelId: string): Promise<string> {
    this.enter("createDnsCname", hostname, tunnelId);
    if (this.foreignNames.has(hostname) || [...this.dns.values()].some((d) => d.hostname === hostname)) {
      throw new HttpError("conflict", `DNS name ${hostname} already exists`);
    }
    const id = `dns-${++this.seq}`;
    this.dns.set(id, { hostname, tunnelId });
    return id;
  }

  async deleteDns(recordId: string, hostname: string): Promise<void> {
    this.enter("deleteDns", recordId, hostname);
    const r = this.dns.get(recordId);
    if (!r) return;
    if (r.hostname !== hostname) throw new Error("refusing to delete foreign DNS record");
    this.dns.delete(recordId);
  }

  async deleteTunnel(tunnelId: string, name: string): Promise<void> {
    this.enter("deleteTunnel", tunnelId, name);
    const t = this.tunnels.get(tunnelId);
    if (!t) return;
    if (t.name !== name) throw new Error("refusing to delete foreign tunnel");
    this.tunnels.delete(tunnelId);
  }
}

let sharedMock: MockCloudflareClient | undefined;
export function sharedMockClient(): MockCloudflareClient {
  return (sharedMock ??= new MockCloudflareClient());
}

export function createCloudflareClient(env: Env, fetcher: typeof fetch = fetch): CloudflareClient {
  if (env.CLOUDFLARE_MODE === "real") {
    return new RealCloudflareClient(env.CF_API_TOKEN ?? "", env.CF_ACCOUNT_ID, env.CF_ZONE_ID, env.ZONE_DOMAIN, fetcher);
  }
  if (env.CLOUDFLARE_MODE === "mock") return sharedMockClient();
  throw new Error(`CLOUDFLARE_MODE must be 'real' or 'mock', got '${env.CLOUDFLARE_MODE}'`);
}
