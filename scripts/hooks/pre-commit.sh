#!/usr/bin/env bash
# Fast pre-commit checks: conflict markers, formatting on staged files, secret
# scan when gitleaks is available.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT" || exit 2

if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  exit 0
fi

STAGED="$(git diff --cached --name-only --diff-filter=ACM)"
[ -z "$STAGED" ] && exit 0

if git diff --cached | grep -qE '^(<<<<<<<|=======|>>>>>>>)'; then
  echo "pre-commit: merge conflict marker found" >&2
  exit 1
fi

if printf '%s\n' "$STAGED" | grep -qE '\.rs$'; then
  scripts/run-rust.sh cargo fmt --all --check || { echo "pre-commit: cargo fmt failed" >&2; exit 1; }
fi

if printf '%s\n' "$STAGED" | grep -qE '^web/'; then
  scripts/run-web.sh npm run format:check || { echo "pre-commit: prettier failed" >&2; exit 1; }
fi

if command -v gitleaks >/dev/null 2>&1; then
  gitleaks protect --staged --redact --no-banner || { echo "pre-commit: secret found" >&2; exit 1; }
fi

exit 0
