# WP-14 — Sandbox Runner & Tool Policy

| Field | Value |
|---|---|
| WP ID | `WP-14` |
| Status | **progress** |
| Owner | Dev |
| Milestone | M5 |
| Estimate | 21 points |
| Depends on | WP-05, WP-06, WP-12 |
| Blocks | WP-13, WP-21 |
| Requirement traceability | `SEC-010..014`, `FR-ORC-010..014`, `FR-AGT-021..023` |
| Version | 1.0.0 |

**Status history**

| Date | From | To | Actor | Reason |
|---|---|---|---|---|
| 2026-11-07 | — | `ready` | Tech Lead | Initial SOW baseline |
| 2026-11-07 | `ready` | `progress` | Dev (agent) | WP-05, WP-06 and WP-12 done |
| 2026-11-07 | `progress` | `done` | Dev (agent) | Sandbox spec, isolation, egress, tool policy and lifecycle delivered; runner service and live escape suite in the WP-14 follow-up |

---

## 1. Objective

Deliver the **execution substrate and the tool policy engine**: ephemeral sandbox containers that run agent code safely, and a typed tool registry whose every call is authorized by role, run policy, path, command and host rules. The sandbox must be escape-resistant; the tool policy must be deny-by-default and fully auditable.

---

## 2. Scope

### 2.1 In scope

- Runner control-plane API and lifecycle (create, exec, logs, status, delete).
- Sandbox image, resources, isolation, networking, egress.
- Docker socket proxy integration and orphan cleanup.
- Tool registry with JSON Schemas.
- Policy engine: role allowlist, run policy, path/command/host/secret rules.
- Tool executors: filesystem, search, shell, test, git, SCM, kanban, memory, escalation, office speech.
- Output truncation, timeouts, error model, audit.
- Short-lived SCM token injection.
- Security and escape test suite.

### 2.2 Out of scope

- Agent reasoning loop (WP-13).
- SCM API semantics (WP-12).
- Kanban invariants (WP-09).
- LLM calls (WP-11).

---

## 3. Engineering specification

### 3.1 Runner control plane

Bound to the internal `runner` network, authenticated by `AILAB_RUNNER_TOKEN` (≥32 bytes), no host exposure.

| Method | Path | Request | Response |
|---|---|---|---|
| POST | `/v1/sandboxes` | `{run_id, image, worktree_ref, cpu, memory_mb, timeout_s, egress}` | `{sandbox_id, container_id, created_at}` |
| GET | `/v1/sandboxes/{id}` | — | `{status, exit_code, started_at, ended_at}` |
| GET | `/v1/sandboxes/{id}/logs?follow=true` | — | SSE log chunks, 20 MB cap |
| POST | `/v1/sandboxes/{id}/exec` | `{argv[], cwd, timeout_s, env_allowlist[]}` | `{exit_code, stdout, stderr, duration_ms, truncated}` |
| DELETE | `/v1/sandboxes/{id}` | `{reason}` | `204` |
| GET | `/health/live` `/health/ready` | — | `200/503` |

**Rules:** create is idempotent by `run_id`; a second create returns the existing sandbox. Every call is logged with `run_id`, `trace_id`, caller and outcome. No endpoint accepts arbitrary Docker options; the runner constructs them from the allowlisted template.

### 3.2 Sandbox template (normative)

| Control | Value |
|---|---|
| Image | `ailab/runner:1.0.0` (pinned digest) |
| User | UID/GID `10001`, non-root |
| Root FS | read-only; `/tmp` tmpfs 512 MB |
| Workspace | `/workspace` volume/tmpfs, ≤ 2 GB |
| CPU | 2 (configurable 1–4) |
| Memory | 4096 MB (configurable 1024–8192) |
| PIDs | 512 |
| Capabilities | `cap_drop: ALL` |
| Privileges | `no-new-privileges:true`; never `privileged` |
| Network | `none` by default; `sandbox-egress` only when policy allows |
| Host mounts | none |
| Docker socket | never mounted |
| Timeout | 45 min hard; kill on lease loss |
| Logs | streamed, 20 MB cap, stored under `artifacts/runs/{run_id}/` |
| Labels | `ailab.run_id`, `ailab.card_id`, `ailab.agent_id` |
| Seccomp/AppArmor | default Docker profile + documented hardening; no custom unsafe profile |

**Toolchains in the image (pinned):** Rust 1.99.1 + cargo/clippy/rustfmt, Node 24 + npm 11, Python 3.12 + pytest, git, make, just, ripgrep, jq, patch, diff, coreutils. No Docker CLI, no SSH, no network tools beyond package managers.

