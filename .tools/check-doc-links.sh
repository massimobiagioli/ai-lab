#!/usr/bin/env bash
# Check relative Markdown links under .docs/product/ and .docs/.
#
# Usage: .tools/check-doc-links.sh [--root DIR]
# Exit: 0 all relative links resolve, 1 a link is broken, 2 environment error.
set -uo pipefail

ROOT=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --root) ROOT="$2"; shift 2 ;;
    -h|--help) sed -n '2,6p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
    *) echo "check-doc-links: unknown argument: $1" >&2; exit 2 ;;
  esac
done
[ -z "$ROOT" ] && ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

FAILED=0
CHECKED=0

while IFS= read -r f; do
  dir="$(dirname "$f")"
  while IFS= read -r link; do
    case "$link" in
      http://*|https://*|mailto:*|'#'*) continue ;;
      *'{'*|*'<'*) continue ;;
    esac
    target="${link%%#*}"
    target="${target%%\?*}"
    target="$(printf '%s' "$target" | sed -E 's/^[[:space:]]+|[[:space:]]+$//g')"
    [ -z "$target" ] && continue
    CHECKED=$((CHECKED + 1))
    if [ ! -e "$dir/$target" ]; then
      printf 'FAIL  broken link in %s -> %s\n' "$f" "$link"
      FAILED=1
    fi
  done < <(grep -oE '\]\([^)]+\)' "$f" 2>/dev/null | sed -E 's/^\]\((.*)\)$/\1/')
done < <(find "$ROOT/.docs" -type f -name '*.md' 2>/dev/null | sort)

echo
if [ "$FAILED" -ne 0 ]; then
  echo "DOC LINKS: FAILED ($CHECKED checked)"
  exit 1
fi
echo "DOC LINKS: OK ($CHECKED checked)"
exit 0
