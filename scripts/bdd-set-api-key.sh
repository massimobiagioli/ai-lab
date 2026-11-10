#!/usr/bin/env bash
# Load the DeepSeek API key into the prepared AI-Lab stack.
#
# Usage: make bdd-set-api-key
#
# The key is read from a hidden prompt, validated, and sent to the provider API.
# It is never accepted as an argument, written to disk or echoed. The command
# creates the DeepSeek provider if it does not exist and rotates the key if it
# does.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
NODE="$ROOT/scripts/run-node.sh"

BASE="${AILAB_BDD_BASE_URL:-http://127.0.0.1:8080}"
BASE="${BASE%/}"
KEY=""

usage() {
  cat <<'EOF'
Usage: make bdd-set-api-key

Reads a DeepSeek API key from a hidden prompt, validates it, and creates or
rotates the DeepSeek provider in the prepared AI-Lab stack.

Rules:
  - the key must start with "sk-" and be 20 to 256 characters long;
  - the key is never accepted as a command-line argument or environment variable;
  - the key is never written to disk, logs or the terminal;
  - the stack must already be running and reachable.

Exit codes:
  0  provider created or rotated
  1  invalid key, unreachable stack or provider API error
  2  usage error
EOF
}

fail() {
  echo "bdd-set-api-key: $1" >&2
  exit 1
}

if [ "${1:-}" = "--help" ] || [ "${1:-}" = "-h" ]; then
  usage
  exit 0
fi
if [ "$#" -ne 0 ]; then
  usage >&2
  exit 2
fi

cleanup() {
  unset KEY
}
trap cleanup EXIT INT TERM

reachable() {
  local code
  code="$(curl -sS -o /dev/null -w '%{http_code}' --max-time 5 "$BASE/healthz" 2>/dev/null || true)"
  if [ "$code" = "200" ]; then return 0; fi
  code="$(curl -sS -o /dev/null -w '%{http_code}' --max-time 5 "$BASE/api/v1/health/live" 2>/dev/null || true)"
  [ "$code" = "200" ]
}

curl_json() {
  if [ -n "${AILAB_API_TOKEN:-}" ]; then
    curl -sS --max-time 20 -H 'accept: application/json' -H 'content-type: application/json' \
      -H "Authorization: Bearer ${AILAB_API_TOKEN}" "$@"
  else
    curl -sS --max-time 20 -H 'accept: application/json' -H 'content-type: application/json' "$@"
  fi
}

curl_write_status() {
  if [ -n "${AILAB_API_TOKEN:-}" ]; then
    curl -sS --max-time 20 -w '\n%{http_code}' -H 'accept: application/json' -H 'content-type: application/json' \
      -H "Authorization: Bearer ${AILAB_API_TOKEN}" "$@"
  else
    curl -sS --max-time 20 -w '\n%{http_code}' -H 'accept: application/json' -H 'content-type: application/json' "$@"
  fi
}

if ! reachable; then
  fail "the prepared stack is not reachable at $BASE; start it and try again"
fi

printf 'DeepSeek API key: ' >&2
IFS= read -rs KEY || fail "no key provided"
printf '\n' >&2

if [ -z "$KEY" ]; then
  fail "the key is empty"
fi
case "$KEY" in
  sk-*) ;;
  *) fail "the key must start with sk-" ;;
esac
if [ "${#KEY}" -lt 20 ] || [ "${#KEY}" -gt 256 ]; then
  fail "the key must be between 20 and 256 characters"
fi
if [[ "$KEY" =~ [[:space:]] ]]; then
  fail "the key must not contain whitespace"
fi

LAST4="${KEY: -4}"

PROVIDERS="$(curl_json -f "$BASE/api/v1/providers")" || fail "could not list providers"
PROVIDER_ID="$(printf '%s' "$PROVIDERS" | "$NODE" -e '
let input = "";
process.stdin.setEncoding("utf8");
process.stdin.on("data", (chunk) => { input += chunk; });
process.stdin.on("end", () => {
  let data = {};
  try { data = JSON.parse(input); } catch {}
  const items = Array.isArray(data) ? data : (data.data || []);
  const provider = items.find((item) => item && item.kind === "deepseek");
  process.stdout.write(provider ? String(provider.id || "") : "");
});
' 2>/dev/null)" || fail "could not parse the provider list"

if [ -z "$PROVIDER_ID" ]; then
  RAW="$(printf '%s' "$KEY" | "$NODE" -e '
let input = "";
process.stdin.setEncoding("utf8");
process.stdin.on("data", (chunk) => { input += chunk; });
process.stdin.on("end", () => {
  const key = input.trim();
  process.stdout.write(JSON.stringify({
    name: "DeepSeek",
    kind: "deepseek",
    base_url: "https://api.deepseek.com",
    api_key: key,
    rate_limit_rpm: 60,
    rate_limit_tpm: 60000,
    timeout_ms: 120000,
  }));
});
' | curl_write_status -X POST "$BASE/api/v1/providers" --data-binary @-)" || fail "could not create the provider"
  HTTP_STATUS="${RAW##*$'\n'}"
  RESPONSE="${RAW%$'\n'*}"
  [[ "$HTTP_STATUS" == 2* ]] || fail "provider API returned HTTP ${HTTP_STATUS}"
  ACTION="created"
else
  RAW="$(printf '%s' "$KEY" | "$NODE" -e '
let input = "";
process.stdin.setEncoding("utf8");
process.stdin.on("data", (chunk) => { input += chunk; });
process.stdin.on("end", () => {
  process.stdout.write(JSON.stringify({ api_key: input.trim() }));
});
' | curl_write_status -X POST "$BASE/api/v1/providers/${PROVIDER_ID}/rotate-key" --data-binary @-)" || fail "could not rotate the provider key"
  HTTP_STATUS="${RAW##*$'\n'}"
  RESPONSE="${RAW%$'\n'*}"
  [[ "$HTTP_STATUS" == 2* ]] || fail "provider API returned HTTP ${HTTP_STATUS}"
  ACTION="rotated"
fi

unset KEY

PARSED="$(printf '%s' "$RESPONSE" | "$NODE" -e '
let input = "";
process.stdin.setEncoding("utf8");
process.stdin.on("data", (chunk) => { input += chunk; });
process.stdin.on("end", () => {
  let data = {};
  try { data = JSON.parse(input); } catch {}
  process.stdout.write(`${data.status || "unknown"}\n${data.api_key_last4 || data.last4 || "????"}`);
});
' 2>/dev/null)" || fail "could not parse the provider response"

STATUS="$(printf '%s\n' "$PARSED" | sed -n '1p')"
LAST4_RESPONSE="$(printf '%s\n' "$PARSED" | sed -n '2p')"
[ -n "$STATUS" ] || STATUS="unknown"
[ -n "$LAST4_RESPONSE" ] || LAST4_RESPONSE="$LAST4"

echo "DeepSeek provider ${ACTION}: status=${STATUS} last4=${LAST4_RESPONSE}"
