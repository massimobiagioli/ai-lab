#!/usr/bin/env bash
# Guarded BDD suite runner. Use `make bdd`; direct invocation is refused.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT" || exit 2

if [ "${AILAB_BDD_VIA_MAKE:-}" != "1" ]; then
  echo "bdd: run this suite only via 'make bdd'." >&2
  exit 2
fi

LOG_DIR="$ROOT/web/e2e-bdd/reports"
LOG_FILE="${AILAB_BDD_LOG:-$LOG_DIR/make-bdd.log}"
if ! mkdir -p "$(dirname "$LOG_FILE")"; then
  echo "bdd: cannot create the log directory for $LOG_FILE." >&2
  exit 2
fi
if ! : > "$LOG_FILE"; then
  echo "bdd: cannot write the log file $LOG_FILE." >&2
  exit 2
fi
exec > >(tee "$LOG_FILE") 2>&1
echo "BDD log: $LOG_FILE"

BASE="${AILAB_BDD_BASE_URL:-http://127.0.0.1:8080}"
BASE="${BASE%/}"
CONTAINER_BASE="${AILAB_BDD_CONTAINER_BASE_URL:-$BASE}"
if [ "$CONTAINER_BASE" = "http://127.0.0.1:8080" ]; then
  CONTAINER_BASE="http://host.docker.internal:8080"
fi
export AILAB_BDD_BASE_URL="$BASE"
export AILAB_DEV_FAKES="${AILAB_DEV_FAKES:-1}"

run_node() {
  "$ROOT/scripts/run-node.sh" "$@"
}

run_node scripts/bdd-preflight.mjs
PREFLIGHT_STATUS=$?
if [ "$PREFLIGHT_STATUS" -ne 0 ]; then
  exit "$PREFLIGHT_STATUS"
fi

if ! run_node .tools/gen-bdd-features.mjs --check; then
  echo "bdd: generated features are out of date; run 'node .tools/gen-bdd-features.mjs --write'." >&2
  exit 1
fi

if ! run_node .tools/check-bdd-coverage.mjs; then
  echo "bdd: coverage or drift check failed; see web/e2e-bdd/reports/coverage.json." >&2
  exit 1
fi

SUITE_ARGS=()
if [ -n "${AILAB_BDD_SUITE:-}" ]; then SUITE_ARGS+=(--suite "$AILAB_BDD_SUITE"); fi
if [ -n "${AILAB_BDD_UP_TO:-}" ]; then SUITE_ARGS+=(--up-to "$AILAB_BDD_UP_TO"); fi

SELECTION="$(run_node scripts/bdd-suites.mjs "${SUITE_ARGS[@]+"${SUITE_ARGS[@]}"}" --tsv)"
SELECTION_STATUS=$?
if [ "$SELECTION_STATUS" -ne 0 ]; then
  exit "$SELECTION_STATUS"
fi
IFS=$'\t' read -r BDD_SUITES BDD_GREP BDD_TOTAL <<< "$SELECTION"
echo "BDD suites: ${BDD_SUITES} (${BDD_TOTAL} scenarios)"
if [ -n "$BDD_GREP" ]; then
  echo "BDD selection: ${BDD_GREP}"
else
  echo "BDD selection: all suites"
fi

if ! curl -fsS --max-time 5 "$BASE/healthz" >/dev/null 2>&1 \
  && ! curl -fsS --max-time 5 "$BASE/api/v1/health/live" >/dev/null 2>&1; then
  echo "BDD suite not started: the prepared test environment is not reachable at $BASE." >&2
  exit 2
fi

IMAGE="${AILAB_BDD_PLAYWRIGHT_IMAGE:-mcr.microsoft.com/playwright:v1.65.0-noble}"
EXTRA_ENV=()
if [ -n "${AILAB_API_TOKEN:-}" ]; then EXTRA_ENV+=(-e AILAB_API_TOKEN); fi

run_web() {
  docker run --rm --user "$(id -u):$(id -g)" \
    --add-host=host.docker.internal:host-gateway \
    -e HOME=/tmp -e NG_CLI_ANALYTICS=false \
    -e AILAB_BDD_VIA_MAKE=1 \
    -e AILAB_BDD_BASE_URL="$CONTAINER_BASE" \
    -e AILAB_BDD_LIVE="${AILAB_BDD_LIVE:-0}" \
    -e AILAB_BDD_REDUCED_MOTION="${AILAB_BDD_REDUCED_MOTION:-no-preference}" \
    -e AILAB_DEV_FAKES="${AILAB_DEV_FAKES:-1}" \
    "${EXTRA_ENV[@]+"${EXTRA_ENV[@]}"}" \
    -v "$ROOT/web:/app" -w /app "$IMAGE" "$@"
}

run_web npx bddgen --config e2e-bdd/playwright.config.ts || exit $?
if [ -n "$BDD_GREP" ]; then
  run_web npx playwright test --config e2e-bdd/playwright.config.ts --grep "$BDD_GREP"
else
  run_web npx playwright test --config e2e-bdd/playwright.config.ts
fi
STATUS=$?

if [ -f "$ROOT/web/e2e-bdd/reports/results.json" ]; then
  run_node -e '
const fs = require("node:fs");
const path = process.argv[1];
const data = JSON.parse(fs.readFileSync(path, "utf8"));
const s = data.summary ?? {};
console.log(`BDD results: total=${s.total ?? 0} passed=${s.passed ?? 0} failed=${s.failed ?? 0} skipped=${s.skipped ?? 0} manual=${s.manual ?? 0} live=${s.live ?? 0}`);
if (Array.isArray(data.failedIds) && data.failedIds.length > 0) console.log(`BDD failed: ${data.failedIds.join(", ")}`);
' "web/e2e-bdd/reports/results.json"
fi

exit "$STATUS"
