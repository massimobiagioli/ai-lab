#!/usr/bin/env python3
"""Fail when the OpenAPI contract advertises a path the edge router does not serve.

Usage: .tools/check-openapi-drift.py [--root DIR]
"""
import argparse
import json
import re
import sys
from pathlib import Path


def normalize(path: str) -> str:
    return re.sub(r"\{[^}]+\}", "{p}", path)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", default=".")
    args = parser.parse_args()
    root = Path(args.root).resolve()
    server = (root / "crates/api/src/server.rs").read_text()
    routes = {normalize(match.group(1)) for match in re.finditer(r'\.route\("([^"]+)"', server)}
    spec = json.loads((root / ".docs/product/api/openapi.json").read_text())
    missing = []
    for path in spec.get("paths", {}):
        full = path if path.startswith("/api/") else f"/api/v1{path}"
        if normalize(full) not in routes:
            missing.append(full)
    if missing:
        print("OPENAPI DRIFT: paths advertised but not served:")
        for path in missing:
            print(f"  {path}")
        return 1
    print(f"OPENAPI DRIFT: OK ({len(spec.get('paths', {}))} paths served)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
