# Melo v2 – architektura

Decentralizovaná síť pro E2E šifrované zprávy. Nody provozuje kdokoli, nikdo jim nevěří, jejich zdroje se měří. Cloudflare slouží jako *control plane*: registrace, verifikace nodů, load balancing a push notifikace. Zprávy a spojení klientů jdou přímo na nody.

> Stav: fáze A a B jsou implementované a lokálně otestované (viz 12.4), nic není nasazené. Body označené **[ověřit]** se potvrdí ve fázi C na reálné infrastruktuře.

## 1. Cíle a hranice

- **Otevřená síť**: nody může spustit kdokoli. Důvěra vzniká z měření, stáří a reputace, ne z identity operátora.
- **Cloudflare mimo data plane**: Worker klientovi jen jednou řekne, na který node se připojit. Pak klient komunikuje výhradně s nodem.
- **Skrytí IP**: nody jsou za Cloudflare subdoménou (IP nodu zná jen Cloudflare). Node nikdy nedostane IP uživatele.
- **Bezpečnost před pohodlím**: node vidí jen ciphertext pod blinded identifikátory.
- **Implementace**: nody a CLI klient v Rustu, Worker v TypeScriptu, grafický klient pro Android a desktop v Tauri 2 s jádrem v Rustu a UI v TypeScriptu (sekce 13).

Mimo rozsah tohoto dokumentu: iOS, tokenová ekonomika (další vynechané části klienta jsou v 13.15).

## 2. Co v současném prototypu (Node.js) nefunguje

| Problém | Kde |
|---|---|
| Jeden sdílený `x-relay-secret` pro celý cluster, jeho únik kompromituje všechny nody | `relay.service.js` |
| `rejectUnauthorized: false` na inter-node komunikaci | `discovery.service.js`, `relay.service.js` |
| Klíč nodu = hash(`NODE_SECRET` + IP): předvídatelný a mění se s IP | `identity.service.js` |
| Ring se skládá z pollingu Flux API s hysterezí, takže ring různých nodů diverguje; cyklické závislosti `discovery ↔ rebalance ↔ relay` | `discovery.service.js` |
| PoW (4 nuly) je jen pro uživatele, nody nic nedokazují | `pow.service.js` |
| FCM tokeny uživatelů leží v DB na každém nodu | `notification.service.js` |
| Každý operátor musí provozovat MongoDB | `models/*` |

Co se přenáší jako návrh (ne kód): blinded mailbox model, ratchet šifrování, tok `send_message` → anchory → push, TTL úklid.

## 3. Přehled

```
                    ┌────────────────────── Cloudflare ──────────────────────┐
                    │  Worker (TS)         Durable Object        KV          │
Klient ─(1) /route─►│  API, tickety  ◄───► registr, skóre, load ─► routing   │
   ▲                │      │               (WS hibernation)       snapshot   │
   │                │   Cron: challenge   Queues: push    R2: bandwidth bloby│
   │                └──────┬─────────────────────────────────────────────────┘
   │(2) top-3 nody + ticket│ Tunnel + DNS node-<id>.<doména>
   │                       ▼
   └─(3) WSS + ticket─► Node A ◄── podepsané requesty ──► Node B, Node C
                         (cloudflared, Rust)             (anchory mailboxu)
```

Tok:
1. Klient zavolá `GET /route`. Worker přečte routing snapshot z KV a vrátí 3 nody s podepsaným ticketem.
2. Klient se připojí na první node přes WSS (Cloudflare Tunnel) a předloží ticket.
3. Node (gateway) spočítá anchory mailboxu příjemce a oznámí jim, že je mailbox online u něj (presence). Odesílatelův node zprávu uloží na anchory, ti ji online příjemci okamžitě přepošlou na jeho gateway. Pokud příjemce online není, zpráva čeká na anchorech a anchor požádá Worker o push (viz 6.3).

## 4. Skrytí IP

### Nody
- Při registraci Worker přes Cloudflare API vytvoří **vlastní Tunnel pro každý node** a DNS záznam `node-<id>.<doména>` (např. `node-qx7c2a5k3m4ztdwp.melonet.cz`). Vše je plně automatické: node zná jen adresu Workeru, sám si vygeneruje klíč, zaregistruje se, dostane tunnel token a spustí `cloudflared`. Nepotřebuje veřejnou IP, otevřený port ani vlastní certifikát.
- **Jméno je jedna úroveň pod zónou**, ne `<id>.nodes.<doména>`: bezplatný univerzální certifikát pokrývá jen `*.<doména>`, dvouúrovňová jména by vyžadovala placený Advanced Certificate Manager. `<id>` je krátký prefix `node_id` (base32, např. 10 bajtů; DNS label má limit 63 znaků). Při kolizi Worker registraci odmítne.
- Worker smí upravovat a mazat **pouze** DNS záznamy s prefixem `node` a tunely, které sám vytvořil. Ostatní záznamy zóny se nedotýká.
- Hostname vlastníme my, takže odpadá `rejectUnauthorized: false` a důkaz kontroly endpointu je jednoduchý (viz 5.1).

### Uživatelé
Cloudflare k požadavku přidává `CF-Connecting-IP`, `X-Forwarded-For`, `True-Client-IP` a podobné hlavičky. Operátor nodu by je viděl, proto:
- **Varianta A (doporučená)**: Transform Rule odstraňující hlavičky s IP návštěvníka, **omezená výrazem na hostname `node-*.<doména>`**. Managed transform „Remove visitor IP headers“ platí pro celou zónu, takže na sdílené (firemní) zóně by změnil chování ostatních služeb a nepoužívá se. Žádný Worker v data plane, nejlevnější a nejrychlejší. **Změřeno v C3: pravidlo IP neskryje.** Odstraní jen hlavičky podvržené klientem; `cf-connecting-ip` a `x-forwarded-for` (od `cloudflared`) s reálnou IP na origin dorazí vždy (sekce 9, bod 1). Varianta A tedy stačí jen pro vlastní nody.
- **Varianta B**: gateway Worker před nodem, který hlavičky sestaví sám. Největší kontrola, ale Worker je v data plane, přidává latenci a účtuje se.
- Ticket je vázaný na ephemeral klíč klienta, ne na IP.

### Důsledky
- IP uživatele zná Cloudflare a (podle měření C3) i operátor nodu, ke kterému se klient připojí. Pro otevřenou síť je nutné rozhodnout o variantě B (sekce 9, bod 1).
- Měření (bandwidth, RTT, witnessy) probíhá přes Cloudflare, měří se tedy to, co dostane klient.
- **[ověřit]** limity počtu Tunnelů a DNS záznamů na účet, podmínky Cloudflare pro WebSocket provoz, případně sdílený tunnel s routingem podle hostname.

### Node bez veřejné IP
- Node nevyžaduje veřejnou IP ani otevřený port. Jediný požadavek je odchozí spojení na Cloudflare (`cloudflared` si ho otevře sám). Funguje za NAT i CGNAT, takže node může běžet na domácím serveru, Raspberry Pi nebo VPS bez veřejné adresy.
- Slabší linka znamená nižší naměřenou kapacitu a tím menší podíl provozu. Bandwidth challenge (5.2) to řeší bez zvláštních pravidel.
- Node ↔ node provoz jde stejnou cestou přes subdoménu, nody se nepotřebují vidět přímo.

### Závislost na Cloudflare a cena
- Subdoména i proxy jsou u Cloudflare, takže Cloudflare může nody odpojit bez ohledu na to, zda běží tunnel. Tunnel tuto závislost nevytváří, jen odstraňuje potřebu veřejné IP. Je to akceptované riziko celého návrhu.
- Transport musí zůstat vyměnitelný: klient a node nesmí do protokolu zapékat nic specifického pro Cloudflare (volitelně záložní přímé adresy v adresáři, případně jiný tunel).
- Cloudflare Tunnel, DNS a proxy provoz jsou zdarma **[ověřit]**. Účtují se Workers, Durable Objects, Queues, KV a R2, ne provoz zpráv přes tunel.
- Rizikové limity **[ověřit]**: počet tunelů na účet, počet DNS záznamů na zónu, idle timeout WebSocketu (heartbeat/ping ho musí držet), limity velikosti požadavků, podmínky služby pro tento typ provozu.

## 5. Control plane (Cloudflare Worker, TypeScript)

### 5.1 Identita a registrace nodu
- Node má **perzistentní náhodný Ed25519 klíč** uložený na disku. `node_id = blake3(pubkey)`.
- Registrace má tři kroky (přesný formát v `spec/PROTOCOL.md` §4):
  1. `POST /register {pubkey, claimed_specs}`: Worker vrátí `challenge_nonce`, hostname a obtížnost hashcash (hashcash je vázaný na nonce, proto až po výzvě).
  2. `POST /register/confirm {pubkey, challenge_nonce, signature, hashcash_counter}`: node podepsal `nonce ‖ hostname ‖ pubkey`. Worker ověří podpis, hashcash a limity, vytvoří Tunnel, nastaví jeho ingress a DNS záznam a vrátí tunnel token. Node je ve stavu `pending`.
  3. `POST /register/activate` (podepsaný request): až po startu `cloudflared` je hostname dosažitelný, takže Worker teprve teď zavolá `https://<hostname>/.well-known/melo-proof` a ověří, že node ten klíč opravdu drží. Node přejde na `active`. Nevyřízený `pending` node po `pending_ttl_s` propadne a jeho zdroje se smažou.
- Provisioning je saga: při selhání kteréhokoli kroku Worker smaže, co už vytvořil. Opakovaná registrace téhož klíče (restart nodu) vydá znovu token bez nových zdrojů.
- **Sybil brzda**: hashcash s adaptivní obtížností (roste při vlně registrací) a limity na ASN a /24 (`request.cf.asn`).
- **Probation**: nový node dostává jen malý podíl provozu. Váha roste s ověřenou uptime a úspěšnými challenge. Tisíc rychle spuštěných nodů tedy nezíská žádný vliv.

### 5.2 Důkazy zdrojů
Principy: `claimed_specs` se nevěří. Efektivní kapacita = `min(claimed, measured)`. Ověření musí být pro Worker levné (účtuje se CPU čas), pro nod drahé.

| Zdroj | Mechanismus |
|---|---|
| RAM + storage | **Proof of Space s Merkle commitmentem.** Worker pošle seed, node vygeneruje memory-hard tabulku N MB/GB (každý blok závisí na předchozím a na 4 náhodných dřívějších) a pošle Merkle root. Worker vyzve k *k* náhodným listům s deadlinem v řádu ms, node vrátí list + Merkle path, Worker ověří *k* hashů. Latence odliší RAM, disk a „počítám za běhu“. Opakuje se s novým seedem. |
| CPU | **Časovaný sekvenční hash-chain.** Node vrátí checkpointy, Worker přepočítá jen pár náhodných krátkých segmentů. Čas odpovědi = single-thread skóre. Multi-core: paralelní nezávislé řetězy. |
| Bandwidth | (a) download náhodného blobu z **R2** (egress zdarma) přes presigned URL, Worker ověří hash; (b) upload omezeného blobu; (c) **peer witnessy**: Worker náhodně vybere 2–3 nody z různých ASN, ti navzájem změří throughput a pošlou podepsané reporty, skóre = medián. |
| Latence / region | RTT z Cloudflare colo při challenge a RTT hlášené klienty (agregované). Region je naměřený, ne tvrzený. |

Průběžně: náhodně načasované re-challenge (DO alarm, cron ho jen znovu spustí), podepsaný heartbeat (load, počet spojení). Selhání snižuje skóre, opakované podvádění znamená ban `pubkey` a penalizaci ASN.

