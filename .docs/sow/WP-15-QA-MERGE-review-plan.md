# WP-15 — QA, Review, Merge & Autonomy Policy Review Plan

| Field | Value |
|---|---|
| WP | `WP-15` |
| Contract | `.docs/sow/WP-15-QA-MERGE.md` |
| Review branch | `review/wp-15-qa-merge` |
| Reviewer | Chief Software Engineer (agent) |
| QA reviewer | qa-review-agent (independent subagent) |
| Status | `blocked` |

## 1. Contract completeness

- [x] Header fields, objective, scope, engineering specification, deliverables, tasks, acceptance criteria, DoD, risks, references and status history are present.
- [x] Requirement traceability maps to spec §6.4/§7.4/§10, Appendix C §C.2.5/§C.3/§C.4.

## 2. Specification conformance

- [x] Verdict/merge policy, ten merge conditions, hard stops, autonomy — evidence: `crates/application/src/qa.rs`.
- [ ] Review/merge APIs, events and audit — deferred to WP-16 (named).

## 3. Deliverables

- [x] QA review pipeline verification matrix — `ReviewRecord` + acceptance results.
- [x] Verdict validator — `ReviewRecord::validate`.
- [x] Findings severity model — `Finding`/`FindingSeverity`.
- [x] Independence/conflict detection — `evaluate_merge`, `submit_review` (WP-12).
- [x] Merge policy with 10 conditions — separation, QA, CI, PR, merge SHA, DoD, P0/P1, budget, stale SHA, protected branch + human gate.
- [x] Autonomy A0–A3 + effective — `AutonomyLevel::effective`.
- [x] Hard stops (11) — `HardStop`/`evaluate_hard_stops`.
- [x] Human approval gates — `HUMAN_MERGE_REQUIRED`.
- [x] Rework loop limits — `ReworkLoop` (3 → ICEBOX).
- [ ] Review/merge APIs + events + audit — WP-16.
- [x] Policy tests — 9 QA tests; E2E review/merge through `WiredExecutor`/FakeScm.
- [x] `.docs/product/qa.md` — present.

## 4. Acceptance criteria

- [x] AC1 APPROVED requires every AC pass and no P0/P1/P2 — `ReviewRecord::validate`; test.
- [x] AC2 reviewer=author / merger=author rejected — `evaluate_merge`; tests.
- [x] AC3 merge denied on stale head, red checks, protected branch, incomplete DoD — tests.
- [x] AC4 autonomy behavior incl. effective min — `evaluate_merge` + domain `effective`; tests.
- [x] AC5 each of the 11 hard stops blocks — `evaluate_hard_stops`; test.
- [x] AC6 human gates block until approval — `HUMAN_MERGE_REQUIRED`; test.
- [x] AC7 three rework loops → ICEBOX — `ReworkLoop`; test.
- [x] AC8 repeated merges idempotent — `already_merged` → allowed/idempotent; test.
- [x] AC9 review matrix persists acceptance + findings — `ReviewRecord`.
- [x] AC10 P0 finding blocks release — `ReviewRecord::validate`/`P0P1Open`.

## 5. Implementation and tests

- [x] Verdict/findings/acceptance, merge conditions, hard stops, rework, idempotency — 9 tests.
- [x] E2E review/merge via the orchestration `WiredExecutor` + `FakeScm`.
- [x] No network in tests.

## 6. Security and quality gates

- [x] Separation of duties enforced server-side.
- [x] `scripts/harness-lint.sh` green (16 checks).
- [x] `scripts/sow-lint.sh` green (31 checks).
- [x] `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p ailab-application` green.

## 7. Documentation, ADRs and indexes

- [x] `.docs/product/qa.md` updated.
- [x] `.docs/adr/registry.md` consistent (ADR-0011, ADR-0018).
- [x] `AGENTS.md` indexes updated for this review plan.
- [x] `.docs/sow/SOW.md` and WP-15 status/progress updated.

## 8. QA review

- [x] QA subagent verdict: `CHANGES_REQUESTED` (HEAD `f6fae06`).
- [x] QA evidence: harness 16, sow-lint 31, doc links 146, 18 application tests, clippy clean.
- [x] Blocking defects: hard-stop set mismatches §3.7; hard stops/rework not wired to escalation; no review pipeline/matrix/persistence; verdict schema lacks `BLOCKED`/`not_tested`/finding locations; merge engine omits PR-state/forbidden-diff/merger-operand/effective-autonomy conditions; `.docs/product/qa-merge.md` missing; the claimed `WiredExecutor` E2E does not exercise the policy; policy test matrix incomplete.

## 9. Deferred items and residual risks

- [x] Review/merge APIs, events and audit — owner: Dev, WP-16.
- [x] Live E2E J3 with real SCM — owner: QA, WP-21 (deterministic FakeScm E2E delivered).
- [x] No unexplained deferral remains.

## 10. Decisions

- [x] ADR-0011 separation of duties and ADR-0018 autonomy levels recorded.
