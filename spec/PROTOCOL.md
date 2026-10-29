# ancord – protokol (verze 1)

Tento dokument je jediná dohoda mezi Workerem (`worker/`) a nodem (`node/`). Obě strany se řídí jen jím a vektory v `test-vectors/`, nesdílejí kód. Architektura a motivace jsou v `../ARCHITECTURE.md`.

## 1. Konvence

### 1.1 Kódování
- **b64u** = base64url (RFC 4648 §5) bez paddingu. Používá se pro veškerá binární data v JSON a v hlavičkách.
- **base32** = RFC 4648, malá písmena `a-z2-7`, bez paddingu.
- Čas je celé číslo, sekundy od epochy Unixu (`t`, `exp`, `issued_at`), není-li uvedeno jinak. Pole s příponou `_ms` jsou milisekundy.
- Všechny JSON zprávy jsou UTF-8. Čísla jsou vždy **celá** (bez zlomku a exponentu), v rozsahu bezpečných celých čísel JavaScriptu (±2^53−1).

### 1.2 Kanonické JSON
Podepisované struktury se serializují jako **RFC 8785 (JCS) omezené na celá čísla**: klíče objektů seřazené podle kódových jednotek UTF-16, bez bílých znaků, řetězce escapované jako `JSON.stringify`. Hodnoty `null` jsou dovoleny, `undefined` neexistuje (pole se vynechá).

### 1.3 Podpisy a doménová separace
Všechny podpisy jsou **Ed25519** (RFC 8032). Podepisuje se vždy

```
DOMAIN ‖ 0x00 ‖ payload
```

kde `DOMAIN` je ASCII řetězec níže a `payload` je definován u každé zprávy. Žádný podpis není platný v jiném kontextu.

| Doména | Použití |
|---|---|
| `ancord/v1/register` | potvrzení registrace |
| `ancord/v1/proof` | důkaz držení klíče na hostname |
| `ancord/v1/req` | podepsaný HTTP request |
| `ancord/v1/heartbeat` | heartbeat po WebSocketu |
| `ancord/v1/directory` | podpis adresáře Workerem |
| `ancord/v1/ticket` | podpis ticketu Workerem |
| `ancord/v1/report` | report peer witnessu |
| `ancord/v1/push-reg` | registrace push tokenu klientem |
| `ancord/v1/chan-node` | node podepisuje přepis handshaku s klientem (§14.1) |
| `ancord/v1/chan-client` | klient (klíč `ek`) podepisuje přepis handshaku (§14.1) |
| `ancord/v1/n2n-resp` | responder podepisuje přepis handshaku node↔node (§14.2) |

Hashe: `blake3` (32 B) pro identitu a HRW, `sha256` pro hashcash, Merkle stromy, hash-chain a těla požadavků.

### 1.4 Chyby
HTTP chyby mají tvar `{"error":{"code":"<kód>","message":"<text>"}}`. Kódy jsou stabilní, text je jen pro člověka.

| Stav | Kód | Význam |
|---|---|---|
| 400 | `bad_request` | neplatný JSON nebo pole |
| 401 | `unauthorized` | chybí nebo je neplatný podpis requestu / admin token |
| 403 | `forbidden` | node není povolen, je zabanován, není anchorem |
| 404 | `not_found` | neznámý node, challenge nebo cesta |
| 409 | `conflict` | kolize hostname, registrace již probíhá, replay |
| 413 | `too_large` | tělo přesahuje limit |
| 422 | `proof_failed` | neplatný podpis registrace, hashcash nebo důkaz hostname |
| 429 | `rate_limited` | limit (ASN, /24, wake). Odpověď nese `Retry-After` |
| 502 | `upstream` | selhalo volání Cloudflare API nebo hostname nodu |
| 503 | `insufficient_nodes` / `stale_snapshot` / `unavailable` | síť je degradovaná, klient má zopakovat později |

## 2. Identita nodu

- Node má perzistentní Ed25519 klíč. `pubkey` je 32 B (b64u).
- `node_id = base32(blake3(pubkey))` (32 B → 52 znaků).
- `hostname = "node-" ‖ node_id[0..16] ‖ "." ‖ zone` (prvních 16 znaků = 10 bajtů hashe). Příklad: `node-qx7c2a5k3m4ztdwp.melonet.cz`. Dvě různé `pubkey` se shodným prefixem jsou kolize, registrace druhého nodu se odmítne (`409`).

## 3. Podepsané HTTP requesty (node → Worker)

Hlavička:

```
Authorization: Ancord-Ed25519 k=<pubkey b64u>,t=<unix>,n=<nonce b64u, 16 B>,s=<podpis b64u>
```

Podepisovaný `payload` (domény `ancord/v1/req`), ASCII spojené znakem `\n`:

```
METHOD ‖ \n ‖ PATH_AND_QUERY ‖ \n ‖ t ‖ \n ‖ n ‖ \n ‖ b64u(sha256(body))
```

`PATH_AND_QUERY` je `pathname + search` tak, jak je request odeslán (bez schématu a hostu). Prázdné tělo má `sha256("")`.

Worker odmítne request (`401`), pokud: `|now − t| > 60 s`, podpis nesedí, nebo se dvojice `(k, n)` už objevila v okně 2 × 60 s (replay, `409`). Klíč `k` musí patřit zaregistrovanému nodu, s výjimkou kroku `register/confirm`, kde se používá podpis v těle.

## 4. Registrace

Stav nodu: `provisioning → pending → active`, dále `retired` (po `POST /deregister`) a `banned`.

### 4.1 `POST /register` – výzva
Tělo:
```json
{"pubkey":"<b64u>","claimed_specs":{"cpu_cores":4,"ram_mb":8192,"storage_gb":200,"bandwidth_mbps":500}}
```
`claimed_specs`: všechna pole celá čísla ≥ 1 (`cpu_cores` ≤ 1024, `ram_mb` ≤ 4 194 304, `storage_gb` ≤ 1 048 576, `bandwidth_mbps` ≤ 400 000).

Odpověď `200`:
```json
{"node_id":"…","hostname":"…","challenge_nonce":"<b64u 32 B>","expires_at":1760000000,"hashcash_bits":0}
```
`hashcash_bits` je 0 pro genesis nody a vypnuté profily. Chyby: `403` (node není na allowlistu, je zabanován), `429` (příliš mnoho otevřených výzev z ASN).

### 4.2 `POST /register/confirm` – potvrzení
Tělo:
```json
{"pubkey":"…","challenge_nonce":"…","signature":"<b64u>","hashcash_counter":12345}
```
- `signature` = Ed25519 přes doménu `ancord/v1/register` a `payload = nonce(32 B) ‖ hostname(UTF-8) ‖ pubkey(32 B)`.
- **Hashcash**: číslo `hashcash_counter` (u64) je platné, pokud `sha256(nonce(32 B) ‖ pubkey(32 B) ‖ counter_le_u64)` má alespoň `hashcash_bits` úvodních nulových bitů. `hashcash_counter` se vynechá při `hashcash_bits = 0`.

Worker ověří výzvu (existuje, nevypršela, patří této `pubkey`, je jednorázová), podpis, hashcash a limity ASN a /24 (`request.cf.asn`, `CF-Connecting-IP`; /24 pro IPv4, /48 pro IPv6). Poté vytvoří Tunnel, nakonfiguruje jeho ingress na `hostname → NODE_ORIGIN` a vytvoří DNS CNAME. Odpověď `200`:
```json
{"node_id":"…","hostname":"…","status":"pending","tunnel_token":"<token cloudflared>","proof_path":"/.well-known/melo-proof","activate_before":1760000900}
```
Opakované potvrzení již zaregistrovaného (a ne zabanovaného) nodu s platnou výzvou vrátí znovu `tunnel_token` bez nových zdrojů (restart nodu). `409`, pokud právě probíhá provisioning.

### 4.3 `POST /register/activate` – aktivace
Podepsaný request (§3), prázdné tělo `{}`. Worker vygeneruje 32 B `c` a zavolá

