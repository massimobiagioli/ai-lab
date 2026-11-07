# AI-Lab Sandbox

| Field | Value |
|---|---|
| Document | `.docs/product/sandbox.md` |
| Version | 1.0.0 |
| Status | Accepted |
| Owner | Dev |
| Related | `.docs/product/tools.md`, `.docs/product/security.md`, `crates/orchestrator/src/sandbox.rs` |

The sandbox is the only place agent code runs. The specification, egress policy, tool policy and lifecycle are implemented in `crates/orchestrator/src/sandbox.rs`; the runner service and actual container creation are owned by WP-14 follow-up/WP-03.

## 1. Specification

- `run_id` is non-empty and cannot contain `/` or `..`.
- `image` must be digest-pinned (`@sha256:` + 64 hex).
- `worktree_ref` cannot escape the workspace.
- CPU 100–4000 millicores, memory 128–8192 MB, timeout 1–7200 s.
- Egress is `Disabled` or an allowlist of domains.

## 2. Isolation

`IsolationFlags` asserts read-only root, `cap_drop=ALL`, non-root UID 10001, PID limit 512, no host mounts, no Docker socket and `no-new-privileges`. `is_secure` must be true.

## 3. Egress

- `Disabled`: no outbound network.
- `Allowlist`: exact domain or subdomain, case-insensitive; `example.com` is denied unless listed.
- Wildcards, schemes, ports and paths are rejected.

## 4. Tool policy

- Role allowlists: Tech Lead/Analyst (`git`, `rg`, `jq`), Dev (`cargo`, `rustc`, `npm`, `npx`, `git`, `rg`, `jq`, `make`), QA (`cargo`, `npm`, `npx`, `git`, `rg`, `jq`).
- Forbidden commands: `sudo`, `mount`, `docker`, `nsenter`, `chroot`, `setcap`, `curl`, `wget`, `nc`, `ssh`, `scp`, `bash`, `sh`.
- `git push --force`/`-f` and `cargo/npm publish` are denied.
- Absolute paths and `..` are denied; `.github/workflows`, `CODEOWNERS`, `.env`, `secrets` and `.git/config` are protected.
- Denials are structured with a rule id (`FORBIDDEN_COMMAND`, `COMMAND_NOT_ALLOWED`, `FORCE_PUSH`, `PUBLISH_FORBIDDEN`, `PATH_TRAVERSAL`, `PROTECTED_PATH`).

## 5. Lifecycle

`Creating → Running → Stopped`. `exec` is allowed only while running and always evaluates the tool policy first. `stop` records the exit code.

## 6. Tests

`cargo test -p ailab-orchestrator` covers spec validation and isolation, egress allow/deny, role allowlists, forbidden commands, force-push, publish, protected paths, traversal and the lifecycle.

## 7. Deferred to the WP-14 follow-up

- Runner service, control-plane API and auth token.
- Docker socket proxy allowlist and egress proxy runtime.
- 20+ tool registry with JSON Schemas and the 8-stage policy engine.
- Real container creation, output caps, cleanup/orphan reconciliation and metrics.
- Escape/resource/egress/token suites with a live Docker host.

## 8. References

- `.docs/sow/WP-14-SANDBOX-RUNNER.md`
- `.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §16.3
- `crates/orchestrator/src/sandbox.rs`
