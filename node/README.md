# ancord node

Node sítě ancord (Rust). Protokol je v `../spec/PROTOCOL.md`, architektura v `../ARCHITECTURE.md`. Se Workerem sdílí jen specifikaci a `../spec/test-vectors/`, žádný kód.

## Stav
- **B1** identita, registrace (`register → confirm → activate`), `cloudflared`, heartbeat, odpovědi na challenge (`cpu`, `space_commit`/`space_open`, `bandwidth`, `witness`).
- **B2** admission klientů (`/client`, ticket + důkaz držení `ek`, PROTOCOL §8.1) a minimální CLI klient.
- **B3** data plane (`PROTOCOL.md` §13): blinded mailboxy v redb, HRW anchory podle epochy, replikace R=3 s kvorem 2, idempotentní zápis, ack s náhrobky, TTL úklid, handoff po změně epochy, podepsané node↔node requesty (`/n/put`, `/n/get`, `/n/ack`). Double Ratchet (X25519 + ChaCha20-Poly1305 + HKDF) je klientská knihovna v `ratchet.rs`, node ho nikdy nepočítá.
- **B6** zabezpečený kanál (`PROTOCOL.md` §14): klient↔node i node↔node šifrované vlastním handshakem (X25519, identita nodu podpisem Ed25519, ChaCha20-Poly1305 / XChaCha20-Poly1305, padding), aby Cloudflare po ukončení TLS neviděl `mailbox_id`, tickety ani `push_id`. Klient ověřuje ticket připnutým klíčem Workeru (`WORKER_PUBKEY`, nutný i pro CLI). Holé `/n/*` API neexistuje. Vektory `channel.json` z nezávislé implementace `spec/reference/channel.py`.
- **B5** presence a online doručení (`PROTOCOL.md` §13.7): `subscribe` u gateway, presence na anchorech (jen v paměti, TTL), anchor novou zprávu přepošle gateway příjemce (`/n/presence`, `/n/deliver`); wake jen když doručení online selže. CLI: `ancord watch`.
- **B4** push (`PROTOCOL.md` §13.6): `send` s volitelným `push_id` příjemce; anchor, který zprávu nově uložil, pošle Workeru podepsaný `POST /wake` (selhání wake zápis nikdy neshodí). 
- **B8** uživatelský tok (`PROTOCOL.md` §15.10 až §15.12): adresa `ancord:<pk>` jako text nebo QR, push klíče schránky (v profilu) a konverzací (ze sdíleného tajemství), registrace push tokenu, online klient s automatickým přijetím kontaktů a znovupřipojením.
- **B7** identita uživatele a funkce z `melo` (`PROTOCOL.md` §15): profil s veřejným klíčem na anchorech (`profile_put`/`profile_get`), X3DH s rotujícím a jednorázovými prekeys a šifrovanou obálkou ve schránce (`user.rs`), bezpečnostní číslo, typing, stav doručení, dávkový `subscribe`, lokální historie. Vektor `pairwise.json` z nezávislé implementace `spec/reference/pairwise.py`.

## Příkazy
```
cargo clippy --all-targets -- -D warnings
cargo test                      # vektory + e2e (viz níže)
cargo run -p ancord-node        # node
WORKER_PUBKEY=$(curl -s http://127.0.0.1:8787/.well-known/ancord-key | jq -r .pubkey) \
cargo run -p ancord-cli -- --worker http://127.0.0.1:8787 route
```

## CLI klient: běžné použití
Dva uživatelé si předají adresu (QR kód nebo text) a jsou online:
```
ancord keygen alice.user                                    # klíče; vypíše adresu ancord:<veřejný klíč>
ancord whoami --qr alice.user                               # adresa jako QR kód a text, předejte ji protistraně
ancord push-register alice.user convs/ --platform fcm --token <token>   # volitelně: push, když nejste online
ancord online alice.user convs/                             # zůstane připojený (Ctrl-C konec)
```
V `online` se píše na vstup: `/start ancord:<adresa> bob` založí konverzaci `bob` (protistrana ji dostane sama, i když je offline: vzbudí ji push), `bob: ahoj` pošle zprávu, `/list` vypíše konverzace. Příchozí zprávy se vypisují jako `bob: text`, nové kontakty i s bezpečnostním číslem (porovnejte ho s protistranou mimo síť). Konverzace jsou soubory `convs/<jméno>.json`. Profil se zveřejní při každém připojení. Jedno spojení odebírá nejvýše 31 konverzací. Dokud `online` běží, ostatní příkazy nad stejnými soubory odmítnou běžet (zámek `*.lock`), aby se stav ratchetu nerozešel.

