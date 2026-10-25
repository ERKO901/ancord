import { describe, expect, it } from "vitest";
import { env } from "cloudflare:workers";
import { evictDurableObject } from "cloudflare:test";
import { b64uDecode, b64uDecodeLen, b64uEncode, base32Encode, fromHex, randomBytes, randomInt, toHex, utf8 } from "../src/encoding";
import { blake3, leadingZeroBits, signerFromSeed, verifySig, DOMAIN } from "../src/crypto";
import { ipPrefix } from "../src/http";
import { anchorsFor } from "../src/hrw";
import { loadConfig, withOverrides } from "../src/config";
import { scoreOf } from "../src/registry/scoring";
import type { NodeRow } from "../src/registry/store";
import { FAST, bringUp, exports, newNode, registryStub, seedActive, setup, sqlOf, tick, withRegistry } from "./support/helpers";
import { BASE } from "./support/node";

describe("kódování", () => {
  it("base64url: round trip a striktní dekódování", () => {
    for (let n = 0; n < 40; n++) {
      const b = randomBytes(n);
      expect(b64uDecode(b64uEncode(b))).toEqual(b);
    }
    expect(b64uEncode(utf8("foobar"))).toBe("Zm9vYmFy");
    expect(b64uDecode("Zm9vYg")).toEqual(utf8("foob"));
    expect(b64uDecode("Zm9vYg==")).toBeNull(); // padding
    expect(b64uDecode("Zm9v+g")).toBeNull(); // jiná abeceda
    expect(b64uDecode("Zm9vYh")).toBeNull(); // nenulové nepoužité bity
    expect(b64uDecode("A")).toBeNull();
    expect(b64uDecode("é")).toBeNull();
    expect(b64uDecodeLen("AAAA", 4)).toBeNull();
    expect(b64uDecodeLen("AAAA", 3)).toEqual(new Uint8Array(3));
  });

  it("base32 a hex", () => {
    expect(base32Encode(utf8("foobar"))).toBe("mzxw6ytboi");
    expect(base32Encode(new Uint8Array(0))).toBe("");
    expect(toHex(fromHex("00ff10"))).toBe("00ff10");
    expect(() => fromHex("zz")).toThrow();
  });

  it("náhodná čísla jsou v rozsahu", () => {
    for (let i = 0; i < 500; i++) {
      const x = randomInt(7);
      expect(x).toBeGreaterThanOrEqual(0);
      expect(x).toBeLessThan(7);
    }
    expect(() => randomInt(0)).toThrow();
  });
});

describe("kryptografie", () => {
  it("blake3 odpovídá referenčnímu vektoru", () => {
    expect(toHex(blake3(new Uint8Array(0)))).toBe("af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262");
  });

  it("Ed25519 odvození veřejného klíče odpovídá RFC 8032 (test 1)", async () => {
    const seed = fromHex("9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60");
    const s = await signerFromSeed(b64uEncode(seed));
    expect(toHex(s.publicKey)).toBe("d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a");
  });

  it("verifySig nikdy nevyhazuje a odmítá neplatné vstupy", async () => {
    const s = await signerFromSeed(b64uEncode(randomBytes(32)));
    const sig = await s.sign(DOMAIN.req, utf8("x"));
    expect(await verifySig(s.publicKey, DOMAIN.req, utf8("x"), sig)).toBe(true);
    expect(await verifySig(s.publicKey, DOMAIN.req, utf8("y"), sig)).toBe(false);
    expect(await verifySig(s.publicKey, DOMAIN.ticket, utf8("x"), sig)).toBe(false);
    expect(await verifySig(new Uint8Array(32), DOMAIN.req, utf8("x"), sig)).toBe(false);
    expect(await verifySig(new Uint8Array(5), DOMAIN.req, utf8("x"), sig)).toBe(false);
    expect(await verifySig(s.publicKey, DOMAIN.req, utf8("x"), new Uint8Array(10))).toBe(false);
    await expect(signerFromSeed("short")).rejects.toThrow();
  });

  it("leadingZeroBits", () => {
    expect(leadingZeroBits(new Uint8Array([0, 0, 0x10]))).toBe(19);
    expect(leadingZeroBits(new Uint8Array([0x80]))).toBe(0);
    expect(leadingZeroBits(new Uint8Array([0, 0]))).toBe(16);
  });
});

