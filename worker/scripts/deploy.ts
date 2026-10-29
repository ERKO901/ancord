/**
 * Nasazení Workeru na Cloudflare (fáze C1 a C2, ARCHITECTURE.md §12.4).
 *
 *   npm run deploy               plán: jen čte účet a zónu, nic nemění
 *   npm run deploy -- --apply    vytvoří chybějící zdroje, nahraje tajemství, nasadí a ověří /health
 *
 * Vstupy (proměnné prostředí, hodnoty se nikdy nevypisují):
 *   CLOUDFLARE_API_TOKEN      token pro nasazení (wrangler)
 *   CLOUDFLARE_ACCOUNT_ID     volitelné; bez něj se použije jediný účet, který token vidí
 *   ANCORD_RUNTIME_CF_TOKEN   užší token Workeru (Tunnel Edit + DNS Edit jen pro zónu); povinný při prvním nasazení
 *   ANCORD_GENESIS_FILE       soubor s node_id genesis nodů (jedno na řádek, `dev/genesis.txt`); bez něj se seznam nemění
 *   ANCORD_PROFILE            profil (výchozí prod)
 *   ANCORD_ADMIN_TOKEN_FILE   kam uložit vygenerovaný ADMIN_TOKEN (výchozí ~/.config/ancord/admin-token, práva 0600)
 *
 * Pojistky pro sdílenou zónu: skript mění jen zdroje s názvem `ancord*`, DNS jméno Workeru musí být volné (nebo už
 * patřit Workeru `ancord`) a Transform Rule se přidává jako samostatné pravidlo s vlastní značkou, cizí pravidla
 * ve stejné fázi zůstávají beze změny.
 */
