# ancord node

Node sítě ancord (Rust). Protokol je v `../spec/PROTOCOL.md`, architektura v `../ARCHITECTURE.md`. Se Workerem sdílí jen specifikaci a `../spec/test-vectors/`, žádný kód.

## Stav
- **B1** identita, registrace (`register → confirm → activate`), `cloudflared`, heartbeat, odpovědi na challenge (`cpu`, `space_commit`/`space_open`, `bandwidth`, `witness`).
- **B2** admission klientů (`/client`, ticket + důkaz držení `ek`, PROTOCOL §8.1) a minimální CLI klient.
- B3 (data plane) a B4 (push) zatím ne.

## Příkazy
```
cargo clippy --all-targets -- -D warnings
cargo test                      # vektory + e2e (viz níže)
cargo run -p ancord-node        # node
cargo run -p ancord-cli -- --worker http://127.0.0.1:8787 route
```

## Konfigurace nodu
Jediný povinný vstup je `WORKER_URL`. Ostatní má výchozí hodnotu nebo se změří na hostiteli (`ancord-node --help`).

| Proměnná | Výchozí | Význam |
|---|---|---|
| `WORKER_URL` | – | adresa Workeru |
| `DATA_DIR` | `./data` | klíč (`identity.key`, práva 0600), připnutý klíč Workeru, soubory Proof of Space. Musí být perzistentní, jinak se změní `node_id` |
| `LISTEN` | `0.0.0.0:8080` | cíl ingressu tunelu (`NODE_ORIGIN` Workeru) |
| `WORKER_PUBKEY` | – | klíč Workeru (b64u); bez něj se při prvním startu stáhne a uloží (trust on first use) |
| `CLOUDFLARED_BIN` | `cloudflared` | `none` tunel nespouští (lokální vývoj) |
| `CLAIM_CPU_CORES`, `CLAIM_RAM_MB`, `CLAIM_STORAGE_GB` | změřeno | tvrzené parametry (Worker jim nevěří, měří) |
| `CLAIM_BANDWIDTH_MBPS` | 100 | tvrzená šířka pásma |
| `MAX_CONNS` | 2000 | strop spojení klientů |
| `MAX_SPACE_LEAVES` | 2^27 | strop Proof of Space, který node přijme (ochrana před přehnanou výzvou) |
| `AUTH_TIMEOUT_S` | 10 | čas klienta na `auth` |

Parametry sítě (R, kvorum, epocha, intervaly) node nezná: dostává je od Workeru v adresáři a v `welcome`.

## Bezpečnostní poznámky
- Privátní klíč se vytváří s právy 0600; node odmítne start, pokud jsou volnější.
- Token tunelu se předává `cloudflared` proměnnou prostředí, ne argumentem.
- URL bandwidth výzvy se bere jen jako cesta vůči `WORKER_URL`, výzva nás nesmí poslat na cizí adresu.
- Peer witness stahuje jen z nodů, které jsou v ověřeném adresáři; zdroj dat obsluhuje jen podepsané requesty nodů z adresáře.

## E2E test (B1 + B2)
`cargo test --test e2e` spustí `wrangler dev` (mock Cloudflare, `PROOF_ORIGIN` místo tunelu) a tři nody; ověří registraci, challenge, `/route`, admission včetně odmítnutí (znovupoužitý ticket, ticket jiného nodu, cizí `ek`, nesmysl, `auth` po limitu) a restart se zachováním `node_id`. Potřebuje `npm install` ve `../worker`; bez něj se přeskočí. Ručně: `npx wrangler dev --var PROOF_ORIGIN:http://127.0.0.1:8080 …` (viz `../worker/README.md`).
