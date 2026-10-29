# dev

Šest nodů v Dockeru za Cloudflare Tunnel (fáze C3, `ARCHITECTURE.md` §10 a §12.5) a lokální běh bez Dockeru.

## Šest nodů v Dockeru (C3)
Image `ancord-node` (`../node/Dockerfile`) obsahuje node i `cloudflared` (pevná verze, ověřený SHA-256). Node se zaregistruje u Workeru, dostane token tunelu a spustí `cloudflared` jako podproces. Žádný kontejner nepublikuje port, node je dosažitelný jen přes `node-<id>.melonet.cz`. Každý node má vlastní volume s klíčem a databází (`node_id` přežije restart i `docker compose down`, ne `down -v`).

Kontejnery běží jako neprivilegovaný uživatel, s read-only kořenovým systémem souborů, bez capabilities a s `no-new-privileges`. `docker stop` node ukončí čistě (SIGTERM).

### Požadavky na server
Docker s Compose v2, odchozí HTTPS a odchozí spojení `cloudflared` na Cloudflare (port 7844, QUIC/UDP s fallbackem na TCP). Žádná veřejná IP ani otevřený port. Profil `prod` dává každému nodu Proof of Space ~4 GB na disku (`space-storage.*` ve volume), šest nodů tedy potřebuje ~25 GB.

Po prvním startu trvá 1–2 minuty, než Cloudflare začne směrovat na nový tunel (do té doby `530`, node aktivaci sám opakuje), a node se objeví v adresáři až na začátku další epochy (`prod`: 300 s). Do té doby klient dostane `insufficient_nodes`.

### Postup
Na stroji, kde poběží nody:
```
cp dev/.env.example dev/.env
dev/bootstrap.sh             # vytvoří klíče 6 nodů ve volume a zapíše jejich node_id do dev/genesis.txt
```
Kde se nasazuje Worker (potřebuje `CLOUDFLARE_API_TOKEN`, viz `../worker/README.md`, Nasazení):
```
cd worker
npm run deploy                                                    # plán: jen čte, vypíše co udělá a co chybí
ANCORD_GENESIS_FILE=../dev/genesis.txt ANCORD_RUNTIME_CF_TOKEN=… npm run deploy -- --apply
```
Deploy vypíše `WORKER_PUBKEY` (ověřený proti klíči, který sám nahrál) a zapíše ho do `dev/.env`, pokud existuje. Běží-li deploy jinde, zkopírujte řádek `WORKER_PUBKEY=…` do `dev/.env` na stroji s nody ručně. Potom:
```
cd dev && docker compose up -d
```

### Kontrola
```
docker compose ps                     # všech 6 „healthy“
docker compose logs -f node1          # registered, cloudflared started, node activated
curl -H "authorization: Bearer $(cat ~/.config/ancord/admin-token)" https://ancord.melonet.cz/admin/nodes
curl https://node-<id>.melonet.cz/health
```
Klient: `WORKER_PUBKEY=… ancord --worker https://ancord.melonet.cz route` (CLI je i v image: `docker compose exec node1 ancord …`, soubory do `/tmp`).

### Failover
```
docker compose stop node3             # Worker ho do heartbeatTimeoutS vyřadí z /route, zápisy dál projdou (R=3, kvorum 2)
docker compose start node3            # stejný node_id, stejný tunel
```

### Nový node, výměna klíče
Genesis allowlist je tajemství Workeru `GENESIS_NODE_IDS`. Nový node (nebo smazaná volume) znamená nový `node_id`: `dev/bootstrap.sh` a znovu `npm run deploy -- --apply` s `ANCORD_GENESIS_FILE` (nasadí jen nový seznam, ostatní tajemství zůstanou).

### Úklid
`dev/cleanup.sh` vypíše DNS záznamy `node*` s komentářem `ancord-managed` a tunely `ancord-*`, `dev/cleanup.sh --apply` je smaže. Cizí záznamy zóny ani Worker nemaže. Registr Workeru o tom neví: po úklidu nody zabanujte (`/admin/nodes/<id>/ban`) nebo smažte jejich volume.

## Lokální běh bez Dockeru a Cloudflare
`dev/local.sh [N]` (výchozí 6) spustí Worker v `wrangler dev` (mock Cloudflare) a N nodů jako procesy. Tunel nahrazuje `local-forward.py` (HTTP i WebSocket, směrování podle `/{hostname}/…`). Data a logy jsou v `/tmp/ancord-local`.

Klient:
```
ancord --worker http://127.0.0.1:8787 --via 'ws://127.0.0.1:8790/{hostname}/client' send alice.json "ahoj"
```
Stav nodů: `curl -H 'authorization: Bearer dev-admin' http://127.0.0.1:8787/admin/nodes`.

Ověřeno: 6 nodů `active` s prošlými challenge, zpráva přes 2 anchory se doručí a po vypnutí jednoho nodu to dál funguje.

Stejně je ověřený `docker-compose.yml` (Worker v `wrangler dev` jako kontejner ve stejné síti, `CLOUDFLARED_BIN=none` a forwarder místo tunelu): `bootstrap.sh` → 6 nodů `active` s genesis, 4/4 challenge u každého, zpráva mezi klienty na různých nodech i s vypnutým nodem, restart se stejným `node_id`.
