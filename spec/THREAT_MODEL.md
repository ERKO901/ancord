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
| Zabrání cizího DNS záznamu / smazání cizích zdrojů | Worker píše a maže jen `node-<16 znaků>.<zóna>`, před zápisem čte existující záznamy, maže jen záznamy s komentářem `ancord-managed` a tunely s prefixem `ancord-`; kontrola přímo v klientovi, ne jen v logice registrace | token s širším oprávněním, než je nutné (viz ARCHITECTURE §12.3: užší token pro Worker) |
| Lhaní o specs | `min(claimed, measured)`; nezměřená dimenze = nulová kapacita; CPU hash-chain s náhodnými segmenty, Proof of Space s Merkle commitmentem a konzistencí `T[i]` ze `T[i−1]`, `T[j(i)]`, bandwidth přes R2 se serverovým měřením | nelze kryptograficky dokázat „mám 16 GB RAM“; obrana je měření, probation a ekonomika |
| Přeposílání challenge jinam | tvrdé deadliny v ms (po odečtení RTT), memory-hard odvození s datově závislým přístupem, náhodné indexy otevření | node s rychlým spojením na silnější stroj projde; takový stroj ale skutečně poskytuje výkon |
| Replay | podepsané requesty s časovým oknem ±60 s a jednorázovým nonce (DO), `seq` heartbeatu, `ts` push registrace, jednorázové výzvy a tokeny | – |
| Podvržení identity při `/ws` | hlavičky `x-ancord-*` pro Registry sestavuje výhradně Worker po ověření podpisu; Registry není dostupné zvenčí | – |
| Push spam / sledování | wake jen od skutečného anchoru (HRW v aktuální a předchozí epoše), token bucket per node, koalescence per `push_id`, obsah push je pevný (`wake`), tokeny šifrované | anchor smí zvonit uživateli, jehož mailbox drží (zná `push_id`). `push_id` schránky je veřejný (v profilu, §15.11): kdokoli může uživatele budit obálkami do schránky, nejvýše jednou za koalescenční interval Workeru. `push_id` mailboxu konverzace zná jen dvojice |
| Přepsání cizího push tokenu | `push_id = blake3(push_pubkey)`, registrace podepsaná push klíčem, starší `ts` se odmítne | – |
| Zahlcení Workeru | `/route` čte jen KV (+ paměť izolátu), DO se neprobouzí; omezení těla požadavků; hashcash a limity chrání registraci | volumetrický DoS řeší Cloudflare edge; **rate limiting a Turnstile pravidla na zóně se nastaví ve fázi C** (WAF pravidla pro `/register*`, `/route`) |
| Výpadek / kompromitace DO | stav v SQLite DO; adresář je podepsaný a neměnný v epoše; klient cachuje adresář | jediná instance DO; multi-signer adresář a shardování jsou otevřené body 6 a 10 |
| Únik tajemství logy | logy nesou jen `node_id` a kódy událostí; tunnel token se nelogguje ani neukládá | – |

## 3. Co Worker vidí
- Při `/route`: `request.cf` klienta (region, souřadnice podle IP geolokace Cloudflare) použité jen k výběru, neukládá se.
- Při `/wake`: `push_id` a `mailbox_id` anchoru, čas. (ARCHITECTURE §5.4 popisuje wake s `mailbox_id`, je to nutné k ověření anchoru.)
- Při registraci push: IP klienta, `push_id` a token. Klient registruje stejný token pod `push_id` schránky i všech svých konverzací, takže Worker může tyto `push_id` spojit k jednomu zařízení a přes veřejný profil i k adrese (`pk`). Zná tedy počet konverzací uživatele a časy, kdy ho která budí; s kým mluví, nezná (`mailbox_id` je blinded).
- U nodů: veřejný klíč, hostname, ASN a hash IP prefixu (HMAC, nelze zpětně odvodit IP), tvrzené a naměřené zdroje, polohu podle spojení (země, kontinent, colo, souřadnice).

## 4. Flux a hostitelé instancí
Hostitel instance (např. na Flux) může vidět privátní klíč nodu. U vlastních nodů se to akceptuje. U otevřené sítě to není model hrozeb, který by šlo předpokládat: operátor nodu je považován za neprůhledného škodlivého, ale ne za schopného číst klíče jiných operátorů. Perzistence klíče při přesunu instance je otevřený bod 9.

## 5. Mimo rozsah
Mobilní klient, UI, tokenová ekonomika. Cloudflare vidí IP adresy, čas a zaokrouhlenou velikost rámců (to je akceptované riziko celého návrhu). Obsah ani `mailbox_id` ne: provoz klient↔node i node↔node je šifrovaný vlastním kanálem (sekce 7).

## 6. Data plane (node)

Operátor anchoru vidí: `mailbox_id` (hash sdíleného tajemství, nespojitelný s identitou), velikost a čas zápisu a čtení, IP nejsou vidět (Cloudflare). Obsah vidí jen jako ciphertext.

