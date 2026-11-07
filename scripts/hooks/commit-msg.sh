#!/usr/bin/env bash
# Validate a commit message: Conventional Commits, and agent trailers when
# AI_LAB_AGENT=1.
set -uo pipefail

MSG_FILE="${1:?usage: commit-msg.sh <message-file>}"
SUBJECT="$(head -1 "$MSG_FILE")"

if ! printf '%s' "$SUBJECT" | grep -Eq '^(feat|fix|chore|docs|test|refactor|perf|build|ci)(\([a-z0-9._/-]+\))?: .{1,72}$'; then
  cat >&2 <<'EOF'
commit-msg: invalid subject.
Use Conventional Commits: type(scope): imperative summary
Types: feat, fix, chore, docs, test, refactor, perf, build, ci
Subject: 1-72 characters after the colon.
EOF
  exit 1
fi

if [ "${AI_LAB_AGENT:-0}" = "1" ]; then
  for trailer in 'AI-Lab-Run:' 'AI-Lab-Card:' 'AI-Lab-Agent:'; do
    if ! grep -q "^${trailer}" "$MSG_FILE"; then
      echo "commit-msg: missing required agent trailer: $trailer" >&2
      exit 1
    fi
  done
fi

if grep -qE '^(<<<<<<<|=======|>>>>>>>)' "$MSG_FILE"; then
  echo "commit-msg: merge conflict marker found" >&2
  exit 1
fi

exit 0