```
GET https://<hostname>/.well-known/melo-proof?c=<b64u c>
```

Node odpoví `200` s `{"pubkey":"<b64u>","signature":"<b64u>"}`, kde podpis je přes doménu `ancord/v1/proof` a `payload = c(32 B) ‖ hostname(UTF-8) ‖ pubkey(32 B)`. Při úspěchu přejde node na `active` (`{"status":"active"}`), jinak `422`/`502`. Nevyřízený `pending` node starší než `pending_ttl_s` se uklidí (tunel a DNS se smažou).

### 4.4 `POST /deregister`
Podepsaný request. Node přejde na `retired`, Worker smaže tunel a DNS záznam, které sám vytvořil, a zavře WebSocket. Nový `register` je možný.

## 5. WebSocket (heartbeat a challenge)

`GET /ws` s `Upgrade: websocket` a podepsaným requestem (§3, prázdné tělo). Node musí být `active`. Jeden node = jedno spojení (nové nahradí staré kódem `4000`). Zprávy jsou textové JSON, `t` je typ, max. 512 KiB.

Worker → node:

| `t` | Obsah |
|---|---|
| `welcome` | `{epoch, heartbeat_interval_s}` po připojení |
| `ack` | `{seq, epoch}` po heartbeatu |
| `directory` | `{directory}` nový podepsaný adresář (§6) |
| `ping` | `{id}`, node odpoví `pong` se stejným `id` |
| `challenge` | viz §9 |
| `bye` | `{reason}` před zavřením |
| `error` | `{reason}` zpráva nodu byla odmítnuta (špatný podpis, `seq`, čas, tvar), spojení zůstává |

Node → Worker:

| `t` | Obsah |
|---|---|
| `hb` | heartbeat (níže) |
| `pong` | `{id}` |
| `challenge_response` | viz §9 |

Zavření spojení: `4000` nahrazeno novým spojením téhož nodu, `4003` node je zabanován, `1000` deregistrace.

Textová zpráva `ping` (bez JSON) obslouží runtime Workeru odpovědí `pong` bez probuzení DO; nody ji posílají v intervalu < 60 s, aby WebSocket nevypršel.

### Heartbeat
```json
{"t":"hb","seq":17,"ts":1760000000,"epoch":4242,
 "load":{"conns":12,"max_conns":2000,"cpu_pct":8,"mem_pct":31},
 "sig":"<b64u>"}
```
`sig` = Ed25519 přes doménu `ancord/v1/heartbeat` a `payload = kanonické JSON zprávy bez pole sig`. `seq` musí striktně růst v rámci jednoho spojení (po připojení Worker čítač vynuluje, první heartbeat může mít `seq = 0`), `|now − ts| ≤ 60 s`. `epoch` je poslední adresář, který node drží (nepovinné). Je-li starší, nebo chybí (node po startu žádný nedrží), Worker pošle `directory`. Interval určuje profil (`heartbeat_interval_s`).

## 6. Adresář a epochy

`epoch = floor(unix / epoch_seconds)`. Adresář epochy `E` se sestaví **jednou**, při prvním přístupu v epoše, z nodů ve stavu `active`, zabanované a neaktivní (poslední heartbeat starší než `heartbeat_timeout_s`) se vynechají. Je neměnný po celou epochu, takže všichni dostanou stejný adresář. Odpojení nodu se tedy projeví až v další epoše.

```json
{"v":1,"epoch":4242,"epoch_seconds":300,"issued_at":1272600,
 "params":{"r":3,"quorum":2},
 "nodes":[{"id":"<node_id>","pk":"<pubkey b64u>","host":"<hostname>"}]}
```
`nodes` seřazeno vzestupně podle `id`. `issued_at = epoch × epoch_seconds`. Obálka:
```json
{"body":{…},"sig":"<b64u>","kid":"<b64u prvních 8 B sha256(pubkey Workeru)>"}
```
`sig` = Ed25519 klíčem Workeru přes doménu `ancord/v1/directory` a `payload = kanonické JSON body`.

`GET /directory` vrací aktuální obálku (`Cache-Control: public, max-age=15`). Klíč Workeru je zapečený v klientovi i nodu, `GET /.well-known/ancord-key` ho vrací jako `{"pubkey":"…","kid":"…"}`.

## 7. Anchory (HRW)

Anchory mailboxu `mailbox_id` (32 B, b64u) v epoše `E`: pro každý node z adresáře `E` spočti

```
weight = blake3("ancord/v1/hrw" ‖ 0x00 ‖ mailbox_id(32 B) ‖ node_id(UTF-8) ‖ 0x00 ‖ E_le_u64)
```
a vezmi `r` nodů s nejvyšší `weight` (porovnání jako big-endian 256bitové číslo, remíza podle `node_id`). Worker při `wake` akceptuje anchory z aktuální a předchozí epochy.

## 8. Routing a ticket

`GET /route?ek=<b64u Ed25519 efemerní veřejný klíč klienta, 32 B>`. Odpověď `200`:

```json
{"epoch":4242,"ttl":120,"nodes":[{"node_id":"…","hostname":"…","ticket":"<ticket>"}]}
```
Počet nodů určuje profil (`route_nodes`, výchozí 3), první je preferovaný. `503 insufficient_nodes`, pokud je živých nodů méně než `min_live_nodes`. Odpověď má `Cache-Control: no-store`.

**Ticket** = `b64u(canonical_json(body)) ‖ "." ‖ b64u(sig)` kde
```json
{"v":1,"nid":"<node_id>","ek":"<ek b64u>","exp":1760000120,"n":"<b64u 16 B>"}
```
`sig` = Ed25519 klíčem Workeru přes doménu `ancord/v1/ticket` a `payload = canonical_json(body)` (přesně ty bajty, které jsou v prvním segmentu). Node ticket přijme, jen když `nid` je jeho `node_id`, `exp > now`, podpis sedí a `n` nebyl v platnosti použit. Klient se navíc prokáže držením `ek` (podpis výzvy nodu efemerním klíčem, definuje protokol data plane).

### 8.1 Admission na nodu

Klient se připojí na `wss://<hostname>/client` (WebSocket). Cloudflare ukončuje TLS, proto se mezi klientem a nodem navazuje vlastní autentizovaný šifrovaný kanál (§14.1). Admission je jeho součástí:

1. Klient **před připojením** ověří ticket klíčem Workeru, který má připnutý mimo síť: podpis, `nid` odpovídá `node_id` z trasy, `ek` je jeho klíč, `exp`. Bez toho by trasu mohl podvrhnout kdokoli na cestě.
2. Node po upgradu pošle `{"t":"hello","node_id":"<node_id>","challenge":"<b64u 32 B>"}`; klient odpoví `init`, node `accept` a od té chvíle je vše šifrované (§14.1).
3. Klient do `auth_timeout_s` (výchozí 10 s) pošle šifrovaně `{"t":"auth","ticket":"<ticket>","sig":"<b64u>"}`, kde `sig` je Ed25519 klíčem `ek` přes doménu `ancord/v1/chan-client` a `payload = th` (přepis handshaku, §14.1). Podpis váže `ek` k tomuto jedinému spojení; ticket sám bez `ek` nestačí.
4. Node ověří ticket (§8: podpis Workeru, `nid`, `exp`), podpis proti `ek` z ticketu a že `n` nebyl použit (jednorázové do `exp`). Při úspěchu odpoví `{"t":"ready"}`, jinak `{"t":"error","reason":"…"}` a zavře spojení kódem `4401`. Bez `auth` do limitu se zavře kódem `4408`. Rámec, který nejde ověřit, zavře spojení kódem `4400`.
5. Po `ready` node odpovídá na `{"t":"ping"}` zprávou `{"t":"pong"}`. Zprávy data plane (`send`, `fetch`, `ack`, `subscribe`) jsou v §13.

