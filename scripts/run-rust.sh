#!/usr/bin/env bash
# Run a Rust command in the pinned Docker image. Host cargo/rustc are never used.
#
# Usage: scripts/run-rust.sh cargo test --workspace
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
IMAGE="${AILAB_RUST_IMAGE:-rust:1.99.1-trixie@sha256:8a4e90c7946b74626488b1446f712c0fe0ed1a596cbd3d871684bc9c5433ae07}"
CACHE="$ROOT/.cache/cargo"
WORKDIR="${AILAB_RUST_WORKDIR:-/src}"
TARGET_VOLUME="${AILAB_CARGO_TARGET_VOLUME:-ailab-cargo-target}"

if ! command -v docker >/dev/null 2>&1; then
  echo "run-rust: docker is required" >&2
  exit 2
fi
if ! docker info >/dev/null 2>&1; then
  echo "run-rust: docker daemon is not reachable" >&2
  exit 2
fi
if ! docker image inspect "$IMAGE" >/dev/null 2>&1; then
  echo "run-rust: pulling $IMAGE ..." >&2
  if ! docker pull "$IMAGE" >/dev/null 2>&1; then
    echo "run-rust: could not pull $IMAGE" >&2
    exit 2
  fi
fi

mkdir -p "$CACHE"
docker volume create "$TARGET_VOLUME" >/dev/null
docker run --rm --user root -v "$TARGET_VOLUME:/target" "$IMAGE" chown "$(id -u):$(id -g)" /target >/dev/null
exec docker run --rm --user "$(id -u):$(id -g)" \
  -e HOME=/tmp \
  -e CARGO_HOME=/tmp/cargo \
  -e PATH=/tmp/cargo/bin:/usr/local/cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin \
  -v "$ROOT:/src" \
  -v "$CACHE:/tmp/cargo" \
  -v "$TARGET_VOLUME:/src/target" \
  -w "$WORKDIR" \
  "$IMAGE" "$@"
