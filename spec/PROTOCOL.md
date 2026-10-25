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
| `ancord/v1/client-auth` | důkaz držení efemerního klíče klienta (admission na nodu) |

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
- `hostname = "node" ‖ node_id[0..16] ‖ "." ‖ zone` (prvních 16 znaků = 10 bajtů hashe). Příklad: `nodeqx7c2a5k3m4ztdwp.melonet.cz`. Dvě různé `pubkey` se shodným prefixem jsou kolize, registrace druhého nodu se odmítne (`409`).

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

Klient se připojí na `wss://<hostname>/client` (WebSocket, textové JSON zprávy, max. 64 KiB). Průběh:

1. Node hned po upgradu pošle `{"t":"hello","node_id":"<node_id>","challenge":"<b64u 32 B>"}`; `challenge` je náhodná a platí jen pro toto spojení.
2. Klient do `auth_timeout_s` (výchozí 10 s) odpoví `{"t":"auth","ticket":"<ticket>","sig":"<b64u>"}`, kde `sig` je Ed25519 klíčem `ek` přes doménu `ancord/v1/client-auth` a `payload = challenge(32 B) ‖ node_id(UTF-8)`. Ticket sám bez `ek` tedy nestačí, ukradený ticket je bez soukromého klíče k ničemu.
3. Node ověří ticket (§8: podpis Workeru, `nid`, `exp`), podpis proti `ek` z ticketu a že `n` nebyl použit (jednorázové do `exp`). Při úspěchu odpoví `{"t":"ready"}`, jinak `{"t":"error","reason":"…"}` a zavře spojení kódem `4401`. Bez `auth` do limitu se zavře kódem `4408`.
4. Po `ready` node odpovídá na `{"t":"ping"}` zprávou `{"t":"pong"}`. Zprávy data plane (`send`, `fetch`, `ack`) jsou v §13.

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
Klient odvodí z mailbox tajemství push klíč (Ed25519) a `push_id = base32(blake3(push_pubkey))`.

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
`test-vectors/*.json` obsahují deterministické vstupy a očekávané výstupy pro: `node_id` a hostname, kanonické JSON, podpis registrace a důkazu, hashcash, podepsaný request, heartbeat, adresář, ticket, HRW, hash-chain, Merkle/space, push registraci, mailbox a ratchet (§13). Obě implementace musí projít všechny.

## 13. Data plane

Node je slepá schránka: ukládá jen ciphertext pod `mailbox_id` a nezná odesílatele ani příjemce. Šifrování je čistě mezi klienty, node ratchet nikdy nepočítá. Dohoda sdíleného tajemství mezi klienty (párování) je mimo rozsah.

### 13.1 Mailbox a zpráva
- Klienti sdílejí tajemství `ss` (32 B). Každý směr má vlastní mailbox, takže do každého píše jediný odesílatel: `dir ∈ {"i2r","r2i"}` (initiator → responder, opačně).
- `mailbox_id(dir) = sha256("ancord/v1/mailbox" ‖ 0x00 ‖ ss ‖ 0x00 ‖ dir)` (32 B, v JSON b64u).
- Zpráva: `id` (16 B náhodných, generuje klient, b64u), `ts` (sekundy klienta, jen k řazení; musí být v rozmezí ±86 400 s od času nodu), `ct` (b64u, po dekódování nejvýše 32 KiB). `(mailbox_id, id)` identifikuje zprávu; opakovaný zápis téhož `id` nic nemění (idempotence).

### 13.2 Ratchet (klient ↔ klient)
Jednosměrný symetrický řetěz na každý mailbox. HKDF je HKDF-SHA256 (RFC 5869), AEAD ChaCha20-Poly1305 (RFC 8439).

