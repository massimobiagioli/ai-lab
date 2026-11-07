#!/usr/bin/env bash
# AI-Lab developer environment doctor.
#
# Usage: scripts/doctor.sh [--generate-key] [--connectivity]
#   --generate-key   write a development AILAB_MASTER_KEY to .env if absent
#   --connectivity   probe DeepSeek, GitHub and Docker Hub with a short timeout
#
# Read-only except for --generate-key. Exit 1 on a hard failure, 0 otherwise.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT" || exit 2

GENERATE_KEY=0
CONNECTIVITY=0
for arg in "$@"; do
  case "$arg" in
    --generate-key) GENERATE_KEY=1 ;;
    --connectivity) CONNECTIVITY=1 ;;
    -h|--help)
      sed -n '2,10p' "$0" | sed 's/^# \{0,1\}//'
      exit 0 ;;
    *) echo "doctor: unknown argument $arg" >&2; exit 2 ;;
  esac
done

FAIL=0
WARN=0
pass() { printf 'PASS  %s\n' "$1"; }
warn() { printf 'WARN  %s\n' "$1"; WARN=$((WARN + 1)); }
fail() { printf 'FAIL  %s\n' "$1"; FAIL=$((FAIL + 1)); }

version_ge() {
  [ "$(printf '%s\n%s\n' "$2" "$1" | sort -V | head -1)" = "$2" ]
}

# ---------------------------------------------------------------------------
# Docker and Compose
# ---------------------------------------------------------------------------
if ! command -v docker >/dev/null 2>&1; then
  fail "docker is not installed"
else
  if docker info >/dev/null 2>&1; then
    docker_version="$(docker version --format '{{.Server.Version}}' 2>/dev/null || echo 0)"
    if version_ge "$docker_version" "27"; then
      pass "Docker $docker_version"
    elif version_ge "$docker_version" "20"; then
      warn "Docker $docker_version is below the recommended 27; the reference host uses 27+"
    else
      fail "Docker $docker_version is too old"
    fi
  else
    fail "Docker daemon is not reachable"
  fi
  if docker compose version >/dev/null 2>&1; then
    compose_version="$(docker compose version --short 2>/dev/null | sed 's/^v//')"
    if version_ge "$compose_version" "2.30"; then
      pass "Docker Compose $compose_version"
    else
      warn "Docker Compose $compose_version is below the recommended 2.30"
    fi
  else
    fail "docker compose plugin is not available"
  fi
fi

# ---------------------------------------------------------------------------
# Toolchains
# ---------------------------------------------------------------------------
for tool in cargo rustc python3 git openssl; do
  if command -v "$tool" >/dev/null 2>&1; then
    pass "$tool $("$tool" --version 2>/dev/null | head -1)"
  else
    fail "$tool is not installed"
  fi
done

if command -v rustc >/dev/null 2>&1; then
  rust_version="$(rustc --version | awk '{print $2}')"
  if version_ge "$rust_version" "1.99.1"; then
    pass "Rust $rust_version meets the 1.99.1 pin"
  else
    fail "Rust $rust_version is below the 1.99.1 pin (run rustup toolchain install 1.99.1)"
  fi
fi

if command -v node >/dev/null 2>&1 && command -v npm >/dev/null 2>&1; then
  pass "node $(node --version), npm $(npm --version)"
elif docker image inspect node:24.22.0-trixie-slim >/dev/null 2>&1; then
  warn "node/npm are not on the host; the container fallback node:24.22.0-trixie-slim is available"
else
  fail "node 24 and npm 11 are not available on the host or as node:24.22.0-trixie-slim"
fi

# ---------------------------------------------------------------------------
# Disk and ports
# ---------------------------------------------------------------------------
avail_kb="$(df -k "$ROOT" | awk 'NR==2 {print $4}')"
if [ -n "$avail_kb" ]; then
  if [ "$avail_kb" -ge $((20 * 1024 * 1024)) ]; then
    pass "disk: $((avail_kb / 1024 / 1024)) GB free"
  else
    fail "disk: less than 20 GB free ($((avail_kb / 1024 / 1024)) GB)"
  fi
