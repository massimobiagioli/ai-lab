#!/usr/bin/env bash
# Self-test for scripts/sow-lint.sh. Builds disposable SOW trees and asserts the
# linter's exit code for each violation class. No network, no database.
#
# Usage: .tools/sow-lint-self-test.sh
# Exit: 0 all cases behave as expected, 1 a case failed, 2 environment issue.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LINT="$ROOT/scripts/sow-lint.sh"

if [ ! -x "$LINT" ]; then
  echo "RED: scripts/sow-lint.sh is missing or not executable" >&2
  exit 1
fi

TMP="$(mktemp -d "${TMPDIR:-/tmp}/sow-lint-test.XXXXXX")"
trap 'rm -rf "$TMP"' EXIT

PASS=0
FAIL=0

write_common() {
  local d="$1"
  mkdir -p "$d/.docs/sow" "$d/.docs/adr" "$d/docs/change-requests" "$d/.github/workflows"
  cat > "$d/.github/required-checks.txt" <<'EOF'
fmt
sow-lint
EOF
  cat > "$d/.github/workflows/ci.yml" <<'EOF'
name: CI
on: [push, pull_request]
jobs:
  fmt:
    runs-on: ubuntu-latest
    steps:
      - run: echo fmt
  sow-lint:
    runs-on: ubuntu-latest
    steps:
      - run: echo sow-lint
EOF
  cat > "$d/docs/change-requests/CR-0001.md" <<'EOF'
# CR-0001 — Valid

| Field | Value |
|---|---|
| id | `CR-0001` |
| title | Valid |
| decision | `approved` |
| approver | Tech Lead, 2026-11-07 |
| date | 2026-11-07 |
| linked_adr | — |

## context
c

## proposed_change
p

## impact_scope
s

## impact_schedule
n

## impact_cost
n

## impact_risk
r

## alternatives
a

## decision
approved

## approver
Tech Lead

## linked_adr
—
EOF
  cat > "$d/docs/change-requests/README.md" <<'EOF'
# CR register

| CR | Title | Decision | Approver | Date | Linked ADR |
|---|---|---|---|---|---|
| [CR-0001](./CR-0001.md) | Valid | approved | Tech Lead | 2026-11-07 | — |
EOF
}