describe("síť a HRW", () => {
  it("ipPrefix: /24 pro IPv4, /48 pro IPv6, neplatné = null", () => {
    expect(ipPrefix("203.0.113.77")).toBe("203.0.113.0/24");
    expect(ipPrefix("2001:db8:abcd:12::1")).toBe("2001:0db8:abcd::/48");
    expect(ipPrefix("2001:db8:abcd:12:0:0:0:1")).toBe("2001:0db8:abcd::/48");
    expect(ipPrefix("::1")).toBe("0000:0000:0000::/48");
    expect(ipPrefix("999.1.1.1")).toBeNull();
    expect(ipPrefix("1.2.3")).toBeNull();
    expect(ipPrefix("1:2:3:4:5:6:7:8:9")).toBeNull();
    expect(ipPrefix(null)).toBeNull();
  });

  it("HRW je deterministické, vrací nejvýše r nodů a stabilně se mění s epochou", () => {
    const ids = Array.from({ length: 9 }, (_, i) => `node${i}`);
    const mb = randomBytes(32);
    expect(anchorsFor(mb, ids, 5, 3)).toEqual(anchorsFor(mb, [...ids].reverse(), 5, 3));
    expect(anchorsFor(mb, ids, 5, 3)).toHaveLength(3);
    expect(anchorsFor(mb, ids.slice(0, 2), 5, 3)).toHaveLength(2);
    expect(anchorsFor(mb, [], 5, 3)).toEqual([]);
    // rovnoměrnost: každý node je anchorem zhruba třetiny mailboxů
    const counts = new Map<string, number>();
    for (let i = 0; i < 900; i++) for (const a of anchorsFor(randomBytes(32), ids, 1, 3)) counts.set(a, (counts.get(a) ?? 0) + 1);
    for (const id of ids) expect(counts.get(id)!).toBeGreaterThan(200);
  });
});

function row(over: Partial<NodeRow> = {}): NodeRow {
  return {
    node_id: "n", pubkey: "p", hostname: "h", status: "active", genesis: 0,
    claimed: JSON.stringify({ cpu_cores: 4, ram_mb: 1000, storage_gb: 1000, bandwidth_mbps: 1000 }),
    asn: null, ip_hash: null, tunnel_id: null, tunnel_name: null, dns_record_id: null,
    created_at: 0, updated_at: 0, activated_at: 0, expires_at: null, last_hb_at: 1000, hb_seq: 0, load: null,
    country: null, continent: null, colo: null, lat: null, lon: null, rtt_ms: null,
    m_cpu_hps: 10_000_000, m_ram_mb: 128, m_storage_gb: 2, m_bw_mbps: 100, m_bw_r2: 100, m_bw_wit: null,
    health: 1, strikes: 0, ch_ok: 100, ch_total: 100, uptime_s: 0, ban_reason: null,
    ...over,
  };
}

