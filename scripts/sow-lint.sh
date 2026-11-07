#!/usr/bin/env bash
# AI-Lab SOW linter.
#
# Validates the work-package contract in .docs/sow: statuses, template structure,
# index/file status agreement, dependency existence and acyclicity, done-package
# checklists and sign-off, progress-log freshness, SOW links, change requests and
# the required CI check names.
#
# Usage: scripts/sow-lint.sh [--root DIR]
#   --root DIR   validate the tree rooted at DIR (default: repository root)
#
# Read-only. No network. Exit 0 when every check passes, 1 on any violation,
# 2 on a usage or environment error.
set -uo pipefail

ROOT=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --root)
      [ "$#" -ge 2 ] || { echo "sow-lint: --root needs a value" >&2; exit 2; }
      ROOT="$2"; shift 2 ;;
    -h|--help)
      sed -n '2,14p' "$0" | sed 's/^# \{0,1\}//'
      exit 0 ;;
    *)
      echo "sow-lint: unknown argument: $1" >&2; exit 2 ;;
  esac
done
if [ -z "$ROOT" ]; then
  ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fi

SOW_DIR="$ROOT/.docs/sow"
SOW_FILE="$SOW_DIR/SOW.md"
CR_DIR="$ROOT/docs/change-requests"
REQ_FILE="$ROOT/.github/required-checks.txt"
WF_FILE="$ROOT/.github/workflows/ci.yml"
GATES_DOC="$ROOT/docs/quality-gates.md"

for p in "$SOW_DIR" "$SOW_FILE"; do
  [ -e "$p" ] || { echo "sow-lint: missing $p" >&2; exit 2; }
done
[ -d "$CR_DIR" ] || { echo "sow-lint: missing $CR_DIR" >&2; exit 2; }

TMPD="$(mktemp -d "${TMPDIR:-/tmp}/sow-lint.XXXXXX")"
trap 'rm -rf "$TMPD"' EXIT

FAILED=0
PASS=0
fail() { printf 'FAIL  %s\n' "$1"; FAILED=1; }
pass() { PASS=$((PASS + 1)); }

FILES_TSV="$TMPD/files.tsv"
INDEX_TSV="$TMPD/index.tsv"
EDGES_FILE="$TMPD/edges.txt"
IDS_FILE="$TMPD/ids.txt"
: > "$FILES_TSV"; : > "$INDEX_TSV"; : > "$EDGES_FILE"; : > "$IDS_FILE"

