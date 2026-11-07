#!/usr/bin/env bash
# Architecture fitness check.
#
# Validates the context ownership manifest and the compile-time dependency graph,
# then scans crates/ for cross-context SQL table references. When a Compose file
# exists, verifies every declared service appears in the service catalog.
#
# Usage: .tools/check-architecture-fitness.sh [--root DIR]
# Exit: 0 green, 1 violation, 2 environment error.
set -uo pipefail

ROOT=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --root) ROOT="$2"; shift 2 ;;
    -h|--help) sed -n '2,9p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
    *) echo "architecture-fitness: unknown argument: $1" >&2; exit 2 ;;
  esac
done
[ -z "$ROOT" ] && ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

CTX="$ROOT/.tools/architecture/contexts.tsv"
DEPS="$ROOT/.tools/architecture/dependencies.tsv"
PATHS="$ROOT/.tools/architecture/paths.tsv"
CATALOG="$ROOT/.docs/product/service-catalog.md"

[ -f "$CTX" ] || { echo "architecture-fitness: missing $CTX" >&2; exit 2; }
[ -f "$DEPS" ] || { echo "architecture-fitness: missing $DEPS" >&2; exit 2; }
[ -f "$PATHS" ] || { echo "architecture-fitness: missing $PATHS" >&2; exit 2; }

FAILED=0
PASS=0
fail() { printf 'FAIL  %s\n' "$1"; FAILED=1; }
pass() { PASS=$((PASS + 1)); }

# ---------------------------------------------------------------------------
# 1. Manifest: exactly one owner per context, no table owned twice.
# ---------------------------------------------------------------------------
DUP_CTX="$(awk -F'\t' '!/^#/ && NF>=3 {print $1}' "$CTX" | sort | uniq -d)"
[ -n "$DUP_CTX" ] && fail "duplicate context definitions: $DUP_CTX"

EMPTY_OWNER="$(awk -F'\t' '!/^#/ && NF>=3 && $2=="" {print $1}' "$CTX")"
[ -n "$EMPTY_OWNER" ] && fail "context without owner: $EMPTY_OWNER"

DUP_TABLE="$(awk -F'\t' '!/^#/ && NF>=3 {n=split($3,a,","); for(i=1;i<=n;i++) if(a[i]!="") print a[i]}' "$CTX" | sort | uniq -d)"
if [ -n "$DUP_TABLE" ]; then
  fail "table owned by more than one context: $(printf '%s' "$DUP_TABLE" | tr '\n' ' ')"
fi
pass "context manifest checked"

# ---------------------------------------------------------------------------
# 2. Dependency graph: known contexts and acyclic.
# ---------------------------------------------------------------------------
CONTEXTS="$(awk -F'\t' '!/^#/ && NF>=3 {print $1}' "$CTX" | sort)"
while IFS=$'\t' read -r from to; do
  case "$from" in ''|'#'*) continue ;; esac
  printf '%s\n' "$CONTEXTS" | grep -qxF "$from" || fail "dependency from unknown context: $from"
  printf '%s\n' "$CONTEXTS" | grep -qxF "$to" || fail "dependency to unknown context: $to"
done < "$DEPS"

deps_of() { awk -F'\t' -v n="$1" '!/^#/ && $1==n {print $2}' "$DEPS"; }
has_cycle_from() {
  local node="$1" path="$2" dep
  case " $path " in *" $node "*) return 0 ;; esac
  while IFS= read -r dep; do
    [ -z "$dep" ] && continue
    if has_cycle_from "$dep" "$path $node"; then return 0; fi
  done < <(deps_of "$node")
  return 1
}
while IFS= read -r c; do
  [ -z "$c" ] && continue
  if has_cycle_from "$c" ""; then
    fail "dependency cycle involving $c"
    break
  fi
done <<< "$CONTEXTS"
while IFS=$'\t' read -r prefix ctx; do
  case "$prefix" in ''|'#'*) continue ;; esac
  printf '%s\n' "$CONTEXTS" | grep -qxF "$ctx" || fail "paths.tsv maps '$prefix' to unknown context '$ctx'"
done < "$PATHS"
pass "dependency graph and path map checked"