import { spawnSync } from "node:child_process";
import { createPrivateKey, createPublicKey, randomBytes } from "node:crypto";
import { chmodSync, existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { homedir, tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const WORKER_DIR = join(dirname(fileURLToPath(import.meta.url)), "..");
const SCRIPT_NAME = "ancord";
const ZONE = "melonet.cz";
const WORKER_HOST = `ancord.${ZONE}`;
const KV = { ROUTING_KV: "ancord-routing", PUSH_KV: "ancord-push" } as const;
const R2_BUCKET = "ancord-bw-blobs";
const QUEUES = ["ancord-push", "ancord-push-dlq"];
const TRANSFORM_REF = "ancord_strip_visitor_ip";
/** Hlavičky s IP návštěvníka, které se odstraní na `node*` (ARCHITECTURE.md §4). `cf-*` Cloudflare odstranit nedovolí. */
const IP_HEADERS = ["x-forwarded-for", "true-client-ip", "x-real-ip"];
const GENERATED_SECRETS = ["WORKER_SIGNING_KEY", "CHALLENGE_HMAC_KEY", "PUSH_ENC_KEY", "ADMIN_TOKEN"] as const;
const API = "https://api.cloudflare.com/client/v4";

const apply = process.argv.includes("--apply");
const token = process.env.CLOUDFLARE_API_TOKEN ?? "";
if (!token) fail("CLOUDFLARE_API_TOKEN is not set");

function fail(msg: string): never {
  console.error(`chyba: ${msg}`);
  process.exit(1);
}

interface Envelope<T> {
  success: boolean;
  errors?: { code: number; message: string }[];
  result: T;
}

async function cf<T>(method: string, path: string, body?: unknown): Promise<T> {
  const init: RequestInit = { method, headers: { authorization: `Bearer ${token}`, "content-type": "application/json" } };
  if (body !== undefined) init.body = JSON.stringify(body);
  const res = await fetch(`${API}${path}`, init);
  const data = (await res.json()) as Envelope<T>;
  if (!res.ok || !data.success) {
    const msg = data.errors?.map((e) => `${e.code} ${e.message}`).join("; ") ?? `HTTP ${res.status}`;
    throw new Error(`${method} ${path.replace(/[0-9a-f]{32}/g, "…")}: ${msg}`);
  }
  return data.result;
}

/** Jako `cf`, ale 404 vrátí jako `null`. */
async function cfMaybe<T>(path: string): Promise<T | null> {
  const res = await fetch(`${API}${path}`, { headers: { authorization: `Bearer ${token}` } });
  if (res.status === 404) return null;
  const data = (await res.json()) as Envelope<T>;
  if (!data.success) {
    // neexistující skript vrací 404 s kódem 10007
    if (data.errors?.some((e) => e.code === 10007)) return null;
    throw new Error(`GET ${path.replace(/[0-9a-f]{32}/g, "…")}: ${data.errors?.map((e) => `${e.code} ${e.message}`).join("; ")}`);
  }
  return data.result;
}

const randomKey = () => randomBytes(32).toString("base64url");

type Step = { what: string; run: () => Promise<void> };
const steps: Step[] = [];
/** Co brání nasazení; plán se vypíše celý, `--apply` pak odmítne pokračovat. */
const blockers: string[] = [];
const plan = (what: string, run: () => Promise<void>) => steps.push({ what, run });

async function main() {
  // --- účet a zóna (C1: kontrola tokenu)
  let accountId = process.env.CLOUDFLARE_ACCOUNT_ID ?? "";
  if (!accountId) {
    const accounts = await cf<{ id: string }[]>("GET", "/accounts");
    if (accounts.length !== 1) fail(`token sees ${accounts.length} accounts, set CLOUDFLARE_ACCOUNT_ID`);
    accountId = accounts[0]!.id;
  }
  const zones = await cf<{ id: string; status: string }[]>("GET", `/zones?name=${ZONE}`);
  const zoneId = zones[0]?.id ?? fail(`zone ${ZONE} is not visible to the token (Zone Read)`);
  console.log(`účet: ok, zóna ${ZONE}: ${zones[0]?.status}`);

  const acct = `/accounts/${accountId}`;
  const profile = process.env.ANCORD_PROFILE ?? "prod";

  // --- DNS jméno Workeru musí být volné nebo už patřit Workeru ancord
  const domains = await cf<{ id: string; hostname: string; service: string }[]>("GET", `${acct}/workers/domains`);
  const ownDomain = domains.find((d) => d.hostname === WORKER_HOST);
  if (ownDomain && ownDomain.service !== SCRIPT_NAME) fail(`${WORKER_HOST} is a custom domain of Worker '${ownDomain.service}'`);
  try {
    const dns = await cf<{ type: string }[]>("GET", `/zones/${zoneId}/dns_records?name=${WORKER_HOST}`);
    if (!ownDomain && dns.length > 0) fail(`DNS record ${WORKER_HOST} already exists (${dns.map((r) => r.type).join(", ")}), refusing to touch it`);
    console.log(`${WORKER_HOST}: ${ownDomain ? "custom domain Workeru ancord" : "volné"}`);
  } catch (e) {
    blockers.push(`nelze číst DNS zóny ${ZONE}, token potřebuje Zone → DNS Edit (${e instanceof Error ? e.message : String(e)})`);
  }

  // --- KV, R2, fronty
  const kvs = await cf<{ id: string; title: string }[]>("GET", `${acct}/storage/kv/namespaces?per_page=100`);
  const kvIds: Record<string, string> = {};
  for (const [binding, title] of Object.entries(KV)) {
    const hit = kvs.find((k) => k.title === title);
    if (hit) kvIds[binding] = hit.id;
    else
      plan(`vytvořit KV namespace ${title}`, async () => {
        kvIds[binding] = (await cf<{ id: string }>("POST", `${acct}/storage/kv/namespaces`, { title })).id;
      });
  }
  const buckets = await cf<{ buckets: { name: string }[] }>("GET", `${acct}/r2/buckets`);
  if (!buckets.buckets.some((b) => b.name === R2_BUCKET))
    plan(`vytvořit R2 bucket ${R2_BUCKET}`, async () => void (await cf("POST", `${acct}/r2/buckets`, { name: R2_BUCKET })));
  const queues = await cf<{ queue_name: string }[]>("GET", `${acct}/queues`);
  for (const q of QUEUES)
    if (!queues.some((x) => x.queue_name === q))
      plan(`vytvořit frontu ${q}`, async () => void (await cf("POST", `${acct}/queues`, { queue_name: q })));

  // --- tajemství: chybějící se vygenerují, existující se nemění
  const script = await cfMaybe<unknown>(`${acct}/workers/scripts/${SCRIPT_NAME}/settings`);
  const existing = new Set(
    script ? (await cf<{ name: string }[]>("GET", `${acct}/workers/scripts/${SCRIPT_NAME}/secrets`)).map((s) => s.name) : [],
  );
  const secrets: Record<string, string> = {};
  for (const name of GENERATED_SECRETS) if (!existing.has(name)) secrets[name] = randomKey();
  const runtimeToken = process.env.ANCORD_RUNTIME_CF_TOKEN;
  if (runtimeToken) secrets.CF_API_TOKEN = runtimeToken;
  else if (!existing.has("CF_API_TOKEN")) blockers.push("chybí ANCORD_RUNTIME_CF_TOKEN (užší token Workeru: Account → Cloudflare Tunnel Edit, Zone → DNS Edit)");
  const genesisFile = process.env.ANCORD_GENESIS_FILE;
  if (genesisFile) {
    const ids = readFileSync(genesisFile, "utf8").split(/\s+/).filter(Boolean);
    for (const id of ids) if (!/^[a-z2-7]{52}$/.test(id)) fail(`${genesisFile}: '${id}' is not a node_id`);
    secrets.GENESIS_NODE_IDS = ids.join(",");
  } else if (!existing.has("GENESIS_NODE_IDS")) {
    console.warn("varování: GENESIS_NODE_IDS není nastaveno, žádný node se nezaregistruje (ANCORD_GENESIS_FILE)");
  }
  const secretNames = Object.keys(secrets);
  console.log(`tajemství: existující ${[...existing].sort().join(", ") || "žádná"}; nahraje se ${secretNames.join(", ") || "nic"}`);

  // --- Transform Rule: odstranit IP hlavičky jen pro node*.melonet.cz (C1)
  const phase = "http_request_late_transform";
  let entry: { id: string; rules?: { id: string; ref?: string }[] } | null = null;
  let entryKnown = true;
  try {
    entry = await cfMaybe<{ id: string; rules?: { id: string; ref?: string }[] }>(`/zones/${zoneId}/rulesets/phases/${phase}/entrypoint`);
  } catch (e) {
    entryKnown = false;
    blockers.push(`nelze číst Transform Rules zóny, token potřebuje Zone → Transform Rules Edit (${e instanceof Error ? e.message : String(e)})`);
  }
  const rule = {
    ref: TRANSFORM_REF,
    description: "ancord: odstranit IP návštěvníka na node*",
    // jen `node<16 znaků base32>.<zóna>`: v zóně jsou cizí `node-xxxxxxxx` (regex `matches` free plán nemá)
    expression: `(starts_with(http.host, "node") and not starts_with(http.host, "node-") and ends_with(http.host, ".${ZONE}") and len(http.host) eq ${"node".length + 16 + ZONE.length + 1})`,
    action: "rewrite",
    action_parameters: { headers: Object.fromEntries(IP_HEADERS.map((h) => [h, { operation: "remove" }])) },
    enabled: true,
  };
  const ourRule = entry?.rules?.find((r) => r.ref === TRANSFORM_REF);
  if (!entryKnown) {
    // bez přečtení fáze nic neplánujeme, PUT by přepsal cizí pravidla
  } else if (!entry) plan(`vytvořit fázi ${phase} s jediným pravidlem ${TRANSFORM_REF}`, async () => void (await cf("PUT", `/zones/${zoneId}/rulesets/phases/${phase}/entrypoint`, { rules: [rule] })));
  else if (!ourRule) plan(`přidat pravidlo ${TRANSFORM_REF} do ${phase} (${entry.rules?.length ?? 0} cizích pravidel beze změny)`, async () => void (await cf("POST", `/zones/${zoneId}/rulesets/${entry.id}/rules`, rule)));
  else plan(`aktualizovat pravidlo ${TRANSFORM_REF}`, async () => void (await cf("PATCH", `/zones/${zoneId}/rulesets/${entry.id}/rules/${ourRule.id}`, rule)));

  // --- nasazení
  plan(`wrangler deploy ${SCRIPT_NAME} (profil ${profile}) na ${WORKER_HOST}`, async () => {
    const config = productionConfig({ accountId, zoneId, profile, kvIds });
    const cfgPath = join(WORKER_DIR, "wrangler.production.json");
    writeFileSync(cfgPath, JSON.stringify(config, null, 2) + "\n");
    // uložit dřív než deploy: když deploy selže až po nahrání tajemství, token se neztratí
    if (secrets.ADMIN_TOKEN) saveAdminToken(secrets.ADMIN_TOKEN);
    const dir = mkdtempSync(join(tmpdir(), "ancord-deploy-"));
    const secretsPath = join(dir, "secrets.json");
    try {
      writeFileSync(secretsPath, JSON.stringify(secrets), { mode: 0o600 });
      const args = ["wrangler", "deploy", "-c", cfgPath];
      if (secretNames.length > 0) args.push("--secrets-file", secretsPath);
      const r = spawnSync("npx", args, {
        cwd: WORKER_DIR,
        stdio: "inherit",
        env: { ...process.env, CLOUDFLARE_ACCOUNT_ID: accountId, WRANGLER_SEND_METRICS: "false" },
      });
      if (r.status !== 0) throw new Error(`wrangler deploy failed (${r.status})`);
    } finally {
      rmSync(dir, { recursive: true, force: true });
    }
  });
  plan(`ověřit https://${WORKER_HOST}/health`, async () => {
    let health: Response | null = null;
    for (let i = 0; i < 60 && !health; i++) {
      try {
        const res = await fetch(`https://${WORKER_HOST}/health`);
        if (res.ok) health = res;
      } catch {
        /* DNS a certifikát custom domény se chvíli rozbíhají */
      }
      if (!health) await new Promise((r) => setTimeout(r, 5000));
    }
    if (!health) throw new Error(`https://${WORKER_HOST}/health does not answer`);
    console.log(`/health: ${health.status} ${await health.text()}`);
    const served = ((await (await fetch(`https://${WORKER_HOST}/.well-known/ancord-key`)).json()) as { pubkey?: string }).pubkey ?? "";
    // nový klíč známe lokálně, takže ho neověřujeme jen tím, co vrátila síť
    const seed = secrets.WORKER_SIGNING_KEY;
    if (seed && served !== publicKeyOf(seed)) throw new Error("served Worker key does not match the uploaded WORKER_SIGNING_KEY");
    console.log(`WORKER_PUBKEY=${served}${seed ? " (ověřeno proti nahranému klíči)" : ""}`);
    pinWorkerKey(served);
  });

  console.log(`\nplán (${apply ? "provádí se" : "nic se nemění, spusťte s --apply"}):`);
  for (const s of steps) console.log(`  - ${s.what}`);
  if (blockers.length > 0) {
    console.log("\nblokuje nasazení:");
    for (const b of blockers) console.log(`  ! ${b}`);
    if (apply) fail("nothing was changed");
  }
  if (!apply) return;
  for (const s of steps) {
    console.log(`\n> ${s.what}`);
    await s.run();
  }
}

/** `wrangler.jsonc` doplněný o produkční hodnoty. Soubor je v .gitignore (obsahuje id účtu, zóny a KV). */
function productionConfig(o: { accountId: string; zoneId: string; profile: string; kvIds: Record<string, string> }) {
  const base = JSON.parse(
    readFileSync(join(WORKER_DIR, "wrangler.jsonc"), "utf8")
      .split("\n")
      .filter((l) => !l.trim().startsWith("//"))
      .join("\n"),
  ) as Record<string, unknown> & { vars: Record<string, string>; kv_namespaces: { binding: string; id?: string }[] };
  delete base.$schema;
  const { GENESIS_NODE_IDS: _genesis, ...vars } = base.vars; // GENESIS_NODE_IDS je tajemství, aby šlo měnit bez nasazení
  void _genesis;
  return {
    ...base,
    name: SCRIPT_NAME,
    account_id: o.accountId,
    workers_dev: false,
    preview_urls: false,
    routes: [{ pattern: WORKER_HOST, custom_domain: true }],
    kv_namespaces: base.kv_namespaces.map((k) => {
      const id = o.kvIds[k.binding];
      if (!id) throw new Error(`KV namespace for ${k.binding} is missing`);
      return { ...k, id };
    }),
    vars: {
      ...vars,
      PROFILE: o.profile,
      ZONE_DOMAIN: ZONE,
      WORKER_HOST,
      CLOUDFLARE_MODE: "real",
      CF_ACCOUNT_ID: o.accountId,
      CF_ZONE_ID: o.zoneId,
      NODE_ORIGIN: "http://localhost:8080",
      PUSH_MODE: process.env.ANCORD_PUSH_MODE ?? "log",
    },
  };
}

/** Veřejný klíč Ed25519 (base64url) ze seedu ve formátu WORKER_SIGNING_KEY. */
function publicKeyOf(seedB64u: string): string {
  const pkcs8 = Buffer.concat([Buffer.from("302e020100300506032b657004220420", "hex"), Buffer.from(seedB64u, "base64url")]);
  const jwk = createPublicKey(createPrivateKey({ key: pkcs8, format: "der", type: "pkcs8" })).export({ format: "jwk" });
  return jwk.x ?? "";
}

/** Zapíše klíč do dev/.env, pokud soubor existuje (nody ho připínají, `WORKER_PUBKEY`). */
function pinWorkerKey(pubkey: string): void {
  const path = join(WORKER_DIR, "..", "dev", ".env");
  if (!existsSync(path) || !/^[A-Za-z0-9_-]{43}$/.test(pubkey)) return;
  const lines = readFileSync(path, "utf8").split("\n").filter((l) => !l.startsWith("WORKER_PUBKEY="));
  lines.splice(lines.findIndex((l) => l.startsWith("WORKER_URL=")) + 1, 0, `WORKER_PUBKEY=${pubkey}`);
  writeFileSync(path, lines.join("\n"));
  console.log(`WORKER_PUBKEY zapsán do ${path}`);
}

function saveAdminToken(value: string): void {
  const path = process.env.ANCORD_ADMIN_TOKEN_FILE ?? join(homedir(), ".config", "ancord", "admin-token");
  mkdirSync(dirname(path), { recursive: true, mode: 0o700 });
  if (existsSync(path)) rmSync(path);
  writeFileSync(path, value + "\n", { mode: 0o600 });
  chmodSync(path, 0o600);
  console.log(`ADMIN_TOKEN uložen do ${path} (0600); přesuňte ho do správce hesel`);
}

main().catch((e: unknown) => fail(e instanceof Error ? e.message : String(e)));
