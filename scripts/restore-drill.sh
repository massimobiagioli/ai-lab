#!/usr/bin/env bash
# Restore a dump into a scratch database and verify the result against the
# backup manifest: dump SHA-256, per-table content checksums, integrity checks
# and an RTO measurement. The scratch database is removed afterwards.
#
# Usage: scripts/restore-drill.sh <dump-file> [--keep]
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
COMPOSE_FILE="$ROOT/infra/compose/docker-compose.yml"
DB_USER="${POSTGRES_USER:-ailab}"
DB_NAME="${POSTGRES_DB:-ailab}"
SCRATCH="ailab_restore_drill"
BACKUP="${1:-}"
KEEP="${2:-}"

[ -n "$BACKUP" ] || { echo "usage: scripts/restore-drill.sh <dump-file> [--keep]" >&2; exit 2; }
[ -f "$BACKUP" ] || { echo "restore-drill: missing $BACKUP" >&2; exit 2; }
[ -f "$COMPOSE_FILE" ] || { echo "restore-drill: missing $COMPOSE_FILE" >&2; exit 2; }
MANIFEST="${BACKUP%.dump}.manifest.json"

psql_src() { docker compose -f "$COMPOSE_FILE" exec -T postgres psql -U "$DB_USER" -d "$DB_NAME" "$@"; }
psql_scratch() { docker compose -f "$COMPOSE_FILE" exec -T postgres psql -U "$DB_USER" -d "$SCRATCH" "$@"; }

START="$(date +%s)"

# 1. The dump must match the manifest checksum.
if [ -f "$MANIFEST" ]; then
  EXPECTED="$(python3 - "$MANIFEST" <<'PY'
import json, sys
print(json.load(open(sys.argv[1])).get("sha256", ""))
PY
)"
  ACTUAL="$(shasum -a 256 "$BACKUP" | awk '{print $1}')"
  if [ -n "$EXPECTED" ] && [ "$EXPECTED" != "$ACTUAL" ]; then
    echo "restore-drill: dump checksum mismatch (manifest $EXPECTED != actual $ACTUAL)" >&2
    exit 1
  fi
  echo "restore-drill: dump checksum verified against the manifest"
else
  echo "restore-drill: no manifest next to the dump; skipping checksum" >&2
fi

# 2. Restore into a fresh scratch database.
docker compose -f "$COMPOSE_FILE" exec -T postgres psql -U "$DB_USER" -d postgres \
  -c "DROP DATABASE IF EXISTS $SCRATCH" >/dev/null
docker compose -f "$COMPOSE_FILE" exec -T postgres psql -U "$DB_USER" -d postgres \
  -c "CREATE DATABASE $SCRATCH" >/dev/null
docker compose -f "$COMPOSE_FILE" exec -T postgres \
  pg_restore -U "$DB_USER" -d "$SCRATCH" --no-owner --no-privileges < "$BACKUP"

# 3. Every base table must have the same row count and content checksum.
TABLES="$(psql_src -tAc "select table_name from information_schema.tables where table_schema='public' and table_type='BASE TABLE' order by table_name")"
FAILED=0
for table in $TABLES; do
  src_rows="$(psql_src -tAc "select count(*) from public.\"$table\"" 2>/dev/null || echo 0)"
  dst_rows="$(psql_scratch -tAc "select count(*) from public.\"$table\"" 2>/dev/null || echo 0)"
  src_sum="$(psql_src -tAc "select coalesce(md5(string_agg(t::text, '' order by t::text)), '') from public.\"$table\" t" 2>/dev/null || echo '')"
  dst_sum="$(psql_scratch -tAc "select coalesce(md5(string_agg(t::text, '' order by t::text)), '') from public.\"$table\" t" 2>/dev/null || echo '')"
  if [ "$src_rows" != "$dst_rows" ]; then
    echo "restore-drill: row count mismatch on $table ($src_rows != $dst_rows)" >&2
    FAILED=1
  fi
  if [ "$src_sum" != "$dst_sum" ]; then
    echo "restore-drill: content checksum mismatch on $table" >&2
    FAILED=1
  fi
done

# 4. The integrity checks must be clean on the restored copy.
if ! docker compose -f "$COMPOSE_FILE" exec -T postgres psql -U "$DB_USER" -d "$SCRATCH" \
  -tA -v ON_ERROR_STOP=1 < "$ROOT/scripts/data-integrity.sql" \
  | python3 -c 'import json,sys; d=json.load(sys.stdin); bad={k:v for k,v in d.items() if v}; print(d); sys.exit(1 if bad else 0)'; then
  echo "restore-drill: integrity checks failed on the restored copy" >&2
  FAILED=1
fi

END="$(date +%s)"
RTO="$((END - START))"

if [ "$KEEP" != "--keep" ]; then
  docker compose -f "$COMPOSE_FILE" exec -T postgres psql -U "$DB_USER" -d postgres \
    -c "DROP DATABASE IF EXISTS $SCRATCH" >/dev/null
fi

if [ "$FAILED" -ne 0 ]; then
  exit 1
fi
echo "restore-drill: OK ($BACKUP restored into $SCRATCH in ${RTO}s; row counts, content checksums and integrity verified)"