field_value() {
  awk -F'|' -v label="$2" '
    $0 ~ ("^\\| " label " \\|") {
      v=$3; gsub(/^[[:space:]]+|[[:space:]]+$/, "", v);
      gsub(/[`*]/, "", v); print v; exit
    }' "$1"
}

section_body() {
  awk -v start="$2" '
    $0 == start {f=1; next}
    f && /^## / {f=0}
    f {print}' "$1"
}

status_history() {
  awk '
    $0 == "**Status history**" {f=1; next}
    f && /^---$/ {exit}
    f {print}' "$1"
}

date_minus_one_day() {
  if date -j -v-1d -f "%Y-%m-%d" "$1" +%Y-%m-%d >/dev/null 2>&1; then
    date -j -v-1d -f "%Y-%m-%d" "$1" +%Y-%m-%d
  else
    date -d "$1 -1 day" +%Y-%m-%d
  fi
}

day_of_week() {
  if date -j -f "%Y-%m-%d" "$1" +%u >/dev/null 2>&1; then
    date -j -f "%Y-%m-%d" "$1" +%u
  else
    date -d "$1" +%u
  fi
}

business_days_ago() {
  local d="$1" count=0 w
  while [ "$count" -lt 3 ]; do
    d="$(date_minus_one_day "$d")"
    w="$(day_of_week "$d")"
    if [ "$w" -le 5 ]; then count=$((count + 1)); fi
  done
  printf '%s\n' "$d"
}

# ---------------------------------------------------------------------------
# 1. Per-WP structure, status and header fields
# ---------------------------------------------------------------------------
HEADINGS=(
  '## 1. Objective'
  '## 2. Scope'
  '### 2.1 In scope'
  '### 2.2 Out of scope'
  '## 3. Engineering specification'
  '## 4. Deliverables (checklist)'
  '## 5. Task breakdown'
  '## 6. Acceptance criteria'
  '## 7. WP Definition of Done'
  '## 8. Risks and mitigations'
  '## 9. References'
)
HEADER_FIELDS=( 'WP ID' 'Status' 'Owner' 'Milestone' 'Estimate' 'Depends on' 'Blocks' 'Requirement traceability' 'Version' )

shopt -s nullglob
WP_FILES=( "$SOW_DIR"/WP-*.md )
if [ "${#WP_FILES[@]}" -eq 0 ]; then
  fail "no WP files found in $SOW_DIR"
fi

for f in "${WP_FILES[@]}"; do
  base="$(basename "$f")"
  id="$(printf '%s' "$base" | sed -E 's/^(WP-[0-9]{2})-.*/\1/')"
  if [ "$id" = "$base" ]; then
    fail "invalid WP filename: $base"
    continue
  fi
  printf '%s\n' "$id" >> "$IDS_FILE"

  status="$(field_value "$f" 'Status')"
  case "$status" in
    ready|progress|done) pass ;;
    *) fail "$base: invalid status '$status'" ;;
  esac
  printf '%s\t%s\n' "$id" "$status" >> "$FILES_TSV"

  for hf in "${HEADER_FIELDS[@]}"; do
    grep -qE "^\\| ${hf} \\|" "$f" || fail "$base: missing header field '$hf'"
  done

  prev=0
  for h in "${HEADINGS[@]}"; do
    ln="$(grep -nF "$h" "$f" | head -1 | cut -d: -f1)"
    if [ -z "$ln" ]; then
      fail "$base: missing heading '$h'"
    elif [ "$ln" -lt "$prev" ]; then
      fail "$base: heading out of order '$h'"
    else
      prev="$ln"
    fi
  done

  deps="$(field_value "$f" 'Depends on')"
  if [ -n "$deps" ] && [ "$deps" != "—" ]; then
    for dep in $(printf '%s' "$deps" | grep -oE 'WP-[0-9]{2}' | sort -u); do
      printf '%s %s\n' "$id" "$dep" >> "$EDGES_FILE"
    done
  fi

  if [ "$status" = "done" ]; then
    unchecked_deliv="$(section_body "$f" '## 4. Deliverables (checklist)' | grep -c '^- \[ \]')"
    if [ "$unchecked_deliv" -gt 0 ]; then
      fail "$base: done WP has $unchecked_deliv unchecked deliverable(s)"
    fi
    unchecked_dod="$(section_body "$f" '## 7. WP Definition of Done' | grep -c '^- \[ \]')"
    if [ "$unchecked_dod" -gt 0 ]; then
      fail "$base: done WP has $unchecked_dod unchecked DoD item(s)"
    fi
    if ! status_history "$f" | awk -F'|' '$4 ~ /done/ {found=1} END {exit !found}'; then
      fail "$base: done WP has no status-history row to 'done'"
    fi
  fi

  if [ "$status" = "progress" ]; then
    if ! grep -qF '## Progress log' "$f"; then
      fail "$base: progress WP has no '## Progress log' section"
    else
      latest="$(section_body "$f" '## Progress log' \
        | grep -E '^[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9][[:space:]]*\|' \
        | cut -c1-10 | sort | tail -1)"
      if [ -z "$latest" ]; then
        fail "$base: progress WP has no dated progress-log line"
      else
        threshold="$(business_days_ago "$(date +%Y-%m-%d)")"
        if [[ "$latest" < "$threshold" ]]; then
          fail "$base: progress log stale (latest $latest, threshold $threshold)"
        fi
      fi
    fi
  fi
done
pass "WP structure checked"

# ---------------------------------------------------------------------------
# 2. Index parsing and status agreement
# ---------------------------------------------------------------------------
awk -F'|' '
  /^\| \[WP-[0-9][0-9]\]/ {
    link=$2; gsub(/^[[:space:]]+|[[:space:]]+$/, "", link);
    if (match(link, /WP-[0-9][0-9]/)) {
      id=substr(link, RSTART, RLENGTH);
      status=$4; gsub(/[*`[:space:]]/, "", status);
      print id "\t" status;
    }
  }' "$SOW_FILE" > "$INDEX_TSV"

if [ ! -s "$INDEX_TSV" ]; then
  fail "SOW.md: no WP rows found in the index"
fi

while IFS=$'\t' read -r id idx_status; do
  [ -z "$id" ] && continue
  file_status="$(awk -F'\t' -v k="$id" '$1 == k {print $2; exit}' "$FILES_TSV")"
  if [ -z "$file_status" ]; then
    fail "SOW.md: index references $id but no WP file exists"
  elif [ "$idx_status" != "$file_status" ]; then
    fail "$id: index status '$idx_status' != WP status '$file_status'"
  fi
done < "$INDEX_TSV"

while IFS=$'\t' read -r id _; do
  [ -z "$id" ] && continue
  if ! awk -F'\t' -v k="$id" '$1 == k {found=1} END {exit !found}' "$INDEX_TSV"; then
    fail "$id: WP file is not referenced from the SOW index"
  fi
