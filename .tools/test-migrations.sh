#!/usr/bin/env bash
# Schema-diff test: the migrations must reproduce Appendix A exactly.
#
# Applies Appendix A + the analytics views to one database and the versioned
# migrations to another, dumps both schemas and compares them.
#
# Usage: .tools/test-migrations.sh
# Exit: 0 identical, 1 drift, 2 environment error.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APPENDIX="$ROOT/.docs/technical-specification/APPENDIX-A-DATABASE.sql"
INIT="$ROOT/migrations/0001_init.sql"
VIEWS="$ROOT/migrations/0002_analytics_views.sql"
IMAGE="${AILAB_TEST_PG_IMAGE:-pgvector/pgvector:pg18}"

for f in "$APPENDIX" "$INIT" "$VIEWS"; do
  [ -f "$f" ] || { echo "ENV ERROR: missing $f" >&2; exit 2; }
done
command -v docker >/dev/null 2>&1 || { echo "ENV ERROR: docker missing" >&2; exit 2; }
docker info >/dev/null 2>&1 || { echo "ENV ERROR: docker not running" >&2; exit 2; }
docker image inspect "$IMAGE" >/dev/null 2>&1 || IMAGE="pgvector/pgvector:pg17"

NAME="ailab-migrations-test-$$"
cleanup() { docker rm -f "$NAME" >/dev/null 2>&1 || true; }
trap cleanup EXIT

docker run -d --name "$NAME" -e POSTGRES_PASSWORD=test "$IMAGE" >/dev/null || exit 2
for _ in $(seq 1 60); do
  docker exec "$NAME" pg_isready -U postgres >/dev/null 2>&1 && break
  sleep 1
done
docker exec "$NAME" pg_isready -U postgres >/dev/null 2>&1 || { echo "ENV ERROR: postgres not ready" >&2; exit 2; }

psql_db() {
  local db="$1"; shift
  docker exec -i "$NAME" psql -U postgres -d "$db" -v ON_ERROR_STOP=1 -q "$@"
}

docker exec "$NAME" createdb -U postgres appendix
docker exec "$NAME" createdb -U postgres migrated
psql_db appendix < "$APPENDIX" >/dev/null
psql_db appendix < "$VIEWS" >/dev/null
psql_db migrated < "$INIT" >/dev/null
psql_db migrated < "$VIEWS" >/dev/null

normalize() {
  sed -E '/^\\(un)?restrict/d; /^SET /d; /^SELECT pg_catalog.set_config/d; /^--/d; /^$/d' "$1"
}
docker exec "$NAME" pg_dump -U postgres -d appendix --schema-only --no-owner --no-privileges > /tmp/ailab-appendix.sql
docker exec "$NAME" pg_dump -U postgres -d migrated --schema-only --no-owner --no-privileges > /tmp/ailab-migrated.sql
normalize /tmp/ailab-appendix.sql > /tmp/ailab-appendix.norm
normalize /tmp/ailab-migrated.sql > /tmp/ailab-migrated.norm

if ! diff -u /tmp/ailab-appendix.norm /tmp/ailab-migrated.norm > /tmp/ailab-schema.diff; then
  echo "MIGRATIONS: FAILED (schema drift)"
  head -60 /tmp/ailab-schema.diff
  exit 1
fi
echo "MIGRATIONS: OK (schema matches Appendix A)"
exit 0