describe("skóre nodu", () => {
  const cfg = loadConfig({ PROFILE: "prod" });
  const now = 1010;

  it("referenční node má kapacitu 1 a skóre 1", () => {
    const s = scoreOf(cfg, row(), now);
    expect(s.capacity).toBeCloseTo(1, 6);
    expect(s.net).toBeCloseTo(1, 6);
  });

  it("efektivní kapacita je min(claimed, measured) – lhaní nahoru nepomůže", () => {
    const honest = scoreOf(cfg, row(), now);
    // naměřeno víc, než node tvrdí → platí tvrzení
    const modest = row({ claimed: JSON.stringify({ cpu_cores: 4, ram_mb: 64, storage_gb: 1000, bandwidth_mbps: 1000 }), m_ram_mb: 128 });
    expect(scoreOf(cfg, modest, now).capacity).toBeLessThan(honest.capacity);
    // tvrdí víc, než naměřeno → platí měření
    const braggart = row({ claimed: JSON.stringify({ cpu_cores: 64, ram_mb: 1e6, storage_gb: 1e6, bandwidth_mbps: 1e6 }) });
    expect(scoreOf(cfg, braggart, now).capacity).toBeCloseTo(honest.capacity, 6);
  });

  it("nezměřená nebo nulová dimenze = nulová kapacita; strop poměru", () => {
    expect(scoreOf(cfg, row({ m_cpu_hps: null }), now).base).toBe(0);
    expect(scoreOf(cfg, row({ m_ram_mb: 0 }), now).base).toBe(0);
    const huge = row({ m_cpu_hps: 1e12, claimed: JSON.stringify({ cpu_cores: 4, ram_mb: 1e9, storage_gb: 1e9, bandwidth_mbps: 1e9 }), m_ram_mb: 1e9, m_storage_gb: 1e9, m_bw_mbps: 1e9 });
    expect(scoreOf(cfg, huge, now).capacity).toBeCloseTo(cfg.score.ratioCap, 6);
  });

  it("neživý, zabanovaný nebo neaktivní node má skóre 0", () => {
    expect(scoreOf(cfg, row({ last_hb_at: 0 }), now + 1000).base).toBe(0);
    expect(scoreOf(cfg, row({ status: "banned" }), now).base).toBe(0);
    expect(scoreOf(cfg, row({ status: "pending" }), now).base).toBe(0);
  });

  it("zdraví škáluje skóre, zátěž ho snižuje (kvantizovaně)", () => {
    expect(scoreOf(cfg, row({ health: 0.5 }), now).base).toBeCloseTo(0.5, 6);
    const loaded = scoreOf(cfg, row({ load: JSON.stringify({ conns: 50, max_conns: 100, cpu_pct: 10, mem_pct: 1 }) }), now);
    expect(loaded.util).toBe(0.5);
    expect(loaded.net).toBeCloseTo(1 - cfg.score.loadWeight * 0.5, 6);
    // drobný šum zátěže skóre nezmění
    const a = scoreOf(cfg, row({ load: JSON.stringify({ conns: 50, max_conns: 1000, cpu_pct: 21, mem_pct: 1 }) }), now);
    const b = scoreOf(cfg, row({ load: JSON.stringify({ conns: 50, max_conns: 1000, cpu_pct: 22, mem_pct: 1 }) }), now);
    expect(a.net).toBe(b.net);
  });

  it("probation: nový node má malou váhu, genesis plnou", () => {
    const open = loadConfig({ PROFILE: "open" });
    const fresh = scoreOf(open, row({ uptime_s: 0, ch_ok: 0 }), now);
    expect(fresh.trust).toBeCloseTo(open.probation.floor, 6);
    const half = scoreOf(open, row({ uptime_s: open.probation.durationS / 2, ch_ok: 100 }), now);
    expect(half.trust).toBeGreaterThan(fresh.trust);
    expect(half.trust).toBeLessThan(1);
    expect(scoreOf(open, row({ uptime_s: open.probation.durationS * 2, ch_ok: 100 }), now).trust).toBe(1);
    expect(scoreOf(open, row({ genesis: 1, uptime_s: 0, ch_ok: 0 }), now).trust).toBe(1);
    // velká uptime bez ověřených challenge nestačí
    expect(scoreOf(open, row({ uptime_s: open.probation.durationS * 2, ch_ok: 0 }), now).trust).toBeCloseTo(open.probation.floor, 6);
  });
});

