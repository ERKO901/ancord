/**
 * Úklid vývojových nodů v Cloudflare (ARCHITECTURE.md §12.3, `dev/cleanup.sh`).
 *
 *   npx tsx scripts/cleanup.ts            vypíše, co by smazal
 *   npx tsx scripts/cleanup.ts --apply    smaže
 *
 * Maže jen DNS záznamy `node-<16 znaků base32>.melonet.cz` s komentářem `ancord-managed` a tunely s prefixem
 * `ancord-`, tedy přesně to, co vytváří Worker. Worker, KV, R2, fronty ani cizí záznamy zóny nemaže.
 * Běžící nody si po smazání při dalším startu zaregistrují tunel a DNS znovu.
 */
import { MANAGED_COMMENT, TUNNEL_NAME_PREFIX, isManagedHostname } from "../src/cloudflare";

const ZONE = "melonet.cz";
const API = "https://api.cloudflare.com/client/v4";
const apply = process.argv.includes("--apply");
const token = process.env.CLOUDFLARE_API_TOKEN ?? "";

interface Envelope<T> {
  success: boolean;
  errors?: { code: number; message: string }[];
  result: T;
}

async function cf<T>(method: string, path: string): Promise<T> {
  const res = await fetch(`${API}${path}`, { method, headers: { authorization: `Bearer ${token}` } });
  const data = (await res.json()) as Envelope<T>;
  if (!res.ok || !data.success) {
    throw new Error(`${method} ${path.replace(/[0-9a-f]{32}/g, "…")}: ${data.errors?.map((e) => `${e.code} ${e.message}`).join("; ") ?? res.status}`);
  }
  return data.result;
}

async function main() {
  if (!token) throw new Error("CLOUDFLARE_API_TOKEN is not set");
  let accountId = process.env.CLOUDFLARE_ACCOUNT_ID ?? "";
  if (!accountId) {
    const accounts = await cf<{ id: string }[]>("GET", "/accounts");
    if (accounts.length !== 1) throw new Error(`token sees ${accounts.length} accounts, set CLOUDFLARE_ACCOUNT_ID`);
    accountId = accounts[0]!.id;
  }
  const zoneId = (await cf<{ id: string }[]>("GET", `/zones?name=${ZONE}`))[0]?.id;
  if (!zoneId) throw new Error(`zone ${ZONE} is not visible to the token`);

  const records = await cf<{ id: string; name: string; type: string; comment?: string | null }[]>(
    "GET",
    `/zones/${zoneId}/dns_records?per_page=5000&name.startswith=node-`,
  );
  const dns = records.filter((r) => isManagedHostname(r.name, ZONE) && r.comment === MANAGED_COMMENT);
  const tunnels = (await cf<{ id: string; name: string }[]>("GET", `/accounts/${accountId}/cfd_tunnel?is_deleted=false&per_page=1000`)).filter(
    (t) => t.name.startsWith(TUNNEL_NAME_PREFIX),
  );

  console.log(`DNS (${dns.length}):`);
  for (const r of dns) console.log(`  ${r.type} ${r.name}`);
  console.log(`tunely (${tunnels.length}):`);
  for (const t of tunnels) console.log(`  ${t.name}`);
  if (!apply) {
    console.log("\nnic se nemění, spusťte s --apply");
    return;
  }
  for (const r of dns) await cf("DELETE", `/zones/${zoneId}/dns_records/${r.id}`);
  for (const t of tunnels) {
    await cf("DELETE", `/accounts/${accountId}/cfd_tunnel/${t.id}/connections`);
    await cf("DELETE", `/accounts/${accountId}/cfd_tunnel/${t.id}`);
  }
  console.log(`smazáno: ${dns.length} DNS, ${tunnels.length} tunelů`);
}

main().catch((e: unknown) => {
  console.error(`chyba: ${e instanceof Error ? e.message : String(e)}`);
  process.exit(1);
});