Volba nodů: viz `worker/` (power-of-two-choices nad váženým výběrem filtrovaným podle vzdálenosti klienta z `request.cf`). Klienti na pořadí nemají spoléhat nad rámec „první je preferovaný“.

## 9. Challenge protokoly

Před každým challenge pošle Worker `ping` s `id` challenge a node odpoví `pong` se stejným `id`; z toho Worker odvodí základní RTT. Teprve pak přijde `{"t":"challenge","id":"<b64u>","kind":"…",…}`. Node odpoví `{"t":"challenge_response","id":"<stejné>",…}` nebo `{"t":"challenge_response","id":"…","error":"<text>"}`. V jednu chvíli běží nejvýše jeden challenge na node. Čas odpovědi měří Worker a odečítá naměřené RTT. `deadline_ms` je informativní limit (u `space_open` závazný pro latenci, u ostatních hard timeout). Výsledek: platná odpověď zvýší naměřenou kapacitu; chybějící odpověď nebo `error` sníží jen zdraví; příliš pomalá odpověď vynuluje kapacitu dané dimenze; neplatný důkaz ji vynuluje a přidá trestný bod (po `ban_strikes` po sobě jdoucích trestných bodech v téže výzvě následuje ban).

### 9.1 `cpu` – časovaný sekvenční hash-chain
Výzva: `{"kind":"cpu","seed":"<b64u 32 B>","iters":N,"every":C,"lanes":L,"deadline_ms":…}`.
Pro `lane ∈ [0, L)`: `h₀ = sha256(seed ‖ lane_le_u32)`, `h_{i+1} = sha256(h_i)`. Node vrátí `checkpoints`: pole `L` polí, v každém `N / C` hodnot (b64u), kde prvek `j` je `h_{(j+1)·C}`. (`N` je násobek `C`.) Worker přepočítá `S` náhodných segmentů (lane, j): z `h_{j·C}` (pro `j = 0` je to `h₀`) spočte `C` hashů a porovná s `checkpoint[j]`. Skóre = `L · N / (čas − RTT)` hashů za sekundu (shora omezené plausibilním maximem). `L = min(cpu_cores, max_lanes)`.

### 9.2 `space_commit` a `space_open` – Proof of Space s Merkle commitmentem
Tabulka má `n` bloků po 32 B (`n` je mocnina dvou):
```
T[0] = sha256("ancord/space/0" ‖ seed)
j(i) = u64_le(T[i−1][0..8]) mod i                  pro i ≥ 1
T[i] = sha256("ancord/space/i" ‖ seed ‖ u64_le(i) ‖ T[i−1] ‖ T[j(i)])
```
Merkle strom: `leaf(i) = sha256(0x00 ‖ T[i])`, `node = sha256(0x01 ‖ L ‖ R)`. Cesta = sourozenecké hashy odspodu nahoru, `bit k` indexu určuje, zda je prvek na úrovni `k` levý (0) nebo pravý (1).

- **`space_commit`**: `{"kind":"space_commit","tier":"ram|storage","seed":"…","leaves":n,"deadline_ms":…}`. Node odpoví `{"root":"<b64u>"}`. Worker si `(tier, seed, root, leaves)` uloží.
- **`space_open`**: `{"kind":"space_open","tier":"…","indices":[i₁…i_k],"deadline_ms":…}` (každé `i ≥ 1`, `i < n`). Pro každé `i` vrátí node `{"i":…,"leaf":T[i],"path":[…],"prev":T[i−1],"prev_path":[…],"ref":T[j(i)],"ref_path":[…]}` (hodnoty a cesty b64u). Worker ověří tři cesty vůči uloženému kořeni, spočte `j(i)` z `prev`, zkontroluje `ref` na indexu `j(i)` a přepočítá `T[i]`.

Po `space_commit` následuje bezprostředně `space_open`; samotný `space_open` se opakuje nad uloženým commitmentem. Latence odpovědi (po odečtení RTT) musí být v `deadline_ms` dané vrstvy (`ram` je přísnější než `storage`). Prokázaná kapacita je `n · 32 B`.

### 9.3 `bandwidth` – R2 blob (download a upload)
Výzva: `{"kind":"bandwidth","bytes":B,"dl_url":"https://…","ul_url":"https://…","deadline_ms":…}`. Node stáhne `GET dl_url` (celé tělo, `B` bajtů), spočte `sha256` a pošle stejné bajty `PUT ul_url`. Pak odpoví `{"sha256":"<b64u>"}`. URL jsou jednorázové (token s platností `token_ttl_s`, vázaný na challenge a směr). Worker měří dobu přenosu na své straně (od začátku odpovědi po poslání posledního bajtu, resp. od začátku po přečtení posledního bajtu uploadu) a kontroluje hash. Skóre = `min(download, upload)` v Mbit/s.

### 9.4 `witness` – peer witness
Worker pošle 2–3 nodům z různých ASN: `{"kind":"witness","target_id":"…","target_host":"…","bytes":B,"deadline_ms":…}`. Svědek stáhne `GET https://<target_host>/.well-known/melo-bw?bytes=B` (podepsaný request §3) a odpoví

```json
{"report":{"target_id":"…","witness_id":"…","bytes":B,"duration_ms":D,"ts":T},"sig":"<b64u>"}
```
`sig` přes doménu `ancord/v1/report` a `canonical_json(report)`. `id` challenge pro svědka je `<id challenge cíle>.<prvních 8 znaků jeho node_id>` a odpověď ho opakuje. Report platí, jen když `target_id`, `witness_id` a `bytes` sedí a podpis ověří klíč svědka. Svědci jsou z navzájem různých ASN a jiných než cíl, cíl se měří, jen když je v síti dost různých ASN (`min_asns`). Kapacita cíle = minimum z R2 měření a mediánu svědků.

## 10. Push

### 10.1 Registrace tokenu
Push klíč je Ed25519 a `push_id = base32(blake3(push_pubkey))`. Klient má dva druhy push klíčů (§15.11): klíč schránky, jehož `push_id` je v profilu, a klíč každého příchozího mailboxu konverzace, odvozený ze sdíleného tajemství. Worker je nerozlišuje. Klient registruje stejný token pod všemi svými `push_id`.

`POST /push/register`:
```json
{"push_pubkey":"<b64u>","platform":"fcm|apns","token":"<řetězec, max 4096 znaků>","ts":1760000000,"sig":"<b64u>"}
```
`sig` přes doménu `ancord/v1/push-reg` a `canonical_json({"op":"register","platform":…,"push_id":…,"token":…,"ts":…})`. `|now − ts| ≤ 300 s`. Registrace s `ts` nižším nebo rovným uloženému se odmítne (`409`, ochrana proti přehrání starší registrace). Odpověď `200 {"push_id":"…"}`. Token je uložen šifrovaně (AES-GCM, AAD = `push_id`).

`DELETE /push/register` s tělem `{"push_pubkey":…,"ts":…,"sig":…}`, kde `sig` je přes `canonical_json({"op":"delete","push_id":…,"ts":…})`, smaže záznam. Klíč, ne `push_id`, určuje, čí záznam se maže.

### 10.2 `POST /wake`
Podepsaný request (§3) anchoru, tělo `{"push_id":"…","mailbox_id":"<b64u 32 B>"}`. Worker ověří: node je v aktuálním nebo předchozím adresáři a je mezi `r` anchory `mailbox_id` (§7), limity (token bucket per node, koalescence per `push_id`), pak zařadí tichý data-push. Odpověď `202 {"status":"queued|coalesced|no_token"}`. Chyby: `403` (node není v adresáři nebo není anchorem), `409` (replay), `429` (limit). Obsah push zprávy je pevný (`wake`), neobsahuje mailbox ani adresy nodů.

## 11. Konfigurační parametry
Hodnoty parametrů (R, kvorum, epocha, intervaly, hashcash, limity ASN, parametry challenge) jsou v profilech `dev`/`prod`/`open` (`worker/src/config.ts`), ne v protokolu. Protokol je přenáší jen v adresáři (`params`) a ve výzvách.

