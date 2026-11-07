# WP-10 — SOW & Planning Engine Review Plan

| Field | Value |
|---|---|
| WP | `WP-10` |
| Contract | `.docs/sow/WP-10-SOW-PLANNING.md` |
| Review branch | `review/wp-10-sow-planning` |
| Reviewer | Chief Software Engineer (agent) |
| QA reviewer | qa-review-agent (independent subagent) |
| Status | `in_review` |

## 1. Contract completeness

- [x] Header fields, objective, scope, engineering specification, deliverables, tasks, acceptance criteria, DoD, risks, references and status history are present and consistent.
- [x] Requirement traceability maps to spec §6.5/§7.2/§7.4 and Appendix B §B.3.1/§B.3.2/§B.3.4.

## 2. Specification conformance

- [x] Specification §6.5 SOW and §7.2/§7.4 planning — evidence: `crates/domain/src/delivery.rs`, `.docs/product/sow.md`, `.docs/product/planning.md`.
- [x] Appendix B §B.3.1–§B.3.4 contracts — evidence: `SowDocument`, `SowSection`.
- [x] Appendix C §C.2.2/§C.2.3 agent SOW/planning — evidence: plan task roles.

## 3. Deliverables

- [x] SOW/plan domain (API WP-16) — evidence: `delivery.rs`.
- [x] SOW aggregate, immutable versions, lifecycle — evidence: `Sow`, `SowVersion`, tests.
- [x] Semantic validation + deterministic Markdown — evidence: `SowDocument::new`, `markdown`, tests; JSON Schema/export WP-16/WP-22.
- [x] Plan aggregate, DAG validator, lifecycle — evidence: `Plan`, `validate_tasks`, `topological_order`, tests.
- [x] Dry-run preview + traceability fields — evidence: `Plan::preview`, `MaterializedCard` (`task_key`, `covers`, `sow_id`); writes WP-16.
- [x] `sow_id`/`task_key` carried by materialized cards — evidence: `MaterializedCard`; API WP-16.
- [x] FR coverage validation — evidence: `PlanTask.covers`, `Plan::validate_coverage`, test.
- [x] `.docs/product/sow.md` and `.docs/product/planning.md` — evidence: files.

## 4. Acceptance criteria

- [x] AC1 idea → 1–20 cards within 3 s; invalid LLM output → one fallback card — DEFERRED to WP-11/WP-16 (LLM pipeline/API).
- [x] AC2 idempotency (same key/text) — DEFERRED to WP-16 (HTTP idempotency/store).
- [x] AC3 schema + semantic validation — evidence: `SowDocument::new` requires all 17 sections, an executive summary >=120 chars, at least one FR and one NFR, and unique FR/NFR ids; `SowDocument::validate_requirements` rejects missing priority/AC, unknown dependencies and missing NFR target/measurement (structured JSON parse is WP-16).
- [x] AC4 versions immutable, approve freezes, supersede requires a new approved version and change note; reject requires a reason and returns to draft — evidence: `revise` change-note guard, `supersede` requires >=2 versions, `reject` reason guard; immutability/reject/supersede tests.
- [x] AC5 cycle/dangling/duplicate/uncovered FR rejected — evidence: `Plan::create` → `topological_order` (cycles), `validate_tasks` (dangling/duplicates), `validate_coverage` (uncovered/unknown FR); test.
- [x] AC6 materialization idempotent; dry-run preview — evidence: `preview` (no mutation), `mark_materialized` guarded; DB idempotency WP-16.
- [x] AC7 every materialized card links `sow_id`/`plan_task_id`; traceability tree — evidence: `MaterializedCard` carries `task_key`, `covers`, `sow_id`; tree API WP-16.
- [x] AC8 `auto_ready` behavior — DEFERRED to WP-16 (card creation + DoR).
- [x] AC9 generation timeout/budget preserves prior version — DEFERRED to WP-11/WP-16.
- [x] AC10 deterministic Markdown/PDF export without scripts — evidence: `markdown` iterates the fixed section order; PDF/HTML export WP-22.

## 5. Implementation and tests

- [x] SOW lifecycle, semantic validation, deterministic Markdown — tests.
- [x] Plan DAG, topological order, leaves, FR coverage, preview — tests.
- [x] No real external API in tests.
- [x] Coverage thresholds met (domain crate).

## 6. Security and quality gates

- [x] No secret, script or external load in SOW/plan domain; Markdown is plain text.
- [x] `scripts/harness-lint.sh` green (16 checks).
- [x] `scripts/sow-lint.sh` green (31 checks).
- [x] `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p ailab-domain` green.

## 7. Documentation, ADRs and indexes

- [x] `.docs/product/sow.md` and `.docs/product/planning.md` updated.
- [x] `.docs/adr/registry.md` consistent.
- [x] `AGENTS.md` indexes updated for this review plan.
- [x] `.docs/sow/SOW.md` and WP-10 status/progress updated.

## 8. QA review

- [ ] QA subagent verdict: `APPROVED`.
- [ ] QA evidence: reproduced gates (harness 16, sow-lint 31, doc links 143, 59 domain tests, clippy, FR coverage).
- [x] First QA pass returned `CHANGES_REQUESTED` for incomplete SOW semantic validation and the reject/supersede lifecycle; fixed with text-level invariants, a structured-requirement validator, reason-guarded reject (back to draft), version-guarded supersede, and lifecycle/immutability tests.

## 9. Deferred items and residual risks

- [x] AC1/AC2/AC8/AC9 generation pipeline and HTTP idempotency — owner: Dev, WP-11/WP-16.
- [x] AC7 traceability API — owner: Dev, WP-16.
- [x] AC10 PDF/HTML export — owner: Analyst, WP-22.
- [x] No unexplained deferral remains.

## 10. Decisions

- [x] SOW versioning and plan DAG semantics recorded in `.docs/product/sow.md`/`planning.md`.
