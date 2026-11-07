#!/usr/bin/env bash
# Run a command in the web workspace.
# Uses the host npm when available, otherwise the pinned Node 24 container.
#
# Usage: scripts/run-web.sh npm run lint
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WEB="$ROOT/web"

if [ ! -d "$WEB" ]; then
  echo "run-web: $WEB does not exist yet (WP-17 owns the app)" >&2
  exit 2
fi
if [ "$#" -eq 0 ]; then
  echo "run-web: no command given" >&2
  exit 2
fi

if command -v npm >/dev/null 2>&1; then
  cd "$WEB" || exit 2
  exec "$@"
fi

IMAGE="${AILAB_NODE_IMAGE:-node:24.22.0-trixie-slim}"
if ! docker image inspect "$IMAGE" >/dev/null 2>&1; then
  echo "run-web: npm is not installed and $IMAGE is not available" >&2
  exit 2
fi

exec docker run --rm \
  --user "$(id -u):$(id -g)" \
  -e HOME=/tmp \
  -e NG_CLI_ANALYTICS=false \
  -v "$WEB:/app" \
  -w /app \
  "$IMAGE" "$@"
