#!/usr/bin/env bash
# Run Node in the pinned Docker image. Host node is never used.
#
# Usage: scripts/run-node.sh scripts/bdd-suites.mjs --list
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
IMAGE="${AILAB_NODE_IMAGE:-node:24.22.0-trixie-slim}"

if ! command -v docker >/dev/null 2>&1; then
  echo "run-node: docker is required" >&2
  exit 2
fi
if ! docker info >/dev/null 2>&1; then
  echo "run-node: docker daemon is not reachable" >&2
  exit 2
fi
if ! docker image inspect "$IMAGE" >/dev/null 2>&1; then
  echo "run-node: pulling $IMAGE ..." >&2
  if ! docker pull "$IMAGE" >/dev/null 2>&1; then
    echo "run-node: could not pull $IMAGE" >&2
    exit 2
  fi
fi

exec docker run --rm -i --user "$(id -u):$(id -g)" \
  -e HOME=/tmp \
  -e NG_CLI_ANALYTICS=false \
  -v "$ROOT:/src" \
  -w /src \
  "$IMAGE" node "$@"
