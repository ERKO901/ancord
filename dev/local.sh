#!/usr/bin/env bash
# Lokální vývojové prostředí bez Dockeru a Cloudflare: Worker v `wrangler dev` (mock Cloudflare) + N nodů jako procesy.
# Tunel nahrazuje local-forward.py (PROOF_ORIGIN a PEER_URL_TEMPLATE). Použití: dev/local.sh [počet nodů, výchozí 6]
set -euo pipefail
N=${1:-6}
ROOT=$(cd "$(dirname "$0")/.." && pwd)
RUN=${RUN_DIR:-/tmp/ancord-local}
WPORT=8787; FPORT=8790; BASE_PORT=8100; ADMIN=dev-admin
rm -rf "$RUN"; mkdir -p "$RUN/logs"
trap 'kill 0' EXIT

cargo build -q --manifest-path "$ROOT/node/Cargo.toml" -p ancord-node -p ancord-cli
NODE=$ROOT/node/target/debug/ancord-node

ids=(); hosts=(); map=()
for i in $(seq 1 "$N"); do
  d=$RUN/node$i
  id=$(WORKER_URL=http://127.0.0.1:1 DATA_DIR=$d CLOUDFLARED_BIN=none RUST_LOG=info timeout 5 "$NODE" 2>&1 | sed -n 's/.*node_id="\([^"]*\)".*/\1/p' | head -1 || true)
  ids+=("$id"); map+=("node-${id:0:16}.melonet.cz=$((BASE_PORT+i))")
done
GENESIS=$(IFS=,; echo "${ids[*]}")

python3 "$ROOT/dev/local-forward.py" $FPORT "${map[@]}" &
(cd "$ROOT/worker" && WRANGLER_SEND_METRICS=false CI=1 exec npx wrangler dev --port $WPORT --persist-to "$RUN/wrangler-state" \
  --var WORKER_SIGNING_KEY:AQIDBAUGBwgJCgsMDQ4PEBESExQVFhcYGRobHB0eHyA \
  --var CHALLENGE_HMAC_KEY:ISIjJCUmJygpKissLS4vMDEyMzQ1Njc4OTo7PD0-Pz8 \
  --var PUSH_ENC_KEY:QEFCQ0RFRkdISUpLTE1OT1BRUlNUVVZXWFlaW1xdXl8 \
  --var ADMIN_TOKEN:$ADMIN --var "GENESIS_NODE_IDS:$GENESIS" \
  --var "PROOF_ORIGIN:http://127.0.0.1:$FPORT/{hostname}" \
  --var 'CONFIG_OVERRIDES:{"epochSeconds":15,"snapshotRefreshS":2,"challenges":{"cpu":{"iters":100000,"every":1000},"space":{"ram":{"maxLeaves":4096},"storage":{"maxLeaves":16384}}}}') \
  >"$RUN/logs/wrangler.log" 2>&1 &
until curl -sf http://127.0.0.1:$WPORT/health >/dev/null; do sleep 1; done

for i in $(seq 1 "$N"); do
  TRUST_WORKER_KEY_ON_FIRST_USE=1 WORKER_URL=http://127.0.0.1:$WPORT DATA_DIR=$RUN/node$i NODE_NAME=node$i LISTEN=127.0.0.1:$((BASE_PORT+i)) CLOUDFLARED_BIN=none \
  CLAIM_CPU_CORES=2 CLAIM_RAM_MB=256 CLAIM_STORAGE_GB=1 PEER_URL_TEMPLATE="http://127.0.0.1:$FPORT/{host}" RUST_LOG=info \
    "$NODE" >"$RUN/logs/node$i.log" 2>&1 &
done
echo "Worker http://127.0.0.1:$WPORT, logy $RUN/logs, admin: curl -H 'authorization: Bearer $ADMIN' http://127.0.0.1:$WPORT/admin/nodes"
wait
