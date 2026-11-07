#!/usr/bin/env bash
# Restore a dump into a scratch database and verify the result.
#
# Usage: scripts/restore-drill.sh <dump-file>
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
COMPOSE_FILE="$ROOT/infra/compose/docker-compose.yml"
DB_USER="${POSTGRES_USER:-ailab}"
DB_NAME="${POSTGRES_DB:-ailab}"
SCRATCH="ailab_restore_drill"
BACKUP="${1:-}"

[ -n "$BACKUP" ] || { echo "usage: scripts/restore-drill.sh <dump-file>" >&2; exit 2; }
[ -f "$BACKUP" ] || { echo "restore-drill: missing $BACKUP" >&2; exit 2; }
[ -f "$COMPOSE_FILE" ] || { echo "restore-drill: missing $COMPOSE_FILE" >&2; exit 2; }

psql_src() { docker compose -f "$COMPOSE_FILE" exec -T postgres psql -U "$DB_USER" -d "$DB_NAME" "$@"; }
psql_scratch() { docker compose -f "$COMPOSE_FILE" exec -T postgres psql -U "$DB_USER" -d "$SCRATCH" "$@"; }

docker compose -f "$COMPOSE_FILE" exec -T postgres psql -U "$DB_USER" -d postgres \
  -c "DROP DATABASE IF EXISTS $SCRATCH" >/dev/null
docker compose -f "$COMPOSE_FILE" exec -T postgres psql -U "$DB_USER" -d postgres \
  -c "CREATE DATABASE $SCRATCH" >/dev/null

docker compose -f "$COMPOSE_FILE" exec -T postgres \
  pg_restore -U "$DB_USER" -d "$SCRATCH" --no-owner --no-privileges < "$BACKUP"

FAILED=0
for table in cards runs agents projects card_events audit_log; do
  src="$(psql_src -tAc "select count(*) from $table" 2>/dev/null || echo 0)"
  dst="$(psql_scratch -tAc "select count(*) from $table" 2>/dev/null || echo 0)"
  if [ "$src" != "$dst" ]; then
    echo "restore-drill: row count mismatch on $table ($src != $dst)" >&2
    FAILED=1
  fi
done

if [ "$FAILED" -ne 0 ]; then
  exit 1
fi

echo "restore-drill: OK ($BACKUP restored into $SCRATCH)"
