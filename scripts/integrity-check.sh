#!/usr/bin/env bash
# Run the integrity checks against the running database and fail on any finding.
#
# Usage: scripts/integrity-check.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
COMPOSE_FILE="$ROOT/infra/compose/docker-compose.yml"
DB_USER="${POSTGRES_USER:-ailab}"
DB_NAME="${POSTGRES_DB:-ailab}"

docker compose -f "$COMPOSE_FILE" exec -T postgres \
  psql -U "$DB_USER" -d "$DB_NAME" -tA -v ON_ERROR_STOP=1 \
  < "$ROOT/scripts/data-integrity.sql" > /tmp/ailab-integrity.json

python3 - /tmp/ailab-integrity.json <<'PY'
import json
import sys

path = sys.argv[1]
data = json.load(open(path))
print(json.dumps(data, sort_keys=True))
bad = {key: value for key, value in data.items() if value}
if bad:
    print("INTEGRITY: FAILED", file=sys.stderr)
    sys.exit(1)
print("INTEGRITY: OK")
PY

# Expose the integrity result to the /metrics textfile collector.
mkdir -p "$ROOT/artifacts/metrics"
MISMATCHES=0
if [ -f "$ROOT/artifacts/integrity-report.json" ]; then
  MISMATCHES="$(python3 -c 'import json,sys; d=json.load(open(sys.argv[1])); print(sum(1 for v in d.values() if v))' "$ROOT/artifacts/integrity-report.json" 2>/dev/null || echo 0)"
fi
printf 'ailab_integrity_mismatches %s\n' "$MISMATCHES" > "$ROOT/artifacts/metrics/integrity.prom"
