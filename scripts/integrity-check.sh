#!/usr/bin/env bash
# Run the integrity checks against the running database and fail on any finding.
#
# Usage: scripts/integrity-check.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
COMPOSE_FILE="$ROOT/infra/compose/docker-compose.yml"
DB_USER="${POSTGRES_USER:-ailab}"
DB_NAME="${POSTGRES_DB:-ailab}"

mkdir -p "$ROOT/artifacts/metrics"
REPORT="$ROOT/artifacts/integrity-report.json"
docker compose -f "$COMPOSE_FILE" exec -T postgres \
  psql -U "$DB_USER" -d "$DB_NAME" -tA -v ON_ERROR_STOP=1 \
  < "$ROOT/scripts/data-integrity.sql" > "$REPORT"

# Expose the result to the /metrics textfile collector even when the check fails.
MISMATCHES="$(python3 - "$REPORT" <<'PY'
import json
import sys

data = json.load(open(sys.argv[1]))
print(sum(1 for value in data.values() if value))
PY
)"
printf 'ailab_integrity_mismatches %s\n' "$MISMATCHES" > "$ROOT/artifacts/metrics/integrity.prom"

python3 - "$REPORT" <<'PY'
import json
import sys

data = json.load(open(sys.argv[1]))
print(json.dumps(data, sort_keys=True))
bad = {key: value for key, value in data.items() if value}
if bad:
    print("INTEGRITY: FAILED", file=sys.stderr)
    sys.exit(1)
print("INTEGRITY: OK")
PY
