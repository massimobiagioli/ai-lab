#!/usr/bin/env bash
# Run cargo-deny in the pinned Rust container, installing it into the Docker
# cargo cache on first use. Host cargo is never used.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if ! "$ROOT/scripts/run-rust.sh" cargo deny --version >/dev/null 2>&1; then
  echo "run-cargo-deny: installing cargo-deny into the Docker cargo cache ..." >&2
  "$ROOT/scripts/run-rust.sh" cargo install cargo-deny --locked || exit $?
fi

exec "$ROOT/scripts/run-rust.sh" cargo deny "$@"