Implementace: challenge běží po heartbeat WebSocketu, nejvýše jeden na node. Výsledek rozlišuje čtyři případy: platný důkaz (kapacita se zapíše), chybějící odpověď (jen zdraví), příliš pomalá odpověď (kapacita dané dimenze 0, bez trestu) a neplatný důkaz (kapacita 0 a trestný bod; trestné body se počítají po druzích výzev a při `score.banStrikes` po sobě jdoucích v téže výzvě následuje ban). Proof of Space se zavazuje jednou (`space_commit`) a pak se opakovaně otevírá nad uloženým kořenem (`space_open`), nový seed jen po `commitMaxAgeS`. Nezměřená dimenze znamená nulovou kapacitu, takže nový node nedostane provoz, dokud neprojde všemi zapnutými výzvami.

**Poctivý limit**: proti odhodlanému lháři nelze kryptograficky dokázat „mám 16 GB RAM“. Obrana je kombinace měření, probation a ekonomické neatraktivity podvodu (node bez reálných zdrojů dostane málo provozu).

### 5.3 Load balancing a routing
- Skóre nodu = `capacity_measured × health × trust − load_penalty`.
- Výběr: **power-of-two-choices** nad váženou množinou kandidátů filtrovanou podle regionu/RTT klienta (`request.cf`). Vrací **3 nody** pro failover.
- DO každých několik sekund publikuje **routing snapshot do KV**. `GET /route` čte jen KV (+ Cache API) a DO neprobouzí: nízká cena a minimální latence.
- **Ticket**: Ed25519 podpis Workeru nad `{nid, ek, exp, n}` (`node_id`, efemerní klíč klienta, expirace, nonce), krátká platnost; každý ze 3 nodů dostane vlastní ticket. Node bez platného ticketu spojení odmítne (admission control). Klient má veřejný klíč Workeru zapečený.
- **Výpadek Workeru**: Worker není v data plane. Klient cachuje poslední podepsaný adresář nodů a připojí se bez něj. Nefunguje jen nový routing, registrace a push.
- **Důvěra ve Worker**: kompromitovaný Worker může cenzurovat nebo routovat na škodlivé nody, ne číst zprávy (E2E). Budoucí zmírnění: více podepisovatelů adresáře nebo transparency log.

### 5.4 Push notifikace
- Worker drží FCM/APNs přihlašovací údaje, **nody je nevidí**.
- Klient registruje push token přímo Workeru pod blinded `push_id` odvozeným z mailbox tajemství. Token je uložen šifrovaně.
- Anchor, který přijme zprávu pro mailbox **bez online odběratele** (nemá platnou presence, viz 6.3), pošle podepsaný `POST /wake {push_id, mailbox_id}`. Worker ověří, že node je registrovaný a v dobrém stavu, že je podle aktuální epochy **skutečným anchorem** daného mailboxu, a aplikuje rate-limit per node a per `push_id` s koalescencí. Pak přes **Queue** odešle tichý data-push bez obsahu a bez adres nodů.
- Worker vidí jen `push_id` a čas.

## 6. Data plane (nody, Rust)

### 6.1 Anchory
Anchory zůstávají. Mění se zdroj členství:
- Ring = **podepsaný adresář s číslem epochy** z Workeru, nody ho dostávají v odpovědi na heartbeat. Všechny nody ve stejné epoše počítají stejné anchory, takže odpadá divergence ringu i hystereze.
- **Vážený rendezvous (HRW) hash** podle naměřené kapacity místo prostého consistent hash.
- Replikace R=3, zápis s kvorem 2. Při změně epochy handoff a rebalance na pozadí.
- Anchory ukládají jen **ciphertext** pod blinded `mailbox_id` (hash párového sdíleného tajemství). TTL úklid.
- Úložiště: embedded (redb nebo RocksDB), žádný MongoDB.

### 6.2 Šifrování a transport
- **Cloudflare ukončuje TLS**, takže samotné WSS/HTTPS přes Tunnel Cloudflare před `mailbox_id`, tickety a `push_id` neskryje. Proto platí dvě vrstvy: vlastní autentizovaný šifrovaný kanál (jako v prototypu `melo`, kde měl `enc_handshake_*` a ratchet nad Socket.IO, jen důsledněji) a nad ním E2E ratchet zpráv. Specifikace: `spec/PROTOCOL.md` §14.
- Klient ↔ node: po `hello` výměna efemerních X25519 klíčů, **node podepíše přepis handshaku svou Ed25519 identitou** a klient ho ověří proti `node_id` z ticketu podepsaného Workerem (klíč Workeru má klient připnutý mimo síť). Pak jsou všechny rámce šifrované (ChaCha20-Poly1305, čítač, padding na 256 B), včetně ticketu a `subscribe`. Cloudflare vidí jen IP, čas a zaokrouhlenou velikost.
- Node ↔ node: **session** navázaná podepsaným requestem iniciátora (`/n/hs`), efemerním X25519 a podepsanou odpovědí respondera (identita z adresáře), pak operace šifrované XChaCha20-Poly1305 (`/n/x`) s ochranou proti replayi. Holé API neexistuje. Session se rotuje po 10 minutách (forward secrecy). Později QUIC (quinn).
- Bez důvěry v první stažení: klíč Workeru je připnutý v konfiguraci nodu i klienta (`WORKER_PUBKEY`), jinak by Cloudflare mohl podvrhnout adresář a tickety.
- Anti-spam: rate-limit a PoW per klient zůstávají, plus kvóty per ticket.

### 6.3 Online doručení a presence

> **Oprava návrhu.** První verze tohoto dokumentu a `spec/PROTOCOL.md` §13 popisovaly jen `send`, `fetch` a `ack`, takže si klient zprávy musel tahat sám a push se posílal vždy. To byl omyl: vypadl z toho hlavní mechanismus původního prototypu (`../melo`). Správně je to takto a implementace i spec se mají podle toho upravit.

Worker slouží klientovi **jen k získání 3 nejbližších a nejvýkonnějších nodů** (5.3). Dál se Worker doručování zpráv neúčastní. Klient se připojí na nejlepší z nich, tomu se říká **gateway**.

Tok:
1. **Presence.** Klient po připojení pošle gateway `subscribe {mailbox_id}`. Gateway spočítá anchory mailboxu (6.1) a každému z nich pošle podepsané `presence {mailbox_id, gateway, exp}`: „tento mailbox je online u mě“. Anchor si to uloží s krátkým TTL. Gateway presence pravidelně obnovuje a při odpojení klienta ji zruší.
2. **Odeslání.** Odesílatelův node (koordinátor) spočítá stejné anchory a zprávu na ně pošle (R=3, kvorum 2, jak v 6.1).
3. **Okamžité doručení.** Anchor zprávu uloží. Pokud má pro mailbox platnou presence, přepošle ji na gateway příjemce (node↔node, podepsaný request). Gateway ji (už zašifrovanou klientem, ratchet je klientská věc) pošle příjemci po otevřeném WSS. Anchor odpoví `delivered_online`.
4. **Označení doručeno.** Zprávu označí za doručenou (náhrobek) klientský `ack` na všech anchorech. Předání gatewayi náhrobek nevytváří: kdyby spojení zaniklo dřív, než klient zprávu přečetl, zpráva by se ztratila. Online doručení tak jen vynechá wake, zpráva zůstává `pending` do `ack` a případné duplicity klient zahodí podle `id`.
5. **Příjemce není online.** Nikdo nemá presence, zpráva zůstane `pending` na anchorech a anchor pošle Workeru `wake` (5.4). Po připojení klient udělá `subscribe` i `fetch` a vyzvedne, co čekalo.
6. **Záloha.** Když doručení online selže (presence je zastaralá, gateway nedostupná), zpráva zůstává `pending` a klient ji dostane přes `fetch`. Doručení je *at least once*, `id` zprávy je idempotentní, takže duplicity klient zahodí.

Důsledky:
- Wake push se posílá jen tehdy, když zprávu nešlo doručit online.
- Anchor je tedy zároveň úložiště i **rozcestník**: ví, kde je příjemce, a zprávu mu pošle sám. Odesílatel nemusí gateway příjemce znát.
- Presence je jen v paměti anchoru s krátkým TTL. Při změně epochy ji nové anchory dostanou z nejbližšího obnovení gateway, handoff presence se nepřenáší.
- Anchor ví z presence, na kterém nodu je mailbox, ale ne kdo za ním stojí (`mailbox_id` je blinded).
- Rozdíl oproti prototypu: tam se používal ještě záložní okruh 5 anchorů a presence v DB. Tady je presence v paměti a kvorum je řízeno adresářem epochy, žádný záložní okruh navíc není, dokud se neukáže potřeba.

### 6.4 Identita uživatele a funkce konverzace

Prototyp `melo` měl uživatele s veřejným klíčem, vyhledání profilu, psaní a stav doručení. Ve v2 to platí také (`spec/PROTOCOL.md` §15):
- **Adresa je veřejný klíč** (Ed25519). K němu patří podepsaný profil s X25519 klíčem pro dohodu tajemství. Profil leží na třech anchorech podle `sha256(pk)`, stejně jako mailboxy (HRW, kvorum), a hledá se podle `pk`.
- **Napsat komukoli lze bez předchozí výměny tajemství**: iniciátor provede X3DH s podepsaným rotujícím prekey a jednorázovým prekey příjemce a pošle mu šifrovanou obálku do jeho schránky (`inbox_id` z adresy). Anchor v obálce nevidí odesílatele. Příjemce ji zpracuje, tajné prekeys po použití resp. rotaci smaže a obě strany pak používají mailboxy a Double Ratchet (podepsaný prekey příjemce je jeho první ratchet klíč). Únik dlouhodobých klíčů tak neprozradí zprávy, jejichž prekeys už neexistují (forward secrecy). Adresu protistrany lze ověřit mimo síť bezpečnostním číslem.
- **Typing** je nespolehlivý efemérní signál přes presence (gateway → anchor → gateway příjemce), nikdy se neukládá.
- **Stav doručení** zjistí odesílatel dotazem na anchory (`pending`/`delivered`/`unknown`).
- **Historie** je lokální u klienta. Server po `ack` drží jen náhrobek a klíč zprávy se v ratchetu zahazuje, takže starý ciphertext je nedešifrovatelný; držet ho déle by zvětšovalo, co operátor vidí, a nic by nepřineslo.
- Dávkové odběry (`mailbox_ids`) jako v prototypu.
- Uživatel je pro node pořád anonymní (ticket, žádné přihlášení); omezení zahlcení je per spojení a hashcash u profilu, ne per uživatel.
- **Sdílení adresy**: `ancord:<pk>` jako text nebo QR kód (spec §15.10). Kdo adresu dostane, napíše bez jakékoli další výměny.
- **Push**: `push_id` schránky je v podepsaném profilu, takže i úplně první zprávu (obálku) anchor může doručit pushem. `push_id` mailboxu konverzace se odvodí ze sdíleného tajemství, zná ho jen dvojice (spec §15.11).
- **Online klient** (spec §15.12): `/route` → připojení na gateway → zveřejnění profilu → `subscribe` schránky a všech konverzací (presence na anchorech) → zprávy chodí živě. Po odpojení presence zanikne a anchory budí pushem. CLI: `ancord online`.

Cena: operátor anchoru profilu vidí, že adresa `pk` existuje a kdy aktualizovala profil. Profil je veřejný z podstaty. Neví, s kým adresa mluví (mailboxy jsou blinded, obálka skrývá odesílatele). Uživatel musí profil zveřejňovat aspoň jednou za týden (rotace prekey). Post-compromise security zajišťuje Double Ratchet (§13.2 specifikace; bod 11).

## 7. Cloudflare zdroje a cena

| Služba | Použití |
|---|---|
| Workers (TS) | API, tickety, ověřování |
| Durable Objects | registr, skóre, load; shardování podle regionu |
| D1 / DO SQLite | perzistence registru |
| KV | routing snapshot |
| Queues | push |
| R2 | bandwidth bloby (egress zdarma) |
| Cron Triggers | sweep challenge |
| Turnstile / Rate Limiting | ochrana registrace |
| Tunnel + DNS | skrytí IP nodů |

