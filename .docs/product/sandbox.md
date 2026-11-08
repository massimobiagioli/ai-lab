# AI-Lab Sandbox

| Field | Value |
|---|---|
| Document | `.docs/product/sandbox.md` |
| Version | 1.0.0 |
| Status | Accepted |
| Owner | Dev |
| Related | `.docs/product/tools.md`, `.docs/product/security.md`, `crates/orchestrator/src/sandbox.rs` |

The sandbox is the only place agent code runs. The specification, egress policy, tool policy and lifecycle are implemented in `crates/orchestrator/src/sandbox.rs`. The runner control-plane API and the Docker backend are implemented in `crates/orchestrator/src/{runner_api,backend,docker_backend}.rs`.

## 1. Specification

- `run_id` is non-empty and cannot contain `/` or `..`.
- `image` must be digest-pinned (`@sha256:` + 64 hex).
- `worktree_ref` cannot escape the workspace.
- CPU 1000–4000 millicores, memory 1024–8192 MB, timeout 1–2700 s; the container is force-removed at the timeout.
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

## 7. Runner control plane

`POST /v1/sandboxes` (idempotent by `run_id`), `GET/DELETE /v1/sandboxes/{id}`, SSE logs, allowlisted `POST /exec`, and health endpoints. Auth is `Bearer AILAB_RUNNER_TOKEN` (constant-time compare). The Docker backend creates the container with `readonly_rootfs`, `cap_drop=ALL`, `no-new-privileges`, non-root 10001, PID/memory/CPU limits, a `tmpfs` `/tmp`, only the run worktree bind-mounted at `/workspace`, and `network_mode=none` unless the egress allowlist is enabled. With egress enabled it injects `HTTP_PROXY`/`HTTPS_PROXY`/`NO_PROXY` on the internal `sandbox-egress` network; a short-lived SCM token is injected only into the `exec` environment as `AILAB_SCM_TOKEN` (never the whole sandbox, never serialized/logged). Exec output is capped at 20 KB per stream; a timeout returns `TOOL_TIMEOUT`. An ignored live escape suite checks the Docker socket and `/etc/shadow` are unreachable.

## 8. Deferred to the WP-14 follow-up

- Runner service, control-plane API and auth token.
- Docker socket proxy allowlist and egress proxy runtime.
- 20+ tool registry with JSON Schemas and the 8-stage policy engine.
- Real container creation, output caps, cleanup/orphan reconciliation and metrics.
- Escape/resource/egress/token suites with a live Docker host.

## 9. References

- `.docs/sow/WP-14-SANDBOX-RUNNER.md`
- `.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §16.3
- `crates/orchestrator/src/sandbox.rs`
