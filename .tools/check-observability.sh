#!/usr/bin/env bash
# Validate the WP-20 observability artifacts (SLOs, alerts, dashboard).
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIR="$ROOT/infra/observability"
python3 -m json.tool "$DIR/grafana-dashboard.json" >/dev/null
for name in AilabHighErrorRate AilabHighLatency AilabNoTraffic; do
  grep -q "alert: $name" "$DIR/alerts.yml" || { echo "missing alert $name" >&2; exit 1; }
done
for name in edge-availability edge-latency run-success; do
  grep -q "name: $name" "$DIR/slo.yml" || { echo "missing SLO $name" >&2; exit 1; }
done
echo "OBSERVABILITY CHECK: OK"