- Největší nákladová položka jsou **heartbeaty**: nody drží WebSocket na DO (hibernation API), interval 30–60 s. **[ověřit]** aktuální ceník (příchozí WS zprávy se účtují s poměrem přibližně 20:1).
- `/route` = čtení z KV + podpis ticketu přes WebCrypto, zlomky ms CPU.

### Proč TypeScript pro Worker
- Cold start isolate je v řádu ms nezávisle na jazyce, Rust→WASM přidá instanciaci a velikost bundlu.
- Ověřování je triviální: Ed25519 a SHA-256 jsou nativně ve WebCrypto, Merkle ověření jsou pár hashů. Chybí výpočetní hotspot, kde by Rust vyhrál.
- Nejlepší podpora DO, KV, Queues a R2. Rust (workers-rs) zvážit, jen pokud profilování ukáže CPU problém.
- Mezi Workerem a nody se sdílí **specifikace a test vektory**, ne kód.

## 8. Hrozby

| Útok | Obrana |
|---|---|
| Sybil (tisíce nodů) | hashcash, limity ASN a /24, probation, váha podle stáří a měření |
| Lhaní o specs | měřené challenge, `min(claimed, measured)`, náhodné re-challenge |
| Node přeposílá challenge jinam | tvrdé deadliny v ms, memory-hard odvození, kontrola RTT |
| Škodlivý node čte zprávy | E2E, blinded mailboxy, jen ciphertext |
| Node zahazuje zprávy | replikace R=3 + kvorum, delivery challenge, skóre |
| Operátor nodu zjišťuje IP uživatelů | Tunnel, odstranění IP hlaviček na zóně, ticket vázaný na ephemeral klíč |
| Push spam | jen skutečný anchor, rate-limit, koalescence |
| Replay a impersonace nodů | podepsané requesty, okno nonce, klíče z adresáře |
| DoS na Worker | Cloudflare edge, KV cache, Turnstile, hashcash |
| Kompromitace Workeru | nečte obsah; budoucí multi-signer adresář |
| Cloudflare jako jediný bod | akceptované riziko; vidí IP a metadata, ne obsah |
| Chyba nebo škodlivá závislost v UI klienta | klíče jen v Rustu, WebView bez sítě (CSP), IPC jen s allowlistem (13.3, 13.5) |
| Screenshot, nahrávání, čtení obrazovky na zařízení | `FLAG_SECURE`, ochrana přístupnostních dat, content protection na desktopu; ne proti rootu a kompromitovanému OS (13.9) |
| Ztráta nebo krádež zařízení | šifrované úložiště, klíč v Keystore/keychainu, volitelné heslo a zámek (13.6) |
| Push `up` zneužitý k útoku na cizí URL | jen `https:443`, jméno hostitele, bez redirectů, limit per hostitel (13.8) |

## 9. Otevřené body

1. Managed transform vs gateway Worker pro odstranění IP uživatele (včetně WebSocket).
2. Tunnel na node vs sdílený tunnel; limity účtu.
3. ~~Parametry Proof of Space~~ vyřešeno měřením (níže): blok odkazuje na 4 dřívější bloky místo jednoho (`spec/PROTOCOL.md` §9.2), *k* = 16 otevření, tabulka `ram` 2^16–2^22 bloků (2–128 MiB), `storage` 2^18–2^26 (8 MiB – 2 GiB), deadline `ram` 150 ms a `storage` 1000 ms (`prod`, `open`). Důkaz je dolní mez s několikanásobnou tolerancí, ne přesné číslo. RAM od NVMe neodliší.
4. Skutečná cena heartbeatů přes DO WebSocket.
5. Zda gateway/Tunnel provoz zprávové aplikace vyhovuje podmínkám Cloudflare.
6. Multi-signer adresář: kdy a jak.
7. ~~Split-brain a expirace adresáře~~ vyřešeno (`spec/PROTOCOL.md` §6.1): nody v heartbeatu hlásí peery, na které se opakovaně nedovolaly. Worker hlášení nevěří, jen podle něj sám ověří `GET /health` přes tunel a node, který dvakrát neprojde, vyřadí z dalšího adresáře i routingu (nejvýše třetinu živých nodů). Adresář nemá expiraci: při výpadku Workeru epocha zamrzne, připojení klienti si píší dál a noví čekají na tickety. Node, který zmeškal epochu, si novější adresář stáhne od peeru, který mu odpoví `403` (podpis Workeru, `/n/directory`). Ověření dělá Worker sám, ne vybraní nody: jde stejnou cestou jako nody (Cloudflare → tunel) a Sybil ho neovlivní.
8. ~~Konfliktní model mailboxu při zápisu s kvorem 2~~ vyřešeno v B3: zápis je idempotentní podle `(mailbox_id, id)`, doručení je náhrobek a „doručeno“ vyhrává, pořadí je podle `(ts, id)`, čtení sjednocuje odpovědi `r − quorum + 1` anchorů (`spec/PROTOCOL.md` §13).
9. Perzistence klíče a dat nodu na Flux (instance se může přesunout na jiný stroj; ztráta klíče znamená nový `node_id` a ztrátu anchorů).
10. Velikost sítě nad stovky nodů: shardování registru (DO) a frekvence epoch. Pro 3–20 nodů se neřeší.
11. ~~Post-compromise security~~ vyřešeno: Double Ratchet s DH krokem, šifrovanou hlavičkou (anchor nevidí `dh`, `pn`, `n`, `seq`) a keepalive zprávou pro uzdravení jednostranných konverzací (`spec/PROTOCOL.md` §13.2, klient `node/crates/ancord-node/src/ratchet.rs`, keepalive v `ancord-cli`). Spam do schránky (`inbox`) zdražuje stráž s proof-of-work (§15.13, `guard.rs`); plně ho nezastaví. Zbývá ztráta klíčů zařízení (záloha vs. forward secrecy).
12. Klient: výkon systémového WebView (zejména WebKitGTK na Linuxu a starší Android System WebView) proti rozpočtu v 13.11. **[ověřit]** v D1 a D3, rozhodnout o minimální verzi WebView.
13. Klient: zda lze na Androidu spustit `sync_once` z receiveru bez Activity a WebView v Tauri mobile (13.10). **Ověřeno (2026-10-30, emulátor API 36 x86_64):** lze, bez Activity a bez WebView. `BroadcastReceiver` (`SyncReceiver.kt`, `goAsync` + vlákno) načte `libancord_client_lib.so` a zavolá JNI `Java_cz_melonet_ancord_SyncReceiver_nativeSync` (`client/src-tauri/src/android_sync.rs`), ta otevře profil ze `dataDir` a spustí `Core::sync_once` bez Tauri runtime (stačí `tauri::async_runtime`). Proces po broadcastu vznikl sám (`Start proc … for broadcast`), `MainActivity` neexistovala, WebView se neinicializoval. Offline zpráva z `ancord send` se přijala (návrat 1), druhý broadcast 0, po otevření aplikace je zpráva uložená a čitelná. Celé kolo trvalo do 1 s na localhostu. Zbývá: spouštění z UnifiedPush receiveru a foreground služby (limity Androidu pro start služby z pozadí), profil s heslem (zůstane zamčený, `sync_once` ho neotevře) a Keystore místo `device.key`. JNI export je jediné `unsafe` (`allow(unsafe_code)` jen na té funkci).
14. Klient: dostupnost a chování `setAccessibilityDataSensitive` (API 34), `setContentSensitivity` (API 35) a `setHideOverlayWindows` (API 31) na reálných zařízeních a v WebView. **[ověřit]** v D6, výsledek do `THREAT_MODEL.md`.
15. Klient: UnifiedPush `up` v Workeru (13.8): přesný tvar endpointů distributorů, zda stačí prosté `POST` bez šifrování WebPush, a limit per hostitel. **[ověřit]** v D7 proti ntfy.
16. Klient: Secret Service na Linuxu bez desktopového prostředí (režim s heslem jako náhrada) a `PR_SET_DUMPABLE`/WER bez `unsafe`. **[ověřit]** v D2 a D5.
17. Klient: souběžné založení konverzace. Když si dva uživatelé přidají adresu navzájem dřív, než jeden z nich zpracuje obálku druhého, vzniknou dvě různé konverzace: každá strana píše do své a obálku protistrany zahodí, protože s ní už konverzaci má (`online.rs`, stav ratchetu se nepřepisuje). Klient to zmírňuje tím, že po připojení nejdřív zpracuje schránku (`attach`) a teprve potom zakládá konverzace s novými kontakty. Řešení (deterministický výběr vítěze, např. menší `pk` iniciátora, a nahrazení dosud nepoužité konverzace) patří do `ancord-core` a spec §15.4. Do té doby `conversation_delete` maže jen historii zpráv, stav ratchetu zůstává, aby šlo s kontaktem dál psát.

**Měření C3 (2026-10-29, 6 nodů v Dockeru za vlastními tunely, zóna Free, Worker `prod`):**
- **Bod 1, IP uživatele:** změřeno echo serverem za tunelem nodu, u běžného GET i u WebSocket upgrade stejně. Transform Rule `ancord_strip_visitor_ip` funguje: hlavičky `x-forwarded-for`, `true-client-ip` a `x-real-ip` podvržené klientem na origin nedorazí. Skutečná IP uživatele ale na origin dorazí vždy, v `cf-connecting-ip` (Cloudflare ji pravidlem odstranit nedovolí) a v `x-forwarded-for`, kterou přidává až `cloudflared`. Dále dorazí `cf-ipcountry`. **Varianta A tedy IP před provozovatelem nodu neskryje**, jen před cizími hlavičkami. Node tyto hlavičky nečte ani neloguje. U vlastních (genesis) nodů je to přijatelné, pro otevřenou síť je potřeba gateway Worker (varianta B) nebo jiný transport. Rozhodnutí je otevřené.
- **Idle timeout WebSocketu** přes tunel: 125 s bez provozu, pak Cloudflare spojení zavře. Klient posílá ping po 25 s nečinnosti, node posílá Workeru keepalive každých 30 s (heartbeat `prod` 45 s), obojí je s rezervou.
- **Bod 2, limity:** 6 tunelů (1 na node) a 6 DNS záznamů prošlo bez problému. Podle dokumentace Cloudflare má účet limit 1000 tunelů a zóna Free založená po 1. 9. 2024 limit 200 DNS záznamů (teď 25). Pro desítky nodů tunel na node stačí, sdílený tunel zatím není potřeba. Nezměřeno, převzato z dokumentace **[ověřit]** při růstu.
- **Rozběh:** nový tunel začne Cloudflare směrovat po 1–2 minutách (mezitím `530`, node aktivaci opakuje sám). Node je v adresáři až od další epochy (`prod`: 300 s). Do té doby klient dostane `insufficient_nodes`. Po restartu nodu zůstávají staré konektory tunelu chvíli viditelné (8 místo 4), provoz to neovlivní.
- **Funkčně:** všech 6 nodů `active` a `live`, `https://node-<id>.melonet.cz/health` 100–220 ms, zpráva klient → WSS → gateway → 2 anchory → příjemce, po vypnutí gateway nodu se další zprávy doručí přes jiné nody.
- **Bod 5 (podmínky Cloudflare)** se měřením neověří, zůstává otevřený.

**Měření D1 (2026-10-30, Linux, WebKitGTK 4.1, Xvfb bez GPU, release build):** binárka shellu 4,6 MB (strip, LTO), JS bundle 25,9 kB gzip, CSS 3,2 kB. Aplikace se spustí a drží se naživu, bez chyby v logu. Paměť po 6 s v klidu: proces shellu 147 MB RSS + `WebKitWebProcess` 157 MB RSS = **~300 MB, cíl 150 MB je překročen**; RSS počítá sdílené stránky knihoven a běželo se bez GPU, proto je potřeba přeměřit PSS na reálném stroji (D3, D8). Instalátor (`tauri build`) a studený start v ms se zatím neměřily (chybí nástroje balení a časování okna).
- Chyba nalezená až na skutečném Cloudflare: `fetch` uložený v objektu a volaný jako metoda selže ve workerd s `Illegal invocation`. Opraveno a testovací fetchery to nyní hlídají.