write_wp() {
  local d="$1" id="$2" status="$3" depends="$4" log="$5" deliverable="$6"
  local file="$d/.docs/sow/${id}-TEST.md"
  cat > "$file" <<EOF
# ${id} — Test package

| Field | Value |
|---|---|
| WP ID | \`${id}\` |
| Status | **${status}** |
| Owner | Dev |
| Milestone | M0 |
| Estimate | 1 points |
| Depends on | ${depends} |
| Blocks | — |
| Requirement traceability | TEST-001 |
| Version | 1.0.0 |

**Status history**

| Date | From | To | Actor | Reason |
|---|---|---|---|---|
| 2026-11-01 | — | \`ready\` | Dev | initial |
| 2026-11-02 | \`ready\` | \`${status}\` | Dev | test fixture |

## 1. Objective

Fixture.

## 2. Scope

### 2.1 In scope

- Fixture.

### 2.2 Out of scope

- Nothing.

## 3. Engineering specification

Fixture.

## 4. Deliverables (checklist)

${deliverable}

## 5. Task breakdown

| Task | Description | Role | Est | Depends on | Acceptance |
|---|---|---|---|---|---|
| ${id}-T01 | Fixture | Dev | 1 | — | Pass |

## 6. Acceptance criteria

1. Pass.

## 7. WP Definition of Done

- [x] Pass.

## 8. Risks and mitigations

| Risk | Impact | Likelihood | Mitigation | Owner |
|---|---|---|---|---|
| None | Low | Low | None | Dev |

## 9. References

- Fixture.

## Progress log

${log}
EOF
}

write_valid() {
  local d="$1" s0="${2:-ready}" s1="${3:-ready}"
  write_common "$d"
  cat > "$d/.docs/sow/SOW.md" <<EOF
# SOW

| WP | Title | Status | Owner | Milestone | Est | Depends on | Primary outputs |
|---|---|---|---|---|---|---|---|
| [WP-00](./WP-00-TEST.md) | First | **${s0}** | Dev | M0 | 1 | — | x |
| [WP-01](./WP-01-TEST.md) | Second | **${s1}** | Dev | M0 | 1 | WP-00 | x |
EOF
  write_wp "$d" WP-00 "$s0" "—" "2026-11-07 | T01 | files | tests | none | next" "- [x] Delivered."
  write_wp "$d" WP-01 "$s1" "WP-00" "2026-11-07 | T01 | files | tests | none | next" "- [x] Delivered."
}

run_case() {
  local name="$1" expected="$2" dir="$3"
  "$LINT" --root "$dir" >/dev/null 2>&1
  local code=$?
  if [ "$expected" = "zero" ]; then
    if [ "$code" -eq 0 ]; then
      printf 'PASS  %s (exit 0)\n' "$name"; PASS=$((PASS+1))
    else
      printf 'FAIL  %s expected exit 0, got %s\n' "$name" "$code"; FAIL=$((FAIL+1))
    fi
  else
    if [ "$code" -ne 0 ]; then
      printf 'PASS  %s (exit %s)\n' "$name" "$code"; PASS=$((PASS+1))
    else
      printf 'FAIL  %s expected non-zero, got 0\n' "$name"; FAIL=$((FAIL+1))
    fi
  fi
}

# 1. Valid tree.
d="$TMP/valid"; write_valid "$d"; run_case "valid tree" zero "$d"

# 2. Invalid status.
d="$TMP/bad-status"; write_valid "$d"; sed -i.bak 's/| Status | \*\*ready\*\* |/| Status | **shipped** |/' "$d/.docs/sow/WP-00-TEST.md"; rm -f "$d/.docs/sow/WP-00-TEST.md.bak"; sed -i.bak 's/| First | \*\*ready\*\* |/| First | **shipped** |/' "$d/.docs/sow/SOW.md"; rm -f "$d/.docs/sow/SOW.md.bak"; run_case "invalid status" nonzero "$d"

# 3. Index/WP status mismatch.
d="$TMP/mismatch"; write_valid "$d"; sed -i.bak 's/| Second | \*\*ready\*\* |/| Second | **done** |/' "$d/.docs/sow/SOW.md"; rm -f "$d/.docs/sow/SOW.md.bak"; run_case "index status mismatch" nonzero "$d"

# 4. Missing dependency.
d="$TMP/missing-dep"; write_valid "$d"; sed -i.bak 's/| Depends on | WP-00 |/| Depends on | WP-99 |/' "$d/.docs/sow/WP-01-TEST.md"; rm -f "$d/.docs/sow/WP-01-TEST.md.bak"; run_case "missing dependency" nonzero "$d"

# 5. Cyclic dependency.
d="$TMP/cycle"; write_valid "$d"; sed -i.bak 's/| Depends on | — |/| Depends on | WP-01 |/' "$d/.docs/sow/WP-00-TEST.md"; rm -f "$d/.docs/sow/WP-00-TEST.md.bak"; run_case "cyclic dependency" nonzero "$d"

# 6. Done WP with an unchecked deliverable.
d="$TMP/done-unchecked"; write_valid "$d" done done; sed -i.bak 's/- \[x\] Delivered\./- [ ] Delivered./' "$d/.docs/sow/WP-00-TEST.md"; rm -f "$d/.docs/sow/WP-00-TEST.md.bak"; run_case "done with unchecked deliverable" nonzero "$d"

# 7. Progress WP with a stale progress log.
d="$TMP/stale-log"; write_valid "$d" progress progress; sed -i.bak 's/2026-11-07 | T01/2020-01-01 | T01/' "$d/.docs/sow/WP-00-TEST.md"; rm -f "$d/.docs/sow/WP-00-TEST.md.bak"; run_case "progress with stale log" nonzero "$d"

# 8. Broken index link.
d="$TMP/broken-link"; write_valid "$d"; sed -i.bak 's|(./WP-01-TEST.md)|(./WP-01-MISSING.md)|' "$d/.docs/sow/SOW.md"; rm -f "$d/.docs/sow/SOW.md.bak"; run_case "broken index link" nonzero "$d"

# 9. Incomplete CR.
d="$TMP/bad-cr"; write_valid "$d"; sed -i.bak '/## impact_cost/,+2d' "$d/docs/change-requests/CR-0001.md"; rm -f "$d/docs/change-requests/CR-0001.md.bak"; run_case "incomplete CR" nonzero "$d"

# 10. Required check missing from workflow.
d="$TMP/missing-check"; write_valid "$d"; sed -i.bak '/^  sow-lint:/,+2d' "$d/.github/workflows/ci.yml"; rm -f "$d/.github/workflows/ci.yml.bak"; run_case "required check missing" nonzero "$d"

# 11. Missing required heading.
d="$TMP/missing-heading"; write_valid "$d"; sed -i.bak 's/^## 8. Risks and mitigations$/## 8. Something else/' "$d/.docs/sow/WP-00-TEST.md"; rm -f "$d/.docs/sow/WP-00-TEST.md.bak"; run_case "missing required heading" nonzero "$d"

echo
if [ "$FAIL" -ne 0 ]; then
  echo "SOW-LINT SELF-TEST: FAILED ($PASS passed, $FAIL failed)"
  exit 1
fi
echo "SOW-LINT SELF-TEST: OK ($PASS cases)"
exit 0
