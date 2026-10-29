#!/usr/bin/env bash
# Vytvoří identity šesti nodů ve volume (bez kontaktu s Workerem) a zapíše jejich node_id do dev/genesis.txt
# pro genesis allowlist Workeru. Idempotentní: existující klíče se jen načtou.
set -euo pipefail
cd "$(dirname "$0")"
[ -f .env ] || { cp .env.example .env; echo "vytvořen dev/.env z .env.example"; }
docker compose build -q
: > genesis.txt.tmp
for i in 1 2 3 4 5 6; do
  id=$(docker compose run --rm --no-deps -T node$i --init-identity | tr -d '\r')
  [[ $id =~ ^[a-z2-7]{52}$ ]] || { echo "node$i: neplatný node_id '$id'" >&2; exit 1; }
  echo "$id" >> genesis.txt.tmp
  echo "node$i  $id  node-${id:0:16}.melonet.cz"
done
mv genesis.txt.tmp genesis.txt
echo "zapsáno dev/genesis.txt; dále: cd worker && ANCORD_GENESIS_FILE=../dev/genesis.txt npm run deploy -- --apply"