```
ck_0     = HKDF(ikm = ss, salt = "ancord/v1/ratchet", info = "chain" ‖ 0x00 ‖ dir, L = 32)
mk_n     = HKDF(ikm = ck_n, salt = (prázdná), info = "ancord/v1/ratchet/msg", L = 44)   key = mk_n[0..32], nonce = mk_n[32..44]
ck_{n+1} = HKDF(ikm = ck_n, salt = (prázdná), info = "ancord/v1/ratchet/chain", L = 32)
```
Prázdná sůl znamená `HashLen` nul (RFC 5869 §2.2). Zpráva s pořadovým číslem `n` (u32, od 0):

```
aad = "ancord/v1/msg" ‖ 0x00 ‖ mailbox_id(32 B) ‖ id(16 B) ‖ n_be_u32
ct  = b64u( n_be_u32 ‖ ChaCha20-Poly1305(key, nonce, aad, plaintext) )      // šifrový text vč. 16 B tagu
```
Klíč i nonce jsou pro každé `n` jiné, takže se nonce nikdy neopakuje s týmž klíčem. Po použití se `ck_n` zahodí (forward secrecy v rámci řetězu). Příjemce drží `(ck, next_n, skipped)`: zpráva s `n = next_n` posune řetěz, `n > next_n` uloží klíče přeskočených zpráv (nejvýše `max_skip = 200` najednou, větší mezera se odmítne jako desync), `n < next_n` použije a smaže uložený přeskočený klíč, jinak se odmítne. Stav se mění až po úspěšném ověření tagu.

### 13.3 Zprávy klient → node
Po `ready` (§8.1). Každý požadavek může nést `rid` (řetězec do 64 znaků), který odpověď zopakuje. Chyba má tvar `{"t":"error","rid":…,"code":"<kód>","reason":"<text>"}`; spojení po ní zůstává.

| Požadavek | Úspěch | Význam |
|---|---|---|
| `{"t":"send","id","mailbox_id","ts","ct","push_id"?}` | `{"t":"sent","id","acks":n}` | zápis na `r` anchorů, hotovo po `quorum` potvrzeních; volitelné `push_id` příjemce (§10.1) spustí wake (§13.6) |
| `{"t":"fetch","mailbox_id","limit"}` | `{"t":"messages","mailbox_id","messages":[{"id","ts","ct"}]}` | nedoručené zprávy seřazené podle `(ts, id)` (`ts` má sekundovou přesnost, přesné pořadí určuje číslo `n` v ratchetu, podle něj zprávy řadí klient); `limit` 1–500 (výchozí 100) |
| `{"t":"ack","mailbox_id","ids":[…]}` | `{"t":"acked","n":k}` | zprávy (nejvýše 500) jsou doručené a anchory je smažou; `n` je počet `ids` v požadavku |

Kódy chyb: `bad_request`, `too_large`, `rate_limited` (limit zpráv na spojení), `no_directory` (node ještě nemá adresář), `insufficient_nodes` (adresář má méně nodů než `quorum`), `quorum_unavailable` (anchory neodpověděly v dostatečném počtu), `mailbox_full`.

- **Anchory** zpracuje node z adresáře své aktuální epochy (§7) s `r` a `quorum` z `params`. Pokud je sám anchorem, zapisuje do vlastního úložiště, jinak volá ostatní (§13.4). Odesílá všem `r` anchorům současně a odpoví, jakmile má `quorum` potvrzení; zbylé zápisy dobíhají na pozadí.
- **`fetch`** se dotáže všech anchorů a skončí, jakmile odpoví aspoň `r − quorum + 1` z nich (takové čtení se vždy protne s jakýmkoli zápisem na kvorum). Výsledek je sjednocení podle `id`. Méně odpovědí je `quorum_unavailable`.
- **`ack`** jde na všechny anchory a uspěje při `quorum` potvrzeních. Je idempotentní.

