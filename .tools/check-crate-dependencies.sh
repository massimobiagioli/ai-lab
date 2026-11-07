#!/usr/bin/env bash
# Crate fitness: dependency direction, acyclicity, unsafe and unwrap gates.
#
# Usage: .tools/check-crate-dependencies.sh
# Exit: 0 green, 1 violation, 2 environment error.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
command -v python3 >/dev/null 2>&1 || { echo "ENV ERROR: python3 missing" >&2; exit 2; }
command -v cargo >/dev/null 2>&1 || { echo "ENV ERROR: cargo missing" >&2; exit 2; }

if ! cargo metadata --no-deps --format-version 1 > /tmp/ailab-crate-metadata.json 2>/tmp/ailab-crate-metadata.err; then
  echo "ENV ERROR: cargo metadata failed" >&2
  cat /tmp/ailab-crate-metadata.err >&2
  exit 2
fi

python3 - /tmp/ailab-crate-metadata.json "$ROOT" <<'PY'
import json
import pathlib
import sys

meta = json.load(open(sys.argv[1]))
root = pathlib.Path(sys.argv[2])
pkgs = {p["name"]: p for p in meta["packages"]}
normal = {
    p["name"]: [d["name"] for d in p["dependencies"] if d.get("kind") in (None, "normal")]
    for p in meta["packages"]
}
failures = []


def workspace(name):
    return name.startswith("ailab-")


for dep in normal.get("ailab-domain", []):
    if workspace(dep) or dep in {"sqlx", "axum", "tokio", "redis", "reqwest", "hyper", "actix-web"}:
        failures.append(f"ailab-domain must not depend on {dep}")

for dep in normal.get("ailab-application", []):
    if workspace(dep) and dep != "ailab-domain":
        failures.append(f"ailab-application must not depend on {dep}")

graph = {name: [dep for dep in deps if dep in pkgs] for name, deps in normal.items()}
color = {name: 0 for name in graph}


def dfs(node):
    color[node] = 1
    for child in graph.get(node, []):
        if color[child] == 1:
            return True
        if color[child] == 0 and dfs(child):
            return True
    color[node] = 2
    return False


for node in list(graph):
    if color[node] == 0 and dfs(node):
        failures.append("cyclic workspace dependency")
        break

for name, package in pkgs.items():
    if not workspace(name):
        continue
    src = pathlib.Path(package["manifest_path"]).parent / "src"
    files = list(src.rglob("*.rs"))
    if not any("#![forbid(unsafe_code)]" in file.read_text() for file in files):
        failures.append(f"{name} missing #![forbid(unsafe_code)]")

for crate in ("domain", "application"):
    for file in (root / f"crates/{crate}/src").rglob("*.rs"):
        production = file.read_text().split("#[cfg(test)]")[0]
        for needle in (".unwrap(", ".expect("):
            if needle in production:
                failures.append(f"{file} uses {needle} in production code")

if failures:
    print("CRATE FITNESS: FAILED")
    for failure in failures:
        print(f" - {failure}")
    sys.exit(1)

print(f"CRATE FITNESS: OK ({len(pkgs)} workspace packages)")
PY
