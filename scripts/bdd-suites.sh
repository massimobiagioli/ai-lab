#!/usr/bin/env bash
# Run the BDD suite helper in the pinned Node container. Host node is never used.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
exec "$ROOT/scripts/run-node.sh" scripts/bdd-suites.mjs "$@"
