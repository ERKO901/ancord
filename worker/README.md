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

## Co je potřeba před nasazením (fáze C)
Nic z toho tento adresář nedělá. Seznam pro C1–C3:
1. R2 bucket `ancord-bw-blobs`, fronty `ancord-push` a `ancord-push-dlq`, KV namespace (wrangler je při prvním deployi nabídne vytvořit).
2. Tajemství z tabulky výše, `CLOUDFLARE_MODE=real`, `CF_ACCOUNT_ID`, `CF_ZONE_ID`, `PROFILE`, `GENESIS_NODE_IDS`.
3. Domény: custom domain `ancord.melonet.cz` pro Worker.
4. Pravidla zóny: rate limiting (WAF) pro `/register*` a `/route`, Transform Rule odstraňující hlavičky s IP návštěvníka jen pro `node*.melonet.cz`.
5. Ověřit blokující body 1, 2 a 5 z ARCHITECTURE §9 a naměřená čísla zapsat tam.