**Měření bodu 3 (2026-10-29, `node/crates/ancord-node/examples/space_bench.rs`, 24 jader, NVMe, release build):**
- **Poctivý node** (4 odkazy): stavba 2^22 v RAM 1,7 s, otevření 16 indexů 0,2 ms. Storage 2^22 11,6 s, 2^26 (2 GiB tabulka + 2 GiB strom) 205 s. Otevření z page cache 1,5–1,9 ms, ze studeného NVMe 55–74 ms (vyprázdněno `posix_fadvise(DONTNEED)`). S jedním odkazem byla stavba 2^26 65 s a studené otevření ~34 ms. Pomalejší stavbu dělají čtyři `pread` po 32 B na blok, proto `commitTimeoutMs` v `prod` a `open` je 1800 s.
- **Útočník s částí tabulky** (drží každý *s*-tý blok, chybějící dopočítá; tabulka 2^22, nejdražší z 16 nezávislých otevření, tj. paralelní útočník):

  | uloženo | 1 odkaz (původní) | 4 odkazy |
  |---|---|---|
  | 1/2 | ~10 hashů | ~7 tisíc |
  | 1/3 | ~150 | ~190 tisíc |
  | 1/4 | ~2 tisíce | ~500 tisíc |
  | 1/8 | ~160 tisíc | ~1,8 milionu |
  | 1/16 | ~1 milion | ~2,8 milionu |

  Jedno jádro spočte ~3,4 milionu hashů za sekundu (4 odkazy), resp. ~5,5 milionu (1 odkaz). Při deadlinu 150 ms tak s původní konstrukcí prošel útočník s **osminou** deklarované RAM. Se čtyřmi odkazy mu stačí nejméně třetina až čtvrtina a s osminou už neprojde. Delší deadline storage (1000 ms) nechává útočníkovi víc času, ale tabulka 2^26 má i dražší dopočítání.
- **Co nepomohlo:** řetězení otevření (další index z předchozího listu). Útočník počítá postupně, ale memo dopočítaných bloků sdílí, takže 64 řetězených otevření stojí jen ~3× víc než 16 paralelních. Zkrácení deadlinu pod nejistotu RTT nejde a poctivé otevření (0,2 ms) je tak rychlé, že ani nejkratší rozumný deadline útočníka nezastaví. Prodražit by ho mohl jen pomalý hash na blok, a ten by stejně zpomalil i poctivou stavbu.
- **Důsledek:** kapacita z Proof of Space je dolní mez s tolerancí zhruba 3–4×, vrstvy `ram` a `storage` se liší jen deadlinem a NVMe projde i jako RAM. Na váhu nodu má vliv jen ve dvou dimenzích (0,2 + 0,2) se stropem `ratioCap`. Pro load balancing to stačí, pro odměňování podle kapacity ne. **[ověřit]** jitter RTT WebSocketu přes tunel na skutečné síti: kdyby byl pod ~20 ms, šlo by deadline `ram` snížit.

**Blokující body** jsou 1, 2 a 5. Ověřit je prototypem (viz fáze 0) dřív, než vznikne další kód.

## 10. Profily dev/prod a bootstrap

Stejný kód běží na 3 i na 9+ nodech. Parametry jsou v konfiguraci (profil), ne v kódu: `R`, kvorum, počet nodů vracených v `/route`, minimum nodů pro peer witnessy, délka epochy, zapnutí probation, hashcash a limitů ASN.

| | dev | prod (start, ~9 nodů) | otevřená síť |
|---|---|---|---|
| nody | 3–6 vlastních | 9+ vlastních (Flux) | kdokoli |
| R / kvorum | 3 / 2 | 3 / 2 | 3 / 2 |
| genesis allowlist | ano | ano | jen počáteční nody |
| probation, hashcash | vypnuto | vypnuto pro genesis | zapnuto |
| limity ASN a /24 | vypnuto | vypnuto pro genesis | zapnuto |
| peer witnessy | vypnuto, jen R2 měření | zapnuto při ≥ 3 různých ASN | zapnuto |
| epocha | pevný interval | pevný interval | pevný interval |

- **Genesis nody** jsou ručně schválené a mají plnou váhu od začátku. Bez toho by probation a Sybil brzda blokovaly i vlastní nody.
- Pod 3 živými nody se zápisy odmítají a klient dostane jasnou chybu. Jiný degradovaný režim není.
- Při `/route` s ≤ 3 nody je load balancing jen failover. Power-of-two-choices dává smysl od ~6 nodů.
- **Bezpečnost Flux nodů**: hostitel instance může vidět privátní klíč nodu. U vlastních nodů se to akceptuje, u otevřené sítě to není model hrozeb, který by šlo předpokládat. Zapsat do `THREAT_MODEL.md`.

### Vývojové prostředí
- Šest Docker kontejnerů, každý s vlastním Rust nodem, vlastním Ed25519 klíčem a vlastním `cloudflared` tunelem na reálnou subdoménu. Nody jsou tedy veřejně dosažitelné, bez veřejné IP, stejně jako v produkci.
- Cílem je ověřit blokující body (tunel, WebSocket, odstranění IP hlaviček, limity účtu, podmínky) na reálné infrastruktuře co nejdřív.
- Omezení: všechny kontejnery sdílejí IP a ASN, takže peer witnessy a bandwidth testy neukážou realistický výkon ani diverzitu. Slouží k ověření funkčnosti, ne výkonu.
- Perzistentní volume pro klíč a databázi každého nodu, aby se `node_id` nezměnil po restartu (stejný požadavek později na Flux).

## 11. Fáze

0. Specifikace: `PROTOCOL.md` (formáty zpráv, ticket, adresář a epochy, challenge protokoly s parametry), `THREAT_MODEL.md`.
1. Worker: registrace, Tunnel, PoSpace/CPU challenge, registr v DO.
2. Rust node (minimální): identita, registrace, heartbeat, odpovědi na challenge. Test: node se spustí za NAT bez příchozích spojení a projde registrací i challenge. Docker prostředí se 6 nody a tunely (sekce 10) slouží zároveň jako ověření limitů Cloudflare.
3. Routing: tickety, KV snapshot, CLI klient v Rustu.
4. Data plane: mailboxy, HRW anchory, replikace, ratchet, handoff.
5. Push přes Worker.
6. Hardening: fuzzing, simulace Sybil a lhaní, load test, cenový model.
7. Klient pro Android a desktop (sekce 13, milníky D0–D8 v 13.14).

## 12. Pokyny pro implementaci (pro Claude Code)

Nová verze se staví **od nuly v novém projektu `ancord`**. **Nejdřív se píše kód Workeru (load balanceru), potom se původní aplikace (`melo`, Node.js) přepíše do Rustu podle nového návrhu; deploy a DNS přijdou až nakonec** (viz 12.4). Starý prototyp je reference chování a návrhu (sekce 2), ne zdroj kódu k přenesení. Tento dokument je zdroj pravdy. Když se s kódem rozejde, nejdřív se upraví dokument.

### 12.1 Struktura projektu

```
ancord/
├── ARCHITECTURE.md        tento dokument
├── CLAUDE.md              krátké pokyny: odkaz na ARCHITECTURE.md, konvence, příkazy
├── spec/
│   ├── PROTOCOL.md        formáty zpráv, ticket, adresář a epochy, challenge protokoly
│   ├── THREAT_MODEL.md
│   └── test-vectors/      sdílené testovací vektory (JSON) pro Worker i node
├── worker/                load balancer + control plane (Cloudflare Worker, TypeScript)
├── node/                  node (Rust); od fáze D i sdílené jádro klienta `crates/ancord-core`
├── client/                grafický klient Android + desktop (Tauri 2, Svelte, TypeScript), fáze D, viz 13.12
└── dev/
    ├── docker-compose.yml 6 nodů
    ├── .env.example       bez tajemství
    └── README.md          jak prostředí spustit
```

- `worker/` a `node/` **nesdílejí kód**, jen `spec/` a `spec/test-vectors/`. Obě strany musí projít stejné vektory (podpis ticketu, Merkle ověření, hash-chain, formát adresáře).
- Tajemství (Cloudflare API token, tunnel tokeny, privátní klíče) nikdy do gitu. `.env` a datové volume jsou v `.gitignore`.

### 12.2 Pravidla práce
1. **Spec před kódem.** Formát, který se používá na obou stranách, se nejdřív zapíše do `spec/PROTOCOL.md` a testovacích vektorů, teprve pak se implementuje.
2. **Po milnících.** Každý milník níže má kritérium přijetí a je hotový až po jeho splnění v běžícím prostředí, ne jen po úspěšné kompilaci.
3. **Body označené [ověřit] se zjišťují prototypem**, ne předpokladem. Výsledek se zapíše do sekce 9 (otevřené body) i s naměřenými čísly.
4. **Parametry v konfiguraci** podle profilů v sekci 10 (`R`, kvorum, epocha, probation…), nic natvrdo.
5. **Nepřidávat nic mimo rozsah** (iOS, tokenová ekonomika, body z 13.15). Klient (sekce 13) je v rozsahu až od fáze D. Při nejasnosti se zeptat, nehádat.
6. Rust: `cargo clippy` a `cargo test` bez varování, žádné `unwrap` v produkční cestě. TypeScript: `strict`, testy přes vitest s Miniflare/workerd.
7. Kód a komentáře odpovídají okolnímu kódu. Dokumentace v češtině, identifikátory anglicky.

### 12.3 Co musí dodat uživatel
Vývoj běží na firemní zóně **`melonet.cz`** (sdílená zóna s existujícími záznamy, proto jsou všechny zásahy omezené na prefixy níže):

| Co | Hostname |
|---|---|
| Load balancer / control plane (Worker, custom domain) | `ancord.melonet.cz` |
| Nody | `node-<id>.melonet.cz` |

- **Přihlašovací údaje** jsou v proměnných prostředí `CLOUDFLARE_API_TOKEN` a `CLOUDFLARE_ACCOUNT_ID` (volitelné, deploy ho zjistí z tokenu). Hodnoty se nikdy nevypisují ani necommitují. Kontrola z C1 (2026-10-29): token je účtový, má oprávnění pro Workers Scripts, KV, R2, Queues a Cloudflare Tunnel, ale **žádná zónová oprávnění pro `melonet.cz`** (DNS čtení vrací `10000 Authentication error`). Pro nasazení je nutné doplnit pro zónu: DNS Edit, Workers Routes Edit, Transform Rules Edit. Seznam oprávnění je v `worker/README.md` (Nasazení). Doplněno týž den; zóna měla 51 záznamů, `ancord.melonet.cz` bylo volné a 33 starých A záznamů `node-xxxxxxxx.melonet.cz` se na pokyn uživatele smazalo. Nody pak přešly na jména `node-<16 znaků base32>` (PROTOCOL.md §2).
- **Dva tokeny**: `CLOUDFLARE_API_TOKEN` slouží k nasazení (wrangler). Worker za běhu používá vlastní, užší token (Tunnel Edit + DNS Edit jen pro zónu) uložený přes `wrangler secret`, ne v kódu.
- **Bezpečnost sdílené zóny**: před jakýmkoli zápisem se přečtou existující DNS záznamy a ověří se, že `ancord` ani `node-*` nic nepřepíšou. Automatika maže jen záznamy `node-<id>` s komentářem `ancord-managed` a tunely, které sama vytvořila. `dev/cleanup` smaže všechny vývojové node záznamy a tunely. Před každým krokem, který v Cloudflare něco vytváří nebo nasazuje, se čeká na potvrzení uživatele.
- Queues a R2 zatím nejsou potřeba (až A4 a A5).

### 12.4 Pořadí prací a milníky

**Stav (fáze A):** A1–A5 jsou implementovány v `worker/` a splňují kritéria přijetí v testech (vitest + Miniflare, `npm test` v `worker/`), včetně sdílených vektorů v `spec/test-vectors/`. Nic není nasazeno.

