#!/usr/bin/env bash
# Self-test for .tools/check-architecture-fitness.sh. Builds disposable roots.
#
# Usage: .tools/architecture-fitness-self-test.sh
# Exit: 0 all cases behave as expected, 1 a case failed, 2 environment issue.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CHECK="$ROOT/.tools/check-architecture-fitness.sh"
[ -x "$CHECK" ] || { echo "RED: $CHECK missing or not executable" >&2; exit 1; }

TMP="$(mktemp -d "${TMPDIR:-/tmp}/arch-fitness-test.XXXXXX")"
trap 'rm -rf "$TMP"' EXIT

PASS=0
FAIL=0

write_base() {
  local d="$1"
  mkdir -p "$d/.tools/architecture" "$d/docs"
  cat > "$d/.tools/architecture/contexts.tsv" <<'EOF'
lab	Lab	labs
inventory	Inventory	agents,projects
kanban	Kanban	cards
delivery	Delivery	runs
EOF
  cat > "$d/.tools/architecture/dependencies.tsv" <<'EOF'
delivery	kanban
kanban	inventory
EOF
  cat > "$d/.tools/architecture/paths.tsv" <<'EOF'
crates/infrastructure/src/delivery	delivery
crates/infrastructure/src/kanban	kanban
EOF
  cat > "$d/docs/service-catalog.md" <<'EOF'
# Catalog

| Service | Owner |
|---|---|
| `api` | WP-16 |
| `web` | WP-17 |
EOF
}

run_case() {
  local name="$1" expected="$2" dir="$3"
  "$CHECK" --root "$dir" >/dev/null 2>&1
  local code=$?
  if [ "$expected" = "zero" ]; then
    if [ "$code" -eq 0 ]; then
      printf 'PASS  %s (exit 0)\n' "$name"; PASS=$((PASS+1))
    else
      printf 'FAIL  %s expected 0, got %s\n' "$name" "$code"; FAIL=$((FAIL+1))
    fi
  else
    if [ "$code" -ne 0 ]; then
      printf 'PASS  %s (exit %s)\n' "$name" "$code"; PASS=$((PASS+1))
    else
      printf 'FAIL  %s expected non-zero, got 0\n' "$name"; FAIL=$((FAIL+1))
    fi
  fi
}

d="$TMP/valid"; write_base "$d"; run_case "valid manifests" zero "$d"

d="$TMP/dup-table"; write_base "$d"; printf 'inventory\tInventory\tagents,cards\n' >> "$d/.tools/architecture/contexts.tsv"; run_case "duplicate table owner" nonzero "$d"

d="$TMP/unknown-dep"; write_base "$d"; printf 'delivery\tghost\n' >> "$d/.tools/architecture/dependencies.tsv"; run_case "unknown dependency" nonzero "$d"

d="$TMP/cycle"; write_base "$d"; printf 'inventory\tdelivery\n' >> "$d/.tools/architecture/dependencies.tsv"; run_case "dependency cycle" nonzero "$d"

d="$TMP/cross"; write_base "$d"; mkdir -p "$d/crates/delivery/src"; printf 'fn q() { sqlx::query("SELECT * FROM cards"); }\n' > "$d/crates/delivery/src/repo.rs"; run_case "cross-context table reference" nonzero "$d"

d="$TMP/cross-schema"; write_base "$d"; mkdir -p "$d/crates/delivery/src"; printf 'fn q() { sqlx::query("SELECT * FROM public.cards"); }\n' > "$d/crates/delivery/src/repo.rs"; run_case "schema-qualified cross-context reference" nonzero "$d"

d="$TMP/cross-quoted"; write_base "$d"; mkdir -p "$d/crates/delivery/src"; printf 'fn q() { sqlx::query("SELECT * FROM \\"cards\\""); }\n' > "$d/crates/delivery/src/repo.rs"; run_case "quoted cross-context reference" nonzero "$d"

d="$TMP/same"; write_base "$d"; mkdir -p "$d/crates/delivery/src"; printf 'fn q() { sqlx::query("SELECT * FROM runs"); }\n' > "$d/crates/delivery/src/repo.rs"; run_case "same-context table reference" zero "$d"

d="$TMP/same-multiline"; write_base "$d"; mkdir -p "$d/crates/delivery/src"; printf 'fn q() { sqlx::query("SELECT *\\nFROM public.runs"); }\n' > "$d/crates/delivery/src/repo.rs"; run_case "multi-line same-context reference" zero "$d"

d="$TMP/unattributed"; write_base "$d"; mkdir -p "$d/crates/infrastructure/src"; printf 'fn q() { sqlx::query("SELECT * FROM cards"); }\n' > "$d/crates/infrastructure/src/shared.rs"; run_case "unattributed infrastructure SQL" nonzero "$d"

d="$TMP/unattributed-api"; write_base "$d"; mkdir -p "$d/crates/api/src"; printf 'fn q() { sqlx::query("select * from cards"); }\n' > "$d/crates/api/src/main.rs"; run_case "unattributed lowercase API SQL" nonzero "$d"

d="$TMP/lowercase-same"; write_base "$d"; mkdir -p "$d/crates/delivery/src"; printf 'fn q() { sqlx::query("select * from runs"); }\n' > "$d/crates/delivery/src/repo.rs"; run_case "lowercase same-context reference" zero "$d"

d="$TMP/compose"; write_base "$d"; printf 'services:\n  api:\n    image: example/api\n  postgres:\n    image: example/pg\n' > "$d/compose.yaml"; run_case "compose service missing from catalog" nonzero "$d"

echo
if [ "$FAIL" -ne 0 ]; then
  echo "ARCHITECTURE FITNESS SELF-TEST: FAILED ($PASS passed, $FAIL failed)"
  exit 1
fi
echo "ARCHITECTURE FITNESS SELF-TEST: OK ($PASS cases)"
exit 0
