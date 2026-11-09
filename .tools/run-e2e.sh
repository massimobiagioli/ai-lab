#!/usr/bin/env bash
# Run the Playwright suite in the official image (browser dependencies included).
# Usage: .tools/run-e2e.sh [playwright args]
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
IMAGE="${AILAB_PLAYWRIGHT_IMAGE:-mcr.microsoft.com/playwright:v1.65.0-noble}"
exec docker run --rm \
  --user "$(id -u):$(id -g)" \
  -e HOME=/tmp \
  -e NG_CLI_ANALYTICS=false \
  -v "$ROOT/web:/app" \
  -w /app \
  "$IMAGE" npx playwright test "$@"