**Stav (fáze B):** B1 až B4 jsou implementovány v `node/` a splňují kritéria přijetí v integračním testu `node/crates/ancord-node/tests/e2e.rs` (tři nody proti lokálně spuštěnému Workeru v `wrangler dev`, mock Cloudflare, bez tunelu), v `tests/dataplane.rs` (kvorum, výpadky, autorizace node↔node, handoff, wake proti falešnému Workeru, bez skutečného Workeru) a ve sdílených vektorech (včetně HRW a ratchetu). Data plane je v `spec/PROTOCOL.md` §13, wake v §13.6. Presence a průběžné doručování online odběrateli jsou hotové (B5, §13.7), wake se posílá jen tehdy, když doručení online nevyjde. Nic není nasazeno.

**Stav (parita s `melo`, §6.4):** identita uživatele, profily na anchorech, X3DH s obálkou ve schránce, typing, stav doručení, dávkové odběry a lokální historie jsou implementované v `node/` (včetně CLI) a otestované v `tests/dataplane.rs`, `tests/e2e.rs` a ve vektoru `pairwise.json`. Nic není nasazeno.

**Stav (uživatelský tok před fází C):** adresa `ancord:<pk>` (text, QR), push klíče schránky (v profilu) a konverzací (ze sdíleného tajemství, vektor `pairwise.json`), registrace tokenu z CLI (`push-register`) a online klient (`ancord online`: jedno spojení, schránka i konverzace, automatické přijetí kontaktů, znovupřipojení s novým `/route`). E2E test ověřuje celý tok: první kontakt vzbudí offline příjemce, po připojení přijme kontakt a zprávy živě, po odpojení ho další zpráva vzbudí znovu. Soubory klíčů a konverzací jsou zamčené, aby dva procesy nepoužily stejný klíč ratchetu dvakrát. Grafický klient (skenování QR, push token) je návrh v sekci 13 (fáze D), rozpracovaný v `client/` (D1, D2 pro desktop, D3 online klient s E2E testem, D4 UI; viz `client/README.md`).

**Stav (fáze C):** C1, C2 a C3 splněné 2026-10-29. Worker `ancord` běží na `https://ancord.melonet.cz` (profil `prod`, `CLOUDFLARE_MODE=real`, `PUSH_MODE=log`), veřejný klíč Workeru `MpUwUO74HgS5ZvLi1guR_aajIEZCoP4olm6gKlfnIo0` (v `dev/.env.example`). Vytvořeno: KV `ancord-routing` a `ancord-push`, R2 `ancord-bw-blobs`, fronty `ancord-push` a `ancord-push-dlq`, Transform Rule `ancord_strip_visitor_ip` (jen `node-<16>.melonet.cz`), custom domain, užší token Workeru `CF_API_TOKEN` (vytvořil uživatel). Šest nodů z `dev/docker-compose.yml` se zaregistrovalo, vytvořily se jim tunely a DNS a jsou dosažitelné bez příchozích portů. Výsledky měření jsou v sekci 9. Testovací nody běžely v dočasném prostředí, trvalý provoz potřebuje vlastní server (`dev/README.md`) a nový `GENESIS_NODE_IDS`.

**Deploy a DNS se ve fázích A a B neřešily.** Nejdřív se napíše a lokálně otestuje kód, nasazení na Cloudflare a Docker prostředí přijdou až ve fázi C. Fáze A a B nic nevytvářejí v Cloudflare a nepotřebují DNS ani `.env`.

**Fáze A: Worker (load balancer a control plane), TypeScript, složka `worker/`. Dělá se jako první.**

| | Obsah | Kritérium přijetí |
|---|---|---|
| **A1** Registrace | `POST /register`, challenge nonce, ověření podpisu, registr nodů v Durable Objectu. Volání Cloudflare API (vytvoření tunelu a DNS `node-<id>.<doména>`) je za rozhraním `CloudflareClient` s implementací pro skutečné API a mockem pro testy. Stavy nodu `pending` → `active`. | Vitest/Miniflare: registrace projde, kolize jména i špatný podpis jsou odmítnuty, Worker maže jen záznamy a tunely, které sám vytvořil. |
| **A2** Heartbeat a adresář | Podepsaný heartbeat přes WebSocket na DO (hibernation), podepsaný adresář s číslem epochy, genesis allowlist, pevný interval epoch, profily dev/prod. | Test: nody ve stejné epoše dostanou stejný adresář, odpojení nodu změní epochu v pevném intervalu. |
| **A3** Routing | KV snapshot, `GET /route` (3 nody, power-of-two-choices, filtr podle `request.cf`), ticket vázaný na efemérní klíč. | Test: `/route` vrací 3 nody, přestane vracet vypnutý node, ticket projde ověřením podle `spec/test-vectors/`. |
| **A4** Challenge | CPU hash-chain, Proof of Space s Merkle commitmentem, bandwidth challenge, skóre nodu. | Test: node, který lže o specs, dostane nižší skóre než poctivý. |
| **A5** Push | `POST /wake`, ověření anchora, rate-limit, odeslání přes abstrakci fronty. | Test: wake od nodu, který není anchorem mailboxu, je odmítnut. |

Spec (`spec/PROTOCOL.md` a testovací vektory) vzniká spolu s A1–A3, protože podle něj se píše i node.

**Fáze B: Node, Rust, složka `node/`. Začíná po dokončení fáze A.** Je to přepis původní aplikace (`../melo`, Node.js) podle nového návrhu. Původní kód slouží jako reference chování (blinded mailbox model, ratchet šifrování, tok `send_message` → anchory → push, TTL úklid), ne jako předloha implementace. Šifrování se ověřuje proti testovacím vektorům z původní verze.

| | Obsah | Kritérium přijetí |
|---|---|---|
| **B1** Identita a registrace | Perzistentní Ed25519 klíč, registrace u Workeru, spuštění `cloudflared` s dodaným tokenem, heartbeat, odpovědi na challenge. | Integrační test proti lokálně spuštěnému Workeru (`wrangler dev`/Miniflare). Restart zachová `node_id`. |
| **B2** Admission a klient | Kontrola ticketu na nodu, WSS server, minimální CLI klient. | Klient dostane z `/route` 3 nody a připojí se s ticketem, bez ticketu je odmítnut. |
| **B3** Data plane | Blinded mailboxy, HRW anchory podle epochy, replikace R=3 s kvorem 2, idempotentní zápis podle ID zprávy, ratchet (ChaCha20-Poly1305 + HKDF), TTL úklid, embedded úložiště (redb), podepsané node↔node requesty. | Zpráva odeslaná přes node A se doručí z node B. Po vypnutí jednoho z anchorů zápis i čtení fungují, po vypnutí druhého se zápisy odmítnou. |
| **B4** Push | Podepsaný `POST /wake` z anchoru. | Wake dorazí na Worker jen od skutečného anchoru. |

**Fáze C: Deploy a vývojové prostředí. Až bude kód hotový.**

| | Obsah | Kritérium přijetí |
|---|---|---|
| **C1** Příprava | Kontrola tokenu a oprávnění, čtení existujících DNS záznamů, Transform Rule pro `node-*.melonet.cz`. | Žádný existující záznam zóny se nezměnil. |
| **C2** Nasazení Workeru | `wrangler deploy` na `ancord.melonet.cz`. | `https://ancord.melonet.cz/health` odpovídá. |
| **C3** Šest nodů | `dev/`: Docker Compose se 6 nody, které znají jen `WORKER_URL`. | `docker compose up` bez dalšího nastavení: všech 6 nodů se zaregistruje, je dosažitelných na `node-<id>.melonet.cz` bez příchozích portů. Změří se a zapíšou do sekce 9: odstranění IP hlaviček u WebSocket upgrade, idle timeout, limity tunelů a DNS záznamů na účtu. |

Pokud limity počtu tunelů nebo DNS záznamů nestačí, rozhodne se o sdíleném tunelu (bod 2 v sekci 9) až na základě naměřených čísel z C3. Body 1, 2 a 5 v sekci 9 jsou blokující pro nasazení, ne pro psaní kódu.

### 12.5 Vývojové prostředí (`dev/`)
- Jeden Docker image `ancord-node`, šest služeb `node1`…`node6` v `docker-compose.yml`.
- Každá služba má vlastní named volume (klíč a databáze), vlastní `NODE_NAME` a vlastní subdoménu. Image obsahuje node i `cloudflared`; node ho spouští jako podproces s tokenem, který dostal při registraci.
- Žádný kontejner nepublikuje port na hostitele. Dosažitelnost je pouze přes tunel, aby se ověřilo chování bez veřejné IP.
- Profil `dev` (sekce 10): genesis allowlist zapnutý, probation, hashcash a limity ASN vypnuté, peer witnessy vypnuté.
- `dev/README.md` popisuje: nastavení `.env`, `docker compose up`, kontrolu registrace, vypnutí jednoho nodu pro test failoveru.

## 13. Klient (Android a desktop)

Grafický klient pro Android a desktop (Windows, macOS, Linux). Jeden kód, jádro v Rustu, UI v TypeScriptu a CSS. Protokol se nemění (kromě jediného rozšíření push, viz 13.8); klient je stejný uživatel sítě jako dnešní CLI (`ancord online`, §6.4).

### 13.1 Cíle a pořadí priorit
1. **Bezpečnost klíčů a obsahu.** Klíče a stav ratchetu nikdy neopustí Rust proces a UI k nim nemá přístup.
2. **Žádná telemetrie.** Klient mluví jen s Workerem a nody. Žádná analytika, crash reporty, Firebase Analytics, fonty ani skripty z internetu, žádný automatický updater, který by se ptal na verzi.
3. **Ochrana proti čtení obrazovky** tam, kde to platforma dovolí (13.9). Co platforma nedovolí, se píše do `THREAT_MODEL.md`, ne do slibů.
4. **Rychlost a malá velikost.** Rozpočet v 13.11 se měří a hlídá, ne jen deklaruje.
5. **UI jako běžný web.** TypeScript a CSS, žádný jazyk ani styling engine navíc.

### 13.2 Volba technologie

| Vrstva | Volba | Důvod |
|---|---|---|
| Shell | **Tauri 2** (desktop i Android) | Jeden kód pro obě platformy, systémový WebView místo vlastního Chromia (instalátor jednotky MB), Rust jádro v procesu, vestavěné `set_content_protected` pro desktop a plugin systém s Kotlinem pro Android |
| UI | **Svelte 5 + TypeScript (`strict`) + čisté CSS** (custom properties jako design tokeny), build Vite | Kompilátor místo runtime s virtuálním DOM, malý bundle, rychlý start. Žádný CSS framework ani knihovna komponent |
| Jádro | **Rust**, crate `ancord-core` sdílený s nodem | Viz níže |
| Úložiště | **redb** s šifrováním po záznamech (13.6) | Stejné jako na nodu, čistý Rust, bez C závislosti (SQLCipher/OpenSSL by ztížilo křížovou kompilaci na Android) |

**Proč Rust a ne celé v TypeScriptu.** Čisté TS by šlo, ale: (a) privátní klíče a stav ratchetu by žily v heapu WebView, který nejde spolehlivě vynulovat a který čte kód celého UI i všech jeho závislostí, kdežto Rust drží klíče v `zeroize` strukturách a UI volá jen příkazy typu `message_send`; (b) šifrování, ratchet, X3DH a formáty jsou už napsané a otestované proti `spec/test-vectors/` v `node/`, druhá implementace v TS by znamenala druhý zdroj chyb a rozcházení s protokolem; (c) celá síť jde mimo WebView, takže odpadá velká část cest, kudy by mohla uniknout data.

**Zvažované a zamítnuté.**