| Hrozba | Obrana v implementaci | Zbytkové riziko |
|---|---|---|
| Čtení zpráv anchorem | E2E: ChaCha20-Poly1305, klíč na každou zprávu z Double Ratchetu (HKDF řetěz + DH krok), AAD váže ciphertext na `mailbox_id` a `id` zprávy | operátor vidí metadata (časování, velikost, počet zpráv v mailboxu) a nešifrovanou hlavičku ratchetu (`dh`, `pn`, `n`, `seq`): ví, kdy odesílatel přijal odpověď, identitu ne |
| Podvržení nebo přehrání zprávy nodem | AEAD tag, stav ratchetu se mění až po ověření tagu, použitý klíč se zahodí; přehraná zpráva (`n < Nr` bez uloženého klíče) se odmítne; DH krok i přeskočené klíče se počítají na kopii stavu, která se přijme až po ověření tagu | node může zprávu zadržet nebo smazat (dostupnost řeší replikace R=3 s kvorem 2) |
| Znovupoužití nonce | nonce i klíč jsou odvozeny z řetězu a unikátní pro každé `n`; klient musí stav uložit **před** odesláním (CLI to dělá) | klient, který stav ratchetu vrátí zpět (záloha, klon zařízení), znovu použije `n` s jiným obsahem; to ratchet sám nezachytí |
| Zahlcení mailboxu | zápis vyžaduje znalost `mailbox_id` (odvozeného z tajemství), strop nedoručených zpráv v mailboxu, limit zpráv na spojení, limit velikosti, ticket je jednorázový a vázaný na efemérní klíč | kdo zná `mailbox_id`, ho může zaplnit až po strop; `mailbox_id` v kódu zná jen dvojice klientů a anchory |
| Falešný anchor nebo podvržený node↔node request | každý request podepsán identitou nodu a ověřen proti adresáři Workeru (aktuální nebo předchozí epocha), nonce proti replayi, příjemce zapisuje a vydává jen mailboxy, pro které je sám anchorem | škodlivý anchor smí číst a mazat mailboxy, jejichž je anchorem (kvorum čtení ho přehlasuje jen částečně, protože čtení sjednocuje) |
| Oživení doručené zprávy | doručení se zapisuje jako náhrobek až do vypršení, `delivered` vyhrává při `put` i handoffu | ack, který nedorazil na žádný z anchorů, zprávu doručí podruhé; klient je odolný díky ratchetu (stejné `n` už nejde otevřít) |
| Ztráta dat při změně epochy | handoff nových anchorů před smazáním (potvrzení aspoň `quorum`), příjemci přijímají zápis i podle předchozí epochy | uzel, který ve chvíli změny epochy padne před dokončením handoffu, nechá data na ostatních dvou kopiích; přežití dvou současných výpadků není garantováno |

## 7. Zabezpečený kanál (`PROTOCOL.md` §14)

Cloudflare ukončuje TLS, takže je to aktivní útočník uprostřed (nebo ho může kdokoli, kdo Cloudflare ovládne). Kanál proto nespoléhá na TLS.

| Hrozba | Obrana v implementaci | Zbytkové riziko |
|---|---|---|
| Pasivní čtení provozu (`mailbox_id`, tickety, `push_id`, zprávy) na okraji Cloudflare | autentizovaný šifrovaný kanál klient↔node a node↔node, X25519 + ChaCha20-Poly1305 / XChaCha20-Poly1305, padding na 256 B | IP, čas, počet a hrubá velikost rámců, kdo se připojuje na který node |
| Man-in-the-middle při handshaku | node podepisuje přepis handshaku (výzva, oba efemerní klíče, jeho klíč a `node_id`) Ed25519 identitou; klient ji ověří proti `node_id` z ticketu podepsaného Workerem; u node↔node ověřuje iniciátor podpis klíčem z adresáře | kdo ukradne soukromý klíč nodu, může se za něj vydávat do jeho vypršení v adresáři |
| Podvržená trasa, ticket nebo adresář | klient ověří ticket připnutým klíčem Workeru před připojením; node a klient se nikdy neřídí klíčem staženým přes Cloudflare bez výslovného vývojového přepínače (`TRUST_WORKER_KEY_ON_FIRST_USE`) | kompromitace samotného klíče Workeru (viz sekce 1); chybně připnutý klíč |
| Krádež ticketu a jeho použití | ticket se posílá až uvnitř kanálu, jednorázový, vázaný na `ek`, podpis `ek` kryje přepis tohoto spojení | – |
| Přehrání, vynechání, prohození rámců | čítač v nonce a v pořadí (WebSocket je uspořádaný), tag ověřen před změnou stavu, nešifrovaný rámec spojení zavře | – |
| Přehrání node↔node requestu | handshake: podepsaný request s nonce a oknem; operace: náhodný 192bitový nonce, okno `ts` ±60 s, duplicity zahazuje responder | – |
| Přiřazení odpovědi k jinému požadavku | AAD odpovědi obsahuje `sid` a nonce požadavku | – |
| Prozrazení starého provozu po kompromitaci klíče | efemerní X25519 na obou stranách, session node↔node rotuje po 10 minutách | provoz probíhající během kompromitace |
| Body malého řádu, vyčerpání session | výstup X25519 samých nul se odmítne, strop session celkem i na volajícího | – |
| Skrytá data v paddingu | přijímá se jen kanonický padding | – |
| Downgrade na nešifrované API | holé `/n/*` a textové rámce po handshaku neexistují | – |

