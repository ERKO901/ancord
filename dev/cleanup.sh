#!/usr/bin/env bash
# Smaže vývojové node DNS záznamy a tunely, které vytvořil Worker (DNS komentář `ancord-managed`, tunel `ancord-*`).
# Bez --apply jen vypíše, co by smazal. Worker, KV, R2 a fronty nemaže.
set -euo pipefail
cd "$(dirname "$0")/../worker"
exec npx tsx scripts/cleanup.ts "$@"