| Možnost | Důvod zamítnutí |
|---|---|
| Electron | 150 MB+, vlastní Chromium, Android nepodporuje |
| Flutter, Compose Multiplatform | Rychlé, ale UI není TypeScript a CSS |
| React Native | Desktop slabý, navíc JS engine s klíči v paměti |
| Slint, Iced, egui | Nejmenší a nejrychlejší, ale bez CSS a TS (požadavek 5) |
| Dioxus | Stejný WebView jako Tauri, UI by se psalo v Rustu |
| Čistý TS v Capacitoru | Klíče v JS, duplicitní implementace protokolu (viz výše) |

**Slabiny Tauri, které se přijímají.** Výkon a funkce závisejí na systémovém WebView: Android System WebView, WebView2 (Windows), WKWebView (macOS), WebKitGTK (Linux, nejslabší). Na Androidu se při startu kontroluje minimální verze WebView (parametr, výchozí hodnotu určí D6) a při nižší verzi aplikace vysvětlí proč nejede. Tauri Android je mladší než desktop, proto D6 začíná prototypem (13.10).

### 13.3 Vrstvy a hranice důvěry

```
┌──────────────────────────── proces aplikace ────────────────────────────┐
│  WebView (nedůvěryhodné)          Rust (důvěryhodné)                     │
│  Svelte + TS + CSS   ──IPC────►   src-tauri: příkazy, události           │
│  CSP, bez sítě                         │                                 │
│                                   ancord-core (sdílený s nodem)          │
│                                     identita, X3DH, ratchet, profil,     │
│                                     kanál (ChaCha20-Poly1305), online    │
│                                     session, sync_once, úložiště         │
│                                        │                                 │
│                                   síť (reqwest + tokio-tungstenite)      │
└────────────────────────────────────────┼─────────────────────────────────┘
                                         ▼  Worker (/route, push-reg) a nody (WSS)
```

- **WebView je nedůvěryhodný.** Předpokládá se, že kód UI nebo závislost může být chybná. Proto UI nedostane klíče, ticketů ani adresy nodů, jen to, co zobrazuje: text zprávy otevřené konverzace, jména, stavy.
- **Síť výhradně v Rustu.** CSP WebView zakazuje spojení kamkoli kromě IPC. Test v D1 to ověří (`fetch` z UI musí selhat).
- **Kořeny důvěry**: veřejný klíč Workeru (`WORKER_PUBKEY`) a URL Workeru jsou zapečené při buildu podle profilu (dev/prod, sekce 10). Přepsat je lze jen v debug buildu.
- **TLS kořeny**: webpki-roots zabalené v aplikaci, ne systémové úložiště (Android trust store se liší podle výrobce a uživatel do něj může přidat CA).

### 13.4 Jádro: crate `ancord-core`

Dnes je klientská logika (`client.rs`, `user.rs`, `ratchet.rs`, `channel.rs`, `profile.rs`, `ticket.rs`, `encoding.rs`, `canonical.rs`, `sign.rs`, `hashcash.rs`, `hrw.rs`) uvnitř `ancord-node`, což táhne `axum`, `redb` a `sysinfo` i do klienta. Milník D0 ji vyčlení:

- `node/crates/ancord-core`: sdílené, bez serverových závislostí (`axum`, `sysinfo`, `supervisor`, `tunnel`). Obsahuje protokolové primitiva a **klientskou stranu**.
- `ancord-node` a `ancord-cli` na `ancord-core` závisejí. Žádná změna chování, `cargo test` v `node/` se musí chovat stejně (včetně `tests/e2e.rs` a `tests/dataplane.rs`) a vektory ze `spec/test-vectors/` prochází tytéž.
- **Perzistence za rozhraním.** CLI dnes ukládá klíče a konverzace do zamčených souborů. `ancord-core` dostane trait (`Store`: identita, konverzace/ratchet, zprávy, nastavení) s dvěma implementacemi: souborovou pro CLI a šifrovaným redb pro klienta (13.6). Rozhodnutí o rozsahu rozhraní padne v D0 podle toho, co kód skutečně potřebuje.
- **Online session** (`ancord online` z CLI, spec §15.12) se přesune do jádra jako samostatný typ (`OnlineSession`): `/route`, připojení, zveřejnění profilu, `subscribe`, zprávy jako stream událostí, znovupřipojení s novým `/route`.
- **Stav D0 (hotovo 2026-10-30).** `ancord-core` obsahuje `encoding`, `canonical`, `sign`, `hashcash`, `hrw`, `ticket`, `channel`, `ratchet`, `user`, `time`, čisté části `profile`, `guard` a `wake` (typy, ověření, `push_id`; chyby jako `ProofError`) a síťovou klientskou stranu (`net`: spojení s nodem, `Session`, profil, stráž, inbox; `online`: `Online`, `OnlineEvent`). `ancord-node` je re-exportuje (`pub use ancord_core::{…}`) a nechává si serverové části (úložiště, replikace, handoff); `ancord-cli` drží jen souborové úložiště (`files.rs`). Rozhraní perzistence je `online::Persist` (už existovalo), souborová implementace zůstává v CLI, šifrovaná redb v `client/`. `OnlineSession` je zatím `online::Online`, `sync_once` patří do D3/D6. `blake3` je ve variantě `pure` (build skript s C kompilátorem by na Androidu vyžadoval NDK). **Kritérium `cargo build -p ancord-core --target aarch64-linux-android` prošlo (2026-10-30) s NDK 26.3 (`CC_aarch64_linux_android=aarch64-linux-android26-clang`); knihovna `client/src-tauri` se pro tento cíl také přeloží.** Bez NDK `ring` neprojde.
- **`sync_once`** (nové): jednorázové vyzvednutí po wake: `/route` → připojení → `subscribe` + `fetch` → uložení → `ack` → odpojení, s tvrdým časovým limitem (výchozí 20 s). Používá ho Android při probuzení pushem (13.8). Ack se posílá až po trvalém uložení zprávy a stavu ratchetu (doručení je *at least once*, duplicity se zahazují podle `id`, §6.3).
- **Jeden vlastník stavu.** Stav ratchetu smí měnit jen jeden proces a v něm jeden `Core` pod zámkem. Na desktopu se to vynucuje single-instance zámkem aplikace, na Androidu je UI i `sync_once` ve stejném procesu. Stejný problém řešil u CLI zámek souborů.
- `unsafe_code = forbid` zůstává. Kód, který ho potřebuje (JNI na Androidu), je v samostatném crate klienta s lokální výjimkou (13.10).

### 13.5 UI a IPC kontrakt

UI nikdy nedostane nic, co není zobrazení. Kontrakt je definovaný jednou, v TypeScriptu typovaně (generované typy z Rust struktur, např. `ts-rs` nebo `specta` [ověřit v D1]), a testuje se z obou stran.

**Příkazy (UI → Rust)**

| Skupina | Příkazy |
|---|---|
| Identita | `identity_status`, `identity_create`, `address_get` (`ancord:<pk>`), `address_qr` (SVG) |
| Kontakty | `contact_add(address)`, `contact_list`, `contact_safety_number(id)`, `contact_mark_verified(id)` |
| Konverzace | `conversation_list`, `conversation_open(id, before?, limit)`, `message_send(id, text)`, `typing_set(id, on)`, `conversation_delete(id)` |
| Stav | `connection_state`, `settings_get`, `settings_set` |
| Zámek | `lock`, `unlock(passphrase?)` |

**Události (Rust → UI)**: `message_received`, `delivery_state` (`pending`/`delivered`/`unknown`, §6.4), `typing`, `connection_state`, `contact_request` (nový kontakt přijatý z obálky).

**Pravidla pro UI**
- Zprávy se zobrazují jako **čistý text**. Žádný HTML ani Markdown, žádné automatické načítání odkazů, náhledů ani obrázků. Odkazy se otevírají v systémovém prohlížeči až po potvrzení.
- Žádné webové fonty a ikony ze sítě: systémové fonty a ikony zabalené v bundlu.
- Seznam zpráv je virtualizovaný (vlastní malá implementace, bez knihovny).
- **Tauri capabilities**: jen vlastní příkazy aplikace a `core:event`. Pluginy `fs`, `shell`, `http`, `opener` (kromě potvrzených odkazů), `updater` a `global-shortcut` se nepřidávají. `withGlobalTauri` je vypnuté, DevTools jen v debug buildu.
- **CSP** (výchozí návrh, ověří se v D1): `default-src 'none'; script-src 'self'; style-src 'self'; img-src 'self' data:; connect-src ipc: http://ipc.localhost`. Bez `unsafe-inline` a `unsafe-eval`.
- Přístupnost: sémantické HTML, popisky, klávesnice. Ochrana před čtečkami (13.9) je přepínač, ne výchozí vypnutí přístupnosti bez vědomí uživatele.

### 13.6 Úložiště a klíče

- **Jeden soubor redb na profil**, tabulky: `identity`, `conversations` (stav ratchetu), `messages` (klíč `(conv_id, ts, id)`), `settings`.
- **Šifrování po záznamech**: hodnota je XChaCha20-Poly1305, klíč tabulky `HKDF(master_key, "ancord/v1/client/" ‖ tabulka)`, AAD = tabulka ‖ klíč záznamu. Čas a ID konverzace v klíči zůstávají čitelné (záznam zná jen lokální útočník, který má stejně přístup k souboru); text, odesílatel a ratchet ne.
- **`master_key`** (32 B náhodných) je zabalený klíčem platformy:
  - Android: klíč v **Android Keystore** (StrongBox, je-li k dispozici), AES-GCM wrap, Kotlin plugin;
  - Windows: DPAPI; macOS: Keychain; Linux: Secret Service (`keyring`) [ověřit na stroji bez Secret Service, tam se nabídne režim s heslem].
- **Režim s heslem** (volitelný): `master_key` se navíc zabalí klíčem z hesla (Argon2id). Cena: na pozadí nelze zprávy dešifrovat, takže wake na Androidu ukáže jen generické oznámení a zprávy se vyzvednou po odemknutí.
- **Zámek aplikace**: UI se po nečinnosti zamkne. Na Androidu volitelně biometrika (BiometricPrompt) před zobrazením UI. Zámek chrání zobrazení, klíč v Keystore je pro wake dostupný stále (bez vazby na biometrii), jinak by nešlo vyzvedávat na pozadí.
- **Citlivé buffery** v Rustu mají `zeroize`. Bez `mlock` (kvůli `unsafe_code = forbid`).
- **Nikde se neloguje** obsah zpráv, klíče ani tickety. Release build loguje jen události bez obsahu a na Androidu žádné citlivé řetězce nejdou do logcatu.
- **Smazání**: `conversation_delete` maže zprávy konverzace v redb, stav ratchetu nechává (bod 17 v sekci 9). Úplné smazání profilu zničí `master_key` v keystore (crypto-erase) a pak soubor.
- Záloha a více zařízení: mimo v1 (otevřený bod 11 v sekci 9). Ztráta zařízení znamená ztrátu identity.

### 13.7 Síť a životní cyklus spojení

- **Popředí** (desktop i Android): `OnlineSession` drží WSS na gateway, zprávy chodí živě, presence na anchorech. Ping po 25 s nečinnosti (idle timeout tunelu 125 s, sekce 9).
- **Pozadí, desktop**: aplikace zavřená do tray drží spojení dál (volba, výchozí vypnuto spolu s autostartem). Zavřená úplně = offline, zprávy čekají na anchorech. Wake push na desktop se nepodporuje (není token).
- **Pozadí, Android**: Android síťová spojení v pozadí ruší. Spojení tedy zaniká s procesem, presence na anchorech vyprší a příchozí zpráva spustí wake push (13.8) → `sync_once`.
- **Cache adresáře**: poslední podepsaný adresář nodů se drží v úložišti, takže při výpadku Workeru se klient připojí na dřív známé nody (§5.3).
- **Offline odesílání**: `message_send` zprávu uloží lokálně jako `queued` a odešle, jakmile je spojení. Stav doručení se dotazuje na anchorech (§6.4).
- Proxy a Tor: mimo rozsah. IP uživatele vidí Cloudflare (sekce 4 a 8), node ne (a podle měření C3 ji u otevřené sítě vidí provozovatel nodu v hlavičkách, bod 1 v sekci 9; klient na to nemá vliv).

