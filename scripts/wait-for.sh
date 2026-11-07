#!/usr/bin/env bash
# Wait for a TCP host:port to accept connections.
# Usage: scripts/wait-for.sh HOST PORT [TIMEOUT_SECONDS]
set -uo pipefail

HOST="${1:?usage: wait-for.sh HOST PORT [TIMEOUT_SECONDS]}"
PORT="${2:?usage: wait-for.sh HOST PORT [TIMEOUT_SECONDS]}"
TIMEOUT="${3:-60}"

start="$(date +%s)"
while :; do
  if command -v nc >/dev/null 2>&1; then
    nc -z "$HOST" "$PORT" >/dev/null 2>&1 && { echo "ready: $HOST:$PORT"; exit 0; }
  else
    (exec 3<>"/dev/tcp/$HOST/$PORT") >/dev/null 2>&1 && { echo "ready: $HOST:$PORT"; exit 0; }
  fi
  now="$(date +%s)"
  if [ $((now - start)) -ge "$TIMEOUT" ]; then
    echo "timeout: $HOST:$PORT after ${TIMEOUT}s" >&2
    exit 1
  fi
  sleep 1
done