## 12. Testovací vektory
`test-vectors/*.json` obsahují deterministické vstupy a očekávané výstupy pro: `node_id` a hostname, kanonické JSON, podpis registrace a důkazu, hashcash, podepsaný request, heartbeat, adresář, ticket, HRW, hash-chain, Merkle/space, push registraci, mailbox a ratchet (§13) a profil, X3DH, obálku a bezpečnostní číslo (`pairwise.json`, §15). Obě implementace musí projít všechny.

## 13. Data plane

Node je slepá schránka: ukládá jen ciphertext pod `mailbox_id` a nezná odesílatele ani příjemce. Šifrování je čistě mezi klienty, node ratchet nikdy nepočítá. Dohoda sdíleného tajemství mezi klienty (párování) je mimo rozsah.

### 13.1 Mailbox a zpráva
- Klienti sdílejí tajemství `ss` (32 B). Každý směr má vlastní mailbox, takže do každého píše jediný odesílatel: `dir ∈ {"i2r","r2i"}` (initiator → responder, opačně).
- `mailbox_id(dir) = sha256("ancord/v1/mailbox" ‖ 0x00 ‖ ss ‖ 0x00 ‖ dir)` (32 B, v JSON b64u).
- Zpráva: `id` (16 B náhodných, generuje klient, b64u), `ts` (sekundy klienta, jen k řazení; musí být v rozmezí ±86 400 s od času nodu), `ct` (b64u, po dekódování nejvýše 32 KiB). `(mailbox_id, id)` identifikuje zprávu; opakovaný zápis téhož `id` nic nemění (idempotence).

### 13.2 Ratchet (klient ↔ klient)
Double Ratchet (Signal) nad dvěma jednosměrnými mailboxy: symetrický řetěz dává forward secrecy, DH krok post-compromise security. HKDF je HKDF-SHA256 (RFC 5869), AEAD ChaCha20-Poly1305 (RFC 8439), DH je X25519 (výstup samých nul se odmítne). Prázdná sůl znamená `HashLen` nul (RFC 5869 §2.2).

**Odvození klíčů.**
```
rk_0         = HKDF(ikm = ss, salt = "ancord/v1/ratchet", info = "root", L = 32)
KDF_RK(rk, d) = okm = HKDF(ikm = d, salt = rk, info = "ancord/v1/ratchet/root", L = 64)   → (rk' = okm[0..32], ck = okm[32..64])
mk_n         = HKDF(ikm = ck_n, salt = (prázdná), info = "ancord/v1/ratchet/msg", L = 44)   key = mk_n[0..32], nonce = mk_n[32..44]
ck_{n+1}     = HKDF(ikm = ck_n, salt = (prázdná), info = "ancord/v1/ratchet/chain", L = 32)
```

**Stav strany:** kořenový klíč `rk`, vlastní ratchet pár `DHs`, poslední ratchet klíč protistrany `DHr`, odesílací a přijímací řetěz `CKs`, `CKr`, čítače `Ns` (zpráv v odesílacím řetězu), `Nr` (v přijímacím), `PN` (délka předchozího odesílacího řetězu), `seq` (zpráv odeslaných celkem) a uložené klíče přeskočených zpráv. Odesílací mailbox je `mailbox_id(dir)` své role, přijímací protistrany (§13.1); mailboxy se kvůli ratchetu nemění.

**Začátek.** Iniciátor zná první ratchet klíč `R` responderu (veřejný) a vygeneruje `DHs`:
```
iniciátor:  rk, CKs = KDF_RK(rk_0, X25519(DHs, R));  DHr = R;  CKr = žádný
responder:  rk = rk_0;  DHs = (tajný klíč k R);  DHr = žádný;  CKs = CKr = žádný
```
`R` určuje způsob dohody: u X3DH (§15.4) je to podepsaný prekey `spk` příjemce, u ručně sdíleného tajemství (bez prekeys) `R = X25519 veřejný klíč k b = HKDF(ikm = ss, salt = "ancord/v1/ratchet", info = "initial-dh", L = 32)`. Responder tedy **nemůže psát, dokud nepřijme první zprávu iniciátora** (nemá odesílací řetěz); `seal` pak selže.

**Zpráva** s hlavičkou `H = dh(32 B) ‖ pn_be_u32 ‖ n_be_u32 ‖ seq_be_u32` (44 B; `dh` je veřejný klíč `DHs` odesílatele, `pn = PN`, `n = Ns`):
```
aad = "ancord/v1/msg" ‖ 0x00 ‖ mailbox_id(32 B) ‖ id(16 B) ‖ H
ct  = b64u( H ‖ ChaCha20-Poly1305(key, nonce, aad, plaintext) )      // šifrový text vč. 16 B tagu
```
Odesílatel vezme `mk` z `CKs`, posune `CKs`, zvýší `Ns` a `seq`. `seq` slouží jen k řazení zpráv u příjemce (`ts` má sekundovou přesnost a `n` se v každém řetězu vrací na nulu).

**Příjem.** Pracuje se na kopii stavu, která se přijme až po ověření tagu, takže podvržená zpráva nic nezmění:
1. Existuje uložený klíč pro `(dh, n)` → použije se a smaže.
2. Jinak, je-li `dh ≠ DHr`: uloží se klíče zbývajících zpráv přijímacího řetězu do `pn` a provede se **DH krok**: `PN = Ns; Ns = 0; Nr = 0; DHr = dh; rk, CKr = KDF_RK(rk, X25519(DHs, DHr)); DHs = nový pár; rk, CKs = KDF_RK(rk, X25519(DHs, DHr))`.
3. Je-li `n < Nr` (a klíč není uložený), zpráva se odmítne jako přehraná; jinak se uloží klíče zpráv `Nr..n` přeskočených v řetězu a použije se `mk_n`.

Mezera větší než `max_skip = 200` zpráv v řetězu (`n − Nr`, resp. `pn − Nr`) se odmítne jako desync; uložených klíčů je nejvýše `max_skip` (nejstarší se zahazují). Po použití se klíč zprávy i `ck_n` zahodí (forward secrecy v rámci řetězu).

**Post-compromise security.** Kdo získá celý stav strany v čase *t*, čte zprávy, dokud tato strana neudělá DH krok s novým párem, který útočník nezná: první odpověď protistrany, kterou přijme, a pak odpověď na ni. Od té chvíle `rk` závisí na DH výstupu, který útočník bez nového tajného klíče nespočítá, a konverzace se uzdraví. Platí to jen, když si strany píšou oba směry: jednostranný proud zpráv se nikdy neuzdraví.

**Hlavička není skrytá.** Anchor ve zprávě vidí `dh`, `pn`, `n` a `seq` (náhodný klíč nesouvisí s identitou). Z toho pozná, kdy odesílatel poprvé přijal odpověď (změna `dh`), a počet zpráv v mailboxu. Šifrování hlavičky (Signal „header encryption“) by to skrylo, ale zvětšilo by stav; zatím se nedělá.

### 13.3 Zprávy klient → node
Po `ready` (§8.1). Každý požadavek může nést `rid` (řetězec do 64 znaků), který odpověď zopakuje. Chyba má tvar `{"t":"error","rid":…,"code":"<kód>","reason":"<text>"}`; spojení po ní zůstává.

