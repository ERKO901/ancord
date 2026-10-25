# ancord – model hrozeb

Doplněk k `ARCHITECTURE.md` §8. Popisuje, co Worker (control plane) chrání, před kým a kde končí záruky. Stav odpovídá implementaci ve `worker/` (sekce 1 až 5) a `node/` (sekce 6).

## 1. Aktiva a hranice důvěry

| Aktivum | Kdo ho drží | Kdo ho nesmí získat |
|---|---|---|
| Obsah zpráv | klienti (E2E) | Worker, operátoři nodů |
| IP uživatele | Cloudflare | operátoři nodů, Worker (jen `request.cf` při `/route`, neukládá se) |
| Privátní klíč Workeru (podpis ticketů a adresáře) | secret `WORKER_SIGNING_KEY` | kdokoli mimo Worker |
| Cloudflare API token Workeru | secret `CF_API_TOKEN` (Tunnel Edit + DNS Edit jen pro zónu) | kdokoli mimo Worker |
| Push tokeny | KV, šifrované AES-GCM (`PUSH_ENC_KEY`, AAD = `push_id`) | nody, útočník s přístupem jen ke KV |
| Tunnel token nodu | node, jednorázově v odpovědi `register/confirm` | ostatní nody |

Worker je **důvěryhodný pro dostupnost a poctivost routingu, nikoli pro obsah**. Kompromitovaný Worker může cenzurovat nebo routovat na škodlivé nody a vydávat falešné adresáře, nemůže číst zprávy.

## 2. Hrozby a obrany (Worker)

| Hrozba | Obrana v implementaci | Zbytkové riziko |
|---|---|---|
| Sybil (mnoho nodů) | hashcash vázaný na nonce a klíč s adaptivní obtížností, limity nodů na ASN a na /24 (/48), probation (váha roste s ověřenou uptime a splněnými challenge), penalizace ASN po banu | odhodlaný útočník s mnoha ASN; ekonomická neatraktivita (málo provozu bez reálných zdrojů) |
| Registrace cizího hostname | podpis `nonce ‖ hostname ‖ pubkey`, jednorázová výzva, důkaz držení klíče na hostname (`melo-proof`) až po startu tunelu | – |
| Zabrání cizího DNS záznamu / smazání cizích zdrojů | Worker píše a maže jen `node<16 znaků>.<zóna>`, před zápisem čte existující záznamy, maže jen záznamy s komentářem `ancord-managed` a tunely s prefixem `ancord-`; kontrola přímo v klientovi, ne jen v logice registrace | token s širším oprávněním, než je nutné (viz ARCHITECTURE §12.3: užší token pro Worker) |
| Lhaní o specs | `min(claimed, measured)`; nezměřená dimenze = nulová kapacita; CPU hash-chain s náhodnými segmenty, Proof of Space s Merkle commitmentem a konzistencí `T[i]` ze `T[i−1]`, `T[j(i)]`, bandwidth přes R2 se serverovým měřením | nelze kryptograficky dokázat „mám 16 GB RAM“; obrana je měření, probation a ekonomika |
| Přeposílání challenge jinam | tvrdé deadliny v ms (po odečtení RTT), memory-hard odvození s datově závislým přístupem, náhodné indexy otevření | node s rychlým spojením na silnější stroj projde; takový stroj ale skutečně poskytuje výkon |
| Replay | podepsané requesty s časovým oknem ±60 s a jednorázovým nonce (DO), `seq` heartbeatu, `ts` push registrace, jednorázové výzvy a tokeny | – |
| Podvržení identity při `/ws` | hlavičky `x-ancord-*` pro Registry sestavuje výhradně Worker po ověření podpisu; Registry není dostupné zvenčí | – |
| Push spam / sledování | wake jen od skutečného anchoru (HRW v aktuální a předchozí epoše), token bucket per node, koalescence per `push_id`, obsah push je pevný (`wake`), tokeny šifrované | anchor smí zvonit uživateli, jehož mailbox drží (zná `push_id`) |
| Přepsání cizího push tokenu | `push_id = blake3(push_pubkey)`, registrace podepsaná push klíčem, starší `ts` se odmítne | – |
| Zahlcení Workeru | `/route` čte jen KV (+ paměť izolátu), DO se neprobouzí; omezení těla požadavků; hashcash a limity chrání registraci | volumetrický DoS řeší Cloudflare edge; **rate limiting a Turnstile pravidla na zóně se nastaví ve fázi C** (WAF pravidla pro `/register*`, `/route`) |
| Výpadek / kompromitace DO | stav v SQLite DO; adresář je podepsaný a neměnný v epoše; klient cachuje adresář | jediná instance DO; multi-signer adresář a shardování jsou otevřené body 6 a 10 |
| Únik tajemství logy | logy nesou jen `node_id` a kódy událostí; tunnel token se nelogguje ani neukládá | – |

