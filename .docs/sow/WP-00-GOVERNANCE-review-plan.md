# WP-00 — Program Governance, Lean/Agile Delivery & Quality System Review Plan

| Field | Value |
|---|---|
| WP | `WP-00` |
| Contract | `.docs/sow/WP-00-GOVERNANCE.md` |
| Review branch | `review/wp-00-governance` |
| Reviewer | Chief Software Engineer (agent) |
| QA reviewer | qa-review-agent (independent subagent) |
| Status | `approved` |

## 1. Contract completeness

- [x] Header fields, objective, scope, engineering specification, deliverables, tasks, acceptance criteria, DoD, risks, references and status history are present and consistent.
- [x] Requirement traceability (`NFR-006`, `NFR-011`, `NFR-012`, `DOC-001..003`) maps to the specification sections cited in §9.

## 2. Specification conformance

- [x] Specification §6.4, §17, §21, §22 and §23 — evidence: `.docs/product/status-model.md`, `.docs/product/metrics.md`, `.docs/product/quality-gates.md`.
- [x] Appendix C.3/C.4 and Appendix D.1–D.3 — evidence: `.docs/product/kanban.md`, `.github/pull_request_template.md`, `.docs/product/metrics.md`.

## 3. Deliverables

- [x] `.docs/product/governance.md` — evidence: file exists and passes the link check.
- [x] `.docs/product/quality-gates.md` — evidence: `.github/required-checks.txt` and `scripts/sow-lint.sh` validate the 16 names.
- [x] `.docs/product/status-model.md` — evidence: file exists; `scripts/sow-lint.sh` enforces the model.
- [x] `.docs/product/metrics.md` — evidence: `.tools/test-analytics-views.sh` 25 assertions.
- [x] `.docs/product/change-control.md` — evidence: `.docs/product/change-requests/CR-0001.md`, `CR-0002.md`, `CR-0003.md`.
- [x] Six templates — evidence: `.docs/product/templates/`.
- [x] `.github/pull_request_template.md` — evidence: file exists and matches Appendix C.4.1.
- [x] `scripts/sow-lint.sh` + CI `sow-lint` — evidence: `.tools/sow-lint-self-test.sh` 11 cases; `.github/workflows/ci.yml`.
- [x] `migrations/0002_analytics_views.sql` — evidence: `.tools/test-analytics-views.sh` 25 assertions.
- [x] `.docs/product/risk-register.md` — evidence: file exists with owners and review dates.
- [x] M0 retrospective with three owned actions — evidence: `.docs/product/retrospectives/M0-retro.md`, `.docs/product/backlog/AIL-101.md`..`AIL-103.md`.
- [x] ADR-0001..0004 — evidence: `.docs/adr/adr-0001.md`..`adr-0004.md`, `.docs/adr/registry.md`.

## 4. Acceptance criteria

- [x] AC1: `sow-lint` exits non-zero for status/link/dependency/checklist/stale-log violations — `.tools/sow-lint-self-test.sh` 11/11.
- [x] AC2: every WP validates against the template — `scripts/sow-lint.sh` 31 checks.
- [x] AC3: the 16 required check names exist and branch protection requires them — `.github/required-checks.txt`, `.github/workflows/ci.yml`, `gh api repos/massimobiagioli/ai-lab/branches/main/protection` (CR-0003).
- [x] AC4: metrics views return expected fixture values — `.tools/test-analytics-views.sh` 25/25.
- [x] AC5: sample CR submitted, approved, recorded — CR-0001/CR-0002.
- [x] AC6: retrospective has exactly three actions linked to TODO cards — `.docs/product/retrospectives/M0-retro.md`.
- [x] AC7: separation-of-duties rejection — `crates/api/tests/separation_of_duties.rs` (integration test: `author_cannot_review_own_card_and_maps_to_422`, `merge_policy_rejects_author_as_reviewer`) asserts HTTP 422 with code `SEPARATION_OF_DUTIES`; backed by `crates/domain/src/kanban.rs` and `crates/application/src/qa.rs` unit tests.

## 5. Implementation and tests

- [x] `scripts/harness-lint.sh` 16 checks.
- [x] `scripts/sow-lint.sh` 31 checks.
- [x] `.tools/test-analytics-views.sh` 25 assertions.
- [x] `.tools/test-supply-chain.sh` 2 negative fixtures.
- [x] `.tools/check-doc-links.sh` resolves all relative links.

## 6. Security and quality gates

- [x] No secret or placeholder in the governance artifacts.
- [x] Required check names match the workflow, `required-checks.txt` and `quality-gates.md`.
- [x] Branch protection applied with the 16 required contexts; admin bypass is global per CR-0003, required PR reviews and linear history are disabled, and force pushes/deletions are blocked.
## 7. Documentation, ADRs and indexes

- [x] Product documentation indexed by `.docs/product/README.md`.
- [x] `.docs/adr/registry.md` lists ADR-0001..0024.
- [x] `AGENTS.md` index updated for this review plan.
- [x] `.docs/sow/SOW.md` and WP-00 status/history remain consistent.

## 8. Status and evidence

- [x] WP status `done`; history has the progress → done sign-off.
- [x] Evidence links resolve.
- [x] Independent review record in `.docs/product/evidence/WP-00-review.md`.
- [x] QA subagent verdict: `APPROVED` (first pass `CHANGES_REQUESTED` for AC7; fixed with `crates/api/tests/separation_of_duties.rs` and re-reviewed).
- [x] QA evidence: reproduced gates — harness 16, sow-lint 31, sow-lint self-test 11, doc links 138, `cargo test -p ailab-api --test separation_of_duties` 2 passed, fmt/clippy clean, analytics views 25.

## 9. Deferred items and residual risks

- [x] Metrics dashboard population is assigned to WP-20 by CR-0002; the verified views and formulas are delivered; owner: Dev, expiry: WP-20 completion.
- [x] Branch protection application is closed by CR-0003 with the review-compatible configuration.

## 10. Decisions

- [x] CR-0003: apply the 16 required contexts, disable enforced admin protection globally for the review workflow, allow merge commits (the review prompt uses `--no-ff`), keep force-pushes and deletions blocked.
