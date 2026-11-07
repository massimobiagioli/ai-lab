# WP-01 — Architecture & Service Topology Review Plan

| Field | Value |
|---|---|
| WP | `WP-01` |
| Contract | `.docs/sow/WP-01-ARCHITECTURE.md` |
| Review branch | `review/wp-01-architecture` |
| Reviewer | Chief Software Engineer (agent) |
| QA reviewer | qa-review-agent (independent subagent) |
| Status | `in_review` |

## 1. Contract completeness

- [x] Header fields, objective, scope, engineering specification, deliverables, tasks, acceptance criteria, DoD, risks, references and status history are present and consistent.
- [x] Requirement traceability (`NFR-001..012`, `FR-ORC-*`, `DB-*`, `BE-001..022`, SOW §11) maps to the specification sections cited in §9.

## 2. Specification conformance

- [x] Specification §4 (architecture), §6 (domain), §13–§15 (contracts, data, operations) — evidence: `.docs/product/architecture.md`, `.docs/product/domain-model.md`, `.docs/product/contracts.md`, `.docs/product/data-ownership.md`.
- [x] Appendix B §B.1–B.4 communication and runner contracts — evidence: `.docs/product/contracts.md` §3.7/§3.8, ADR-0023.
- [x] Appendix D §D.3 traceability — evidence: `.docs/sow/WP-01-ARCHITECTURE.md` §6/§9.

## 3. Deliverables

- [x] `.docs/product/architecture.md` (C4 L1/L2/L3, trust boundaries, flows) — evidence: file exists; diagrams embedded.
- [x] `.docs/product/domain-model.md` — evidence: file exists.
- [x] `.docs/product/service-catalog.md` — evidence: file exists; `.tools/check-architecture-fitness.sh` compose coverage check.
- [x] `.docs/product/contracts.md` — evidence: file exists.
- [x] `.docs/product/data-ownership.md` — evidence: `.tools/architecture/contexts.tsv`, fitness check.
- [x] `.docs/product/failure-modes.md` — evidence: file exists.
- [x] Four Mermaid sources + four fresh SVG exports — evidence: `.docs/product/diagrams/` (SVGs newer than `.mmd`).
- [x] ADR-0001…ADR-0023 committed and linked — evidence: `.docs/adr/registry.md`, harness-lint ADR checks.
- [x] Fitness test + self-test wired in CI — evidence: `.tools/check-architecture-fitness.sh` 4 checks; `.tools/architecture-fitness-self-test.sh` 13 cases; `ci.yml`.
- [x] Independent review record — evidence: `.docs/product/evidence/WP-01-review.md`.

## 4. Acceptance criteria

- [x] AC1 every context has one owner, aggregates, owned tables, published/consumed events — evidence: `.docs/product/domain-model.md`, `.docs/product/data-ownership.md`, `.tools/architecture/contexts.tsv` (32/32 tables mapped once).
- [x] AC2 fitness test fails on cross-context table access — evidence: `.tools/architecture-fitness-self-test.sh` 13/13, including schema-qualified, quoted, escaped, multi-line, lowercase and unattributed SQL.
- [x] AC3 service catalog covers every Compose container — evidence: `.tools/check-architecture-fitness.sh` scans `infra/compose/docker-compose*.yml`; all 8 base services plus the `tls`, `observability` and `egress` profile services are catalogued; self-test 15/15.
- [x] AC4 runner API contract implemented by WP-14 and consumed by WP-13 without drift — DEFERRED to WP-13/WP-14 with ADR-0023 as the canonical contract; compatibility test owned there.
- [x] AC5 every ADR exists, follows the template and is linked — evidence: 24 ADRs in `.docs/adr/registry.md`; harness-lint ADR field check.
- [x] AC6 failure matrix exercised by chaos tests — DEFERRED to WP-21 (chaos suite owner); matrix delivered in `.docs/product/failure-modes.md`.
- [x] AC7 diagrams build from source and are fresh — evidence: `.docs/product/diagrams/*.svg` newer than `*.mmd`.

## 5. Implementation and tests

- [x] Bounded-context ownership enforced — test: `.tools/check-architecture-fitness.sh` 4/4.
- [x] Crate dependency direction and acyclicity — test: `.tools/check-crate-dependencies.sh` (12 packages).
- [x] Regression coverage for fitness extraction edge cases — test: `.tools/architecture-fitness-self-test.sh` 13/13.
- [x] No real external API in tests.
- [x] Coverage thresholds met (docs-only WP; no new executable production code in this WP).

## 6. Security and quality gates

- [x] Trust boundaries and forbidden-access rules documented and enforced — `.docs/product/architecture.md`, `.docs/product/data-ownership.md`, fitness check.
- [x] `scripts/harness-lint.sh` green (16 checks).
- [x] `scripts/sow-lint.sh` green (31 checks).
- [x] `.tools/check-doc-links` green (139 links).
- [x] `.tools/check-architecture-fitness.sh` green; `.tools/check-crate-dependencies.sh` green.

## 7. Documentation, ADRs and indexes

- [x] `.docs/product/architecture.md` and consumers updated.
- [x] `.docs/adr/registry.md` lists ADR-0001..0024 consistent with the files.
- [x] `AGENTS.md` indexes updated for this review plan.
- [x] `.docs/sow/SOW.md` and WP-01 status/progress updated.

## 8. QA review

- [ ] QA subagent verdict: `APPROVED`.
- [ ] QA evidence: reproduced gates (harness 16, sow-lint 31, doc links 139, fitness 4/4, self-test 15/15, crate fitness OK).
- [x] First QA pass returned `CHANGES_REQUESTED` for the compose-coverage false negative, ADR template conformance, contradictory AC3 text and pre-asserted approval; all fixed and sent for re-review.

## 9. Deferred items and residual risks

- [x] AC4 runner contract integration + compatibility test — owner: Dev, WP-13/WP-14, contract ADR-0023.
- [x] AC6 chaos coverage — owner: QA, WP-21, matrix in `.docs/product/failure-modes.md`.
- [x] No unexplained deferral remains.

## 10. Decisions

- [x] ADR-0002 modular monolith + ADR-0023 runner control-plane API — decision, alternatives and consequences recorded.
