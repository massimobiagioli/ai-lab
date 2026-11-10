#!/usr/bin/env bash
# Negative supply-chain tests.
#
# 1. A known vulnerable npm dependency must fail `npm audit --audit-level=high`.
# 2. A disallowed license must fail `cargo deny check licenses`.
#
# Usage: .tools/test-supply-chain.sh
# Exit: 0 both negative fixtures were rejected, 1 a gate failed to reject,
#       2 environment unavailable (docker or cargo-deny missing).
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
NPM_FIXTURE="$ROOT/.tools/fixtures/supply-chain/npm"
CARGO_FIXTURE="$ROOT/.tools/fixtures/supply-chain/cargo"
NODE_IMAGE="${AILAB_NODE_IMAGE:-node:24.22.0-trixie-slim}"

FAIL=0
PASS=0
fail() { printf 'FAIL  %s\n' "$1"; FAIL=1; }
pass() { printf 'PASS  %s\n' "$1"; PASS=$((PASS + 1)); }

# ---------------------------------------------------------------------------
# npm audit negative fixture
# ---------------------------------------------------------------------------
if ! command -v docker >/dev/null 2>&1 || ! docker info >/dev/null 2>&1; then
  echo "ENV ERROR: docker is required for the npm audit fixture" >&2
  exit 2
fi
if ! docker image inspect "$NODE_IMAGE" >/dev/null 2>&1; then
  echo "pulling $NODE_IMAGE ..." >&2
  if ! docker pull "$NODE_IMAGE" >/dev/null 2>&1; then
    echo "ENV ERROR: $NODE_IMAGE is not available and could not be pulled" >&2
    exit 2
  fi
fi

TMP="$(mktemp -d "${TMPDIR:-/tmp}/supply-chain-npm.XXXXXX")"
trap 'rm -rf "$TMP"' EXIT
cp "$NPM_FIXTURE/package.json" "$TMP/package.json"

docker run --rm --user "$(id -u):$(id -g)" -e HOME=/tmp \
  -v "$TMP:/app" -w /app "$NODE_IMAGE" \
  sh -c 'npm install --package-lock-only --silent && npm audit --audit-level=high' \
  >/tmp/ailab-npm-audit-fixture.log 2>&1
npm_code=$?
if [ "$npm_code" -eq 0 ]; then
  fail "npm audit accepted the vulnerable lodash fixture"
else
  pass "npm audit rejected the vulnerable fixture (exit $npm_code)"
fi

# ---------------------------------------------------------------------------
# cargo-deny license negative fixture
# ---------------------------------------------------------------------------
WORK="$ROOT/.cache/supply-chain-cargo"
rm -rf "$WORK"
mkdir -p "$WORK"
cp -R "$CARGO_FIXTURE"/. "$WORK"/
if ! "$ROOT/scripts/run-cargo-deny.sh" --version >/dev/null 2>&1; then
  echo "ENV ERROR: cargo-deny could not be prepared in the Rust container" >&2
  exit 2
fi
AILAB_RUST_WORKDIR=/src/.cache/supply-chain-cargo "$ROOT/scripts/run-cargo-deny.sh" check licenses >/tmp/ailab-cargo-deny-fixture.log 2>&1
cargo_code=$?
if [ "$cargo_code" -eq 0 ]; then
  fail "cargo-deny accepted the GPL-3.0 fixture"
else
  pass "cargo-deny rejected the disallowed license (exit $cargo_code)"
fi

echo
if [ "$FAIL" -ne 0 ]; then
  echo "SUPPLY CHAIN TESTS: FAILED ($PASS passed)"
  exit 1
fi
echo "SUPPLY CHAIN TESTS: OK ($PASS negative fixtures rejected)"
exit 0