| Požadavek | Úspěch | Význam |
|---|---|---|
| `{"t":"send","id","mailbox_id","ts","ct","push_id"?}` | `{"t":"sent","id","acks":n}` | zápis na `r` anchorů, hotovo po `quorum` potvrzeních; volitelné `push_id` příjemce (§10.1) spustí wake (§13.6) |
| `{"t":"fetch","mailbox_id","limit"}` | `{"t":"messages","mailbox_id","messages":[{"id","ts","ct"}]}` | nedoručené zprávy seřazené podle `(ts, id)` (`ts` má sekundovou přesnost, přesné pořadí určuje číslo `n` v ratchetu, podle něj zprávy řadí klient); `limit` 1–500 (výchozí 100) |
| `{"t":"ack","mailbox_id","ids":[…]}` | `{"t":"acked","n":k}` | zprávy (nejvýše 500) jsou doručené a anchory je smažou; `n` je počet `ids` v požadavku |
| `{"t":"subscribe","mailbox_id"}` | `{"t":"subscribed","mailbox_id","anchors":n}` | node se stává gateway mailboxu: oznámí anchorům presence (§13.7); `n` je počet potvrzených anchorů. Nejvýše 32 odběrů na spojení |
| `{"t":"unsubscribe","mailbox_id"}` | `{"t":"unsubscribed","mailbox_id"}` | zruší odběr a presence |

Po `subscribe` posílá node asynchronně `{"t":"message","mailbox_id","id","ts","ct"}` (bez `rid`), když anchor přepošle novou zprávu (§13.7). Zpráva zůstává na anchorech `pending`, dokud ji klient nepotvrdí `ack`.

Kódy chyb: `bad_request`, `too_large`, `rate_limited` (limit zpráv na spojení), `no_directory` (node ještě nemá adresář), `insufficient_nodes` (adresář má méně nodů než `quorum`), `quorum_unavailable` (anchory neodpověděly v dostatečném počtu), `mailbox_full`.

- **Anchory** zpracuje node z adresáře své aktuální epochy (§7) s `r` a `quorum` z `params`. Pokud je sám anchorem, zapisuje do vlastního úložiště, jinak volá ostatní (§13.4). Odesílá všem `r` anchorům současně a odpoví, jakmile má `quorum` potvrzení; zbylé zápisy dobíhají na pozadí.
- **`fetch`** se dotáže všech anchorů a skončí, jakmile odpoví aspoň `r − quorum + 1` z nich (takové čtení se vždy protne s jakýmkoli zápisem na kvorum). Výsledek je sjednocení podle `id`. Méně odpovědí je `quorum_unavailable`.
- **`ack`** jde na všechny anchory a uspěje při `quorum` potvrzeních. Je idempotentní.

### 13.4 Node ↔ node
`POST /n/put`, `/n/get`, `/n/ack` na `https://<hostname>` cílového nodu, tělo JSON (nejvýše 128 KiB), podepsaný request podle §3. Příjemce odmítne:
- `401` neplatný podpis, `409` replay (nonce okno jako §3),
- `403` volající není v adresáři aktuální ani předchozí epochy, nebo příjemce není anchorem `mailbox_id` v aktuální ani předchozí epoše,
- `413` příliš velké tělo, `400` neplatné tělo.

O jednu epochu napřed bývá odesílatel jen na několik vteřin, příjemce adresář ještě neobdržel a `403` (i při handshaku) je dočasné. Odesílatel proto `403` zopakuje dvakrát s prodlevou 400 ms (s novým nonce), než ho vyhodnotí jako chybu.

| Cesta | Tělo | Odpověď `200` |
|---|---|---|
| `/n/put` | `{"mailbox_id","id","ts","exp","ct","push_id"?}` nebo `{"mailbox_id","id","ts","exp","delivered":true}` (náhrobek, bez `ct`) | `{"status":"stored"\|"duplicate"}` |
| `/n/get` | `{"mailbox_id","limit"}` | `{"messages":[{"id","ts","exp","ct"}]}` |
| `/n/ack` | `{"mailbox_id","ids":[…]}` | `{"n":k}` (počet změněných záznamů) |

`put` s existujícím `(mailbox_id, id)` vrací `duplicate` a nemění nic, jen `delivered: true` převede pending záznam na doručený (delivered vyhrává vždy). `ack` vyrobí z neexistující zprávy záznam „doručeno“, takže pozdní `put` zprávu neoživí. Plný mailbox odpoví `507 mailbox_full`.

`POST /n/presence` (gateway → anchor) `{"mailbox_id","exp"}` → `{"ok":true}`: volající je gateway (z podpisu), `exp` je unixový čas konce platnosti (`0` ruší). Příjemce musí být anchorem `mailbox_id`.
`POST /n/deliver` (anchor → gateway) `{"mailbox_id","id","ts","ct"}` → `{"delivered":bool}`: `403`, pokud volající není anchorem `mailbox_id`. `delivered` je `true`, když zprávu dostal aspoň jeden odběratel (nebo ji gateway už předala, duplicity od dalších anchorů se zahazují).

### 13.5 Úložiště, TTL, handoff
- Záznam je `pending` (nese `ct`) nebo `delivered` (jen náhrobek bez `ct`). Oba mají `exp`; po `exp` je smaže úklid. Příjemce si `exp` omezí na `now + message_ttl_s` (výchozí 30 dní), takže handoff ani opakovaný zápis životnost neprodlužují.
- Strop nedoručených zpráv v jednom mailboxu na nodu je `max_mailbox_messages` (výchozí 10 000).
- **Handoff**: po přijetí adresáře nové epochy projde node svá uložená data. Pro každý mailbox spočte anchory nové epochy a každý záznam (i náhrobek) pošle `put` všem z nich kromě sebe. Pokud sám už anchorem není a záznam přijalo aspoň `quorum` nových anchorů, smaže ho. Neúspěšné pokusy se opakují při dalším průchodu údržby, dokud se neuspěje; po dobu, co by nové anchory ještě neznaly novou epochu, je příjemce přijme díky pravidlu o předchozí epoše.
- Pod `quorum` nodů v adresáři se zápisy odmítají (`insufficient_nodes`), jiný degradovaný režim není.

### 13.6 Wake (push)
- `push_id` je 52 znaků `[a-z2-7]` (§10.1). Odesílatel ho zná: u mailboxu konverzace ho odvodí ze sdíleného tajemství, u schránky příjemce ho má z jeho profilu (§15.11). Koordinující node ho přidá do `/n/put` všem anchorům. Neplatný tvar se odmítne (`bad_request`). `push_id` se neukládá.
- Anchor, který zprávu **nově uložil** (`stored`, ne `duplicate`, ne náhrobek) a dostal `push_id`, na pozadí pošle Workeru podepsaný `POST /wake` (§10.2) s `{push_id, mailbox_id}`. Odpověď Workeru se na zápis zprávy nijak nepromítá: selhání wake nikdy neshodí `send`.
- Anchor koaleskuje i sám: pro stejný `push_id` pošle nejvýše jeden wake za `wake_min_interval_s` (výchozí 10 s). Odpověď `no_token` se do koalescence nepočítá (push neodešel; příjemce mohl token právě zaregistrovat). Worker koaleskuje znovu (per `push_id`) a limituje per node, takže až `r` souběžných wake od anchorů téže zprávy skončí nejvýš jednou jako push.
- Wake se posílá, jen když doručení online nevyšlo (§13.7): anchor nemá platnou presence, gateway nedostupná, nebo gateway odběratele nemá.

### 13.7 Presence a online doručení
Worker klientovi jen vybere nejbližší nody (§8); anchory si počítají nody. Viz `ARCHITECTURE.md` §6.3.
- **Presence.** Po `subscribe` gateway spočte anchory mailboxu (§7) a každému pošle `/n/presence` s `exp = now + presence_ttl_s` (výchozí 60 s). `subscribe` skončí po potvrzení `quorum` anchorů; zbylé dobíhají. Gateway presence obnovuje každých `presence_renew_s` (výchozí 20 s), při odpojení klienta ji zruší (`exp = 0`). Anchor ji drží jen v paměti, TTL si omezí na vlastní `presence_ttl_s`, takže po výpadku gateway presence sama zanikne. Po změně epochy ji nové anchory dostanou z nejbližšího obnovení.
- **Doručení.** Anchor, který zprávu nově uložil (`stored`), při platné presence zavolá `/n/deliver` na gateway, ta ji pošle klientovi po WSS. Anchor běží na pozadí a odeslání (`send`) tím nezdržuje.
- **Wake.** Když `/n/deliver` vrátí `delivered: false` (a presence se tím zahodí) nebo selže, anchor pošle wake (§13.6), má-li `push_id`.
- **Spolehlivost.** Doručení je *at least once*. Zpráva zůstává `pending`, dokud klient nepošle `ack`, takže ji nelze ztratit, když spojení mezi předáním a přečtením zanikne. Klient proto po `subscribe` provede `fetch` a duplicity zahazuje podle `id` (ratchet je přijme jen jednou).
- **Soukromí.** Anchor ví, na kterém nodu je mailbox online, ale `mailbox_id` je blinded; gateway nezná odesílatele.