fi

if command -v lsof >/dev/null 2>&1; then
  for port in 8080 3000 5432 6379; do
    if lsof -iTCP:"$port" -sTCP:LISTEN >/dev/null 2>&1; then
      fail "port $port is already in use"
    else
      pass "port $port is free"
    fi
  done
fi

# ---------------------------------------------------------------------------
# Environment and secrets
# ---------------------------------------------------------------------------
ENV_FILE="$ROOT/.env"
if [ "$GENERATE_KEY" -eq 1 ]; then
  if [ ! -f "$ENV_FILE" ]; then
    cp "$ROOT/.env.example" "$ENV_FILE"
  fi
  if grep -qE '^AILAB_MASTER_KEY=(replace-.*)?$' "$ENV_FILE"; then
    key="$(openssl rand -base64 32)"
    if sed -i.bak "s|^AILAB_MASTER_KEY=.*|AILAB_MASTER_KEY=${key}|" "$ENV_FILE"; then
      rm -f "$ENV_FILE.bak"
      pass "generated AILAB_MASTER_KEY in .env (development only)"
    else
      fail "could not write AILAB_MASTER_KEY to .env"
    fi
  else
    pass "AILAB_MASTER_KEY already configured in .env"
  fi
  if grep -qE '^AILAB_RUNNER_TOKEN=$' "$ENV_FILE"; then
    token="$(openssl rand -hex 32)"
    if sed -i.bak "s|^AILAB_RUNNER_TOKEN=.*|AILAB_RUNNER_TOKEN=${token}|" "$ENV_FILE"; then
      rm -f "$ENV_FILE.bak"
      pass "generated AILAB_RUNNER_TOKEN in .env (development only)"
    else
      fail "could not write AILAB_RUNNER_TOKEN to .env"
    fi
  else
    pass "AILAB_RUNNER_TOKEN is present in .env"
  fi
elif [ -f "$ENV_FILE" ] \
  && grep -qE '^AILAB_MASTER_KEY=[A-Za-z0-9+/=]{20,}' "$ENV_FILE" \
  && grep -qE '^AILAB_RUNNER_TOKEN=[A-Za-z0-9._-]{16,}' "$ENV_FILE"; then
  pass "AILAB_MASTER_KEY and AILAB_RUNNER_TOKEN are present in .env"
else
  fail "AILAB_MASTER_KEY or AILAB_RUNNER_TOKEN is missing; run scripts/doctor.sh --generate-key for development"
fi

if [ -f "$ENV_FILE" ]; then
  missing="$(comm -23 \
    <(grep -E '^[A-Z0-9_]+=' "$ROOT/.env.example" | cut -d= -f1 | sort) \
    <(grep -E '^[A-Z0-9_]+=' "$ENV_FILE" | cut -d= -f1 | sort) | tr '\n' ' ')"
  if [ -n "$missing" ]; then
    warn ".env is missing variables: $missing"
  else
    pass ".env covers every .env.example variable"
  fi
fi

# ---------------------------------------------------------------------------
# Connectivity (optional)
# ---------------------------------------------------------------------------
if [ "$CONNECTIVITY" -eq 1 ]; then
  for url in https://api.deepseek.com https://api.github.com https://registry-1.docker.io; do
    if curl -sSfi --max-time 5 "$url" >/dev/null 2>&1 || curl -sSfI --max-time 5 "$url" >/dev/null 2>&1; then
      pass "connectivity: $url"
    else
      warn "connectivity: $url is not reachable"
    fi
  done
fi

echo
if [ "$FAIL" -ne 0 ]; then
  echo "DOCTOR: FAILED ($FAIL failures, $WARN warnings)"
  exit 1
fi
echo "DOCTOR: OK ($WARN warnings)"
exit 0
