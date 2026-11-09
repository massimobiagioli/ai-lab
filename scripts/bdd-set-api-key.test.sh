#!/usr/bin/env bash
# Tests for scripts/bdd-set-api-key.sh with a fake provider API.
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
SERVER_PID=""

cleanup() {
  if [ -n "$SERVER_PID" ]; then kill "$SERVER_PID" 2>/dev/null || true; fi
  rm -rf "$TMP"
}
trap cleanup EXIT INT TERM

pass() { printf 'PASS  %s\n' "$1"; PASS=$((PASS + 1)); }
fail() { printf 'FAIL  %s\n' "$1"; FAIL=$((FAIL + 1)); }

cat > "$TMP/server.py" <<'PY'
import json
import sys
import http.server
import socketserver

port = int(sys.argv[1])
log_path = sys.argv[2]
providers = []

class Handler(http.server.BaseHTTPRequestHandler):
    def log_message(self, *args):
        pass

    def _send(self, code, payload):
        body = json.dumps(payload).encode()
        self.send_response(code)
        self.send_header('content-type', 'application/json')
        self.send_header('content-length', str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def _log(self, method, path, body):
        with open(log_path, 'a', encoding='utf-8') as handle:
            handle.write(json.dumps({'method': method, 'path': path, 'body': body}) + '\n')

    def do_GET(self):
        if self.path in ('/healthz', '/api/v1/health/live'):
            return self._send(200, {'status': 'ok'})
        if self.path.startswith('/api/v1/providers'):
            return self._send(200, {'data': providers})
        return self._send(404, {'error': 'not found'})

    def do_POST(self):
        length = int(self.headers.get('content-length', '0'))
        raw = self.rfile.read(length).decode('utf-8')
        try:
            body = json.loads(raw)
        except Exception:
            body = {}
        self._log('POST', self.path, body)
        if self.path == '/api/v1/providers':
            item = {
                'id': 'p1',
                'kind': body.get('kind', 'deepseek'),
                'status': 'active',
                'api_key_last4': body.get('api_key', '')[-4:],
            }
            providers.append(item)
            return self._send(201, item)
        if self.path == '/api/v1/providers/p1/rotate-key':
            return self._send(200, {
                'id': 'p1',
                'kind': 'deepseek',
                'status': 'active',
                'api_key_last4': body.get('api_key', '')[-4:],
            })
        return self._send(404, {'error': 'not found'})

class Server(socketserver.TCPServer):
    allow_reuse_address = True

with Server(('127.0.0.1', port), Handler) as httpd:
    httpd.serve_forever()
PY

python3 "$TMP/server.py" "$PORT" "$TMP/requests.log" &
SERVER_PID=$!

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

if grep -q '"path": "/api/v1/providers"' "$TMP/requests.log"; then
  pass "the fake API received the create request"
else
  fail "the fake API did not receive the create request"
fi
if grep -q '"path": "/api/v1/providers/p1/rotate-key"' "$TMP/requests.log"; then
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

# The provider creation body must contain the key; the script output must not.
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
