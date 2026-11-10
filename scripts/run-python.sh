#!/usr/bin/env bash
# Run Python in the pinned Docker image. Host python3 is never used.
#
# Usage: scripts/run-python.sh .tools/gen-asyncapi.py --root .
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
IMAGE="${AILAB_PYTHON_IMAGE:-python:3.12.6-slim-bookworm}"

if ! command -v docker >/dev/null 2>&1; then
  echo "run-python: docker is required" >&2
  exit 2
fi
if ! docker info >/dev/null 2>&1; then
  echo "run-python: docker daemon is not reachable" >&2
  exit 2
fi
if ! docker image inspect "$IMAGE" >/dev/null 2>&1; then
  echo "run-python: pulling $IMAGE ..." >&2
  if ! docker pull "$IMAGE" >/dev/null 2>&1; then
    echo "run-python: could not pull $IMAGE" >&2
    exit 2
  fi
fi

exec docker run --rm -i --user "$(id -u):$(id -g)" \
  -e HOME=/tmp \
  -v "$ROOT:/src" \
  -w /src \
  "$IMAGE" python "$@"
