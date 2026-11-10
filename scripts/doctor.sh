#!/usr/bin/env bash
# AI-Lab host doctor.
#
# Usage: scripts/doctor.sh [--connectivity]
#   --connectivity   Probe DeepSeek, GitHub and Docker Hub with a short timeout.
#
# The doctor checks the host prerequisites only. Rust, Node, Python and OpenSSL
# run inside Docker; the host toolchains are intentionally not required.
#
# Read-only. Exit 1 on a hard failure, 0 otherwise.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT" || exit 2

CONNECTIVITY=0
for arg in "$@"; do
  case "$arg" in
    --connectivity) CONNECTIVITY=1 ;;
    -h|--help)
      sed -n '2,12p' "$0" | sed 's/^# \{0,1\}//'
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
# Docker images
# ---------------------------------------------------------------------------
for image in \
  "rust:1.99.1-trixie@sha256:8a4e90c7946b74626488b1446f712c0fe0ed1a596cbd3d871684bc9c5433ae07" \
  "node:24.22.0-trixie-slim" \
  "python:3.12.6-slim-bookworm" \
  "mcr.microsoft.com/playwright:v1.65.0-noble"; do
  if docker image inspect "$image" >/dev/null 2>&1; then
    pass "image present: $image"
  else
    warn "image not pulled yet: $image (it will be pulled on first use)"
  fi
done

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

COMPOSE_FILE="$ROOT/infra/compose/docker-compose.yml"
stack_ports() {
  [ -f "$COMPOSE_FILE" ] || return 0
  docker compose -f "$COMPOSE_FILE" ps -q 2>/dev/null | while IFS= read -r container; do
    [ -n "$container" ] || continue
    docker inspect --format '{{range $p, $conf := .NetworkSettings.Ports}}{{range $conf}}{{.HostPort}} {{end}}{{end}}' "$container" 2>/dev/null
  done
}

STACK_PORTS="$(stack_ports)"
if command -v lsof >/dev/null 2>&1; then
  for port in 8080 3000 5432 6379; do
    if printf '%s\n' "$STACK_PORTS" | tr ' ' '\n' | grep -qx "$port"; then
      pass "port $port is used by the running AI-Lab stack"
    elif lsof -iTCP:"$port" -sTCP:LISTEN >/dev/null 2>&1; then
      fail "port $port is already in use by another process"
    else
      pass "port $port is free"
    fi
  done
fi

# ---------------------------------------------------------------------------
# Environment and secrets
# ---------------------------------------------------------------------------
ENV_FILE="$ROOT/.env"
if [ ! -f "$ENV_FILE" ]; then
  fail ".env is missing; run make generate-key"
else
  master="$(grep -E '^AILAB_MASTER_KEY=' "$ENV_FILE" | head -1 | cut -d= -f2-)"
  runner="$(grep -E '^AILAB_RUNNER_TOKEN=' "$ENV_FILE" | head -1 | cut -d= -f2-)"
  master_bytes="$(printf '%s' "$master" | base64 -d 2>/dev/null | wc -c | tr -d ' ')"
  if [ "$master_bytes" = "32" ]; then
    pass "AILAB_MASTER_KEY is a base64 32-byte key"
  else
    fail "AILAB_MASTER_KEY is missing or invalid; run make generate-key"
  fi
  if [ "${#runner}" -ge 32 ]; then
    pass "AILAB_RUNNER_TOKEN is present (${#runner} chars)"
  else
    fail "AILAB_RUNNER_TOKEN is missing or too short; run make generate-key"
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