Průběh: `online` si od Workeru vyžádá 3 nody (`/route`), připojí se na první, který ho přijme, a odebírá schránku i všechny konverzace. Gateway to oznámí anchorům (presence) a ti nové zprávy přeposílají rovnou. Po odpojení presence zanikne a anchory pošlou přes Worker push na tokeny z `push-register`. Při výpadku nodu se `online` připojí znovu přes nový `/route`.

## CLI klient: konverzace podle veřejného klíče (jednotlivé kroky)
```
ancord keygen alice.user                                # vypíše adresu (veřejný klíč), kterou předáte dál
ancord --worker $WORKER_URL publish alice.user          # rotuje prekey, doplní jednorázové a zveřejní profil
ancord --worker $WORKER_URL start alice.user ancord:<adresa-boba> alice-bob.json   # X3DH + obálka do Bobovy schránky
ancord --worker $WORKER_URL inbox bob.user convs/       # Bob vyzvedne obálky a založí konverzace
ancord safety alice.user <adresa-boba>                  # bezpečnostní číslo: porovnat s protistranou mimo síť
ancord --worker $WORKER_URL send alice-bob.json "ahoj"  # vypíše id zprávy
ancord --worker $WORKER_URL typing alice-bob.json       # protistrana vidí „(typing…)“ ve `watch`
ancord --worker $WORKER_URL status alice-bob.json <id>  # pending / delivered / unknown
ancord history alice-bob.json                           # lokální historie (server ji nedrží)
```
Po `inbox` spusťte znovu `publish` (použité jednorázové prekeys jsou smazané) a profil obnovujte aspoň jednou týdně. Soubory `*.user` a konverzace obsahují tajné klíče (práva 0600), zálohou stavu ratchetu se poškodí.

## CLI klient: zprávy se sdíleným tajemstvím
```
ancord init alice.json --role initiator                 # vypíše sdílené tajemství
ancord init bob.json --role responder --secret <b64u>   # protistrana zadá totéž tajemství
ancord --worker $WORKER_URL send alice.json "ahoj"
ancord --worker $WORKER_URL recv bob.json               # vypíše, dešifruje a potvrdí
```
Soubor konverzace obsahuje klíče (práva 0600) a stav ratchetu; ukládá se před odesláním, aby se číslo zprávy nepoužilo dvakrát. Obnova ze zálohy stavu ratchet poškodí. Ratchet je Double Ratchet: konverzace se po úniku stavu uzdraví, jakmile si strany vymění zprávy v obou směrech. Responder může psát až po první zprávě iniciátora.

## Instalace pro uživatele
`curl -fsSL https://github.com/ERKO901/ancord/releases/latest/download/install.sh | sh`, potom příkaz `ancord` (menu, výkon, autostart, odinstalace). Vše je v `~/.ancord`, viz `../install/` a `../ARCHITECTURE.md` §12.6.

## Docker
`Dockerfile` sestaví image s `ancord-node`, CLI `ancord` a `cloudflared` (pevná verze a SHA-256, amd64 i arm64): `docker build -t ancord-node node/`. Kontejner běží jako uživatel `ancord` (uid 10001), data jsou ve volume `/data`, `HEALTHCHECK` volá `/health`. Šest nodů v Compose: `../dev/README.md`.

`ancord-node --init-identity` jen vytvoří (nebo načte) klíč v `DATA_DIR`, vypíše `node_id` a skončí, bez kontaktu s Workerem; slouží k sestavení genesis allowlistu. Node končí čistě na Ctrl-C i SIGTERM.

## Konfigurace nodu
Jediný povinný vstup je `WORKER_URL`. Ostatní má výchozí hodnotu nebo se změří na hostiteli (`ancord-node --help`).

