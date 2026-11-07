#!/usr/bin/env bash
# AI-Lab harness validation. Read-only checks; no network; exits non-zero on failure.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT" || exit 1

FAILED=0
PASS=0
fail() { printf 'FAIL  %s\n' "$1"; FAILED=1; }
pass() { printf 'PASS  %s\n' "$1"; PASS=$((PASS + 1)); }

# ---------------------------------------------------------------------------
# 1. Required files
# ---------------------------------------------------------------------------
required=(
  "AGENTS.md"
  "README.md"
  ".docs/CONSTITUTION.md"
  ".docs/PRINCIPLES.md"
  ".docs/STACK.md"
  ".docs/adr/registry.md"
  ".opencode/commands/impl-task.md"
  ".opencode/commands/impl-wp.md"
)
for f in "${required[@]}"; do
  [ -f "$f" ] && pass "required file: $f" || fail "required file missing: $f"
done

# ---------------------------------------------------------------------------
# 2. Skills: frontmatter and AGENTS.md index
# ---------------------------------------------------------------------------
skill_count=0
for d in .opencode/skills/*/; do
  [ -d "$d" ] || continue
  skill_count=$((skill_count + 1))
  id="$(basename "$d")"
  f="$d/SKILL.md"
  if [ ! -f "$f" ]; then
    fail "skill missing SKILL.md: $id"
    continue
  fi
  if ! sed -n '1,20p' "$f" | grep -qE '^name: .+'; then
    fail "skill frontmatter missing name: $id"
  fi
  if ! sed -n '1,20p' "$f" | grep -qE '^description: .+'; then
    fail "skill frontmatter missing description: $id"
  fi
  if ! grep -Fq ".opencode/skills/$id/SKILL.md" AGENTS.md; then
    fail "skill not indexed in AGENTS.md: $id"
  fi
done
[ "$skill_count" -ge 14 ] && pass "skill count >= 14 ($skill_count)" || fail "skill count < 14 ($skill_count)"

# ---------------------------------------------------------------------------
# 3. Documentation index: every .docs file except adr-*.md is in AGENTS.md
# ---------------------------------------------------------------------------
while IFS= read -r f; do
  case "$f" in
    .docs/adr/adr-*.md) continue ;;
  esac
  if ! grep -Fq "$f" AGENTS.md; then
    fail "doc not indexed in AGENTS.md: $f"
  fi
done < <(find .docs -type f \( -name '*.md' -o -name '*.sql' \) | sort)
pass "documentation index checked"

# ---------------------------------------------------------------------------
# 4. ADR registry consistency
# ---------------------------------------------------------------------------
for f in .docs/adr/adr-*.md; do
  [ -f "$f" ] || continue
  b="$(basename "$f")"
  if ! grep -Fq "$b" .docs/adr/registry.md; then
    fail "ADR missing from registry: $b"
  fi
done
while IFS= read -r b; do
  [ -z "$b" ] && continue
  if [ ! -f ".docs/adr/$b" ]; then
    fail "registry points to missing ADR: $b"
  fi
done < <(grep -oE 'adr-[0-9]{4}\.md' .docs/adr/registry.md | sort -u)
pass "ADR registry consistency checked"

# ---------------------------------------------------------------------------
# 5. ADR required fields
# ---------------------------------------------------------------------------
for f in .docs/adr/adr-*.md; do
  [ -f "$f" ] || continue
  for field in Status Date Context Decision Consequences; do
    if ! grep -qF "$field" "$f"; then
      fail "ADR missing $field: $f"
    fi
  done
done
pass "ADR fields checked"

# ---------------------------------------------------------------------------
# 6. Commands
# ---------------------------------------------------------------------------
for c in impl-task impl-wp; do
  f=".opencode/commands/$c.md"
  if [ ! -f "$f" ]; then
    fail "command missing: $f"
    continue
  fi
  if ! grep -qF '$ARGUMENTS' "$f"; then
    fail "command missing \$ARGUMENTS: $f"
  fi
  for word in Red Green Refactor; do
    if ! grep -qF "$word" "$f"; then
      fail "command missing TDD step $word: $f"
    fi
  done
  if ! grep -qF 'progress' "$f" || ! grep -qF 'done' "$f"; then
    fail "command missing status transitions: $f"
  fi
done
pass "commands checked"

# ---------------------------------------------------------------------------
# 7. AGENTS.md required content
# ---------------------------------------------------------------------------
if ! grep -qF 'README.md' AGENTS.md; then
  fail "AGENTS.md does not reference README.md"
fi
if ! grep -qF '.docs/adr/registry.md' AGENTS.md; then
  fail "AGENTS.md does not reference the ADR registry"
fi
h1_count="$(grep -c '^# ' AGENTS.md || true)"
if [ "$h1_count" -ne 1 ]; then
  fail "AGENTS.md must contain exactly one H1 (found $h1_count)"
fi
for section in 'Read order' 'Skills index' 'Documentation index' 'ADR registry' 'Commands' 'Quality gates' 'Status model' 'Update rules'; do
  if ! grep -qF "$section" AGENTS.md; then
    fail "AGENTS.md missing section: $section"
  fi
done
pass "AGENTS.md content checked"

# ---------------------------------------------------------------------------
# 8. No placeholders in harness files
# ---------------------------------------------------------------------------
scan_files=(
  AGENTS.md
  .docs/CONSTITUTION.md
  .docs/PRINCIPLES.md
  .docs/STACK.md
  .opencode/commands/impl-task.md
  .opencode/commands/impl-wp.md
)
while IFS= read -r f; do scan_files+=("$f"); done < <(find .docs/adr -name 'adr-*.md' | sort)
while IFS= read -r f; do scan_files+=("$f"); done < <(find .opencode/skills -name 'SKILL.md' | sort)
for f in "${scan_files[@]}"; do
  [ -f "$f" ] || continue
  if grep -nE '(TBD|TODO|FIXME|<placeholder>)' "$f" >/dev/null 2>&1; then
    fail "placeholder marker in $f"
  fi
done
pass "placeholder scan checked"

# ---------------------------------------------------------------------------
# 9. Relative Markdown links resolve
# ---------------------------------------------------------------------------
check_links() {
  local file="$1"
  local dir
  dir="$(dirname "$file")"
  while IFS= read -r link; do
    case "$link" in
      http://*|https://*|mailto:*|'#'*) continue ;;
    esac
    link="${link%%#*}"
    [ -z "$link" ] && continue
    if [ ! -e "$dir/$link" ]; then
      fail "broken link in $file -> $link"
    fi
  done < <(grep -oE '\]\([^)]+\)' "$file" 2>/dev/null | sed -E 's/^\]\((.*)\)$/\1/')
}
check_links AGENTS.md
while IFS= read -r f; do check_links "$f"; done < <(find .docs -type f -name '*.md' | sort)
pass "link check finished"

# ---------------------------------------------------------------------------
# 10. Summary
# ---------------------------------------------------------------------------
echo
if [ "$FAILED" -ne 0 ]; then
  echo "HARNESS LINT: FAILED ($PASS checks passed)"
  exit 1
fi
echo "HARNESS LINT: OK ($PASS checks passed)"
exit 0
