# WP-08 — Lab & Three-Tier Memory Review Plan

| Field | Value |
|---|---|
| WP | `WP-08` |
| Contract | `.docs/sow/WP-08-LAB-MEMORY.md` |
| Review branch | `review/wp-08-lab-memory` |
| Reviewer | Chief Software Engineer (agent) |
| QA reviewer | qa-review-agent (independent subagent) |
| Status | `in_review` |

## 1. Contract completeness

- [x] Header fields, objective, scope, engineering specification, deliverables, tasks, acceptance criteria, DoD, risks, references and status history are present and consistent.
- [x] Requirement traceability maps to spec §6.2/§6.3/§7.5 and Appendix A §10.

## 2. Specification conformance

- [x] Specification §6.2/§6.3 lab and memory, §7.5 retrieval — evidence: `crates/domain/src/lab.rs`, `crates/domain/src/memory.rs`, `.docs/product/memory.md`.
- [x] Appendix A §10 memory tables and vector index — evidence: `migrations/0001_init.sql`.
- [x] Appendix C §C.2.6 memory rules — evidence: `memory.rs`.

## 3. Deliverables

- [x] Lab aggregate + settings validation + archive — evidence: `Lab`, `LabSettings`, tests.
- [x] Lab creation enforces one active Tech Lead — evidence: `Lab::create` → `Agent::validate_team`.
- [x] Document/chunk model + version — evidence: `MemoryDocument`, `MemoryChunk`.
- [x] 384d embedding contract + deterministic fake — evidence: `EMBEDDING_DIM`, tests.
- [x] Deterministic 800/120 chunking — evidence: `chunk_text`, test.
- [x] Near-duplicate dedupe + secret quarantine — evidence: `DEDUPE_COSINE`, `contains_secret`, tests.
- [x] Hybrid scoring + scope filter — evidence: `hybrid_search`, tests.
- [x] Key builder/TTLs — evidence: `crates/infrastructure/src/redis_keys.rs`; client WP-16.
- [x] Compaction/retention policy — evidence: `plan_compaction`, test.
- [x] Leakage + formula tests — evidence: scope-filter and formula tests; nDCG/perf WP-21.
- [x] `.docs/product/memory.md` — evidence: file.

## 4. Acceptance criteria

- [x] AC1 lab cannot be created without a Tech Lead — evidence: `Lab::create` → `validate_team`; test `lab_requires_one_tech_lead`.
- [x] AC2 settings reject out-of-range values with field errors — evidence: `LabSettings` models WIP, autonomy flags, scheduler, office and maintenance blocks; `validate` enforces `doing 1-4`, `review 1-10`, budgets `0..=1e12` micro-USD, `tick 250-5000`, `lease 60-3600`, `retries 0-5`, non-empty theme/cron, `stale_days >= 1`; upper-bound tests; unknown keys are rejected by the WP-16 JSON Schema (`additionalProperties=false`).
- [x] AC3 chunking is deterministic — evidence: `chunking_is_deterministic`.
- [x] AC4 secret document is quarantined (not embedded) and a security event is emitted — evidence: `MemoryDocument::new` returns `MEMORY_QUARANTINED`, test; persistence as blocked + event emission is WP-16.
- [x] AC5 near-duplicate (cosine ≥ 0.92) deduped — evidence: `near_duplicates_are_deduped`.
- [x] AC6 retrieval never returns out-of-scope chunks — evidence: `retrieval_is_scope_filtered`.
- [x] AC7 hybrid score matches the formula within 1e-6 — evidence: `hybrid_score_matches_the_formula`; nDCG@8 ≥ 0.8 is WP-21.
- [x] AC8 cold vs warm cache ranking — DEFERRED to WP-16 (Redis/client); ranking is deterministic for a fixed clock.
- [x] AC9 compaction > 30 days, pinned preserved, raw retention > 180 days — evidence: `compaction_respects_pins_and_retention`.
- [x] AC10 retrieval p95 ≤ 80 ms / explorer citations — DEFERRED to WP-21 (performance) and WP-16/WP-19 (explorer).
- [x] AC11 deleting a lab removes documents/chunks/episodes/caches — DEFERRED to WP-16 (cascade + cache invalidation); FK `ON DELETE CASCADE` in Appendix A.

## 5. Implementation and tests

- [x] Lab settings, team invariant, archive — tests.
- [x] Chunking, secret quarantine, dedupe, scope filter, formula, compaction — tests.
- [x] No real external API in tests.
- [x] Coverage thresholds met (domain crate).

## 6. Security and quality gates

- [x] Secret shapes detected (provider/SCM/cloud prefixes and PEM); no secret embedded.
- [x] `scripts/harness-lint.sh` green (16 checks).
- [x] `scripts/sow-lint.sh` green (31 checks).
- [x] `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p ailab-domain` green.

## 7. Documentation, ADRs and indexes

- [x] `.docs/product/memory.md` updated.
- [x] `.docs/adr/registry.md` consistent (ADR-0016).
- [x] `AGENTS.md` indexes updated for this review plan.
- [x] `.docs/sow/SOW.md` and WP-08 status/progress updated.

## 8. QA review

- [ ] QA subagent verdict: `APPROVED`.
- [ ] QA evidence: reproduced gates (harness 16, sow-lint 31, doc links 143, domain tests, clippy).
- [x] First QA pass returned `CHANGES_REQUESTED` for incomplete lab-settings validation; fixed by modelling the full §3.1 settings schema and enforcing every documented range with upper-bound tests.

## 9. Deferred items and residual risks

- [x] AC4 blocked-document persistence + security event — owner: Dev, WP-16.
- [x] AC8 Redis client/cache — owner: Dev, WP-16.
- [x] AC10 nDCG/perf — owner: QA, WP-21.
- [x] AC11 deletion cascade/caches — owner: Dev, WP-16.
- [x] fastembed adapter — owner: Dev, WP-08 follow-up (explicitly deferred in §4).
- [x] No unexplained deferral remains.

## 10. Decisions

- [x] ADR-0016 three-tier memory with SQL-level scope isolation recorded.