| Proměnná | Výchozí | Význam |
|---|---|---|
| `WORKER_URL` | – | adresa Workeru |
| `DATA_DIR` | `./data` | klíč (`identity.key`, práva 0600), připnutý klíč Workeru, soubory Proof of Space. Musí být perzistentní, jinak se změní `node_id` |
| `LISTEN` | `0.0.0.0:8080` | cíl ingressu tunelu (`NODE_ORIGIN` Workeru) |
| `WORKER_PUBKEY` | – | klíč Workeru (b64u), **v produkci povinný**; zjistí se při nasazení z `/.well-known/ancord-key` a předá se mimo síť |
| `TRUST_WORKER_KEY_ON_FIRST_USE` | vypnuto | jen vývoj: bez `WORKER_PUBKEY` klíč stáhne při prvním startu přes Cloudflare (kdo ho podvrhne, ovládne důvěru nodu) |
| `CLOUDFLARED_BIN` | `cloudflared` | `none` tunel nespouští (lokální vývoj) |
| `CLAIM_CPU_CORES`, `CLAIM_RAM_MB`, `CLAIM_STORAGE_GB` | změřeno | tvrzené parametry (Worker jim nevěří, měří) |
| `CLAIM_BANDWIDTH_MBPS` | 100 | tvrzená šířka pásma |
| `MAX_CONNS` | 2000 | strop spojení klientů |
| `MAX_SPACE_LEAVES` | 2^27 | strop Proof of Space, který node přijme (ochrana před přehnanou výzvou) |
| `AUTH_TIMEOUT_S` | 10 | čas klienta na `auth` |
| `PEER_URL_TEMPLATE` | `https://{host}` | adresa jiného nodu pro node↔node requesty (lokální vývoj a testy ji přesměrují) |
| `PEER_TIMEOUT_S` | 5 | timeout jednoho node↔node requestu |
| `MESSAGE_TTL_S` | 2 592 000 (30 dní) | jak dlouho se drží nedoručená zpráva |
| `MAX_MAILBOX_MESSAGES` | 10 000 | strop nedoručených zpráv v mailboxu na nodu |
| `CLIENT_MSG_RATE` | 20 | zpráv za sekundu na spojení klienta (dávka dvojnásobek) |
| `REACH_MIN_FAILURES` | 3 | po kolika neúspěšných voláních po sobě se peer hlásí Workeru jako nedosažitelný (spec §6.1) |
| `REACH_WINDOW_S` | 120 | jak dlouho od posledního neúspěchu se peer hlásí |
| `MAINTENANCE_INTERVAL_S` | 30 | jak často se kontroluje, zda je třeba handoff |
| `JANITOR_INTERVAL_S` | 3600 | jak často se uklízí vypršené zprávy |
| `PRESENCE_TTL_S` | 60 | platnost presence na anchoru bez obnovení |
| `PRESENCE_RENEW_S` | 20 | jak často gateway presence obnovuje |
| `WAKE_MIN_INTERVAL_S` | 10 | nejkratší odstup mezi wake téhož `push_id` z tohoto nodu |

Úložiště mailboxů je `DATA_DIR/mailboxes.redb` (součást perzistentní volume). Parametry sítě (R, kvorum, epocha, intervaly) node nezná: dostává je od Workeru v adresáři a v `welcome`.

## Bezpečnostní poznámky
- Privátní klíč se vytváří s právy 0600; node odmítne start, pokud jsou volnější.
- Token tunelu se předává `cloudflared` proměnnou prostředí, ne argumentem.
- URL bandwidth výzvy se bere jen jako cesta vůči `WORKER_URL`, výzva nás nesmí poslat na cizí adresu.
- Peer witness stahuje jen z nodů, které jsou v ověřeném adresáři; zdroj dat obsluhuje jen podepsané requesty nodů z adresáře.

## E2E test (B1 až B4)
`cargo test --test e2e` spustí `wrangler dev` (mock Cloudflare, `PROOF_ORIGIN` místo tunelu) a tři nody; ověří registraci, challenge, `/route`, admission včetně odmítnutí (znovupoužitý ticket, ticket jiného nodu, cizí `ek`, nesmysl, `auth` po limitu) restart se zachováním `node_id`, výměnu zpráv mezi dvěma klienty přes různé nody, zprávu přežívající restart nodu, zápis a čtení s jedním vypnutým anchorem a odmítnutí zápisu při dvou vypnutých. Ověří i push: zpráva s `push_id` zaregistrovaného tokenu vyvolá wake od anchorů a Worker (profil dev) push zaloguje. `cargo test --test dataplane` totéž pro kvorum, autorizaci node↔node a handoff bez Workeru. Potřebuje `npm install` ve `../worker`; bez něj se přeskočí. Ručně: `npx wrangler dev --var PROOF_ORIGIN:http://127.0.0.1:8080 …` (viz `../worker/README.md`).