# ---------------------------------------------------------------------------
# 3. Cross-context table references in Rust source.
# ---------------------------------------------------------------------------
owner_of_table() {
  awk -F'\t' -v t="$1" '!/^#/ && NF>=3 {n=split($3,a,","); for(i=1;i<=n;i++) if(a[i]==t){print $1; exit}}' "$CTX"
}

# Extract SQL table references from Rust string literals only, so prose in
# comments (for example "into a host") is not mistaken for SQL.
extract_tables() {
  tr '\n' ' ' < "$1" \
    | grep -oE 'r#"([^"]*)"#|"([^"\\]|\\.)*"' \
    | tr '\n' ' ' \
    | sed -E 's/\\"/"/g' \
    | grep -oiE '(FROM|JOIN|INTO|UPDATE|DELETE[[:space:]]+FROM)[[:space:]]+("?[a-z_][a-z0-9_]*"?\.)?"?[a-z_][a-z0-9_]*"?' \
    | awk '{print $NF}' \
    | sed -E 's/.*\.//' \
    | tr -d '"' \
    | sort -u
}

context_of_path() {
  local rel="$1" prefix ctx best="" bestlen=0 c
  while IFS=$'\t' read -r prefix ctx; do
    case "$prefix" in ''|'#'*) continue ;; esac
    case "$rel" in
      "$prefix"|"$prefix"/*)
        if [ "${#prefix}" -gt "$bestlen" ]; then best="$ctx"; bestlen=${#prefix}; fi
        ;;
    esac
  done < "$PATHS"
  if [ -n "$best" ]; then printf '%s' "$best"; return 0; fi
  while IFS= read -r c; do
    [ -z "$c" ] && continue
    case "/$rel/" in *"/$c/"*) printf '%s' "$c"; return 0 ;; esac
  done <<< "$CONTEXTS"
  return 1
}

if [ -d "$ROOT/crates" ]; then
  while IFS= read -r f; do
    rel="${f#"$ROOT"/}"
    tables="$(extract_tables "$f")"
    [ -z "$tables" ] && continue
    if ! ctx="$(context_of_path "$rel")"; then
      case "$rel" in
        crates/infrastructure/*|crates/api/*|crates/worker/*|crates/cli/*)
          fail "$rel contains SQL table references but is not attributed to a context; add a prefix to paths.tsv"
          ;;
      esac
      continue
    fi
    while IFS= read -r table; do
      [ -z "$table" ] && continue
      owner="$(owner_of_table "$table")"
      [ -z "$owner" ] && continue
      if [ "$owner" != "$ctx" ]; then
        fail "$rel ($ctx) references table '$table' owned by $owner"
      fi
    done <<< "$tables"
  done < <(find "$ROOT/crates" -type f -name '*.rs')
fi
pass "cross-context table references checked"

# ---------------------------------------------------------------------------
# 4. Compose services appear in the catalog.
# ---------------------------------------------------------------------------
COMPOSE=""
for candidate in "$ROOT/compose.yaml" "$ROOT/compose.yml" "$ROOT/docker-compose.yml" "$ROOT/docker-compose.yaml"; do
  [ -f "$candidate" ] && COMPOSE="$candidate" && break
done
if [ -n "$COMPOSE" ]; then
  [ -f "$CATALOG" ] || fail "compose exists but $CATALOG is missing"
  while IFS= read -r svc; do
    [ -z "$svc" ] && continue
    grep -qF "\`$svc\`" "$CATALOG" || fail "compose service '$svc' is missing from the service catalog"
  done < <(awk '/^services:/{f=1;next} /^[^[:space:]]/{f=0} f && /^  [a-zA-Z0-9_.-]+:/{gsub(/^  /,""); sub(/:.*/,""); print}' "$COMPOSE")
fi
pass "compose coverage checked"

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
echo
printf 'ARCHITECTURE FITNESS: %s checks passed\n' "$PASS"
if [ "$FAILED" -ne 0 ]; then
  echo "ARCHITECTURE FITNESS: FAILED"
  exit 1
fi
echo "ARCHITECTURE FITNESS: OK"
exit 0