## 14. Zabezpečený kanál

Cloudflare Tunnel ukončuje TLS na okraji, takže bez další vrstvy by Cloudflare viděl `mailbox_id`, tickety, `push_id` a strukturu provozu. Klient s nodem a nody mezi sebou proto vedou vlastní autentizovaný šifrovaný kanál. Cloudflare pak vidí IP adresy, čas a (zaokrouhlenou) velikost rámců, obsah ne. Žádná operace není dostupná mimo kanál a žádný režim nedegraduje na nešifrovaný.

Primitiva: X25519 (RFC 7748, výstup samých nul se odmítá), HKDF-SHA256 (RFC 5869), SHA-256, Ed25519 (§1.3), ChaCha20-Poly1305 (RFC 8439) mezi klientem a nodem a XChaCha20-Poly1305 (draft-irtf-cfrg-xchacha) mezi nody. Referenční implementace a vektory: `reference/channel.py`, `test-vectors/channel.json`.

**Padding.** Každá otevřená data se před šifrováním zarovnají: `u32be(len) ‖ data ‖ 0x00…` na násobek 256 B (nejméně 256). Příjemce přijme jen kanonický tvar (správná délka, samé nuly).

### 14.1 Klient ↔ node
Klient zná z trasy `node_id` a ticket, a z konfigurace připnutý klíč Workeru (§8.1 bod 1). Identita nodu se ověřuje takto:
1. Node → `hello {node_id, challenge}` (text, §8.1).
2. Klient → `{"t":"init","ce":"<b64u X25519 efemerní veřejný klíč>"}` (text).
3. Node → `{"t":"accept","ne":"<b64u X25519>","npk":"<b64u Ed25519 identita nodu>","sig":"<b64u>"}` (text), kde

```
th  = SHA256( "ancord/v1/chan" ‖ 0x00 ‖ challenge(32) ‖ ce(32) ‖ ne(32) ‖ npk(32) ‖ node_id(UTF-8) )
sig = Ed25519(npk, domain "ancord/v1/chan-node", payload = th)
```
Klient odmítne spojení, pokud `node_id ≠ base32(blake3(npk))` (§2; `node_id` pochází z ticketu podepsaného Workerem) nebo `sig` nesedí. Cloudflare tedy nemůže uprostřed podvrhnout vlastní klíč: nezná soukromý klíč nodu a nemůže podvrhnout ani `node_id`, protože ten je v ticketu podepsaném Workerem.

4. Klíče: `shared = X25519(ce, ne)`, `okm = HKDF-SHA256(ikm = shared, salt = th, info = "ancord/v1/chan/keys", L = 64)`, `k_c2n = okm[0..32]`, `k_n2c = okm[32..64]`.
5. Od teď jsou všechny rámce WebSocket **binární** (textový rámec spojení zavře). Rámec `n` v daném směru je `ChaCha20-Poly1305(key, nonce = 0x00000000 ‖ u64be(n), aad = "ancord/v1/chan/frame" ‖ th, pad(JSON))`. Čítač `n` začíná 0 v každém směru, příjemce vyžaduje přesně další hodnotu (WebSocket je uspořádaný), takže přehrání, vynechání ani prohození rámce neprojde. Při 2^32 rámcích se spojení zavře a naváže se nové. Nonce se tak nikdy neopakuje a každý směr má vlastní klíč.
6. Klient pošle první rámec `auth` (§8.1 bod 3).

Forward secrecy: oba efemerní klíče se po výpočtu zahodí, kompromitace identity nodu ani Workeru neodhalí starý provoz.

### 14.2 Node ↔ node
Oba nody jsou v adresáři (podepsaném Workerem), takže každý zná `pubkey` druhého. Session má jednu dvojici klíčů na směr a žije `session_ttl_s = 600` (volající ji obnoví 30 s předem).

**Handshake `POST /n/hs`** – podepsaný request podle §3 (podepisuje volající, tj. iniciátor; nonce `n` je 16 B), tělo `{"ce":"<b64u X25519>"}`. Responder odmítne `401` (podpis), `403` (volající není v adresáři), `409` (replay), `400` (nesmysl, bod malého řádu), `503` (příliš mnoho session). Odpověď `200`:

```
{"ne":"<b64u X25519>","sid":"<b64u 16 B náhodných>","sig":"<b64u>"}
th  = SHA256( "ancord/v1/n2n" ‖ 0x00 ‖ ipk(32) ‖ rpk(32) ‖ n(16) ‖ ce(32) ‖ ne(32) ‖ sid(16) )
sig = Ed25519(rpk, domain "ancord/v1/n2n-resp", payload = th)
```
`ipk` je veřejný klíč iniciátora, `rpk` responderu (z adresáře), `n` nonce požadavku. Iniciátor ověří `sig` klíčem `rpk` **z adresáře**; node, který tam není (nebo Cloudflare), odpověď nepodepíše. Klíče: `okm = HKDF-SHA256(ikm = X25519(ce, ne), salt = th, info = "ancord/v1/n2n/keys", L = 64)`, `k_i2r = okm[0..32]`, `k_r2i = okm[32..64]`. Iniciátora prokazuje podpis requestu, který kryje `ce`; klíče odvodí jen držitel soukromého `ce`. Responder drží nejvýše 4096 session, z toho 8 na volajícího (nejstarší se zahazuje).

**Operace `POST /n/x`** – tělo `{"sid":"<b64u>","n":"<b64u 24 B náhodných>","c":"<b64u>"}`:

```
c = XChaCha20-Poly1305(k_i2r, nonce = n, aad = "ancord/v1/n2n/req" ‖ sid, pad({"op":"put|get|ack|presence|deliver","ts":<t>,"body":{…}}))
```
Responder zahodí požadavek, pokud session neexistuje nebo vypršela (`401 unknown_session`, volající ji naváže znovu a zopakuje), šifrový text nejde ověřit (`401`), volající už není v adresáři (`403`), `|now − ts| > 60 s` nebo se dvojice `(sid, n)` už objevila v okně 2 × 60 s (replay, `409`). Odpověď `200`:

```
{"n":"<b64u 24 B>","c":"<b64u>"}
c = XChaCha20-Poly1305(k_r2i, nonce = n', aad = "ancord/v1/n2n/res" ‖ sid ‖ n, pad({"status":<HTTP stav>,"body":{…}}))
```
Odpověď je vázaná na `n` požadavku, takže ji nelze přiřadit k jinému požadavku. Stavy operací (§13.4) jsou uvnitř; `200` vnějšího HTTP znamená jen „session platí“. Nonce jsou náhodné 192bitové, takže session unese libovolný počet požadavků bez rizika kolize.

### 14.3 Co kanál nekryje
- IP adresy, čas a hrubá velikost rámců (padding 256 B) jsou Cloudflare vidět; odesílatel a příjemce nejsou před nodem skryti kromě blinded `mailbox_id` (`THREAT_MODEL.md`).
- Spojení nody ↔ Worker (heartbeat, `/wake`, `/route`) vede přes infrastrukturu Cloudflare z podstaty: Worker je Cloudflare. Vidí `push_id` a u `/wake` i `mailbox_id`; obsah zpráv nikdy.
- Klient musí mít klíč Workeru připnutý mimo síť (`WORKER_PUBKEY`). Nody ho smějí poprvé stáhnout jen s výslovným `TRUST_WORKER_KEY_ON_FIRST_USE` (vývoj).

