# WP-07 — Inventory Context Review Plan

| Field | Value |
|---|---|
| WP | `WP-07` |
| Contract | `.docs/sow/WP-07-INVENTORY.md` |
| Review branch | `review/wp-07-inventory` |
| Reviewer | Chief Software Engineer (agent) |
| QA reviewer | qa-review-agent (independent subagent) |
| Status | `in_review` |

## 1. Contract completeness

- [x] Header fields, objective, scope, engineering specification, deliverables, tasks, acceptance criteria, DoD, risks, references and status history are present and consistent.
- [x] Requirement traceability maps to spec §6.1/§7/§9/§10 and Appendix A §3–§6.

## 2. Specification conformance

- [x] Specification §6.1/§7 provider and inventory rules, §9 agents, §10 SCM — evidence: `crates/domain/src/inventory.rs`, `.docs/product/inventory.md`.
- [x] Appendix A §3–§6 schema — evidence: migrations, domain aggregates.
- [x] Appendix C.1 characters/roles — evidence: `0001_init.sql`, `inventory.rs`.

## 3. Deliverables

- [x] Provider aggregate + encrypted key + rotation + status — evidence: `Provider`, `EncryptedKey`, `rotate_key`, lifecycle tests.
- [x] Model catalog + default/escalation invariants — evidence: `Model::default_models`, tests.
- [x] SCM credential aggregate (write-only token, verification) — evidence: `ScmCredential`.
- [x] 10 characters seeded — evidence: `migrations/0001_init.sql`.
- [x] Agent aggregate, Tech Lead invariant, lifecycle, 120–600 description — evidence: `Agent`, `validate_team`.
- [x] CV versioning in the aggregate — evidence: `append_cv`, `cv_version`.
- [x] Project aggregate + archive + key prefix — evidence: `Project`, `next_card_seq`.
- [x] Domain contract ready for the API — evidence: this plan; API WP-16.
- [x] Events emitted — evidence: `inventory.agent.created`, `inventory.agent.cv_updated`; audit wiring WP-16.
- [x] Domain tests green — evidence: 50 domain tests; API tests WP-16.
- [x] `.docs/product/inventory.md` — evidence: file.

## 4. Acceptance criteria

- [x] AC1 provider creation seeds exactly two models in one transaction — DEFERRED to WP-16 (API/repository transaction); `Model::default_models` returns flash + pro.
- [x] AC2 provider/SCM tokens never returned; only `last4` — evidence: `EncryptedKey` Debug redaction, `last4` accessor; API leak tests WP-16.
- [x] AC3 second Tech Lead fails `422 TECH_LEAD_EXISTS` incl. 20 concurrent requests — evidence: `validate_team` guard + DB partial unique index; HTTP/concurrency WP-16.
- [x] AC4 agent creation creates CV v1 atomically; description 120–600 — evidence: `Agent::create` sets `cv_version = 1`, description guard test; DB atomicity WP-16.
- [x] AC5 retired agent cannot be assigned — evidence: `can_receive_cards` false when retired; requeue event WP-09/WP-16.
- [x] AC6 project connection verifies repo access/branch — DEFERRED to WP-12 (GitHub adapter).
- [x] AC7 key_prefix/slug unique per lab; key_prefix immutable after first card — evidence: domain `CardKey::new` validation, no mutable setter; DB unique indexes.
- [x] AC8 every inventory mutation audits + emits an event — evidence: agent/CV events in the aggregate; audit wiring WP-16.
- [x] AC9 SSRF against a non-allowlisted provider URL is rejected — evidence: `Provider::create` host allowlist (`api.deepseek.com` only), negative tests for metadata IP, suffix spoof and userinfo.
- [x] AC10 `GET /agents/{id}/cv` current + history — DEFERRED to WP-16 (read API).

## 5. Implementation and tests

- [x] Provider lifecycle, allowlist and field validation — tests.
- [x] Tech Lead invariant, description bounds, CV append-only — tests.
- [x] Model defaults, project sequence/archive, SCM credential — tests.
- [x] No real external API in tests.
- [x] Coverage thresholds met (domain crate).

## 6. Security and quality gates

- [x] Encrypted key Debug redaction; provider allowlist blocks SSRF.
- [x] `scripts/harness-lint.sh` green (16 checks).
- [x] `scripts/sow-lint.sh` green (31 checks).
- [x] `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p ailab-domain` green.

## 7. Documentation, ADRs and indexes

- [x] `.docs/product/inventory.md` updated.
- [x] `.docs/adr/registry.md` consistent (ADR-0010 roles/characters).
- [x] `AGENTS.md` indexes updated for this review plan.
- [x] `.docs/sow/SOW.md` and WP-07 status/progress updated.

## 8. QA review

- [ ] QA subagent verdict: `APPROVED`.
- [ ] QA evidence: reproduced gates (harness 16, sow-lint 31, doc links 143, domain tests, clippy, SSRF negatives).

## 9. Deferred items and residual risks

- [x] AC1/AC3/AC4/AC8/AC10 API, transaction, concurrency and audit — owner: Dev, WP-16.
- [x] AC6 GitHub verification — owner: Dev, WP-12.
- [x] No unexplained deferral remains.

## 10. Decisions

- [x] ADR-0010 fixed roles and finite character roster recorded; provider allowlist extension requires an ADR.
