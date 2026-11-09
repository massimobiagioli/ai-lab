#!/usr/bin/env bash
# Run the k6 edge load scenario against the API container's network namespace.
# Usage: .tools/run-load.sh [k6 args]
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
COMPOSE="$ROOT/infra/compose/docker-compose.yml"
API_ID="$(docker compose -f "$COMPOSE" ps -q api)"
[ -n "$API_ID" ] || { echo "run-load: api container is not running" >&2; exit 2; }
TOKEN="${AILAB_API_TOKEN:-change-me-in-env-please-change-123456}"
LAB_ID="${AILAB_LAB_ID:-d0000000-0000-7000-8000-000000000001}"
exec docker run --rm \
  --network "container:$API_ID" \
  -e BASE_URL=http://127.0.0.1:3000 \
  -e TOKEN="$TOKEN" \
  -e LAB_ID="$LAB_ID" \
  -v "$ROOT/infra/load:/scripts:ro" \
  -w /scripts \
  grafana/k6:1.2.3 run board.js "$@"