### 13.8 Push (Android)

Dnes Worker umí platformy `fcm` a `apns` (`worker/src/push/senders.ts`, spec §10). Pro klienta bez Google závislostí se přidává **UnifiedPush**:

- **Spec (nejdřív, pravidlo 12.2/1)**: `platform` v `POST /push/register` dostane hodnotu `up`. `token` je pak URL endpointu distributora (UnifiedPush, např. ntfy), `https`, max 4096 znaků. Worker při wake udělá `POST` na tuto URL s pevným tělem `wake` (hlavičky `TTL: 60`, `Urgency: high`), odpověď ignoruje, `404` a `410` záznam smaže. Obsah push je stále pevný, stejně jako u FCM a APNs.
- **Ochrana Workeru jako odesílatele (SSRF a zesílení).** Registrovat token smí kdokoli s libovolným push klíčem, takže útočník by mohl nasměrovat wake na cizí URL. Worker proto: přijímá jen `https` na portu 443, jméno hostitele (ne IP literál, ne `localhost`/interní přípony), `redirect: "manual"`, vlastní limit **per hostitel** (kromě limitu per `push_id` a per node) a krátký timeout. Test vektory a testy ve Workeru pokryjí odmítnuté URL a limit.
- **Tok na zařízení**: Kotlin `MessagingReceiver` (UnifiedPush) → předá `wake` do Rustu → `sync_once` → lokální oznámení. Klient registruje stejný endpoint pod všemi svými `push_id` (schránka + každá konverzace, §15.11), jako dnes u FCM tokenu.
- **Oznámení**: výchozí text je jen „Nová zpráva“, bez odesílatele a bez obsahu, viditelnost `VISIBILITY_PRIVATE`. Náhled obsahu je volba uživatele. V režimu s heslem (13.6) je oznámení vždy generické.
- **Co distributor vidí**: čas wake a IP Workeru, ne obsah ani `push_id` odesílatele. U FCM by to byl Google (timing, token); proto je FCM jen volitelný pozdější doplněk (Worker ho už umí), ne součást v1.
- **Bez distributora** (UnifiedPush aplikace není nainstalovaná): nabídne se volba „Trvalé spojení“, foreground služba drží `OnlineSession` s trvalým oznámením. Stojí baterii, proto není výchozí.

### 13.9 Soukromí a ochrana proti čtení obrazovky

Žádná z ochran není absolutní: root, kompromitovaný OS, fyzický snímek obrazovky kamerou ani škodlivý přístupnostní nástroj s nejvyšším oprávněním je nemusí respektovat. To patří do `THREAT_MODEL.md`.

| Hrozba | Android | Windows | macOS | Linux |
|---|---|---|---|---|
| Screenshot, nahrávání, náhled v přehledu aplikací | `FLAG_SECURE` na okně (plugin) | `WDA_EXCLUDEFROMCAPTURE` (Tauri `set_content_protected`) | `NSWindow.sharingType = none` (stejné API) | bez ochrany (Wayland ji neumožňuje, uživatel nevyžaduje) |
| Čtení obsahu přístupnostními službami | `setAccessibilityDataSensitive` (API 34+), na starších `importantForAccessibility` skrývající WebView; přepínač „Zvýšená ochrana“, výchozí zapnuto, s vysvětlením pro čtečky | n/a | n/a | n/a |
| Překryvná okna a tapjacking | `Window.setHideOverlayWindows` (API 31+), `filterTouchesWhenObscured` | n/a | n/a | n/a |
| Citlivý obsah v nahrávání a oznámeních | `View.setContentSensitivity` (API 35) [ověřit dostupnost] | n/a | n/a | n/a |
| Klávesnice se učí z textu | `IME_FLAG_NO_PERSONALIZED_LEARNING` na vstupu zprávy, incognito režim IME | n/a | n/a | n/a |
| Autofill | `importantForAutofill = no` | n/a | n/a | n/a |
| Schránka | `ClipDescription.EXTRA_IS_SENSITIVE` při kopírování | historie schránky: upozornění v nastavení | n/a | n/a |
| Cloudová záloha | `allowBackup = false`, `dataExtractionRules` bez dat | n/a | n/a | n/a |
| Core dump, crash report | žádný crash reporter; `PR_SET_DUMPABLE=0` [ověřit bez `unsafe`] | vypnout WER pro proces [ověřit] | vypnout crash reporter | `ulimit -c 0` při spuštění |
| Ladění WebView | `setWebContentsDebuggingEnabled(false)`, `allowFileAccess = false` v release | DevTools jen v debug | totéž | totéž |

Telemetrie dále: žádná závislost, která volá ven (kontrola v D5: `cargo deny` / `cargo tree` a `npm ls` nad celým stromem, zákaz Firebase a Google Play Services v Androidu, test provozu, který zaznamená všechna odchozí spojení běžícího klienta a porovná je s povolenými hostiteli). Žádný automatický updater: aktualizace jsou podepsané balíčky, které si uživatel nainstaluje sám (kontrola verze by prozradila IP a čas).

### 13.10 Platformní specifika

**Android**
- `minSdk` 26 (parametr, potvrdit v D6), jen `arm64-v8a` pro release (plus `x86_64` pro emulátor v debug).
- Kotlin plugin Tauri (`client/src-tauri/gen/android` + vlastní plugin): `FLAG_SECURE`, nastavení přístupnosti a overlayů, Keystore wrap, UnifiedPush receiver, oznámení, skener QR (CameraX), kontrola verze WebView, biometrika.
- **Prototyp na začátku D6 [ověřit]**: zda lze `sync_once` spustit v procesu bez vytvoření Activity/WebView (z `BroadcastReceiver`/krátké foreground služby) a zda to Tauri mobile dovolí. Pokud ne, `sync_once` poběží přes tenký JNI most v samostatném crate (`unsafe` jen tam) nebo se Activity spustí neviditelně. Výsledek se zapíše do sekce 9.
- Oprávnění: `INTERNET`, `POST_NOTIFICATIONS`, `CAMERA` (až při skenu), případně `FOREGROUND_SERVICE` pro volitelné trvalé spojení. Nic dalšího (kontakty, poloha, úložiště se nežádají).

**Desktop**
- Single-instance zámek (stav ratchetu).
- Tray a autostart volitelné, výchozí vypnuto. Oznámení přes systémové API, výchozí text generický (13.8).
- Windows: WebView2 Evergreen (bootstrapper v instalátoru, Windows 11 ho má). macOS: podepsání a notarizace. Linux: balíčky `.deb`/`.AppImage`, WebKitGTK 4.1.

### 13.11 Rozpočet výkonu (cíle, měří se)

Čísla jsou **cíle k ověření**, ne naměřené hodnoty. Měří se v D1 (kostra), D3 (reálná data) a znovu v D8, výsledek se zapíše do sekce 9.

| Metrika | Cíl |
|---|---|
| Instalátor desktop | < 15 MB |
| APK (arm64) | < 20 MB |
| JS bundle (gzip) | < 150 kB |
| Studený start do použitelného UI, desktop | < 600 ms |
| Studený start, střední Android | < 1,2 s |
| Idle paměť desktop (celý proces včetně WebView) | < 150 MB |
| Otevření konverzace s 10 000 zprávami | < 150 ms do prvního vykreslení |
| Plynulost seznamu zpráv | 60 fps při rolování |
| Odeslání zprávy: stisk → zobrazení lokálně | < 50 ms |
| `sync_once` po wake | < 5 s typicky, tvrdý limit 20 s |

### 13.12 Struktura `client/`

```
client/
├── package.json, vite.config.ts, tsconfig.json   (strict)
├── src/                      Svelte + TS + CSS
│   ├── lib/ipc.ts            typované volání příkazů a události (generované typy)
│   ├── lib/ui/               malé komponenty bez závislostí, virtuální seznam
│   ├── styles/               tokeny (custom properties), světlé/tmavé téma
│   └── routes/               seznam konverzací, konverzace, kontakty, nastavení, zámek
├── src-tauri/                Rust shell: příkazy, události, capabilities, CSP
│   ├── tauri.conf.json
│   └── gen/android/          generovaný Android projekt + Kotlin plugin
└── README.md                 build, spuštění, profily
```

`ancord-core` je v `node/crates/` (společný Cargo workspace s nodem); `client/src-tauri` na něj odkazuje cestou. `client/` nesdílí kód s `worker/`, jen `spec/` a vektory (jako node).

### 13.13 Testování

- **Jádro**: sdílené vektory ze `spec/test-vectors/` (už procházejí), unit testy úložiště (šifrování po záznamech, ztráta a poškození záznamu), test `sync_once` (limit, ack až po uložení, idempotence).
- **E2E bez UI**: dva klienti na jádře proti lokálnímu `wrangler dev` a třem nodům (rozšíření `tests/e2e.rs`), včetně wake přes falešný `up` endpoint (lokální HTTP server).
- **IPC a UI**: vitest + `mockIPC` z Tauri, test, že UI nic neposílá mimo IPC (CSP), snapshot rozpočtu bundle (selhání při překročení).
- **Desktop smoke**: `tauri-driver` (WebDriver) na Linuxu [ověřit v D1 v tomto prostředí].
- **Android**: instrumentovaný test na emulátoru nebo zařízení (`FLAG_SECURE` aktivní, oznámení bez obsahu, `sync_once` z receiveru). Vyžaduje emulátor s KVM nebo fyzické zařízení; pokud prostředí nemá, kritérium D6 potvrdí uživatel na zařízení a stav se zapíše.
- Rust: `cargo clippy --all-targets -- -D warnings` a `cargo test` bez varování, žádné `unwrap` v produkční cestě (jako v `node/`). TypeScript: `strict`, `svelte-check`, vitest.

### 13.14 Milníky fáze D

**Fáze D: Klient, složka `client/`, jádro `node/crates/ancord-core`. Začíná po dokončení fáze B a po zapsání rozhodnutí o rozsahu do `CLAUDE.md`.** Jako ve fázích A a B se nic nenasazuje, nic se nevytváří v Cloudflare a nečte se `.env`. Vše běží proti lokálnímu `wrangler dev`. Změna Workeru v D7 se nasadí až po potvrzení uživatele.

| | Obsah | Kritérium přijetí |
|---|---|---|
| **D0** Vyčlenění jádra | `ancord-core` z `ancord-node` (13.4), `Store` rozhraní, `OnlineSession` do jádra. Beze změny chování. | `cargo clippy` a `cargo test` v `node/` bez varování, shodný výsledek jako před změnou (vektory, `dataplane.rs`, `e2e.rs`). `cargo check -p ancord-core --target aarch64-linux-android` projde. |
| **D1** Kostra | `client/`: Tauri 2 + Svelte + TS, CSP, capabilities, IPC kontrakt s generovanými typy, vytvoření identity a zobrazení adresy a QR. | Aplikace se spustí na desktopu. Test: `fetch` z WebView selže, žádný příkaz mimo kontrakt. Změřen instalátor, bundle a start (13.11). |
| **D2** Úložiště a klíče | Šifrované redb, `master_key` v keychainu platformy, režim s heslem, zámek. | Test: poškozený nebo cizí záznam se neotevře, restart zachová identitu a konverzace, `grep` na čitelný text zprávy v souboru profilu nic nenajde. |
| **D3** Online klient | Napojení `OnlineSession`: přidání kontaktu podle adresy, příjem obálky, zprávy živě, offline fronta, typing, stav doručení. | E2E test: dva klienti přes lokální Worker a 3 nody si píší živě, offline příjemce zprávu dostane po připojení, duplicitní doručení se zahodí. |
| **D4** UI | Seznam konverzací, konverzace s virtuálním seznamem, kontakty, ověření bezpečnostním číslem, nastavení, světlé/tmavé téma. | Test UI (mock IPC) pro všechny obrazovky. Změřené otevření konverzace s 10 000 zprávami do cíle z 13.11. |
| **D5** Desktop hardening | `set_content_protected`, single-instance, tray a autostart (volitelné), zákaz sítě z WebView, kontrola závislostí a odchozího provozu (13.9). | Test odchozích spojení: klient mluví jen s Workerem a nody. `cargo tree`/`npm ls` bez analytiky. Packaging pro Windows, macOS, Linux. |
| **D6** Android | Prototyp `sync_once` bez Activity (13.10), Kotlin plugin: `FLAG_SECURE`, přístupnost a overlay, Keystore, IME a autofill, oznámení, skener QR, kontrola WebView. | Na emulátoru nebo zařízení: screenshot černý, náhled v přehledu aplikací prázdný, zprávy se posílají a přijímají, QR kontakt funguje. |
| **D7** Push `up` | Spec §10 (`platform: up`), vektory, Worker (sender, SSRF guard, limit per hostitel, testy), UnifiedPush receiver, volitelné trvalé spojení. | Test: wake od skutečného anchora dorazí na falešný `up` endpoint, endpoint s IP literálem nebo `http` se odmítne, limit per hostitel drží. Na zařízení: offline příjemce dostane generické oznámení a po otevření zprávy. |
| **D8** Vydání | Podepsané balíčky, reprodukovatelnost kde to jde, finální měření rozpočtu, `THREAT_MODEL.md` pro klienta, dokumentace. | Rozpočet z 13.11 splněn, nebo rozdíly zapsané v sekci 9 s důvodem. |

