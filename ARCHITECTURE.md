# Melo v2 – architektura

Decentralizovaná síť pro E2E šifrované zprávy. Nody provozuje kdokoli, nikdo jim nevěří, jejich zdroje se měří. Cloudflare slouží jako *control plane*: registrace, verifikace nodů, load balancing a push notifikace. Zprávy a spojení klientů jdou přímo na nody.

> Stav: fáze A a B jsou implementované a lokálně otestované (viz 12.4), nic není nasazené. Body označené **[ověřit]** se potvrdí ve fázi C na reálné infrastruktuře.

## 1. Cíle a hranice

- **Otevřená síť**: nody může spustit kdokoli. Důvěra vzniká z měření, stáří a reputace, ne z identity operátora.
- **Cloudflare mimo data plane**: Worker klientovi jen jednou řekne, na který node se připojit. Pak klient komunikuje výhradně s nodem.
- **Skrytí IP**: nody jsou za Cloudflare subdoménou (IP nodu zná jen Cloudflare). Node nikdy nedostane IP uživatele.
- **Bezpečnost před pohodlím**: node vidí jen ciphertext pod blinded identifikátory.
- **Implementace**: nody a CLI klient v Rustu, Worker v TypeScriptu.

Mimo rozsah tohoto dokumentu: mobilní klient, UI, tokenová ekonomika.

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
   │(2) top-3 nody + ticket│ Tunnel + DNS node<id>.<doména>
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
- Při registraci Worker přes Cloudflare API vytvoří **vlastní Tunnel pro každý node** a DNS záznam `node<id>.<doména>` (např. `node7k2m4q9x3a.melonet.cz`). Vše je plně automatické: node zná jen adresu Workeru, sám si vygeneruje klíč, zaregistruje se, dostane tunnel token a spustí `cloudflared`. Nepotřebuje veřejnou IP, otevřený port ani vlastní certifikát.
- **Jméno je jedna úroveň pod zónou**, ne `<id>.nodes.<doména>`: bezplatný univerzální certifikát pokrývá jen `*.<doména>`, dvouúrovňová jména by vyžadovala placený Advanced Certificate Manager. `<id>` je krátký prefix `node_id` (base32, např. 10 bajtů; DNS label má limit 63 znaků). Při kolizi Worker registraci odmítne.
- Worker smí upravovat a mazat **pouze** DNS záznamy s prefixem `node` a tunely, které sám vytvořil. Ostatní záznamy zóny se nedotýká.
- Hostname vlastníme my, takže odpadá `rejectUnauthorized: false` a důkaz kontroly endpointu je jednoduchý (viz 5.1).

### Uživatelé
Cloudflare k požadavku přidává `CF-Connecting-IP`, `X-Forwarded-For`, `True-Client-IP` a podobné hlavičky. Operátor nodu by je viděl, proto:
- **Varianta A (doporučená)**: Transform Rule odstraňující hlavičky s IP návštěvníka, **omezená výrazem na hostname `node*.<doména>`**. Managed transform „Remove visitor IP headers“ platí pro celou zónu, takže na sdílené (firemní) zóně by změnil chování ostatních služeb a nepoužívá se. Žádný Worker v data plane, nejlevnější a nejrychlejší. **[ověřit]**, že pravidlo pokrývá WebSocket upgrade a že nic neprosakuje.
- **Varianta B**: gateway Worker před nodem, který hlavičky sestaví sám. Největší kontrola, ale Worker je v data plane, přidává latenci a účtuje se.
- Ticket je vázaný na ephemeral klíč klienta, ne na IP.

### Důsledky
- IP uživatele zná jen Cloudflare. Cloudflare je jediný bod, který vidí metadata sítě.
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
| RAM + storage | **Proof of Space s Merkle commitmentem.** Worker pošle seed, node vygeneruje memory-hard tabulku N MB/GB a pošle Merkle root. Worker vyzve k *k* náhodným listům s deadlinem v řádu ms, node vrátí list + Merkle path, Worker ověří *k* hashů. Latence odliší RAM, disk a „počítám za běhu“. Opakuje se s novým seedem. |
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

