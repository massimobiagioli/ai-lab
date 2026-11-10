#!/usr/bin/env bash
# Run a command in the web workspace inside Docker. Host npm/node are never used.
#
# Usage: scripts/run-web.sh npm run lint
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WEB="$ROOT/web"
IMAGE="${AILAB_NODE_IMAGE:-node:24.22.0-trixie-slim}"
NPM_CACHE="$ROOT/.cache/npm"

if [ ! -d "$WEB" ]; then
  echo "run-web: $WEB does not exist" >&2
  exit 2
fi
if [ "$#" -eq 0 ]; then
  echo "run-web: no command given" >&2
  exit 2
fi
if ! command -v docker >/dev/null 2>&1; then
  echo "run-web: docker is required" >&2
  exit 2
fi
if ! docker info >/dev/null 2>&1; then
  echo "run-web: docker daemon is not reachable" >&2
  exit 2
fi
if ! docker image inspect "$IMAGE" >/dev/null 2>&1; then
  echo "run-web: pulling $IMAGE ..." >&2
  if ! docker pull "$IMAGE" >/dev/null 2>&1; then
    echo "run-web: could not pull $IMAGE" >&2
    exit 2
  fi
fi

mkdir -p "$NPM_CACHE"
exec docker run --rm --user "$(id -u):$(id -g)" \
  -e HOME=/tmp \
  -e NG_CLI_ANALYTICS=false \
  -e npm_config_cache=/tmp/npm-cache \
  -v "$WEB:/app" \
  -v "$NPM_CACHE:/tmp/npm-cache" \
  -w /app \
  "$IMAGE" "$@"
