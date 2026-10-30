# ancord klient (Android a desktop)

Tauri 2 + Svelte 5 + TypeScript (`strict`) + čisté CSS. Návrh je v `ARCHITECTURE.md` §13. Hotové jsou **D1 (kostra), D2 (úložiště, desktop), D3 (online klient) a velká část D4 (UI)**: Rust shell umí identitu, adresu, QR, kontakty (bezpečnostní číslo), nastavení, šifrované úložiště a zprávy přes síť (živě, offline fronta, psaní, stav doručení).

## Stav proti milníkům (§13.14)

| | Stav |
|---|---|
| D0 vyčlenění `ancord-core` | **hotovo** (viz `ARCHITECTURE.md` §13.4); shell závisí na `ancord-core`, ne na `ancord-node` |
| D1 kostra | shell, CSP, capabilities (příkazy jen z kontraktu), identita + adresa + QR. Test hranic důvěry běží ve vitestu; skutečné spuštění okna a změření instalátoru a startu zatím ne |
| D2 úložiště a klíče | **hotovo pro desktop**: `src-tauri/src/store.rs` (redb, XChaCha20-Poly1305 po záznamech s AAD, HKDF klíč tabulky, `master_key` zabalený klíčem zařízení a volitelně Argon2id z hesla), klíč zařízení v Keychain / Credential Manager / Secret Service (`keyring`), bez něj souborový `device.key` 0600 (jen při zakládání profilu, existující profil se bez svého keychainu neotevře). Identita, kontakty a nastavení přežijí restart, profil s heslem se při `lock` zahodí z paměti, klíče (`SigningKey`, `StaticSecret`) se při uvolnění nulují. Automatický zámek po nečinnosti dělá UI (`App.svelte`). **Zbývá:** Android Keystore (D6), ověření Secret Service na stroji bez desktopu (bod 16 v §9) |
| D3 online klient | **hotovo**: `src-tauri/src/net.rs` (jedna úloha s `ancord_core::online::Online`, `/route`, admission s připnutým klíčem Workeru, znovupřipojení s backoffem, ping po 25 s), `core.rs` (`Persist` nad šifrovaným redb: přijatá zpráva se zapíše do `messages` před `ack`, stav ratchetu bez historie v `conversations`; fronta `queued` → `pending` → `delivered`/`unknown` podle `status` anchorů). E2E test `src/e2e.rs`: dva klienti přes `dev/local.sh 3` (Worker ve `wrangler dev` a 3 nody). **Zbývá:** přeměření paměti (PSS) s reálnými daty (bod 12 v §9), `sync_once` (D6), souběžné založení konverzace (bod 17 v §9) |
| D4 UI | seznam, konverzace s virtuálním seznamem, kontakty, bezpečnostní číslo, moje adresa, nastavení, zámek, světlé/tmavé téma |
| D5–D8 | není |

## Příkazy

```
npm install
npm run dev        # UI v prohlížeči s mock backendem (src/lib/mock), bez Rustu
npm run check      # svelte-check + vitest + build + rozpočet bundle
npm run tauri dev  # okno aplikace (vyžaduje WebKitGTK 4.1 na Linuxu)
```

Rust shell (`cd src-tauri`): `cargo clippy --all-targets -- -D warnings`, `cargo test`. Má vlastní workspace, `node/` se tím nemění. `cargo test` zahrnuje E2E test D3: spustí `dev/local.sh 3` na volných portech (potřebuje `npm install` ve `worker/`, jinak se přeskočí; `ANCORD_SKIP_E2E=1` ho vypne), trvá kolem minuty.

## Síť

Adresa a veřejný klíč Workeru jsou kořeny důvěry (§13.3) a zapékají se při buildu: `ANCORD_WORKER_URL`, `ANCORD_WORKER_PUBKEY` (b64u, z `/.well-known/ancord-key` při nasazení), volitelně `ANCORD_NODE_URL` (šablona adresy nodu, výchozí `wss://{hostname}/client`) a `ANCORD_POW_BITS` (obtížnost profilu, výchozí 10). Debug build je čte i za běhu z prostředí, release ne. Bez URL a klíče zůstane klient offline a zprávy čekají ve frontě.

Proti `dev/local.sh`:
```
ANCORD_WORKER_URL=http://127.0.0.1:8787 \
ANCORD_WORKER_PUBKEY=$(curl -s http://127.0.0.1:8787/.well-known/ancord-key | sed 's/.*"pubkey":"\([^"]*\)".*/\1/') \
ANCORD_NODE_URL='ws://127.0.0.1:8790/{hostname}/client' npm run tauri dev
```

## Architektura UI

- `src/lib/types.ts` – IPC kontrakt (příkazy, události). Rust strana je `src-tauri/src/contract.rs`; `test/contract.test.ts` hlídá, že seznamy příkazů v TS, `build.rs`, `lib.rs` a capabilities jsou shodné.
- `src/lib/ipc.ts` – jediná cesta do Rustu. Mimo Tauri (`vite dev`) se v dev buildu nahradí `src/lib/mock/backend.ts`; v release buildu je ta větev odstraněná.
- `src/lib/app.svelte.ts` – stav (runes), události z Rustu, zámek vymaže obsah z paměti UI.
- `src/lib/ui/VirtualList.svelte` + `virtual.ts` – vlastní virtuální seznam s proměnnou výškou řádků.
- `src/styles/tokens.css` – design tokeny (tmavé a světlé téma). Žádné webové fonty, ikony jsou v bundlu.

## Pravidla (§13.5)

Zprávy jsou čistý text (žádné HTML, Markdown, náhledy). UI nemá síť: CSP `connect-src ipc:`, žádný `fetch`. Klíče, tickety a adresy nodů UI nikdy nedostane.
