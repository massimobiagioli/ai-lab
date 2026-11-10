#!/usr/bin/env bash
# Generate or validate the local master key and runner token in .env.
# The command runs in the pinned Node container; host openssl/node are never used.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
exec "$ROOT/scripts/run-node.sh" scripts/generate-key.mjs
