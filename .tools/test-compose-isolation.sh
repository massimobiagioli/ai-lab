#!/usr/bin/env bash
# Compose isolation and hardening test.
#
# Renders the core profile to JSON and asserts the platform invariants from
# WP-03 §3.2, §3.7 and §3.8: no privileged containers, no host networking, no
# published database ports, read-only non-root services, dropped capabilities,
# resource limits, isolated sandbox networks and a single Docker socket mount.
#
# Usage: .tools/test-compose-isolation.sh
# Exit: 0 green, 1 violation, 2 environment error.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
COMPOSE_FILE="$ROOT/infra/compose/docker-compose.yml"
[ -f "$COMPOSE_FILE" ] || { echo "ENV ERROR: missing $COMPOSE_FILE" >&2; exit 2; }
command -v docker >/dev/null 2>&1 || { echo "ENV ERROR: docker missing" >&2; exit 2; }
command -v python3 >/dev/null 2>&1 || { echo "ENV ERROR: python3 missing" >&2; exit 2; }

TMP="$(mktemp "${TMPDIR:-/tmp}/ailab-compose.XXXXXX.json")"
trap 'rm -f "$TMP"' EXIT

if ! docker compose -f "$COMPOSE_FILE" --profile core config --format json > "$TMP" 2>/tmp/ailab-compose.err; then
  echo "ENV ERROR: docker compose config failed" >&2
  cat /tmp/ailab-compose.err >&2
  exit 2
fi

python3 - "$TMP" <<'PY'
import json
import sys

data = json.load(open(sys.argv[1]))
services = data.get("services", {})
failures = []


def check(condition, message):
    if not condition:
        failures.append(message)


for name, svc in services.items():
    check(svc.get("privileged") is not True, f"{name}: privileged container")
    check(svc.get("network_mode") != "host", f"{name}: host network")
    check("latest" not in str(svc.get("image", "")), f"{name}: latest image tag")
    check(svc.get("mem_limit") not in (None, ""), f"{name}: missing mem_limit")
    check(svc.get("pids_limit") not in (None, ""), f"{name}: missing pids_limit")

for name in ("api", "worker", "runner", "web"):
    svc = services.get(name, {})
    check(svc.get("read_only") is True, f"{name}: root filesystem is writable")
    check("ALL" in (svc.get("cap_drop") or []), f"{name}: capabilities not dropped")
    check(
        any("no-new-privileges" in opt for opt in (svc.get("security_opt") or [])),
        f"{name}: no-new-privileges missing",
    )

# The socket proxy needs a writable config directory at startup. It still drops
# all capabilities, and the Docker socket mount must be read-only.
proxy = services.get("docker-socket-proxy", {})
check("ALL" in (proxy.get("cap_drop") or []), "docker-socket-proxy: capabilities not dropped")
check(
    any("no-new-privileges" in opt for opt in (proxy.get("security_opt") or [])),
    "docker-socket-proxy: no-new-privileges missing",
)
for volume in proxy.get("volumes") or []:
    source = volume.get("source", "") if isinstance(volume, dict) else str(volume)
    if "docker.sock" in source:
        check(
            isinstance(volume, dict) and volume.get("read_only") is True,
            "docker-socket-proxy: Docker socket mount must be read-only",
        )

for name, svc in services.items():
    ports = svc.get("ports") or []
    if name == "web":
        check(len(ports) > 0, "web: must publish 127.0.0.1:8080")
    else:
        check(len(ports) == 0, f"{name}: must not publish host ports")

for name in ("postgres", "redis"):
    networks = set((services.get(name, {}).get("networks") or {}).keys())
    check(networks == {"backbone"}, f"{name}: networks {sorted(networks)} != ['backbone']")

for name, svc in services.items():
    for volume in svc.get("volumes") or []:
        source = volume.get("source", "") if isinstance(volume, dict) else str(volume)
        if "docker.sock" in source:
            check(name == "docker-socket-proxy", f"{name}: mounts the Docker socket")

for network in ("runner", "sandbox-control"):
    check(
        (data.get("networks", {}).get(network, {}) or {}).get("internal") is True,
        f"{network}: network must be internal",
    )

if failures:
    print("COMPOSE ISOLATION: FAILED")
    for failure in failures:
        print(f" - {failure}")
    sys.exit(1)

print(f"COMPOSE ISOLATION: OK ({len(services)} services)")
PY
