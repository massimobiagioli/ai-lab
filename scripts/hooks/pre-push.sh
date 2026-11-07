#!/usr/bin/env bash
# Pre-push: run the fast local gate. Full suites run in CI.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT" || exit 2
exec make check
