#!/usr/bin/env bash
# Enforce the coverage thresholds (backend 80, domain 90, web 80).
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
status=0

backend="$(cargo llvm-cov --workspace --summary-only 2>/dev/null | awk '/^TOTAL/{v=$10; gsub(/%/,"",v); print v}' | tail -1)"
awk -v v="${backend:-0}" 'BEGIN { printf "backend line coverage: %s%% (min 80)\n", v; exit !(v+0 >= 80) }' || status=1

domain="$(cargo llvm-cov -p ailab-domain --summary-only 2>/dev/null | awk '/^TOTAL/{v=$10; gsub(/%/,"",v); print v}' | tail -1)"
awk -v v="${domain:-0}" 'BEGIN { printf "domain line coverage: %s%% (min 90)\n", v; exit !(v+0 >= 90) }' || status=1

web_out="$(mktemp)"
scripts/run-web.sh npm test -- --coverage >"$web_out" 2>&1 || true
web="$(sed -E 's/\x1B\[[0-9;]*[mK]//g' "$web_out" | awk -F'|' '/All files/{gsub(/ /,"",$5); gsub(/%/,"",$5); print $5}' | tail -1)"
rm -f "$web_out"
awk -v v="${web:-0}" 'BEGIN { printf "web line coverage: %s%% (min 80)\n", v; exit !(v+0 >= 80) }' || status=1

[ "$status" -eq 0 ] || { echo "check-coverage: FAILED" >&2; exit 1; }
echo "check-coverage: OK"
