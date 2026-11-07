#!/usr/bin/env bash
# Copy the canonical AsyncAPI source to the generated contract.
# The server-side generator is owned by the WP-16 follow-up.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SRC="$ROOT/docs/api/asyncapi.source.json"
OUT="$ROOT/docs/api/asyncapi.json"
[ -f "$SRC" ] || { echo "gen-asyncapi: missing $SRC" >&2; exit 1; }
cp "$SRC" "$OUT"
echo "gen-asyncapi: wrote docs/api/asyncapi.json"