## 15. Uživatelská identita, profil a funkce konverzace

Funkční parita s prototypem `melo`: uživatel má veřejný klíč, najde protistranu podle něj, napíše jí bez předchozí výměny tajemství a vidí psaní a stav doručení. Node zůstává slepý: zná jen veřejný profil (to je smysl adresy) a blinded `mailbox_id`.

### 15.1 Identita uživatele
- `uk`: Ed25519 identitní klíč uživatele. **Adresa uživatele** je `pk = b64u(uk_pub)` (32 B).
- `bk`: dlouhodobý X25519 klíč pro dohodu tajemství (identita pro DH).
- `spk`: krátkodobý X25519 **podepsaný prekey**. Uživatel ho rotuje každých `SPK_ROTATE_S` = 7 dní. Tajný klíč starého prekey uchová `SPK_KEEP_S` = 21 dní (profil smí být 14 dní starý) a pak ho **nenávratně smaže**.
- `otks`: až 32 **jednorázových** X25519 prekeys. Tajný klíč se smaže hned, jak ho použije první zpracovaná obálka.
- `pushk`: trvalý náhodný Ed25519 **push klíč schránky** (§15.11). Jeho `push_id` je v profilu.
- Klíče uživatele nemají nic společného s `ek` (efemerní klíč klienta pro ticket, §8) ani s klíči nodů.

### 15.2 Profil
Profil je `{"pk","bk","spk","otks":[…],"push_id","ts","pow","sig"}` (klíče a `sig` v b64u, `push_id` podle §10.1):
- `payload = bk ‖ spk ‖ n(1 B) ‖ otks (n × 32 B) ‖ ts_be_u64 ‖ push_id (52 B ASCII)`,
- `sig = Ed25519(uk, "ancord/v1/profile" ‖ 0x00 ‖ payload)`: pod podpisem je vše, co klient z profilu použije,
- `pow`: hashcash (§4.2) s `nonce = sha256("ancord/v1/profile-pow" ‖ 0x00 ‖ payload)`, `pubkey = pk` a `bits = profile_pow_bits` (výchozí 10); chrání anchory před zaplavením profily,
- `ts` je čas vytvoření a musí být v rozmezí ±86 400 s od času nodu, `n ≤ 32`, `push_id` má tvar podle §10.1 (jinak `bad_request`).

Anchor drží vždy profil s nejvyšším `ts`; starší nebo shodný `ts` nic nemění. Záznam žije `profile_ttl_s` (výchozí 90 dní) od posledního přijetí. Uživatel profil znovu zveřejňuje po rotaci `spk`, po použití jednorázových prekeys a nejpozději jednou za 7 dní. Profil je veřejný z podstaty (kdo zná `pk`, ho získá), proto se ukládá v čitelné podobě.

`profile_id = sha256("ancord/v1/profile-id" ‖ 0x00 ‖ pk)` (32 B). Anchory profilu se počítají podle §7 s `profile_id` místo `mailbox_id`.

**Co musí klient ověřit.** Podpis profilu klíčem `pk`, shodu `pk` s adresou, kterou chtěl, a čerstvost: pro novou konverzaci odmítne profil starší než `PROFILE_MAX_AGE_S` = 14 dní, aby mu škodlivý anchor nemohl podstrčit starý prekey, jehož tajný klíč už unikl. Anchor profil podvrhnout nemůže, jen zadržet nebo vrátit starší. Že adresa patří opravdu dané osobě, se ověřuje mimo síť bezpečnostním číslem (§15.4).

### 15.3 Zprávy klient → node
| Požadavek | Úspěch | Význam |
|---|---|---|
| `{"t":"profile_put","pk","bk","spk","otks","push_id","ts","pow","sig"}` | `{"t":"profile_stored","acks":n}` | ověří podpis a `pow`, zapíše na `r` anchorů, hotovo po `quorum` potvrzeních |
| `{"t":"profile_get","pk"}` | `{"t":"profile","pk","bk","spk","otks","push_id","ts","pow","sig"}` | přečte `r − quorum + 1` anchorů, vrátí profil s nejvyšším `ts`; `not_found`, když ho nemá žádný |

Nové chybové kódy: `not_found`, `invalid_proof` (podpis nebo `pow` profilu neplatí).

### 15.4 Zahájení konverzace (X3DH) a obálka
Iniciátor (Alice) zná ověřený čerstvý profil příjemce (Bob). Alice vygeneruje efemerní X25519 klíč `ek`, z `otks` Bobova profilu vybere náhodně jeden `otk` (nejsou-li, vynechá se) a spočítá:
```
dh1 = X25519(bk_A, spk_B)    dh2 = X25519(ek, bk_B)    dh3 = X25519(ek, spk_B)    dh4 = X25519(ek, otk)   // jen s otk
ss  = HKDF(salt = "ancord/v1/x3dh", ikm = 0xFF×32 ‖ dh1 ‖ dh2 ‖ dh3 [‖ dh4],
           info = pk_A ‖ pk_B ‖ ek_pub ‖ spk_B ‖ [otk], L = 32)
```
Samé nuly z libovolného DH se odmítnou. Dál platí §13.1 a §13.2: `ss` určuje oba mailboxy a počáteční kořenový klíč ratchetu. První ratchet klíč responderu `R` (§13.2) je jeho podepsaný prekey `spk_B`, který Alice zná z profilu; Bob jeho tajný klíč zkopíruje do stavu ratchetu (z klíčů uživatele ho maže rotace podle §15.1) a v ratchetu ho nahradí při prvním DH kroku, tedy po přijetí první zprávy Alice. **Iniciátor** je ten, kdo poslal obálku; Alice smí začít psát ihned, zprávy čekají v mailboxu. Bob může psát až po přijetí její první zprávy (§13.2).

**Schránka (inbox).** `inbox_id = sha256("ancord/v1/inbox" ‖ 0x00 ‖ pk)`. Je to běžný mailbox (§13), takže pro node vypadá jako kterýkoli jiný. Alice do ní pošle obálku obyčejným `send` s `push_id` z Bobova profilu, takže offline Boba vzbudí push (§13.6).

**Obálka** (hodnota `ct`): `0x01 ‖ spk_id(8 B) ‖ ek_pub(32 B) ‖ AEAD`, kde `spk_id = sha256(spk_B)[0..8]`:
```
okm = HKDF(salt = "ancord/v1/envelope", ikm = X25519(ek, spk_B), info = inbox_id ‖ ek_pub ‖ spk_id, L = 44)
AEAD = ChaCha20-Poly1305(key = okm[0..32], nonce = okm[32..44], aad = "ancord/v1/envelope" ‖ inbox_id ‖ ek_pub ‖ spk_id, plaintext = pk_A ‖ otk(32 B, nuly = žádný))
```
Anchor schránky vidí jen náhodný `ek_pub` a ciphertext; **adresu odesílatele ani jeho použitý `otk` nevidí**.

**Příjemce** zpracuje obálku takto: najde `spk` podle `spk_id`, dešifruje, získá `pk_A` a `otk`, z anchorů si stáhne a ověří Alicin profil (§15.2), spočítá stejné `dh1..dh4` s opačnými rolemi, uloží nový stav klíčů **a teprve potom** obálku potvrdí (`ack`). Použitý `otk` se při uložení smaže. Poškozené nebo cizí obálky se potvrdí a zahodí (uvolní schránku); obálka, jejíž odesílatelův profil se nepodařilo najít, zůstane na později. Replay obálky je neškodný: `otk` už neexistuje, a obálka bez `otk` vytvoří stejné `ss`, které už příjemce zná (klient existující konverzaci nepřepíše).

**Bezpečnostní číslo.** `sha256("ancord/v1/safety" ‖ 0x00 ‖ min(pk_A, pk_B) ‖ max(pk_A, pk_B))`, prvních 24 B jako 12 skupin po 2 B, každá `mod 100000` na 5 číslic. Obě strany musí vidět totéž; porovnává se mimo síť (osobně, telefonem). Bez toho by škodlivý anchor mohl při první komunikaci podstrčit cizí adresu.

