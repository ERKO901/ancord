# ancord

Decentralizovaná síť pro E2E šifrované zprávy. Nody provozuje kdokoli, Cloudflare slouží jako control plane (registrace, load balancing, push). Nová verze, stavěná od nuly.

**Zdroj pravdy je `ARCHITECTURE.md`.** Před jakoukoli prací si přečti sekce 3, 10 a 12. Když se kód a dokument rozejdou, nejdřív uprav dokument.

## Struktura
- `worker/` – load balancer a control plane (Cloudflare Worker, TypeScript)
- `node/` – node (Rust)
- `spec/` – `PROTOCOL.md`, `THREAT_MODEL.md`, `test-vectors/` (jediné, co sdílejí worker a node)
- `dev/` – Docker Compose se 6 nody připojenými přes Cloudflare Tunnel

Složky vznikají postupně podle milníků v sekci 12.4.

## Konvence
- Starý Node.js prototyp (`../melo`) je reference chování. Aplikaci přepiš do Rustu podle nového návrhu, nepřenášej kód 1:1.
- Spec a testovací vektory před implementací. `worker/` a `node/` nesdílejí kód.
- Parametry (R, kvorum, epocha, probation…) jsou v konfiguraci podle profilů dev/prod, ne v kódu.
- Body označené **[ověřit]** zjisti prototypem a výsledek zapiš do sekce 9 v `ARCHITECTURE.md`.
- Nic mimo rozsah (mobilní klient, UI, tokenová ekonomika). Při nejasnosti se zeptej.
- Tajemství (Cloudflare API token, tunnel tokeny, privátní klíče) nikdy do gitu. `.env` a datová volume jsou v `.gitignore`.
- Rust: `cargo clippy` a `cargo test` bez varování, žádné `unwrap` v produkční cestě.
- TypeScript: `strict`, testy přes vitest s Miniflare/workerd.
- Dokumentace česky, identifikátory anglicky.

## Postup
Pořadí (podrobně v sekci 12.4):
1. **Fáze A:** napiš kód Workeru (load balancer a control plane) v `worker/`, A1 až A5. Testuj lokálně (vitest, Miniflare).
2. **Fáze B:** přepiš původní aplikaci (`../melo`, Node.js) do Rustu v `node/` podle nového návrhu. Původní kód je reference chování, ne předloha.
3. **Fáze C:** deploy, DNS a Docker se 6 nody. Až bude kód hotový.

Ve fázích A a B nic nenasazuj, nevytvářej nic v Cloudflare a nečti `.env`. Milník je hotový až po splnění kritéria přijetí.

## Cloudflare (firemní zóna melonet.cz)
- Worker: `ancord.melonet.cz`, nody: `node<id>.melonet.cz` (jedna úroveň pod zónou).
- Přihlašovací údaje jsou v proměnných prostředí `CLOUDFLARE_API_TOKEN` a `CLOUDFLARE_ACCOUNT_ID`. Hodnoty nikdy nevypisuj.
- Zóna je sdílená s existujícími službami: před zápisem přečti existující DNS záznamy, měň jen záznamy s prefixem `node` a `ancord`, nic jiného. Před každým krokem, který v Cloudflare něco vytváří nebo nasazuje, počkej na potvrzení uživatele.

## Příkazy
Worker (`cd worker`):
- `npm install` – závislosti
- `npm test` – vitest + Miniflare (workerd), všechny milníky A1–A5
- `npm run typecheck` – `tsc --noEmit` (strict)
- `npm run check` – typecheck + testy
- `npm run vectors` – přegeneruje `spec/test-vectors/` z referenčních funkcí (po změně formátu protokolu; změnu pak musí splnit i node)
- `npx wrangler dev` – lokální běh (profil `dev`, mock Cloudflare API)

Node (`cd node`, Cargo workspace `ancord-node` + `ancord-cli`):
- `cargo clippy --all-targets -- -D warnings` – bez varování
- `cargo test` – vektory ze `spec/test-vectors/` + data plane (`tests/dataplane.rs`) + e2e test B1–B3 (spustí `wrangler dev` a 3 nody; vyžaduje `npm install` ve `worker/`, jinak se přeskočí; `ANCORD_SKIP_E2E=1` ho vypne)
- `cargo run -p ancord-node` – node (`WORKER_URL` je jediný povinný vstup, viz `node/README.md`)

`dev/` zatím neexistuje.
