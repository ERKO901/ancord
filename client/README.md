# ancord klient (Android a desktop)

Tauri 2 + Svelte 5 + TypeScript (`strict`) + čisté CSS. Návrh je v `ARCHITECTURE.md` §13. Tento adresář je zatím **D1 (kostra) a velká část D4 (UI)**: UI je hotové a otestované nad mock IPC, Rust shell umí identitu, adresu, QR, kontakty (bezpečnostní číslo), nastavení a šifrované úložiště. Zprávy a síť ještě ne.

## Stav proti milníkům (§13.14)

| | Stav |
|---|---|
| D0 vyčlenění `ancord-core` | **hotovo** (viz `ARCHITECTURE.md` §13.4); shell závisí na `ancord-core`, ne na `ancord-node` |
| D1 kostra | shell, CSP, capabilities (příkazy jen z kontraktu), identita + adresa + QR. Test hranic důvěry běží ve vitestu; skutečné spuštění okna a změření instalátoru a startu zatím ne |
| D2 úložiště a klíče | **hotovo pro desktop**: `src-tauri/src/store.rs` (redb, XChaCha20-Poly1305 po záznamech s AAD, HKDF klíč tabulky, `master_key` zabalený klíčem zařízení a volitelně Argon2id z hesla), klíč zařízení v Keychain / Credential Manager / Secret Service (`keyring`), bez něj souborový `device.key` 0600 (jen při zakládání profilu, existující profil se bez svého keychainu neotevře). Identita, kontakty a nastavení přežijí restart, profil s heslem se při `lock` zahodí z paměti, klíče (`SigningKey`, `StaticSecret`) se při uvolnění nulují. Automatický zámek po nečinnosti dělá UI (`App.svelte`). **Zbývá:** Android Keystore (D6), ověření Secret Service na stroji bez desktopu (bod 16 v §9) |
| D3 online klient | není: `message_send` vrací chybu, seznam konverzací je prázdný |
| D4 UI | seznam, konverzace s virtuálním seznamem, kontakty, bezpečnostní číslo, moje adresa, nastavení, zámek, světlé/tmavé téma |
| D5–D8 | není |

## Příkazy

```
npm install
npm run dev        # UI v prohlížeči s mock backendem (src/lib/mock), bez Rustu
npm run check      # svelte-check + vitest + build + rozpočet bundle
npm run tauri dev  # okno aplikace (vyžaduje WebKitGTK 4.1 na Linuxu)
```

Rust shell (`cd src-tauri`): `cargo clippy --all-targets -- -D warnings`, `cargo test`. Má vlastní workspace, `node/` se tím nemění.

## Architektura UI

- `src/lib/types.ts` – IPC kontrakt (příkazy, události). Rust strana je `src-tauri/src/contract.rs`; `test/contract.test.ts` hlídá, že seznamy příkazů v TS, `build.rs`, `lib.rs` a capabilities jsou shodné.
- `src/lib/ipc.ts` – jediná cesta do Rustu. Mimo Tauri (`vite dev`) se v dev buildu nahradí `src/lib/mock/backend.ts`; v release buildu je ta větev odstraněná.
- `src/lib/app.svelte.ts` – stav (runes), události z Rustu, zámek vymaže obsah z paměti UI.
- `src/lib/ui/VirtualList.svelte` + `virtual.ts` – vlastní virtuální seznam s proměnnou výškou řádků.
- `src/styles/tokens.css` – design tokeny (tmavé a světlé téma). Žádné webové fonty, ikony jsou v bundlu.

## Pravidla (§13.5)

Zprávy jsou čistý text (žádné HTML, Markdown, náhledy). UI nemá síť: CSP `connect-src ipc:`, žádný `fetch`. Klíče, tickety a adresy nodů UI nikdy nedostane.