## 9. Otevřené body

1. Managed transform vs gateway Worker pro odstranění IP uživatele (včetně WebSocket).
2. Tunnel na node vs sdílený tunnel; limity účtu.
3. Parametry Proof of Space: velikost tabulky, počet výzev *k*, deadline.
4. Skutečná cena heartbeatů přes DO WebSocket.
5. Zda gateway/Tunnel provoz zprávové aplikace vyhovuje podmínkám Cloudflare.
6. Multi-signer adresář: kdy a jak.
7. Chování při split-brain (nody se nevidí, ale vidí Worker) a expirace adresáře při dlouhém výpadku Workeru.
8. ~~Konfliktní model mailboxu při zápisu s kvorem 2~~ vyřešeno v B3: zápis je idempotentní podle `(mailbox_id, id)`, doručení je náhrobek a „doručeno“ vyhrává, pořadí je podle `(ts, id)`, čtení sjednocuje odpovědi `r − quorum + 1` anchorů (`spec/PROTOCOL.md` §13).
9. Perzistence klíče a dat nodu na Flux (instance se může přesunout na jiný stroj; ztráta klíče znamená nový `node_id` a ztrátu anchorů).
10. Velikost sítě nad stovky nodů: shardování registru (DO) a frekvence epoch. Pro 3–20 nodů se neřeší.
11. ~~Post-compromise security~~ vyřešeno: Double Ratchet s DH krokem (`spec/PROTOCOL.md` §13.2, klient `node/crates/ancord-node/src/ratchet.rs`). Zbývá šifrování hlavičky ratchetu (anchor vidí `dh`, `pn`, `n`, `seq`) a uzdravení jednostranných konverzací (prázdná „keepalive“ zpráva). Dále spam do schránky (`inbox`) a ztráta klíčů zařízení (záloha vs. forward secrecy).

Poznámka k bodu 1 (zatím nezměřeno): custom Transform Rule smí odstranit `x-forwarded-for`, `true-client-ip` a `x-real-ip`, hlavičky `cf-*` (včetně `cf-connecting-ip`) podle dokumentace Cloudflare nikoli **[ověřit]**. Node žádnou z těchto hlaviček nečte ani neloguje a `cloudflared` je předává jen lokálně na `localhost:8080`, takže IP uživatele neopustí kontejner nodu; provozovatel nodu ji ale vidět může.

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
├── node/                  node (Rust)
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
5. **Nepřidávat nic mimo rozsah** (mobilní klient, UI, tokenová ekonomika). Při nejasnosti se zeptat, nehádat.
6. Rust: `cargo clippy` a `cargo test` bez varování, žádné `unwrap` v produkční cestě. TypeScript: `strict`, testy přes vitest s Miniflare/workerd.
7. Kód a komentáře odpovídají okolnímu kódu. Dokumentace v češtině, identifikátory anglicky.

### 12.3 Co musí dodat uživatel
Vývoj běží na firemní zóně **`melonet.cz`** (sdílená zóna s existujícími záznamy, proto jsou všechny zásahy omezené na prefixy níže):

| Co | Hostname |
|---|---|
| Load balancer / control plane (Worker, custom domain) | `ancord.melonet.cz` |
| Nody | `node<id>.melonet.cz` |