done < "$FILES_TSV"
pass "index and statuses checked"

# ---------------------------------------------------------------------------
# 3. Dependency existence and acyclicity
# ---------------------------------------------------------------------------
while read -r from to; do
  [ -z "$from" ] && continue
  if ! grep -qxF "$to" "$IDS_FILE"; then
    fail "$from: dependency '$to' does not exist"
  fi
done < "$EDGES_FILE"

if ! awk '
  !/^#/ && NF >= 2 {
    from = $1; to = $2
    adj[from] = adj[from] " " to
    indeg[to]++
    nodes[from] = 1; nodes[to] = 1
  }
  END {
    n = 0
    for (k in nodes) n++
    head = 1; tail = 0; visited = 0
    for (k in nodes) {
      if (!(k in indeg) || indeg[k] == 0) queue[++tail] = k
    }
    while (head <= tail) {
      v = queue[head++]; visited++
      m = split(adj[v], next_nodes, " ")
      for (i = 1; i <= m; i++) {
        u = next_nodes[i]
        if (u == "") continue
        indeg[u]--
        if (indeg[u] == 0) queue[++tail] = u
      }
    }
    if (visited < n) exit 1
  }' "$EDGES_FILE"; then
  fail "dependency graph contains a cycle"
fi
pass "dependencies checked"

# ---------------------------------------------------------------------------
# 4. SOW links resolve
# ---------------------------------------------------------------------------
while IFS= read -r link; do
  case "$link" in
    http://*|https://*|mailto:*|'#'*) continue ;;
  esac
  link="${link%%#*}"
  [ -z "$link" ] && continue
  [ -e "$SOW_DIR/$link" ] || fail "SOW.md: broken link -> $link"
done < <(grep -oE '\]\([^)]+\)' "$SOW_FILE" | sed -E 's/^\]\((.*)\)$/\1/')
pass "SOW links checked"

# ---------------------------------------------------------------------------
# 5. Change requests
# ---------------------------------------------------------------------------
for cr in "$CR_DIR"/CR-*.md; do
  [ -f "$cr" ] || continue
  base="$(basename "$cr")"
  for row in id title decision approver date linked_adr; do
    grep -qE "^\\| ${row} \\|" "$cr" || fail "$base: missing field '$row'"
  done
  for head in context proposed_change impact_scope impact_schedule impact_cost impact_risk alternatives decision approver linked_adr; do
    grep -qE "^## ${head}$" "$cr" || fail "$base: missing section '$head'"
  done
done
if [ -f "$CR_DIR/README.md" ]; then
  while IFS= read -r link; do
    case "$link" in
      http://*|https://*|'#'*) continue ;;
    esac
    link="${link%%#*}"
    [ -z "$link" ] && continue
    [ -e "$CR_DIR/$link" ] || fail "CR register: broken link -> $link"
  done < <(grep -oE '\]\([^)]+\)' "$CR_DIR/README.md" | sed -E 's/^\]\((.*)\)$/\1/')
fi
pass "change requests checked"

# ---------------------------------------------------------------------------
# 6. Required CI check names exist in the workflow and the gate document
# ---------------------------------------------------------------------------
if [ ! -f "$REQ_FILE" ]; then
  fail "missing $REQ_FILE"
elif [ ! -f "$WF_FILE" ]; then
  fail "missing $WF_FILE"
else
  while IFS= read -r name; do
    case "$name" in ''|'#'*) continue ;; esac
    if ! grep -qE "^[[:space:]]+${name}:" "$WF_FILE"; then
      fail "workflow is missing job '$name'"
    fi
    if [ -f "$GATES_DOC" ] && ! grep -qF "\`$name\`" "$GATES_DOC"; then
      fail "docs/quality-gates.md is missing check '$name'"
    fi
  done < "$REQ_FILE"
fi
pass "required checks checked"

# ---------------------------------------------------------------------------
# 7. ADR registry consistency (defence in depth; harness-lint also checks it)
# ---------------------------------------------------------------------------
if [ -f "$ROOT/.docs/adr/registry.md" ]; then
  while IFS= read -r b; do
    [ -f "$ROOT/.docs/adr/$b" ] || fail "ADR registry points to missing $b"
  done < <(grep -oE 'adr-[0-9]{4}\.md' "$ROOT/.docs/adr/registry.md" | sort -u)
fi
pass "ADR registry checked"

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
echo
printf 'SOW LINT: %s checks passed\n' "$PASS"
if [ "$FAILED" -ne 0 ]; then
  echo "SOW LINT: FAILED"
  exit 1
fi
echo "SOW LINT: OK"
exit 0
