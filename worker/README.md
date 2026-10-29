# ancord Worker

Load balancer a control plane sítě ancord (Cloudflare Worker, TypeScript). Protokol je v `../spec/PROTOCOL.md`, architektura v `../ARCHITECTURE.md`, hrozby v `../spec/THREAT_MODEL.md`.

## Příkazy
```
npm install
npm test            # vitest + Miniflare (workerd)
npm run typecheck   # tsc --noEmit (strict)
npm run check       # obojí
npm run vectors     # přegeneruje ../spec/test-vectors
npx wrangler dev    # lokálně, profil dev, mock Cloudflare API
```

## Struktura
```
src/
  index.ts            router, cron, konzument fronty
  config.ts           profily dev / prod / open (žádné parametry v kódu)
  cloudflare.ts       CloudflareClient: skutečné API (s pojistkami pro sdílenou zónu) a mock
  registry/           Durable Object Registry (SQLite): registrace, WebSocket a heartbeat,
                      adresář a epochy, skóre, routing snapshot do KV
  challenge/          CPU hash-chain, Proof of Space, bandwidth (R2), peer witnessy, plánovač
  push/               registrace tokenu, /wake, fronta, odesílatelé FCM a APNs
  limiter.ts          Durable Object RateLimiter (token bucket + nonce)
  routing.ts          výběr nodů (power-of-two-choices)
  ticket.ts, hrw.ts, auth.ts, hashcash.ts, crypto.ts, canonical.ts, encoding.ts
test/                 testy po milnících + sdílené vektory (vectors.test.ts)
scripts/gen-vectors.ts
```

## Konfigurace
- Proměnná `PROFILE` (`dev` | `prod` | `open`) vybírá profil z `src/config.ts`. Pro ladění lze přepsat jednotlivé hodnoty JSON proměnnou `CONFIG_OVERRIDES`; neznámý klíč nebo špatný typ je chyba při startu.
- `ZONE_DOMAIN`, `WORKER_HOST`, `NODE_ORIGIN`, `CF_ACCOUNT_ID`, `CF_ZONE_ID`, `GENESIS_NODE_IDS` (node_id oddělené čárkou) jsou proměnné ve `wrangler.jsonc`.
- `PROOF_ORIGIN` (volitelné, jen s `CLOUDFLARE_MODE=mock`): základ URL, na který Worker při aktivaci pošle žádost o důkaz místo `https://<hostname>`. Slouží k lokálnímu běhu nodu bez tunelu: `npx wrangler dev --var PROOF_ORIGIN:http://127.0.0.1:8080`. Řetězec `{hostname}` se nahradí hostname nodu (jeden lokální rozcestník pro víc nodů). V režimu `real` se ignoruje.
- `CLOUDFLARE_MODE`: `mock` (výchozí, nic se nevolá) nebo `real`. `PUSH_MODE`: `log` nebo `real`.

### Tajemství (`wrangler secret put`, nikdy do gitu)
| Název | Obsah |
|---|---|
| `WORKER_SIGNING_KEY` | Ed25519 seed, 32 B base64url (podpis ticketů a adresáře) |
| `CF_API_TOKEN` | užší token Workeru: Tunnel Edit + DNS Edit jen pro zónu |
| `ADMIN_TOKEN` | bearer pro `/admin/*` |
| `CHALLENGE_HMAC_KEY` | 32 B base64url (tokeny bandwidth výzev, HMAC hashe IP prefixů) |
| `PUSH_ENC_KEY` | 32 B base64url (šifrování push tokenů) |
| `FCM_SERVICE_ACCOUNT`, `APNS_KEY_P8`, `APNS_KEY_ID`, `APNS_TEAM_ID`, `APNS_TOPIC` | jen pro `PUSH_MODE=real` |

Náhodný klíč: `head -c 32 /dev/urandom | basenc --base64url | tr -d '='`.

## Správa (`Authorization: Bearer $ADMIN_TOKEN`)
- `GET /admin/nodes` – stav, naměřené hodnoty, skóre
- `POST /admin/nodes/<node_id>/ban` `{"reason":"…"}` – ban, uvolní tunel a DNS
- `POST /admin/nodes/<node_id>/unban`

## Nasazení (fáze C1 a C2)
`npm run deploy` (`scripts/deploy.ts`) bez argumentů jen čte účet a zónu a vypíše plán i to, co nasazení blokuje. `npm run deploy -- --apply` plán provede:
1. ověří, že `ancord.melonet.cz` není v DNS ani custom doménou jiného Workeru,
2. vytvoří chybějící KV (`ancord-routing`, `ancord-push`), R2 `ancord-bw-blobs` a fronty `ancord-push`, `ancord-push-dlq`,
3. přidá Transform Rule `ancord_strip_visitor_ip` (fáze `http_request_late_transform`, jen `node*.melonet.cz`, cizí pravidla nemění),
4. vygeneruje chybějící tajemství (`WORKER_SIGNING_KEY`, `CHALLENGE_HMAC_KEY`, `PUSH_ENC_KEY`, `ADMIN_TOKEN`), existující nemění,
5. sestaví `wrangler.production.json` (v .gitignore: `CLOUDFLARE_MODE=real`, id účtu, zóny a KV, custom domain) a spustí `wrangler deploy` s tajemstvími,
6. počká na `https://ancord.melonet.cz/health` a vypíše `WORKER_PUBKEY` ověřený proti nahranému klíči (zapíše ho do `../dev/.env`).

| Proměnná | Význam |
|---|---|
| `CLOUDFLARE_API_TOKEN` | token pro nasazení. Účet: Workers Scripts, Workers KV Storage, Workers R2 Storage, Queues (Edit). Zóna `melonet.cz`: Zone Read, DNS Edit, Workers Routes Edit, Transform Rules Edit |
| `CLOUDFLARE_ACCOUNT_ID` | volitelné, jinak jediný účet tokenu |
| `ANCORD_RUNTIME_CF_TOKEN` | užší token, se kterým Worker za běhu vytváří tunely a DNS nodů (`CF_API_TOKEN`). Účet: Cloudflare Tunnel Edit. Zóna `melonet.cz`: DNS Edit. Povinný při prvním nasazení |
| `ANCORD_GENESIS_FILE` | `../dev/genesis.txt` z `dev/bootstrap.sh`; `GENESIS_NODE_IDS` je tajemství, takže se mění bez úpravy konfigurace |
| `ANCORD_PROFILE` | profil, výchozí `prod` |
| `ANCORD_PUSH_MODE` | `log` (výchozí) nebo `real` (pak je třeba nahrát FCM/APNs tajemství přes `wrangler secret put -c wrangler.production.json`) |
| `ANCORD_ADMIN_TOKEN_FILE` | kam uložit vygenerovaný `ADMIN_TOKEN` (výchozí `~/.config/ancord/admin-token`, 0600) |

Rotace tajemství: `npx wrangler secret put <NÁZEV> -c wrangler.production.json`. Výměna `WORKER_SIGNING_KEY` znamená nové `WORKER_PUBKEY` u všech nodů i klientů.

Zbývá ručně: rate limiting (WAF) pro `/register*` a `/route` (Worker má vlastní limiter v DO, WAF pravidlo je druhá vrstva) a ověření blokujících bodů 1, 2 a 5 z ARCHITECTURE §9 na nasazených nodech.
