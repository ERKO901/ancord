# ancord

**A decentralized network for end-to-end encrypted messaging.** Anyone can run a node, nobody has to trust them, and their resources are measured. Cloudflare acts only as a control plane (registration, load balancing, push) and never touches message data.

Decentralizovaná síť pro E2E šifrované zprávy. Nody může provozovat kdokoli, nikdo jim nevěří a jejich zdroje se měří. Cloudflare slouží jen jako *control plane* (registrace, verifikace nodů, load balancing, push notifikace), zprávy a spojení klientů jdou přímo na nody.

> **Stav: pre-alpha.** Worker, node a klient jsou implementované a lokálně otestované, nic zatím není nasazené do produkce. Protokol ani formáty nejsou stabilní a projekt neprošel nezávislým bezpečnostním auditem. Nepoužívejte ho pro citlivou komunikaci.

## Principy

- **Otevřená síť:** důvěra vzniká z měření (CPU, místo na disku, propustnost, svědci), stáří a reputace, ne z identity operátora.
- **Cloudflare mimo data plane:** Worker klientovi jen jednou řekne, na který node se připojit (`/route`, podepsaný ticket). Pak klient komunikuje výhradně s nodem.
- **Skrytí IP:** každý node je za vlastním Cloudflare Tunnelem (`node-<id>.<doména>`), jeho IP zná jen Cloudflare. Node nikdy nedostane IP uživatele.
- **Node vidí jen ciphertext** pod blinded identifikátory schránek. Zprávy se replikují na několik anchorů (HRW podle epochy, kvorum), šifrování je X3DH + Double Ratchet.
- **Zabezpečený kanál nad TLS:** vlastní handshake klient↔node i node↔node, aby Cloudflare po ukončení TLS neviděl identifikátory schránek ani tickety.
- **Parametry v konfiguraci**, ne v kódu (profily `dev` / `prod` / `open`).

## Architektura

```
Klient ─(1) /route─► Worker (TS) + Durable Object + KV + Queues + R2   [Cloudflare]
   │                       │
   │ (2) top-3 nody + ticket│ Tunnel + DNS node-<id>.<doména>
   └─(3) WSS + ticket─► Node A ◄── podepsané requesty ──► Node B, Node C   [Rust]
```

Podrobně v [`ARCHITECTURE.md`](ARCHITECTURE.md) (zdroj pravdy), protokol v [`spec/PROTOCOL.md`](spec/PROTOCOL.md), model hrozeb v [`spec/THREAT_MODEL.md`](spec/THREAT_MODEL.md).

## Struktura repozitáře

| Složka | Obsah |
|---|---|
| [`worker/`](worker/README.md) | Cloudflare Worker (TypeScript): registrace, verifikace nodů, skóre, routing, push |
| [`node/`](node/README.md) | Node (Rust), sdílené jádro `ancord-core` a souborový CLI klient `ancord-cli` |
| [`client/`](client/README.md) | Grafický klient pro desktop a Android (Tauri 2 + Svelte 5) |
| [`spec/`](spec) | Specifikace protokolu, model hrozeb, referenční implementace a testovací vektory sdílené workerem a nodem |
| [`dev/`](dev/README.md) | Docker Compose se 6 nody za Cloudflare Tunnel a skripty pro lokální běh |

Worker a node nesdílejí žádný kód, jen specifikaci a testovací vektory ze `spec/test-vectors/`.

## Rychlý start (lokálně, bez Cloudflare)

Vyžaduje Node.js, Rust (stable) a `jq`.

```bash
cd worker && npm install && npm run check   # typecheck + testy (vitest, Miniflare/workerd)
cd ../node && cargo clippy --all-targets -- -D warnings && cargo test
cd ../dev && ./local.sh 3                   # Worker (wrangler dev) a 3 nody lokálně
```

`cargo test` zahrnuje end-to-end test, který spustí `wrangler dev` a 3 nody (vypnutí: `ANCORD_SKIP_E2E=1`). Použití CLI klienta je popsané v [`node/README.md`](node/README.md), grafický klient v [`client/README.md`](client/README.md).

## Stav

| Fáze | Obsah | Stav |
|---|---|---|
| A | Worker: registrace, challenge, routing, push | implementováno, otestováno lokálně |
| B | Node v Rustu: data plane, zabezpečený kanál, presence, push, X3DH + ratchet | implementováno, otestováno lokálně |
| C | Nasazení, DNS, 6 nodů v Dockeru | připraveno, zatím nenasazeno |
| D | Grafický klient (desktop, Android) | rozpracováno (D1–D5 hotové, D6 Android částečně) |

Přesné milníky a kritéria přijetí jsou v [`ARCHITECTURE.md`](ARCHITECTURE.md) §12.4 a §13.14. Mimo rozsah: iOS a tokenová ekonomika.

## Bezpečnost

Návrh počítá s nedůvěryhodnými nody a s Cloudflare jako pozorovatelem. Hrozby a jejich mitigace jsou v [`spec/THREAT_MODEL.md`](spec/THREAT_MODEL.md). Zranitelnost prosím nehlaste veřejným issue, ale soukromě přes GitHub Security Advisories (záložka *Security* repozitáře).

## Přispívání

Pull requesty jsou vítané. Zásady:

- Spec a testovací vektory před implementací; když se kód a [`ARCHITECTURE.md`](ARCHITECTURE.md) rozejdou, nejdřív se upraví dokument.
- Rust: `cargo clippy --all-targets -- -D warnings` a `cargo test` bez varování, žádné `unwrap` v produkční cestě.
- TypeScript: `strict`, testy přes vitest. Změna formátu protokolu znamená `npm run vectors` a úpravu nodu.
- Tajemství (API tokeny, tunnel tokeny, privátní klíče) nikdy do gitu.
- Dokumentace česky, identifikátory anglicky.

## Licence

[MIT](LICENSE)