## 3. Co Worker vidí
- Při `/route`: `request.cf` klienta (region, souřadnice podle IP geolokace Cloudflare) použité jen k výběru, neukládá se.
- Při `/wake`: `push_id` a `mailbox_id` anchoru, čas. (ARCHITECTURE §5.4 popisuje wake s `mailbox_id`, je to nutné k ověření anchoru.)
- U nodů: veřejný klíč, hostname, ASN a hash IP prefixu (HMAC, nelze zpětně odvodit IP), tvrzené a naměřené zdroje, polohu podle spojení (země, kontinent, colo, souřadnice).

## 4. Flux a hostitelé instancí
Hostitel instance (např. na Flux) může vidět privátní klíč nodu. U vlastních nodů se to akceptuje. U otevřené sítě to není model hrozeb, který by šlo předpokládat: operátor nodu je považován za neprůhledného škodlivého, ale ne za schopného číst klíče jiných operátorů. Perzistence klíče při přesunu instance je otevřený bod 9.

## 5. Mimo rozsah
Mobilní klient, UI, tokenová ekonomika. Ochrana metadat před Cloudflarem (ten vidí IP a časování, ne obsah) je akceptované riziko celého návrhu.

## 6. Data plane (node)

Operátor anchoru vidí: `mailbox_id` (hash sdíleného tajemství, nespojitelný s identitou), velikost a čas zápisu a čtení, IP nejsou vidět (Cloudflare). Obsah vidí jen jako ciphertext.

| Hrozba | Obrana v implementaci | Zbytkové riziko |
|---|---|---|
| Čtení zpráv anchorem | E2E: ChaCha20-Poly1305, klíč na každou zprávu z jednosměrného HKDF řetězu, AAD váže ciphertext na `mailbox_id` a `id` zprávy | operátor vidí metadata (časování, velikost, počet zpráv v mailboxu) |
| Podvržení nebo přehrání zprávy nodem | AEAD tag, stav ratchetu se mění až po ověření tagu, použitý klíč se zahodí; přehraná zpráva (`n < next_n` bez uloženého klíče) se odmítne | node může zprávu zadržet nebo smazat (dostupnost řeší replikace R=3 s kvorem 2) |
| Znovupoužití nonce | nonce i klíč jsou odvozeny z řetězu a unikátní pro každé `n`; klient musí stav uložit **před** odesláním (CLI to dělá) | klient, který stav ratchetu vrátí zpět (záloha, klon zařízení), znovu použije `n` s jiným obsahem; to ratchet sám nezachytí |
| Zahlcení mailboxu | zápis vyžaduje znalost `mailbox_id` (odvozeného z tajemství), strop nedoručených zpráv v mailboxu, limit zpráv na spojení, limit velikosti, ticket je jednorázový a vázaný na efemérní klíč | kdo zná `mailbox_id`, ho může zaplnit až po strop; `mailbox_id` v kódu zná jen dvojice klientů a anchory |
| Falešný anchor nebo podvržený node↔node request | každý request podepsán identitou nodu a ověřen proti adresáři Workeru (aktuální nebo předchozí epocha), nonce proti replayi, příjemce zapisuje a vydává jen mailboxy, pro které je sám anchorem | škodlivý anchor smí číst a mazat mailboxy, jejichž je anchorem (kvorum čtení ho přehlasuje jen částečně, protože čtení sjednocuje) |
| Oživení doručené zprávy | doručení se zapisuje jako náhrobek až do vypršení, `delivered` vyhrává při `put` i handoffu | ack, který nedorazil na žádný z anchorů, zprávu doručí podruhé; klient je odolný díky ratchetu (stejné `n` už nejde otevřít) |
| Ztráta dat při změně epochy | handoff nových anchorů před smazáním (potvrzení aspoň `quorum`), příjemci přijímají zápis i podle předchozí epochy | uzel, který ve chvíli změny epochy padne před dokončením handoffu, nechá data na ostatních dvou kopiích; přežití dvou současných výpadků není garantováno |
