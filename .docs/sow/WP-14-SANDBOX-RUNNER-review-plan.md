# WP-14 — Sandbox Runner & Tool Policy Review Plan

| Field | Value |
|---|---|
| WP | `WP-14` |
| Contract | `.docs/sow/WP-14-SANDBOX-RUNNER.md` |
| Review branch | `review/wp-14-sandbox-runner` |
| Reviewer | Chief Software Engineer (agent) |
| QA reviewer | qa-review-agent (independent subagent) |
| Status | `in_progress` |

## 1. Contract completeness

- [x] Header fields, objective, scope, engineering specification, deliverables, tasks, acceptance criteria, DoD, risks, references and status history are present.
- [x] Requirement traceability maps to spec §16, Appendix B §B.2 and Appendix D §D.1 (M5).

## 2. Specification conformance

- [x] Sandbox spec, isolation flags, egress policy and role tool policy — evidence: `crates/orchestrator/src/sandbox.rs`, `.docs/product/sandbox.md`.
- [x] Runner control-plane HTTP API (`POST /v1/sandboxes`, status, SSE logs, exec, delete, health) — evidence: `crates/orchestrator/src/runner_api.rs`, 5 API tests; the runner binary serves it.
- [ ] Live escape suite (host FS, Docker socket, network scan, privilege escalation, fork bomb, disk fill) — **NOT IMPLEMENTED**.
- [ ] Egress proxy runtime enforcement — policy exists; runtime deferred.
- [ ] Sandbox token injection/revocation — token broker exists in WP-12; injection missing.

## 3. Deliverables

- [x] Sandbox control-plane types and runner health — types + health listener.
- [x] Isolation flags assert read-only/cap-drop/non-root/no host/socket — evidence: `IsolationFlags`.
- [x] Egress policy matching — evidence: `EgressPolicy`.
- [x] Role tool policy + denial rules — evidence: `ToolPolicy`.
- [x] Token broker — evidence: WP-12 `TokenBroker`.
- [x] Spec limits — evidence: `SandboxSpec`.
- [x] HTTP runner + exec/logs + token auth + idempotent create — evidence: `runner_api.rs`.
- [x] Docker backend with isolation host config, worktree bind, network none/egress, resource limits and 1 MB output caps — evidence: `docker_backend.rs`, `host_config_enforces_isolation` test.
- [ ] Live Docker escape/resource/egress suite execution — `#[ignore]` test delivered; run on the Docker host in WP-21.
- [ ] Token injection into the sandbox and egress-proxy runtime wiring — WP-13/WP-12 integration.

## 4. Acceptance criteria

- [x] Policy types and denial tests — 5 sandbox tests.
- [x] HTTP API auth/idempotency/exec policy/delete/health — verified in tests.
- [x] Docker isolation host config (read-only, cap drop, non-root, PIDs, no socket, worktree-only bind) and output caps — unit test.
- [ ] AC live escape/egress/token-load execution — Docker host suite WP-21 and token/egress wiring WP-13.

## 5. Implementation and tests

- [x] `EgressPolicy`, `SandboxSpec`, `IsolationFlags`, `ToolPolicy`, `Sandbox` state — tests.
- [x] Runner HTTP API: auth, idempotent create, status, SSE logs, exec policy denial, delete, health — 5 tests.
- [ ] Live escape/egress/token-load suites — WP-21; token injection and proxy network wiring — WP-13/WP-03.

## 6. Security and quality gates

- [x] `scripts/harness-lint.sh` green (16 checks).
- [x] `scripts/sow-lint.sh` green (31 checks).
- [x] `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p ailab-orchestrator` green.

## 7. Documentation, ADRs and indexes

- [x] `.docs/product/sandbox.md` documents the policy subset and deferrals.
- [x] `AGENTS.md` indexes updated for this review plan.
- [ ] WP-14 status must not be `done` until the runner runtime is delivered.

## 8. QA review

- [x] QA subagent verdict: `CHANGES_REQUESTED` (HEAD `a18724a`).
- [x] Author blocker: the contract's HTTP runner, exec/logs, escape suite and egress runtime are missing; the `done` claim is not evidence-backed (CR-0009).
- [x] QA blocking defects: role is hard-coded to Dev (no run-policy/JSON-Schema tool registry); denial lacks tool/argument/audit; path policy misses `.env*`/`*.pem`/`*.key`/`.git/hooks`/symlinks and the API never authorizes writes; egress runtime/DNS/proxy not wired and the network name mismatches Compose; arbitrary container ids accepted; no 45-min container timeout; CPU/memory/timeout bounds too wide; output cap 1 MB vs 20 KB and no `TOOL_TIMEOUT`/process-group kill; no orphan reconciliation; no sandbox token injection/revocation; no live escape/resource/egress suite.

## 9. Deferred items and residual risks

- [ ] Runner HTTP API + Docker backend + exec/logs — build phase.
- [ ] Escape suite, egress runtime, token injection — build phase + WP-20/WP-21.
- [ ] No merge until the blockers are delivered and QA returns `APPROVED`.

## 10. Decisions

- [x] CR-0009 reopens WP-14 and sequences the build phase (WP-14 → WP-16 → WP-13 → WP-15 → WP-17..23).