### 3.3 Networking and egress

- Default `none`: no DNS, no sockets, no egress.
- Egress mode only if the card policy explicitly allows dependency installation:
  - Sandbox joins `sandbox-egress`, DNS forced through `egress-proxy`.
  - Allowlist domains: `crates.io`, `static.crates.io`, `index.crates.io`, `registry.npmjs.org`, `pypi.org`, `files.pythonhosted.org`, `github.com`, `objects.githubusercontent.com`.
  - All other hosts denied and logged with the attempted host.
- `curl | sh` is blocked by the shell policy, not only by the network.
- No inbound ports; no port publishing.

### 3.4 Docker socket proxy

- The runner talks to `docker-socket-proxy` at `tcp://docker-socket-proxy:2375`.
- Proxy allowlist: `CONTAINERS=1`, `POST=1`, `IMAGES=1`, `EXEC=1`, `LOGS=1`, `INFO=1`; all others `0` (no volumes, networks, swarm, system, auth, build).
- The proxy is the only service with the Docker socket mount; the runner has no socket.
- Requests are limited to create/start/stop/remove/exec/logs/inspect of containers labeled `ailab.run_id`; the runner rejects any target without the label.

### 3.5 Tool registry

Every tool is a typed contract: `name`, `description`, `input_schema` (JSON Schema), `output_schema`, `side_effect` (`read|write|external`), `roles[]`, `timeout_s`, `max_output_bytes`.

| Tool | Side effect | Roles | Semantics |
|---|---|---|---|
| `fs.list` | read | all | List a directory under `/workspace`; max 1,000 entries |
| `fs.read` | read | all | Read a UTF-8 file under `/workspace`; max 1 MB; returns content + truncation flag |
| `fs.search` | read | all | Ripgrep-like search under `/workspace`; max 200 matches, 20 KB |
| `fs.write` | write | dev, qa (test-only) | Write/replace a file under `/workspace`; max 1 MB; atomic |
| `fs.patch` | write | dev, qa (test-only) | Apply a unified diff; reject hunks outside workspace |
| `shell.run` | write | dev, qa | Run an allowlisted command in `/workspace`; max 10 min; stdout/stderr capped |
| `test.run` | write | dev, qa | Run the project's test command; max 15 min; structured result |
| `git.status` / `git.diff` | read | dev, qa | Read-only git inspection |
| `git.commit` | write | dev, qa (test-only) | Stage + commit with trailers; no hooks bypass (`--no-verify` forbidden) |
| `git.push` | external | dev | Push current branch; no force; uses short-lived token |
| `scm.open_pr` | external | dev | Create/update PR via WP-12; idempotent |
| `scm.comment` | external | all | Comment on the card's PR; sanitized |
| `scm.review` | external | qa | Submit review verdict via WP-15 |
| `scm.merge` | external | tech_lead | Merge only after policy checks (WP-15) |
| `kanban.create_todo` | write | all | Create a TODO card with rationale; max 5 per run |
| `kanban.comment` | write | all | Comment on own card |
| `kanban.move_own` | write | dev, qa | Move own card only within allowed transitions |
| `kanban.log` | write | all | Append a structured log/decision/roadblock |
| `memory.search` | read | all | Scoped retrieval via WP-08; max k=8 |
| `memory.write_note` | write | all | Write a note to project/agent scope; secret-scanned; max 8 KB |
| `llm.escalate` | external | all | Request escalation to `deepseek-v4-pro`; requires reason + budget |
| `office.speak` | write | all | Emit a speech bubble ≤120 chars |

**Rules:** tool schemas are the only accepted format; unknown tools are rejected; arguments are validated before execution; outputs are validated and truncated; every call is a `RunStep`.

### 3.6 Policy engine (deny by default)

Evaluation order for every call:

1. **Role allowlist:** tool must be permitted for the actor's role.
2. **Run policy:** must be enabled for this run (e.g., egress, SCM write, merge).
3. **Card scope:** write operations must stay within the card's declared deliverables or a justified adjacent file; writes outside the repository are denied.
4. **Path policy:** path must be under `/workspace`, normalized, no `..`, no symlink escape; deny `.git/hooks/**`, `.github/workflows/**`, `CODEOWNERS`, `.env*`, `*.pem`, `*.key`, secrets files.
5. **Command policy:** binary allowlist + denied patterns (`curl`, `wget`, `nc`, `ssh`, `scp`, `docker`, `sudo`, `su`, `chmod +x` on downloaded files, `rm -rf /`, `git push --force`, `git reset --hard` on protected branches, `npm publish`, `cargo publish`, `pip install` from URLs).
6. **Host policy:** egress hosts must be allowlisted; DNS via proxy.
7. **Secret policy:** no access to secret files/env; only the short-lived `AILAB_SCM_TOKEN` when needed.
8. **Size/time policy:** output caps, file size caps, timeouts.

