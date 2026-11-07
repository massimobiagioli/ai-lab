#!/usr/bin/env bash
# Database and artifact backup with a manifest.
#
# Usage: scripts/backup.sh
# Writes artifacts/backups/<UTC timestamp>.dump and a .manifest.json.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
COMPOSE_FILE="$ROOT/infra/compose/docker-compose.yml"
DB_USER="${POSTGRES_USER:-ailab}"
DB_NAME="${POSTGRES_DB:-ailab}"
TS="$(date -u +%Y%m%dT%H%M%SZ)"
OUT="$ROOT/artifacts/backups"

[ -f "$COMPOSE_FILE" ] || { echo "backup: missing $COMPOSE_FILE" >&2; exit 2; }
mkdir -p "$OUT"
DUMP="$OUT/$TS.dump"

docker compose -f "$COMPOSE_FILE" exec -T postgres \
  pg_dump -U "$DB_USER" -d "$DB_NAME" --format=custom > "$DUMP"

tar -czf "$OUT/$TS-artifacts.tgz" -C "$ROOT" artifacts --exclude=artifacts/backups 2>/dev/null || true
SHA="$(shasum -a 256 "$DUMP" | awk '{print $1}')"
GIT="$(git -C "$ROOT" rev-parse --short HEAD 2>/dev/null || echo no-git)"
MIGRATION="$(docker compose -f "$COMPOSE_FILE" exec -T postgres \
  psql -U "$DB_USER" -d "$DB_NAME" -tAc \
  "select version from _sqlx_migrations order by version desc limit 1" 2>/dev/null || echo unknown)"

cat > "$OUT/$TS.manifest.json" <<EOF
{
  "created_at": "$TS",
  "dump": "$(basename "$DUMP")",
  "sha256": "$SHA",
  "git": "$GIT",
  "migration": "${MIGRATION:-unknown}"
}
EOF

echo "backup: $DUMP"
echo "backup: manifest $OUT/$TS.manifest.json"
