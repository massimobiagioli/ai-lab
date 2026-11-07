# WP-11 — LLM Platform & Prompt Engineering Review Plan

| Field | Value |
|---|---|
| WP | `WP-11` |
| Contract | `.docs/sow/WP-11-LLM-PLATFORM.md` |
| Review branch | `review/wp-11-llm-platform` |
| Reviewer | Chief Software Engineer (agent) |
| QA reviewer | qa-review-agent (independent subagent) |
| Status | `in_review` |

## 1. Contract completeness

- [x] Header fields, objective, scope, engineering specification, deliverables, tasks, acceptance criteria, DoD, risks, references and status history are present and consistent.
- [x] Requirement traceability maps to spec §9/§16, Appendix C §C.2 and Appendix B §B.3.

## 2. Specification conformance

- [x] Specification §9 LLM and §16 security — evidence: `crates/llm/src/lib.rs`, `.docs/product/llm.md`, `.docs/product/agents.md`.
- [x] Appendix C §C.2 prompt/role contracts — evidence: `PromptAssembler`.
- [x] Appendix B §B.3 structured outputs — evidence: `complete_structured`, `DraftCard`.

## 3. Deliverables

- [x] `LlmProvider` port, types, errors — evidence: `lib.rs`.
- [x] `DeepSeekProvider` complete/health over injectable transport — evidence: `DeepSeekProvider`, `HttpTransport`.
- [x] Prompt assembler normative order (role charter, character, CV, hard rules, tools, output contract, task, untrusted) + SHA-256 `prompt_hash` — evidence: `PromptAssembler::assemble`, test.
- [x] Hard rules, role charter, 600-char cap — evidence: `new`/`assemble`, tests.
- [x] Schema validation + exactly one repair — evidence: `complete_structured`, tests.
- [x] Routing with a recorded reason — evidence: `route`, test; persistence WP-16.
- [x] Retry, exponential backoff, breaker; Redis limiter WP-16 — evidence: `RetryPolicy`, `CircuitBreaker`, `complete`.
- [x] Micro-USD cost, budgets, hard stop; ledger WP-16 — evidence: `ModelPrice::cost`, `Budget::charge`.
- [x] Secret guard before the call — evidence: `contains_secret`, `assemble`; persistence WP-16.
- [x] Health probe; metrics/traces WP-20 — evidence: `health`.
- [x] Fake provider/transport + determinism tests — evidence: `FakeTransport`, tests.
- [x] `.docs/product/agents.md` and `.docs/product/llm.md` — evidence: files.

## 4. Acceptance criteria

- [x] AC1 same inputs → same `prompt_hash`; deterministic section order — evidence: `prompt_is_deterministic_and_wraps_untrusted_content` asserts order and hash; role/purpose snapshots remain a WP-11 follow-up.
- [x] AC2 schema validation + exactly one repair then `InvalidOutput` — evidence: `structured_output_repairs_once`, `structured_output_fails_after_one_repair`.
- [x] AC3 429/5xx/timeouts retry with 4 attempts and 1/2/4/8 s ±20% backoff honoring `Retry-After`; 4xx auth no retry — evidence: `RetryPolicy`, `complete` with `futures_timer::Delay`; tests.
- [x] AC4 breaker opens after 5 failures, half-open after 60 s — evidence: `CircuitBreaker`, test.
- [x] AC5 micro-USD cost and budget hard stop — evidence: `ModelPrice::cost`, `Budget::charge`; DB/Redis ledger WP-16.
- [x] AC6 Redis unavailable → DB ledger — DEFERRED to WP-16.
- [x] AC7 provider keys never in logs/prompts/errors — evidence: `contains_secret` guard (provider/SCM/cloud prefixes, PEM), tests; log/trace wiring WP-20.
- [x] AC8 escalation recorded with reason and cost — evidence: `route` reason; persistence WP-16.
- [x] AC9 streaming deltas/usage — DEFERRED (explicit deliverable deferral).
- [x] AC10 health probe updates status + event — evidence: breaker-based `health`; status persistence/event WP-16/WP-20.

## 5. Implementation and tests

- [x] Determinism, prompt secret guard, routing, budget, cost, retry, breaker, structured repair — tests.
- [x] No real external API in tests (fake transport).
- [x] Coverage thresholds met (llm crate).

## 6. Security and quality gates

- [x] Secret guard before the call; no key in the prompt.
- [x] `scripts/harness-lint.sh` green (16 checks).
- [x] `scripts/sow-lint.sh` green (31 checks).
- [x] `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p ailab-llm` green.

## 7. Documentation, ADRs and indexes

- [x] `.docs/product/llm.md` and `.docs/product/agents.md` updated.
- [x] `.docs/adr/registry.md` consistent (ADR-0005, ADR-0015, ADR-0017).
- [x] `AGENTS.md` indexes updated for this review plan.
- [x] `.docs/sow/SOW.md` and WP-11 status/progress updated.

## 8. QA review

- [ ] QA subagent verdict: `APPROVED`.
- [ ] QA evidence: reproduced gates (harness 16, sow-lint 31, doc links 143, 10 llm tests, clippy, retry/backoff, section order, secret guard).
- [x] First QA pass returned `CHANGES_REQUESTED` for the retry policy values, blocking sleep and prompt section order; fixed with 4 attempts at 1/2/4/8 s ±20% jitter via `futures_timer`, the Appendix C.2 section order, and updated tests/docs.

## 9. Deferred items and residual risks

- [x] Streaming, tools/reasoning, wiremock — named WP-11 follow-up.
- [x] Redis limiter, DB/Redis ledger, persistence, health events — owner: Dev, WP-16/WP-20.
- [x] No unexplained deferral remains.

## 10. Decisions

- [x] ADR-0005 DeepSeek port, ADR-0015 structured outputs, ADR-0017 budgets recorded.