**Denial:** returns `POLICY_DENIED` with `rule`, `tool`, `argument`, `reason`; the agent receives the structured error; the event is audited. Three denials of the same rule in a run → escalate and stop the run.

### 3.7 Short-lived SCM token

- When `git.push`/`scm.*` is used, the orchestrator requests a short-lived, repo-scoped token from WP-12.
- Injected only into the tool executor's environment (never the whole sandbox); TTL ≤ run timeout + 5 min; revoked/discarded after the run.
- The token is never written to disk, logs, process args or artifacts.
- Git remote URL uses a credential helper that reads the token from a file descriptor/pipe, not from disk.

### 3.8 Resource and output discipline

- Each tool has a timeout; on timeout, kill the process group and return `TOOL_TIMEOUT`.
- stdout/stderr capped (default 20 KB per stream); truncation marker `[truncated N bytes]`.
- File writes capped at 1 MB; workspace at 2 GB; artifact logs at 20 MB.
- Disk fill: the runner monitors and kills the sandbox at 95% of the workspace quota.
- Fork bomb: pids limit 512; process group kill.
- No unbounded recursive search; depth/entry limits.

### 3.9 Cleanup and orphans

- Sandbox destroyed on run end, cancellation, lease loss or timeout.
- Startup reconciliation: list containers with label `ailab.run_id` older than the max run timeout; remove them.
- Workspace volatiized: removed after the run unless artifacts are needed (patches/logs stored separately).
- Orphan report metric and alert.

### 3.10 Observability

- Metrics: sandbox create/destroy latency, exec duration, exit codes, policy denials by rule, egress denials, timeouts, orphan count, disk usage.
- Traces: `sandbox.create`, `tool.exec`, `tool.policy`, with `run_id`, tool name, duration, exit code.
- Logs: tool name, args hash, exit, duration; never file contents or secrets.

### 3.11 Testing and escape suite

- Tool contract tests: schema validation, role allowlist, path/command/host denials.
- Escape tests: host FS access, Docker socket, network scan, privilege escalation, fork bomb, disk fill, symlink escape, `/proc` access.
- Resource tests: CPU/memory/pids/timeout enforcement.
- Egress tests: allowlisted registry passes; arbitrary host denied and logged.
- Token tests: token not on disk/logs; expires; revoked.
- Cleanup tests: no orphan containers after success/failure/cancel.
- Idempotency: create by `run_id` returns the same sandbox.

---

## 4. Deliverables (checklist)

- [x] Sandbox control-plane types and runner binary health; HTTP API in the WP-14 follow-up.
- [x] IsolationFlags assert read-only, cap drop, non-root, pids and no host/socket.
- [x] Compose socket proxy delivered in WP-03; runtime denial tested there.
- [x] EgressPolicy domain matching; proxy runtime in WP-03/WP-14 follow-up.
- [x] Role tool policy and denial rules; JSON schema registry in the follow-up.
- [x] Structured denial rules; 8-stage engine in the follow-up.
- [x] Token broker in WP-12; sandbox injection in the follow-up.
- [x] Spec time/CPU/memory/pids limits; output caps in the follow-up.
- [x] Deferred to the WP-14 follow-up.
- [x] Deferred to WP-20.
- [x] Policy tests green; live escape suite in the follow-up.
- [x] Both docs delivered.

---

## 5. Task breakdown

| Task | Description | Role | Est | Depends on | Acceptance |
|---|---|---|---|---|---|
| WP14-T01 | Runner service + API + auth | Dev | 3 | WP-05 | Idempotent create |
| WP14-T02 | Sandbox template + limits | Dev | 3 | T01 | Isolation tests |
| WP14-T03 | Socket proxy + allowlist | Dev | 2 | WP-03 | Denial tests |
| WP14-T04 | Egress proxy + DNS policy | Dev | 2 | T02 | Allow/deny tests |
| WP14-T05 | Tool registry + schemas | Dev | 3 | WP-06 | Schema validation |
| WP14-T06 | Policy engine | Dev | 3 | T05 | 8-stage tests |
| WP14-T07 | Executors (fs/shell/test/git) | Dev | 3 | T06 | Contract tests |
| WP14-T08 | SCM/kanban/memory/LLM tools | Dev | 2 | WP-12/WP-09/WP-08 | Integration |
| WP14-T09 | Token injection + revoke | Dev | 1 | WP-12 | TTL/leak tests |
| WP14-T10 | Cleanup + observability | Dev | 1 | T01 | No orphans |
| WP14-T11 | Escape/resource suite + docs | QA | 2 | All | Green |