### 13.4 Node ↔ node
`POST /n/put`, `/n/get`, `/n/ack` na `https://<hostname>` cílového nodu, tělo JSON (nejvýše 128 KiB), podepsaný request podle §3. Příjemce odmítne:
- `401` neplatný podpis, `409` replay (nonce okno jako §3),
- `403` volající není v adresáři aktuální ani předchozí epochy, nebo příjemce není anchorem `mailbox_id` v aktuální ani předchozí epoše,
- `413` příliš velké tělo, `400` neplatné tělo.

O jednu epochu napřed bývá odesílatel jen na několik vteřin, příjemce adresář ještě neobdržel a `403` je dočasné. Odesílatel proto `403` zopakuje dvakrát s prodlevou 400 ms (s novým nonce), než ho vyhodnotí jako chybu.

| Cesta | Tělo | Odpověď `200` |
|---|---|---|
| `/n/put` | `{"mailbox_id","id","ts","exp","ct","push_id"?}` nebo `{"mailbox_id","id","ts","exp","delivered":true}` (náhrobek, bez `ct`) | `{"status":"stored"\|"duplicate"}` |
| `/n/get` | `{"mailbox_id","limit"}` | `{"messages":[{"id","ts","exp","ct"}]}` |
| `/n/ack` | `{"mailbox_id","ids":[…]}` | `{"n":k}` (počet změněných záznamů) |

`put` s existujícím `(mailbox_id, id)` vrací `duplicate` a nemění nic, jen `delivered: true` převede pending záznam na doručený (delivered vyhrává vždy). `ack` vyrobí z neexistující zprávy záznam „doručeno“, takže pozdní `put` zprávu neoživí. Plný mailbox odpoví `507 mailbox_full`.

### 13.5 Úložiště, TTL, handoff
- Záznam je `pending` (nese `ct`) nebo `delivered` (jen náhrobek bez `ct`). Oba mají `exp`; po `exp` je smaže úklid. Příjemce si `exp` omezí na `now + message_ttl_s` (výchozí 30 dní), takže handoff ani opakovaný zápis životnost neprodlužují.
- Strop nedoručených zpráv v jednom mailboxu na nodu je `max_mailbox_messages` (výchozí 10 000).
- **Handoff**: po přijetí adresáře nové epochy projde node svá uložená data. Pro každý mailbox spočte anchory nové epochy a každý záznam (i náhrobek) pošle `put` všem z nich kromě sebe. Pokud sám už anchorem není a záznam přijalo aspoň `quorum` nových anchorů, smaže ho. Neúspěšné pokusy se opakují při dalším průchodu údržby, dokud se neuspěje; po dobu, co by nové anchory ještě neznaly novou epochu, je příjemce přijme díky pravidlu o předchozí epoše.
- Pod `quorum` nodů v adresáři se zápisy odmítají (`insufficient_nodes`), jiný degradovaný režim není.

### 13.6 Wake (push)
- `push_id` je 52 znaků `[a-z2-7]` (§10.1). Odesílatel ho zná, protože ho odvodí ze sdíleného tajemství mailboxu; koordinující node ho přidá do `/n/put` všem anchorům. Neplatný tvar se odmítne (`bad_request`). `push_id` se neukládá.
- Anchor, který zprávu **nově uložil** (`stored`, ne `duplicate`, ne náhrobek) a dostal `push_id`, na pozadí pošle Workeru podepsaný `POST /wake` (§10.2) s `{push_id, mailbox_id}`. Odpověď Workeru se na zápis zprávy nijak nepromítá: selhání wake nikdy neshodí `send`.
- Anchor koaleskuje i sám: pro stejný `push_id` pošle nejvýše jeden wake za `wake_min_interval_s` (výchozí 10 s). Worker koaleskuje znovu (per `push_id`) a limituje per node, takže až `r` souběžných wake od anchorů téže zprávy skončí nejvýš jednou jako push.
- Průběžné doručování online odběrateli (a tedy vynechání wake, když je odběratel online) tu zatím není: klient čte `fetch`. Wake se proto posílá vždy.
