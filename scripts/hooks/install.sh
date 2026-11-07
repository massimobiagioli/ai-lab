#!/usr/bin/env bash
# Install commit hooks. Uses lefthook when available, otherwise installs native
# git hooks from scripts/hooks/.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT" || exit 2

if command -v lefthook >/dev/null 2>&1; then
  lefthook install
  echo "hooks: installed with lefthook"
  exit 0
fi

if [ ! -d .git ]; then
  echo "hooks: not a git repository; run 'git init' first" >&2
  exit 2
fi

for hook in pre-commit commit-msg pre-push; do
  cp "scripts/hooks/${hook}.sh" ".git/hooks/${hook}"
  chmod +x ".git/hooks/${hook}"
done
echo "hooks: lefthook not found; installed native git hooks"
