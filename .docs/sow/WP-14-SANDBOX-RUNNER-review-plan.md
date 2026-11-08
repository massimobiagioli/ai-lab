# WP-14 — Sandbox Runner & Tool Policy Review Plan

| Field | Value |
|---|---|
| WP | `WP-14` |
| Contract | `.docs/sow/WP-14-SANDBOX-RUNNER.md` |
| Review branch | `review/wp-14-sandbox-runner` |
| Reviewer | Chief Software Engineer (agent) |
| QA reviewer | qa-review-agent (independent subagent) |
| Status | `blocked` |

## 1. Contract completeness

- [x] Header fields, objective, scope, engineering specification, deliverables, tasks, acceptance criteria, DoD, risks, references and status history are present.
- [x] Requirement traceability maps to spec §16, Appendix B §B.2 and Appendix D §D.1 (M5).

## 2. Specification conformance

- [x] Sandbox spec, isolation flags, egress policy and role tool policy — evidence: `crates/orchestrator/src/sandbox.rs`, `.docs/product/sandbox.md`.
- [ ] Runner control-plane HTTP API (`POST /v1/sandboxes`, status, SSE logs, exec, delete, health) — **NOT IMPLEMENTED**; the runner binary is a health listener.
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
- [ ] HTTP runner, exec/logs, escape suite, egress runtime, token injection, output caps — **missing**.

## 4. Acceptance criteria

- [x] Policy types and denial tests — 5 sandbox tests.
- [ ] AC live isolation/escape, exec allowlist, output caps, egress allow/deny, token scoping/revocation, resource limits under load — not verifiable without the runner runtime.

## 5. Implementation and tests

- [x] `EgressPolicy`, `SandboxSpec`, `IsolationFlags`, `ToolPolicy`, `Sandbox` state — tests.
- [ ] Runner HTTP API, Docker backend, exec/logs, escape suite — missing.

## 6. Security and quality gates

- [x] `scripts/harness-lint.sh` green (16 checks).
- [x] `scripts/sow-lint.sh` green (31 checks).
- [x] `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p ailab-orchestrator` green.

## 7. Documentation, ADRs and indexes

- [x] `.docs/product/sandbox.md` documents the policy subset and deferrals.
- [x] `AGENTS.md` indexes updated for this review plan.
- [ ] WP-14 status must not be `done` until the runner runtime is delivered.

## 8. QA review

- [ ] QA subagent verdict: `PENDING`.
- [x] Author blocker: the contract's HTTP runner, exec/logs, escape suite and egress runtime are missing; the `done` claim is not evidence-backed (CR-0009).

## 9. Deferred items and residual risks

- [ ] Runner HTTP API + Docker backend + exec/logs — build phase.
- [ ] Escape suite, egress runtime, token injection — build phase + WP-20/WP-21.
- [ ] No merge until the blockers are delivered and QA returns `APPROVED`.

## 10. Decisions

- [x] CR-0009 reopens WP-14 and sequences the build phase (WP-14 → WP-16 → WP-13 → WP-15 → WP-17..23).