- **Přihlašovací údaje** jsou v proměnných prostředí `CLOUDFLARE_API_TOKEN` a `CLOUDFLARE_ACCOUNT_ID` (volitelné, deploy ho zjistí z tokenu). Hodnoty se nikdy nevypisují ani necommitují. Kontrola z C1 (2026-10-29): token je účtový, má oprávnění pro Workers Scripts, KV, R2, Queues a Cloudflare Tunnel, ale **žádná zónová oprávnění pro `melonet.cz`** (DNS čtení vrací `10000 Authentication error`). Pro nasazení je nutné doplnit pro zónu: DNS Edit, Workers Routes Edit, Transform Rules Edit. Seznam oprávnění je v `worker/README.md` (Nasazení). Doplněno týž den; zóna má 51 záznamů, `ancord.melonet.cz` je volné a je v ní 33 cizích A záznamů `node-xxxxxxxx.melonet.cz` (s pomlčkou). Naše jména `node<16 znaků base32>` s nimi nekolidují, ale každý výraz nad `node*` je musí vyloučit (Transform Rule kontroluje i délku hostname a `node-`).
- **Dva tokeny**: `CLOUDFLARE_API_TOKEN` slouží k nasazení (wrangler). Worker za běhu používá vlastní, užší token (Tunnel Edit + DNS Edit jen pro zónu) uložený přes `wrangler secret`, ne v kódu.
- **Bezpečnost sdílené zóny**: před jakýmkoli zápisem se přečtou existující DNS záznamy a ověří se, že `ancord` ani `node*` nic nepřepíšou. Automatika maže jen záznamy s prefixem `node` a tunely, které sama vytvořila. `dev/cleanup` smaže všechny vývojové node záznamy a tunely. Před každým krokem, který v Cloudflare něco vytváří nebo nasazuje, se čeká na potvrzení uživatele.
- Queues a R2 zatím nejsou potřeba (až A4 a A5).

### 12.4 Pořadí prací a milníky

**Stav (fáze A):** A1–A5 jsou implementovány v `worker/` a splňují kritéria přijetí v testech (vitest + Miniflare, `npm test` v `worker/`), včetně sdílených vektorů v `spec/test-vectors/`. Nic není nasazeno.

**Stav (fáze B):** B1 až B4 jsou implementovány v `node/` a splňují kritéria přijetí v integračním testu `node/crates/ancord-node/tests/e2e.rs` (tři nody proti lokálně spuštěnému Workeru v `wrangler dev`, mock Cloudflare, bez tunelu), v `tests/dataplane.rs` (kvorum, výpadky, autorizace node↔node, handoff, wake proti falešnému Workeru, bez skutečného Workeru) a ve sdílených vektorech (včetně HRW a ratchetu). Data plane je v `spec/PROTOCOL.md` §13, wake v §13.6. Presence a průběžné doručování online odběrateli jsou hotové (B5, §13.7), wake se posílá jen tehdy, když doručení online nevyjde. Nic není nasazeno.

**Stav (parita s `melo`, §6.4):** identita uživatele, profily na anchorech, X3DH s obálkou ve schránce, typing, stav doručení, dávkové odběry a lokální historie jsou implementované v `node/` (včetně CLI) a otestované v `tests/dataplane.rs`, `tests/e2e.rs` a ve vektoru `pairwise.json`. Nic není nasazeno.

**Stav (uživatelský tok před fází C):** adresa `ancord:<pk>` (text, QR), push klíče schránky (v profilu) a konverzací (ze sdíleného tajemství, vektor `pairwise.json`), registrace tokenu z CLI (`push-register`) a online klient (`ancord online`: jedno spojení, schránka i konverzace, automatické přijetí kontaktů, znovupřipojení s novým `/route`). E2E test ověřuje celý tok: první kontakt vzbudí offline příjemce, po připojení přijme kontakt a zprávy živě, po odpojení ho další zpráva vzbudí znovu. Soubory klíčů a konverzací jsou zamčené, aby dva procesy nepoužily stejný klíč ratchetu dvakrát. Mobilní klient (skenování QR, FCM/APNs token) zůstává mimo rozsah.

