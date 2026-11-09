#!/usr/bin/env bash
# Restart the worker and confirm it recovers to healthy (WP-24 chaos).
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
COMPOSE="$ROOT/infra/compose/docker-compose.yml"
echo "chaos: killing the worker"
docker compose -f "$COMPOSE" kill worker >/dev/null
sleep 2
echo "chaos: restarting the worker"
docker compose -f "$COMPOSE" up -d worker >/dev/null
for _ in $(seq 1 30); do
  id="$(docker compose -f "$COMPOSE" ps -q worker)"
  if [ -n "$id" ] && [ "$(docker inspect --format '{{.State.Health.Status}}' "$id")" = "healthy" ]; then
    echo "chaos: worker recovered healthy"
    exit 0
  fi
  sleep 2
done
echo "chaos: worker did not recover" >&2
exit 1