**Forward secrecy.**
- Únik dlouhodobých klíčů (`uk`, `bk`) neprozradí zprávy, jejichž `spk` už příjemce smazal (po `SPK_KEEP_S`) ani jejichž `otk` už byl použit.
- Zprávy navázané s `otk` jsou chráněny i proti úniku `spk` po použití `otk`.
- Zprávy navázané bez `otk` (příjemci došly) jsou ohroženy únikem `spk` po dobu jeho života (nejvýše 21 dní), proto uživatel profil doplňuje.
- Uvnitř konverzace dává forward secrecy jednosměrný řetěz klíčů a post-compromise security DH krok Double Ratchetu (§13.2).
- **Meze:** Alicino první řetězec (zprávy před první odpovědí Boba) závisí na `spk_B` stejně jako X3DH; konverzace, v níž píše jen jedna strana, se neuzdraví; stav ratchetu je nutné chránit před zálohou a klonováním (znovupoužití `n`).

### 15.5 Psaní (typing)
`{"t":"typing","mailbox_id","on":bool}` → `{"t":"typing_ok"}`. Odesílatel signalizuje do mailboxu, do kterého píše (§13.1). Gateway pošle anchorům `/n/signal`; anchor s platnou presence (§13.7) předá signál `/n/typing` gateway příjemce, ta ho odběrateli pošle jako `{"t":"typing","mailbox_id","on"}`. Signál je nespolehlivý a nikdy se neukládá. Stejný stav se opakovaně nepřeposílá dřív než za `typing_min_interval_s` (výchozí 2 s). Obsah je blinded jako zbytek; příjemce nemá od nodu důkaz odesílatele (stejně jako u zpráv, ověřuje ho až ratchet).

### 15.6 Stav doručení
`{"t":"status","mailbox_id","ids":[…]}` (nejvýše 500) → `{"t":"status","mailbox_id","states":["pending"|"delivered"|"unknown",…]}` ve stejném pořadí jako `ids`. Čte `r − quorum + 1` anchorů; `delivered` vyhrává nad `pending`, `pending` nad `unknown`. `delivered` znamená, že příjemce zprávu potvrdil (`ack`); stav `unknown` mají zprávy po vypršení nebo ty, které nikdy nedorazily.

### 15.7 Historie
Server historii nedrží: po `ack` zbývá jen náhrobek a ratchet klíč zprávy hned zahodí, takže starý ciphertext už nikdo nedešifruje. Historie je proto lokální u klienta (CLI ji ukládá do souboru konverzace s právy 0600). To je záměr, ne chybějící funkce: delší držení ciphertextu na anchorech by prodlužovalo, co operátor vidí, a nic by nepřineslo.

### 15.8 Dávkové odběry
`subscribe` i `unsubscribe` smí místo `mailbox_id` nést `mailbox_ids` (pole, v součtu s existujícími odběry nejvýše 32). Odpověď má `mailbox_ids` a `anchors` jako pole potvrzených anchorů ve stejném pořadí; chyba kteréhokoli mailboxu zruší celý požadavek.

### 15.9 Node ↔ node
Přes zabezpečený kanál (§14.2), všechny podepsané jako `put`/`get` (§13.4):
| Operace | Tělo | Odpověď | Kdo smí volat |
|---|---|---|---|
| `profile_put` | profil (§15.2) | `{"status":"stored"\|"duplicate"}` | člen adresáře; příjemce musí být anchorem `profile_id` |
| `profile_get` | `{"pk"}` | `{"profile":{…}\|null}` (`null`, když ho anchor nemá) | člen adresáře; příjemce musí být anchorem `profile_id` |
| `signal` | `{"mailbox_id","on"}` | `{"ok":true}` | člen adresáře; příjemce musí být anchorem `mailbox_id`; má-li presence, předá `typing` |
| `typing` | `{"mailbox_id","on"}` | `{"delivered":bool}` | anchor mailboxu (jako `deliver`) |
| `status` | `{"mailbox_id","ids":[…]}` | `{"states":[…]}` | člen adresáře; příjemce musí být anchorem `mailbox_id` |

Příjemce `profile_put` profil znovu ověří (podpis, `pow`, `ts`); volající mu nevěří. Při handoffu (§13.5) se profily přesouvají jako zprávy, anchory podle `profile_id`.

### 15.10 Adresa ke sdílení
Adresa se předává mimo síť: QR kódem nebo zkopírováním textu (zpráva, e-mail, cokoli). Textový tvar je URI `ancord:<pk>` (`pk` je b64u, 43 znaků); QR kód obsahuje přesně tento řetězec. Klient přijme i holé `pk` bez předpony. Před použitím ověří, že `pk` je platný bod Ed25519. Překlep v adrese tak většinou skončí chybou, a pokud ne, profil pro takovou adresu neexistuje (`not_found`). Že adresa patří správné osobě, potvrdí bezpečnostní číslo (§15.4).

### 15.11 Push klíče klienta
Aby anchor mohl offline příjemce vzbudit (§13.6), musí odesílatel znát `push_id` mailboxu, do kterého píše, a příjemce pod ním mít u Workeru zaregistrovaný token (§10.1).
- **Schránka.** `push_id` schránky je `push_id` klíče `pushk` (§15.1) a je podepsaný v profilu. Zná ho každý, kdo zná adresu, stejně jako schránku samotnou. Registrovat (a tím i mazat) token pod ním může jen držitel `pushk`.
- **Mailbox konverzace.** `push_seed = HKDF(salt = "ancord/v1/push", ikm = ss, info = mailbox_id, L = 32)` je Ed25519 seed, z něj `push_pubkey` a `push_id` (§10.1). Příjemce registruje token pod `push_id` svého příchozího mailboxu, odesílatel k `send` přikládá `push_id` mailboxu, do kterého píše. Tento `push_id` zná jen dvojice, takže nikdo jiný nemůže příjemce budit zprávami do konverzace. Protistrana by mohla registraci přepsat, tím ale ovlivní jen push z vlastní konverzace.
- Push klíče nemají nic společného s identitou (`uk`) ani s klíči ratchetu. Worker vidí, že několik `push_id` má stejný token (stejné zařízení); počet konverzací uživatele tak před ním skrytý není (`THREAT_MODEL.md`).

### 15.12 Online klient
Doporučený průběh klienta, který je připojený (CLI `ancord online`):
1. `GET /route` (§8): Worker vrátí 3 nody a tickety. Klient se připojí na první, který ho přijme (§8.1, §14.1). Worker se dál doručování neúčastní.
2. Je-li potřeba (rotace `spk`, použité `otks`, nejpozději po 7 dnech), klient zveřejní profil (§15.2). CLI ho zveřejní při každém připojení.
3. `subscribe` schránky a příchozích mailboxů všech konverzací (§15.8, nejvýše 32 na spojení). Gateway spočítá anchory a oznámí jim presence. Pak `fetch` toho, co čekalo.
4. Nová zpráva ve schránce: klient obálku zpracuje (§15.4), uloží klíče a konverzaci, potvrdí obálku, přihlásí odběr nového mailboxu a znovu zveřejní profil (spotřebovaný `otk`).
5. Zprávy konverzací anchor přeposílá přes gateway (§13.7). Klient je dešifruje, uloží stav ratchetu a teprve pak potvrdí (`ack`).
6. Po odpojení gateway presence zruší, nebo sama vyprší po `presence_ttl_s`. Další zprávy zůstávají na anchorech a anchor pošle wake (§13.6). Push probudí aplikaci, ta se připojí znovu (krok 1) a zprávy vyzvedne.
7. Při výpadku gateway se klient připojí znovu s novým `/route`. Presence na starém nodu vyprší sama.

Token registruje klient přímo u Workeru (§10.1) pro schránku a pro příchozí mailbox každé konverzace, tedy i nově založené.
