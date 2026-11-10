#!/usr/bin/env bash
# Fixture-based self-test for gen-bdd-features and check-bdd-coverage.
#
# Usage: .tools/bdd-self-test.sh
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP="$(mktemp -d)"
PASS=0
FAIL=0

TOOLS="/repo/.tools"
FIXTURE="/tmp/bdd"

cleanup() { rm -rf "$TMP"; }
trap cleanup EXIT INT TERM

run_node() {
  docker run --rm --user "$(id -u):$(id -g)" -e HOME=/tmp \
    -v "$ROOT:/repo" -v "$TMP:/tmp/bdd" -w /repo node:24.22.0-trixie-slim node "$@"
}

pass() { printf 'PASS  %s\n' "$1"; PASS=$((PASS + 1)); }
fail() { printf 'FAIL  %s\n' "$1"; FAIL=$((FAIL + 1)); }

write_fixture() {
  local first="$1"
  local second="${2:-}"
  mkdir -p "$TMP/.docs/e2e" "$TMP/web/e2e-bdd/features" "$TMP/web/e2e-bdd/steps"
  cat > "$TMP/.docs/e2e/BDD.md" <<EOF
# Mini BDD

## 1. Mini area

\`\`\`gherkin
@mini @p0
Feature: Mini area

  Scenario: [ ] ${first} A prepared environment
    Given a prepared environment
    When I call GET /api/v1/health/live
    Then the response status is 200

  Scenario: [ ] ${second} A second scenario
    Given a prepared environment
    When I call GET /api/v1/health/live
    Then the response status is 200
\`\`\`
EOF
  cat > "$TMP/web/e2e-bdd/steps/mini.steps.ts" <<'EOF'
import { createBdd } from 'playwright-bdd';
import { test } from '../../../support/fixtures';

const { Given, When, Then } = createBdd(test);

Given('a prepared environment', async () => {});
When(/^I call (\w+) (.+)$/, async () => {});
Then('the response status is {int}', async () => {});
EOF
  cat > "$TMP/web/e2e-bdd/manual-scenarios.md" <<'EOF'
# Manual scenarios

| E2E ID | Owner | Reason | Evidence | Review date |
|---|---|---|---|---|
EOF
}

expect_exit() {
  local expected="$1"
  local name="$2"
  shift 2
  "$@" >/dev/null 2>&1
  local code=$?
  if [ "$code" -eq "$expected" ]; then pass "$name"; else fail "$name (exit $code, expected $expected)"; fi
}

write_fixture E2E-001 E2E-002

expect_exit 0 "generator writes the fixture" run_node "$TOOLS/gen-bdd-features.mjs" --root "$FIXTURE" --expected 2 --write
expect_exit 0 "generator check passes" run_node "$TOOLS/gen-bdd-features.mjs" --root "$FIXTURE" --expected 2 --check
expect_exit 0 "coverage check passes" run_node "$TOOLS/check-bdd-coverage.mjs" --root "$FIXTURE" --expected 2

write_fixture E2E-001
expect_exit 1 "missing id is rejected" run_node "$TOOLS/gen-bdd-features.mjs" --root "$FIXTURE" --expected 2 --check

write_fixture E2E-001 E2E-001
expect_exit 1 "duplicate id is rejected" run_node "$TOOLS/gen-bdd-features.mjs" --root "$FIXTURE" --expected 2 --check

write_fixture E2E-001 E2E-002
run_node "$TOOLS/gen-bdd-features.mjs" --root "$FIXTURE" --expected 2 --write >/dev/null 2>&1
printf '@stale\nFeature: Stale\n\n  Scenario: stale\n    Then nothing\n' > "$TMP/web/e2e-bdd/features/99-stale.feature"
expect_exit 1 "stale feature is rejected" run_node "$TOOLS/check-bdd-coverage.mjs" --root "$FIXTURE" --expected 2

write_fixture E2E-001 E2E-002
sed -i.bak 's/the response status is 200/the impossible assertion/' "$TMP/.docs/e2e/BDD.md" 2>/dev/null || true
run_node "$TOOLS/gen-bdd-features.mjs" --root "$FIXTURE" --expected 2 --write >/dev/null 2>&1
expect_exit 1 "unmatched step is rejected in strict mode" run_node "$TOOLS/check-bdd-coverage.mjs" --root "$FIXTURE" --expected 2 --strict

echo
if [ "$FAIL" -ne 0 ]; then
  echo "bdd-self-test: FAILED ($FAIL failures, $PASS passed)"
  exit 1
fi
echo "bdd-self-test: OK ($PASS passed)"