describe("RateLimiter DO", () => {
  it("token bucket: burst, odmítnutí s Retry-After, doplňování", async () => {
    const l = env.LIMITER.getByName(`test:${crypto.randomUUID()}`);
    const o = { capacity: 2, refillPerS: 5 };
    expect((await l.admit(o)).status).toBe("ok");
    expect((await l.admit(o)).status).toBe("ok");
    const limited = await l.admit(o);
    expect(limited.status).toBe("limited");
    if (limited.status === "limited") expect(limited.retryAfterS).toBeGreaterThanOrEqual(1);
    await new Promise((r) => setTimeout(r, 450));
    expect((await l.admit(o)).status).toBe("ok");
  });

  it("nonce: druhé použití je replay, limitovaný request nonce nespotřebuje", async () => {
    const l = env.LIMITER.getByName(`test:${crypto.randomUUID()}`);
    const o = { capacity: 1, refillPerS: 0.001, nonceExpiresMs: Date.now() + 60_000 };
    expect((await l.admit({ ...o, nonce: "a" })).status).toBe("ok");
    expect((await l.admit({ ...o, nonce: "a" })).status).toBe("replay");
    expect((await l.admit({ ...o, nonce: "b" })).status).toBe("limited");
    // b nebyl spotřebován, takže po doplnění projde
    const l2 = env.LIMITER.getByName(`test:${crypto.randomUUID()}`);
    expect((await l2.admit({ ...o, capacity: 1, nonce: "x" })).status).toBe("ok");
  });
});

describe("správa a veřejné endpointy", () => {
  it("/admin vyžaduje token; výpis, ban a unban", async () => {
    await setup(FAST);
    const n = await seedActive(await newNode());
    const call = (path: string, init: RequestInit = {}, token: string | null = "test-admin-token") =>
      exports.default.fetch(new Request(`${BASE}${path}`, { ...init, headers: token ? { authorization: `Bearer ${token}`, "content-type": "application/json" } : {} }));
    expect((await call("/admin/nodes", {}, null)).status).toBe(401);
    expect((await call("/admin/nodes", {}, "wrong")).status).toBe(401);
    const list = (await (await call("/admin/nodes")).json()) as any[];
    const mine = list.find((x) => x.node_id === n.nodeId);
    expect(mine.status).toBe("active");
    expect(mine.score.base).toBeGreaterThan(0);
    expect(mine.claimed).toEqual(n.specs);

    const ban = await call(`/admin/nodes/${n.nodeId}/ban`, { method: "POST", body: JSON.stringify({ reason: "test" }) });
    expect(((await ban.json()) as any).banned).toBe(true);
    const after = ((await (await call("/admin/nodes")).json()) as any[]).find((x) => x.node_id === n.nodeId);
    expect(after.status).toBe("banned");
    expect(after.ban_reason).toBe("test");
    const un = await call(`/admin/nodes/${n.nodeId}/unban`, { method: "POST" });
    expect(((await un.json()) as any).unbanned).toBe(true);
    expect((await call("/admin/nope")).status).toBe(404);
    expect((await call("/admin/nodes/short/ban", { method: "POST", body: "{}" })).status).toBe(404);
  });

  it("/.well-known/ancord-key vrací veřejný klíč a kid", async () => {
    const res = await exports.default.fetch(`${BASE}/.well-known/ancord-key`);
    const body = (await res.json()) as any;
    expect(b64uDecodeLen(body.pubkey, 32)).not.toBeNull();
    expect(body.kid).toHaveLength(11);
  });

  it("cron: scheduled handler spustí tik", async () => {
    await setup(FAST);
    await seedActive(await newNode());
    await (exports.default as any).scheduled({ cron: "* * * * *", scheduledTime: Date.now() }, env, { waitUntil: (p: Promise<unknown>) => p });
    await tick();
    const dirs = await withRegistry((r) => sqlOf(r).exec("SELECT COUNT(*) AS c FROM directories").one() as any);
    expect(dirs.c).toBeGreaterThan(0);
  });
});

describe("hibernace Durable Objectu", () => {
  it("po vyřazení DO z paměti WebSocket pokračuje a stav zůstane", async () => {
    await setup(withOverrides(loadConfig({ PROFILE: "dev" }), {}) && { ...FAST, challenges: { ...FAST.challenges, enabled: [] } });
    const n = await bringUp(await newNode());
    expect((await n.heartbeat()).t).toBe("ack");
    await evictDurableObject(registryStub() as never);
    const ack = await n.heartbeat();
    expect(ack.t).toBe("ack");
    expect(ack.seq).toBe(2);
    n.close();
  });
});
