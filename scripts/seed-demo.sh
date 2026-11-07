#!/usr/bin/env bash
# Seed the deterministic demo dataset.
#
# Usage: scripts/seed-demo.sh
# Requires the core stack to be running (postgres healthy).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
COMPOSE_FILE="$ROOT/infra/compose/docker-compose.yml"
DB_USER="${POSTGRES_USER:-ailab}"
DB_NAME="${POSTGRES_DB:-ailab}"

[ -f "$COMPOSE_FILE" ] || { echo "seed-demo: missing $COMPOSE_FILE" >&2; exit 2; }
[ -f "$ROOT/scripts/seed-demo.sql" ] || { echo "seed-demo: missing scripts/seed-demo.sql" >&2; exit 2; }

if ! docker compose -f "$COMPOSE_FILE" exec -T postgres pg_isready -U "$DB_USER" -d "$DB_NAME" >/dev/null 2>&1; then
  echo "seed-demo: postgres is not running; run make up first" >&2
  exit 1
fi

docker compose -f "$COMPOSE_FILE" exec -T postgres \
  psql -U "$DB_USER" -d "$DB_NAME" -v ON_ERROR_STOP=1 -q < "$ROOT/scripts/seed-demo.sql"

echo "seed-demo: demo lab, 4 agents, project, SOW, plan, 12 cards and events seeded"
