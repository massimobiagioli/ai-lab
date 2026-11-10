#!/usr/bin/env bash
# Run the BDD suite helper with host Node or the pinned Node container.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if command -v node >/dev/null 2>&1; then
  exec node "$ROOT/scripts/bdd-suites.mjs" "$@"
fi

exec docker run --rm --user "$(id -u):$(id -g)" -e HOME=/tmp \
  -v "$ROOT:/app" -w /app node:24.22.0-trixie-slim node scripts/bdd-suites.mjs "$@"