Co kanál z podstaty neřeší: Worker je Cloudflare a vidí své vlastní vstupy (`/route`: geolokace, `ek`; `/wake`: `push_id`, `mailbox_id`). Časová korelace provozu na obou koncích spojení (kdo kdy odešle a kdo kdy přijme) není skryta.

## 8. Identita uživatele a profil (`PROTOCOL.md` §15)

Profil (`pk`, `bk`, `spk`, jednorázové prekeys) je veřejný. Anchor profilu vidí `pk`, čas aktualizace a kdo se na něj ptá (jen gateway, ne IP). Nevidí, s kým `pk` komunikuje: `mailbox_id` je blinded a obálka v `inbox` skrývá odesílatele.

| Hrozba | Obrana v implementaci | Zbytkové riziko |
|---|---|---|
| Podvržení profilu cizí adresy | profil je podepsaný `uk` (včetně všech prekeys); anchor i klient podpis ověřují, klient navíc proti očekávané adrese | – |
| Podvržení adresy samotné (škodlivý anchor podstrčí cizí adresu při první komunikaci) | adresa je klíč; bezpečnostní číslo se porovnává mimo síť | uživatel, který číslo neporovná, se spoléhá na to, že adresu získal důvěryhodně |
| Zadržení nebo vrácení staršího profilu (zastaralý prekey) | čtení z `r − quorum + 1` anchorů, vyhrává nejvyšší `ts`; klient odmítne profil starší 14 dní | všichni dotazovaní anchory zadrží; klient pak vidí `not_found` |
| Únik dlouhodobých klíčů (`uk`, `bk`) | X3DH: tajemství závisí i na prekeys, jejichž tajné klíče příjemce maže (`spk` po 21 dnech, `otk` při prvním použití) | zprávy navázané bez `otk` jsou ohroženy únikem `spk` v jeho životě; aktivní útočník s `bk` se může vydávat za uživatele do výměny klíčů |
| Únik stavu ratchetu | jednosměrný řetěz chrání starší zprávy (forward secrecy); DH krok s novým párem uzdraví konverzaci (post-compromise security): po jedné odpovědi protistrany a odpovědi na ni útočník budoucí zprávy nečte | do uzdravení čte útočník s uloženým stavem (a pasivním přístupem k mailboxům) další zprávy; konverzace, kde píše jen jedna strana, se neuzdraví; první zprávy iniciátora před odpovědí závisí na `spk` responderu |
| Útočník vidí, kdo koho oslovil | obálka je zašifrovaná klíčem z `X25519(ek, spk)`; odesílatele ani `otk` anchor nevidí | anchor vidí, že do schránky pro `pk` přišla obálka, její čas a velikost |
| Replay obálky | `otk` po použití neexistuje; obálka bez `otk` dá totéž `ss`, existující konverzaci klient nepřepíše | – |
| Souběžné použití téhož `otk` dvěma iniciátory | druhá obálka se odmítne (`PrekeyGone`) a zahodí | druhý iniciátor musí začít znovu po novém profilu; příjemce profil po zpracování obálek znovu zveřejní |
| Zahlcení anchorů profily | hashcash na profil, limit zpráv na spojení, TTL, vždy jen jeden profil na `pk` | útočník s výpočetním výkonem; obtížnost je v konfiguraci |
| Zahlcení schránky (`inbox`) cizími obálkami | strop nedoručených zpráv v mailboxu, limit zpráv na spojení; příjemce neplatné obálky zahazuje a uvolňuje | schránka je veřejná z adresy: útočník ji může zaplnit až po strop (10 000) a zablokovat nové kontakty, dokud ji příjemce nevyčistí |
| Enumerace uživatelů | profil se hledá jen podle celého `pk` (256 bitů), neexistuje seznam ani vyhledávání | kdo zná `pk`, zjistí, že profil existuje |
| Falešný typing nebo stav doručení škodlivým anchorem | typing nemá důkaz původu a je jen UI, stav `delivered` čte kvorum a `delivered` vyhrává | škodlivý anchor může signál podvrhnout nebo zahodit; obsah zpráv to neovlivní |
| Zahlcení signály | token bucket na spojení, signál se neukládá, koalescence stejného stavu | – |
| Krádež zařízení / souboru s klíči | soubory 0600, atomický zápis | klíče nejsou šifrované heslem (otevřený bod) |
| Zachycení zpráv na serveru, historie | server drží jen ciphertext do `ack` (nejvýše 30 dní), po něm jen náhrobek bez obsahu; historie je výhradně lokální u uživatele | operátor vidí metadata (čas, velikost) |
