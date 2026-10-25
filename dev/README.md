# dev

`docker-compose.yml` (C3, 6 nodů za Cloudflare Tunnel) zatím není; vzniká s fází C.

## Lokální běh bez Dockeru a Cloudflare
`dev/local.sh [N]` (výchozí 6) spustí Worker v `wrangler dev` (mock Cloudflare) a N nodů jako procesy. Tunel nahrazuje `local-forward.py` (HTTP i WebSocket, směrování podle `/{hostname}/…`). Data a logy jsou v `/tmp/ancord-local`.

Klient:
```
ancord --worker http://127.0.0.1:8787 --via 'ws://127.0.0.1:8790/{hostname}/client' send alice.json "ahoj"
```
Stav nodů: `curl -H 'authorization: Bearer dev-admin' http://127.0.0.1:8787/admin/nodes`.

Ověřeno: 6 nodů `active` s prošlými challenge, zpráva přes 2 anchory se doručí a po vypnutí jednoho nodu to dál funguje.