**Stav (fáze C):** připravený kód, nic není nasazeno. `worker/scripts/deploy.ts` (`npm run deploy`) provádí C1 a C2: bez `--apply` jen čte a vypíše plán i blokery, s `--apply` vytvoří KV, R2, fronty, Transform Rule pro `node*`, tajemství a nasadí Worker na custom doménu. `dev/docker-compose.yml` (C3) se 6 nody, `node/Dockerfile` (node + `cloudflared`), `dev/bootstrap.sh` (klíče nodů a genesis allowlist před prvním startem, `ancord-node --init-identity`), `dev/cleanup.sh`. Compose je ověřený lokálně proti `wrangler dev` (mock Cloudflare, forwarder místo tunelu): 6 nodů `active`, všechny challenge prošly, doručení přes různé nody i po výpadku jednoho, restart se stejným `node_id`. Nasazení blokují chybějící zónová oprávnění tokenu (12.3) a užší token Workeru (`ANCORD_RUNTIME_CF_TOKEN`). Měření C3 (sekce 9) zbývá.

**Deploy a DNS se ve fázích A a B neřešily.** Nejdřív se napíše a lokálně otestuje kód, nasazení na Cloudflare a Docker prostředí přijdou až ve fázi C. Fáze A a B nic nevytvářejí v Cloudflare a nepotřebují DNS ani `.env`.

**Fáze A: Worker (load balancer a control plane), TypeScript, složka `worker/`. Dělá se jako první.**

| | Obsah | Kritérium přijetí |
|---|---|---|
| **A1** Registrace | `POST /register`, challenge nonce, ověření podpisu, registr nodů v Durable Objectu. Volání Cloudflare API (vytvoření tunelu a DNS `node<id>.<doména>`) je za rozhraním `CloudflareClient` s implementací pro skutečné API a mockem pro testy. Stavy nodu `pending` → `active`. | Vitest/Miniflare: registrace projde, kolize jména i špatný podpis jsou odmítnuty, Worker maže jen záznamy a tunely, které sám vytvořil. |
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
| **C1** Příprava | Kontrola tokenu a oprávnění, čtení existujících DNS záznamů, Transform Rule pro `node*.melonet.cz`. | Žádný existující záznam zóny se nezměnil. |
| **C2** Nasazení Workeru | `wrangler deploy` na `ancord.melonet.cz`. | `https://ancord.melonet.cz/health` odpovídá. |
| **C3** Šest nodů | `dev/`: Docker Compose se 6 nody, které znají jen `WORKER_URL`. | `docker compose up` bez dalšího nastavení: všech 6 nodů se zaregistruje, je dosažitelných na `node<id>.melonet.cz` bez příchozích portů. Změří se a zapíšou do sekce 9: odstranění IP hlaviček u WebSocket upgrade, idle timeout, limity tunelů a DNS záznamů na účtu. |

Pokud limity počtu tunelů nebo DNS záznamů nestačí, rozhodne se o sdíleném tunelu (bod 2 v sekci 9) až na základě naměřených čísel z C3. Body 1, 2 a 5 v sekci 9 jsou blokující pro nasazení, ne pro psaní kódu.

### 12.5 Vývojové prostředí (`dev/`)
- Jeden Docker image `ancord-node`, šest služeb `node1`…`node6` v `docker-compose.yml`.
- Každá služba má vlastní named volume (klíč a databáze), vlastní `NODE_NAME` a vlastní subdoménu. Image obsahuje node i `cloudflared`; node ho spouští jako podproces s tokenem, který dostal při registraci.
- Žádný kontejner nepublikuje port na hostitele. Dosažitelnost je pouze přes tunel, aby se ověřilo chování bez veřejné IP.
- Profil `dev` (sekce 10): genesis allowlist zapnutý, probation, hashcash a limity ASN vypnuté, peer witnessy vypnuté.
- `dev/README.md` popisuje: nastavení `.env`, `docker compose up`, kontrolu registrace, vypnutí jednoho nodu pro test failoveru.
