#!/usr/bin/env bash
# Generate the AsyncAPI contract from the domain EVENT_CATALOG.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
python3 "$ROOT/.tools/gen-asyncapi.py" --root "$ROOT"