**Stav D3 (hotovo 2026-10-30).** `client/src-tauri/src/net.rs` drží jednu síťovou úlohu s `online::Online` (to je zatím `OnlineSession` z 13.4): `/route` s novým efemerním klíčem, `connect_any` s ověřením ticketu připnutým `WORKER_PUBKEY`, `attach`, potom rámce a pokyny z UI (`Flush`, `Typing`) v jedné smyčce, ping po 25 s, znovupřipojení s backoffem 1–60 s. Úloha je za běhu jediný vlastník stavu ratchetu; `Core` s ní sdílí jen stav pod zámkem, který se nedrží přes `await`, a při zámku v režimu s heslem ji zruší (stav je uložený před `ack` i před odesláním, takže zrušení nic neztratí). Perzistence (`Persist` v `core.rs`): přijatá zpráva se z historie `Session` zapíše do `messages` (`conv/ts_ms/id`) dřív, než `Online` pošle `ack`; záznam v `conversations` nese stav ratchetu bez historie. Odesílání: `message_send` uloží zprávu jako `queued` do `messages` a do fronty kontaktu, úloha ji odešle po připojení (`pending`) a dokud čeká, ptá se anchorů každé 2 s na `status` (`delivered`/`unknown`). Konverzace s novým kontaktem se zakládá po `attach`, aby se nejdřív zpracovaly obálky od protistrany (bod 17 v sekci 9). Kritérium splněno: E2E test `client/src-tauri/src/e2e.rs` (dva klienti, `dev/local.sh 3`) ověřuje frontu odesílatele bez sítě, živé zprávy a psaní oběma směry, stav doručení, offline příjemce (zpráva `pending`, po jeho připojení doručena) a že po opakovaném restartu (`attach` a `fetch` znovu) je každá zpráva v historii právě jednou. Výkon s reálnými daty (bod 12) se v tomto prostředí neměřil: stroj nemá WebKitGTK ani grafické prostředí, Rust běžel v kontejneru.

**Stav D4 (hotovo 2026-10-30).** Všechny obrazovky mají test s mock IPC (`client/test/screens.test.ts`). Otevření konverzace s 10 000 zprávami (`stressMessages: 10_000`) trvá v jsdom ~4 ms nad pevným čekáním testu a vykreslí jen okno (~11 řádků), cíl je < 150 ms (13.11). Je to měření logiky virtuálního seznamu, ne reálného WebView; výkon ve WebKitGTK a na Androidu zůstává pro D8.

**Stav D5 (hotovo 2026-10-30 pro Linux).** `tauri-plugin-single-instance` (druhý proces jen zaostří okno prvního), `set_content_protected` řízené nastavením `enhanced_protection` (start, `settings_set`, `unlock`; zamčený profil startuje s ochranou zapnutou), `PR_SET_DUMPABLE=0` přes crate `prctl` (bez `unsafe`, bod 16), testy závislostí `deps_audit` (`Cargo.lock`) a `client/test/deps.test.ts` (`package-lock.json`). Test odchozích spojení je součástí E2E z D3: vlákno vzorkuje `/proc/net/{tcp,tcp6,udp,udp6}` soketů vlastního procesu a po konci testu ověří, že všechna vzdálená spojení vedla jen na loopback port Workeru nebo přesměrování k nodům (krátké spojení mezi vzorky může uniknout; klient drží dlouhé WebSocket spojení). Packaging pro Linux ověřen (`tauri build --bundles deb`, 3,1 MiB). Neověřeno: instalátory pro Windows a macOS (potřebují cílový OS), WER a macOS crash reporter, tray a autostart (volitelné, nedělá se). Kritérium přijetí je splněno pro Linux, zbytek patří do D8 s podpisy.

**Stav D6 (rozpracováno, 2026-10-30).** Hotovo: Rust `sync_once` (`net::sync_once`, `Core::sync_once`), build a běh aplikace na emulátoru (`tauri android init`, `gen/android/` je v gitu, build artefakty ne) a prototyp 13.10 (bod 13 v sekci 9): `SyncReceiver.kt` → JNI `android_sync.rs` → `Core::sync_once` bez Activity a WebView, ověřeno příjmem offline zprávy na emulátoru (API 36, x86_64, KVM). Build na hostu: `npx tauri android build --debug --target x86_64 --apk` s `ANDROID_HOME`, `NDK_HOME` (26.3), `JAVA_HOME` a vypálenými `ANCORD_WORKER_URL`, `ANCORD_WORKER_PUBKEY`, `ANCORD_NODE_URL`; Worker a nody z `dev/local.sh` se emulátoru zpřístupní přes `adb reverse`. Hotovo i ochrana okna v `MainActivity.kt` (`FLAG_SECURE`, přístupnost, overlay, obsah citlivý; `setHideOverlayWindows` vyžaduje oprávnění `HIDE_OVERLAY_WINDOWS`, bez něj aplikace spadla na `SecurityException`; na emulátoru API 36 je screenshot černý, chování na reálných zařízeních a v WebView (bod 14) zbývá ověřit). Zbývá: Keystore wrap, IME a autofill, oznámení bez obsahu, skener QR, kontrola WebView), UnifiedPush receiver a foreground služba jako spouštěč `sync_once`, profil s heslem na pozadí. Kritérium D6 (černý screenshot, prázdný náhled v přehledu aplikací, QR kontakt) zatím nesplněno.

**D6: ověřeno na produkci a co zbývá (2026-10-30).** Dvě instance klienta na emulátorech (API 36, x86_64) vyměnily zprávu přes produkční Worker `ancord.melonet.cz` a produkční nody (APK s vypálenou adresou a klíčem Workeru, bez `ANCORD_NODE_URL`). Nejdřív se klient nepřipojil: node odmítl první zprávu po admission (`bad_request: unknown message type`), protože nasazené nody byly starší než protokol klienta; po nasazení nodů z aktuálního `main` vše funguje. Poučení: klient a nody se musí nasazovat na stejné verzi protokolu, klient zatím chyby spojení nehlásí uživateli (`net.rs::run` je zahazuje, v debug buildu je vypíše `eprintln!`).

Zbývá udělat (pořadí podle závislostí):
1. **Hlášení chyb spojení v UI** — *hotovo* (2026-10-30): událost `connection_issue` (`ConnectionIssue`: Worker nedostupný / chyba Workeru / trasa neověřená / node nedostupný / node odmítl `bad_request`), `net.rs::classify`, text v seznamu a v Composeru (`connText`), test v `screens.test.ts`. Verze protokolu: node ji posílá v `hello` (`v`, `ancord_core::PROTOCOL_VERSION`, `spec/PROTOCOL.md` §8.1), klient ji porovná před výměnou klíčů a při nesouladu (i u starého nodu bez `v`) hlásí `protocol_mismatch`. Původní zadání: rozlišit stavy offline (bez sítě), Worker nedostupný, trasa neověřená a node odmítl (`bad_request`, nejspíš nesoulad verzí), a ukázat uživateli důvod; podpora verze protokolu v `/route` nebo admission, aby nesoulad šel poznat dřív než po první zprávě.
2. **Skener QR** — *hotovo v kódu, sestaveno, ověřeno jen na virtuální kameře emulátoru* (2026-10-30): `ScanActivity.kt` (CameraX + zxing-core, bez Google Play služeb, `FLAG_SECURE`, snímky jen v paměti), most `window.AncordScan.scan()` v `MainActivity` (`addJavascriptInterface`, vrací jen text QR; výsledek vyplní pole adresy v `AddContact.svelte`, přidání potvrdí uživatel; Rust adresu validuje). Oprávnění `CAMERA` se žádá až při prvním skenu. Na emulátoru (virtuální scéna s QR plakáty, `ScanActivity` dočasně exportovaná) se skener po spatření kódu sám zavřel, na výchozí scéně zůstal otevřený; obsah vráceného textu a celý tok přes UI se neověřoval, chybí zkouška na skutečné kameře.
3. **Keystore wrap** klíče zařízení — *hotovo a ověřeno na emulátoru* (2026-10-30): `KeystoreKey.kt` (AES-256-GCM v Android Keystore, bez nutnosti autentizace a odemčeného zařízení, aby fungoval `sync_once` při zamčeném telefonu, proto zrušení zámku obrazovky klíč nesmaže), `android_keystore.rs` (JNI, třída držená jako globální reference z `JNI_OnLoad`), nový profil ukládá `device.key.ks` místo `device.key`, po restartu se otevře. Existující `device.key` má přednost (bez migrace). Bez klíče v Keystore (přeinstalace, obnova ze zálohy) se profil neotevře. Neověřeno: StrongBox a zařízení s poškozeným Keystore.
4. **Oznámení bez obsahu, UnifiedPush receiver a foreground služba** jako spouštěč `sync_once` (prototyp 13.10 už běží z `BroadcastReceiver`); profil s heslem na pozadí.
5. **Kontrola minimální verze WebView** — *hotovo v kódu* (`MainActivity.checkWebViewVersion`, `MIN_WEBVIEW_MAJOR` = 110, dialog a zavření aplikace); neověřeno na starém WebView, hodnotu do §9 bod 12 zapsat po měření.
6. **IME a autofill** — *částečně* : `importantForAutofill=noExcludeDescendants`, `saveFormData=false`, `autocorrect=off` u polí. Zbývá `IME_FLAG_NO_PERSONALIZED_LEARNING` (WebView vytváří wry, chce podtřídu s `onCreateInputConnection`).
7. **`minSdk`** — *hotovo*: 26 v `tauri.conf.json` (`bundle.android.minSdkVersion`) i `build.gradle.kts`.
8. **Ověření kritéria D6**: prázdný náhled v přehledu aplikací (`adb` screenshot je černý, náhled zatím nezkontrolován), QR kontakt, a chování `setAccessibilityDataSensitive` / `setContentSensitivity` / `setHideOverlayWindows` na reálném zařízení a ve WebView (bod 14 v §9, výsledek do `THREAT_MODEL.md`).
9. Spojení více procesů: `sync_once` nesmí běžet vedle online úlohy (řešeno kontrolou), ověřit na zařízení při současném běhu receiveru a aplikace.

### 13.15 Mimo rozsah v1 klienta
iOS, webový klient, přílohy a obrázky, hovory, skupinové konverzace, záloha a více zařízení (bod 11 v sekci 9), FCM (Worker ho umí, klient ne), proxy/Tor, tokenová ekonomika. Co je ve specifikaci protokolu nedefinované, se v klientovi nevymýšlí.