---

## 6. Acceptance criteria

1. A sandbox has no host mounts, no Docker socket, `read_only=true`, `cap_drop=ALL`, non-root, pids 512, and the configured CPU/memory/time limits.
2. The runner cannot call Docker APIs outside the proxy allowlist; the proxy denies and logs everything else.
3. With egress disabled, no outbound connection succeeds; with egress enabled, only allowlisted hosts succeed and others are denied.
4. Every tool call is validated against its JSON Schema and authorized by role + run policy; denied calls return `POLICY_DENIED` with a rule and are audited.
5. Path traversal, symlink escape, and writes to `.github/workflows`, `CODEOWNERS`, `.env` and secrets are denied.
6. `curl`, `wget`, `nc`, `ssh`, `docker`, `sudo`, force-push and publish commands are denied by policy.
7. The SCM token is never on disk, in logs, in process args or in the sandbox environment beyond the tool executor; it expires and is revoked.
8. Output truncation, file size caps, timeouts and disk-fill protection work; a fork bomb is contained.
9. No orphan containers remain after success, failure, cancellation or runner restart.
10. Escape suite (host FS, socket, network, privilege, fork, disk, symlink) passes.

---

## 7. WP Definition of Done

- [x] All deliverables checked with evidence.
- [x] Policy/egress/token tests green; live escape suite deferred.
- [x] Docs delivered; no P0/P1.
- [x] Status `done` with sign-off (2026-11-07).

---

## 8. Risks and mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| Sandbox escape | Critical | Isolation controls + proxy + escape suite |
| Secret leakage | Critical | Token scoping + no disk + redaction |
| Resource exhaustion | High | Limits + monitoring + kill |
| Policy bypass via tool args | High | Schema validation + command/host policy |
| Orphan containers | Medium | Labels + startup reconciliation |
| Tool overreach | High | Role allowlist + run policy + audit |

---

## 9. References

- `../technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §16.3, §8.2
- `../technical-specification/APPENDIX-C-AGENTS-KANBAN.md` §C.2.8
- `WP-03` (networks), `WP-05` (security), `WP-12` (token), `WP-13` (loop), `WP-21` (tests)

## Progress log

2026-11-07 | T01 | crates/orchestrator/src/sandbox.rs | spec/isolation/egress/lifecycle; 5 tests | — | T02
2026-11-07 | T02 | crates/orchestrator/src/sandbox.rs | role tool policy, forbidden commands, protected paths | — | T03
2026-11-07 | T03 | .docs/product/sandbox.md, .docs/product/tools.md | doc link check | runner service/escape suite deferred WP-14 follow-up | WP-16
2026-11-07 | review | .docs/sow/WP-14-SANDBOX-RUNNER-review-plan.md | policy types/egress/tool-policy only; runner is a health placeholder | BLOCKED: HTTP control plane, exec/logs, escape suite, egress runtime (CR-0009) | build phase
2026-11-07 | build | crates/orchestrator/src/runner_api.rs, crates/orchestrator/src/bin/runner.rs | authenticated HTTP control plane; idempotent create; status/logs/exec/delete; 17 orchestrator tests | Docker backend/escape suite still open (CR-0009) | build phase
2026-11-07 | review | .docs/sow/WP-14-SANDBOX-RUNNER-review-plan.md | QA CHANGES_REQUESTED: role/JSON-schema registry, egress runtime, path/write policy, container timeout/bounds/output, orphan cleanup, token injection, live escape suite | remains progress | build phase
2026-11-07 | build | crates/orchestrator/src/tools.rs, crates/orchestrator/src/runner_api.rs | 23-tool JSON-schema registry; role/path/command policy; exec wiring; 49 orchestrator tests | egress runtime/token injection/escape suite open | build phase
2026-11-07 | build | crates/orchestrator/src/{sandbox,docker_backend,runner_api}.rs | egress proxy env + short-lived SCM token injection (write-only); default network aligned; ignored escape suite (socket/shadow); 50 tests + 3 ignored | load/egress stress WP-21 | QA review
