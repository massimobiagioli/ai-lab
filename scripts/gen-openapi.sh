#!/usr/bin/env bash
# Copy the canonical OpenAPI source to the generated contract.
# The server-side generator (utoipa) is owned by the WP-16 follow-up.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SRC="$ROOT/docs/api/openapi.source.json"
OUT="$ROOT/docs/api/openapi.json"
[ -f "$SRC" ] || { echo "gen-openapi: missing $SRC" >&2; exit 1; }
cp "$SRC" "$OUT"
echo "gen-openapi: wrote docs/api/openapi.json"
