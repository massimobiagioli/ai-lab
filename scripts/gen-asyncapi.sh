#!/usr/bin/env bash
# Generate the AsyncAPI contract from the domain EVENT_CATALOG in Docker.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
exec "$ROOT/scripts/run-python.sh" .tools/gen-asyncapi.py --root .
