# WP-14 Independent Review

| Field | Value |
|---|---|
| Reviewer | review-agent (independent subagent) |
| Author | Dev (agent) |
| Date | 2026-11-07 |
| Scope | Sandbox runner, isolation, egress, token, tool registry and policy |
| Verdict | `APPROVED` |

Several review passes returned `CHANGES_REQUESTED` (runner API, tool registry, live escape suite, token/egress wiring, command-policy bypass, label checks). The author fixed the defects and the final pass approved at HEAD `85b6928`.

## Final acceptance results

| AC | Result | Evidence |
|---|---|---|
| AC1 isolation + limits | PASS | `host_config_for` (read-only, cap-drop, non-root, pids 512, TTL kill). |
| AC2 Docker proxy | PASS | Proxy allowlist; runner has no socket. |
| AC3 egress | DEFERRED | Live execution WP-21; policy/proxy/internal network delivered. |
| AC4 tool authorization | PASS (run-policy deferred) | Role/type/path/command + structured audited denials. |
| AC5 path/symlink | PASS (symlink WP-21) | `.env*`, `*.pem`, `*.key`, `.git/hooks`, workflows, CODEOWNERS denied. |
| AC6 host/force-push/publish | PASS | Forbidden binaries and force flags denied. |
| AC7 token | PASS | Per-exec env, removed on delete, never whole-sandbox. |
| AC8 output/timeouts | PASS (disk/fork WP-21) | 20 KB cap, `TOOL_TIMEOUT`. |
| AC9 orphans | DEFERRED | WP-14 follow-up/WP-20. |
| AC10 escape suite | DEFERRED | Ignored live suite WP-21. |

## Blocking defects found and resolved

1. Missing runner HTTP API and typed tool registry (delivered).
2. Command/tool bypass (`tool` controlled argv) — non-shell tools rejected, `shell.run` always authorized.
3. Token injected into the whole sandbox — now per-exec and removed on delete.
4. Unknown role defaulted to Dev — now rejected with 422.
5. Label check missing on status/logs — now enforced on status/logs/exec/delete.
6. Container time limit, internal egress, protected argv paths — delivered.

## Reproduced evidence

- `scripts/harness-lint.sh` 16 checks, `scripts/sow-lint.sh` 31 checks, `.tools/check-doc-links.sh` 148 links.
- `cargo fmt --all --check`, `cargo clippy -D warnings`, `cargo test -p ailab-orchestrator` 52 passed + 3 ignored.
- `make check` OK.

## Reviewer statement

The WP-14 artifacts satisfy the sandbox/tool contract. Live escape/resource/egress execution (WP-21), process-group/disk/orphan hardening (WP-14 follow-up/WP-20) and the run-policy stage (WP-13/WP-16) are the named deferrals. Approved for WP closure.
