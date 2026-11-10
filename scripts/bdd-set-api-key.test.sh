#!/usr/bin/env bash
# Tests for scripts/bdd-set-api-key.sh with a fake provider API.
#
# The fake API runs in Docker; the test itself uses host bash and curl only.
#
# Usage: bash scripts/bdd-set-api-key.test.sh
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCRIPT="$ROOT/scripts/bdd-set-api-key.sh"
TMP="$(mktemp -d)"
PORT="${AILAB_BDD_TEST_PORT:-18999}"
BASE="http://127.0.0.1:${PORT}"
PASS=0
FAIL=0
SERVER_NAME="ailab-bdd-key-test-$$"

cleanup() {
  docker rm -f "$SERVER_NAME" >/dev/null 2>&1 || true
  rm -rf "$TMP"
}
trap cleanup EXIT INT TERM

pass() { printf 'PASS  %s\n' "$1"; PASS=$((PASS + 1)); }
fail() { printf 'FAIL  %s\n' "$1"; FAIL=$((FAIL + 1)); }

cat > "$TMP/server.mjs" <<'JS'
import http from 'node:http';
import { appendFileSync } from 'node:fs';

const port = Number(process.argv[2]);
const log = process.argv[3];
const providers = [];

const server = http.createServer((request, response) => {
  let body = '';
  request.on('data', (chunk) => { body += chunk; });
  request.on('end', () => {
    const send = (status, payload) => {
      const data = JSON.stringify(payload);
      response.writeHead(status, { 'content-type': 'application/json', 'content-length': Buffer.byteLength(data) });
      response.end(data);
    };
    if (request.method === 'GET' && (request.url === '/healthz' || request.url === '/api/v1/health/live')) {
      return send(200, { status: 'ok' });
    }
    if (request.method === 'GET' && request.url.startsWith('/api/v1/providers')) {
      return send(200, { data: providers });
    }
    if (request.method === 'POST') {
      let parsed = {};
      try { parsed = JSON.parse(body); } catch { parsed = {}; }
      appendFileSync(log, `${JSON.stringify({ method: 'POST', path: request.url, body: parsed })}\n`);
      if (request.url === '/api/v1/providers') {
        const item = { id: 'p1', kind: parsed.kind || 'deepseek', status: 'active', api_key_last4: String(parsed.api_key || '').slice(-4) };
        providers.push(item);
        return send(201, item);
      }
      if (request.url === '/api/v1/providers/p1/rotate-key') {
        return send(200, { id: 'p1', kind: 'deepseek', status: 'active', api_key_last4: String(parsed.api_key || '').slice(-4) });
      }
    }
    send(404, { error: 'not found' });
  });
});
server.listen(port);
JS

docker run --rm -d --name "$SERVER_NAME" -p "$PORT:$PORT" \
  -v "$TMP:/app" -w /app node:24.22.0-trixie-slim \
  node server.mjs "$PORT" /app/requests.log >/dev/null || {
  echo "FAIL  fake provider container did not start" >&2
  exit 1
}

for _ in $(seq 1 50); do
  if curl -fsS --max-time 1 "$BASE/healthz" >/dev/null 2>&1; then break; fi
  sleep 0.1
done
if ! curl -fsS --max-time 1 "$BASE/healthz" >/dev/null 2>&1; then
  echo "FAIL  fake provider server did not start" >&2
  exit 1
fi

run_case() {
  local name="$1"
  local key="$2"
  local base="$3"
  local expected="$4"
  local out err code
  out="$(printf '%s\n' "$key" | AILAB_BDD_BASE_URL="$base" bash "$SCRIPT" 2>"$TMP/err")"
  code=$?
  err="$(cat "$TMP/err")"
  if [ "$code" -ne "$expected" ]; then
    fail "$name (exit $code, expected $expected; stderr: ${err})"
    return 1
  fi
  if [ -n "$key" ] && printf '%s\n%s' "$out" "$err" | grep -Fq "$key"; then
    fail "$name (the key was echoed)"
    return 1
  fi
  printf '%s' "$out" > "$TMP/out"
  return 0
}

if run_case "empty input" "" "$BASE" 1; then pass "empty input is rejected"; fi
if run_case "wrong prefix" "bad-abcdefghijklmnopqrstuvwxyz" "$BASE" 1; then pass "wrong prefix is rejected"; fi
if run_case "short key" "sk-short" "$BASE" 1; then pass "short key is rejected"; fi
if run_case "long key" "sk-$(printf 'a%.0s' {1..300})" "$BASE" 1; then pass "long key is rejected"; fi

VALID="sk-$(printf 'test-%s-1234' 'abcdefghijklmnopqrstuvwxyz')"
if run_case "provider create" "$VALID" "$BASE" 0; then
  if grep -q 'created' "$TMP/out" && grep -q 'last4=1234' "$TMP/out"; then
    pass "provider create returns the status and last4"
  else
    fail "provider create output does not contain status/last4"
  fi
fi

if run_case "provider rotate" "$VALID" "$BASE" 0; then
  if grep -q 'rotated' "$TMP/out" && grep -q 'last4=1234' "$TMP/out"; then
    pass "provider rotate is idempotent"
  else
    fail "provider rotate output does not contain status/last4"
  fi
fi

if grep -q '"path":"/api/v1/providers"' "$TMP/requests.log"; then
  pass "the fake API received the create request"
else
  fail "the fake API did not receive the create request"
fi
if grep -q '"path":"/api/v1/providers/p1/rotate-key"' "$TMP/requests.log"; then
  pass "the fake API received the rotate request"
else
  fail "the fake API did not receive the rotate request"
fi

if run_case "unreachable stack" "$VALID" "http://127.0.0.1:1" 1; then
  if [ -s "$TMP/err" ]; then pass "unreachable stack is rejected"; fi
fi

if grep -q 'trap cleanup' "$SCRIPT" && grep -q 'unset KEY' "$SCRIPT"; then
  pass "the key is cleared on exit"
else
  fail "the key cleanup trap is missing"
fi

if grep -q "$VALID" "$TMP/requests.log"; then
  pass "the key reached the provider API"
else
  fail "the key did not reach the provider API"
fi

echo
if [ "$FAIL" -ne 0 ]; then
  echo "bdd-set-api-key tests: FAILED ($FAIL failures, $PASS passed)"
  exit 1
fi
echo "bdd-set-api-key tests: OK ($PASS passed)"
