# WP-07 Independent Review

| Field | Value |
|---|---|
| Reviewer | review-agent (independent subagent) |
| Author | Dev (agent) |
| Date | 2026-11-07 |
| Scope | Provider, encrypted key, model catalog, SCM credential, characters, agent, CV, project |
| Verdict | `APPROVED` |

One review pass returned `CHANGES_REQUESTED`; the author fixed the defects and the re-review approved.

## Final acceptance results

| AC | Result | Evidence |
|---|---|---|
| AC1 provider seeds two models in one transaction | DEFERRED | `Model::default_models` (flash default + pro escalation); DB transaction is WP-16. |
| AC2 tokens never returned; only last4 | PASS | `EncryptedKey` Debug redaction; no Serialize; API leak tests WP-16. |
| AC3 second Tech Lead rejected incl. concurrency | DEFERRED | `validate_team` + DB partial unique index; HTTP/concurrency WP-16. |
| AC4 CV v1 and 120–600 description without the LLM | PASS | `cv_version = 1`; `Agent::template_description` valid fallback; DB atomicity WP-16. |
| AC5 retired agent cannot be assigned | PASS | `can_receive_cards` false after retire; requeue WP-09/WP-16. |
| AC6 project verification | DEFERRED | WP-12. |
| AC7 key_prefix/slug uniqueness + immutability | PASS | Domain validation + DB unique indexes; no mutable setter. |
| AC8 audit + event per mutation | DEFERRED | Agent/project events emitted; audit rows WP-16. |
| AC9 SSRF allowlist | PASS | `ALLOWED_PROVIDER_HOSTS`; metadata IP, suffix spoof, userinfo rejected. |
| AC10 CV read API | DEFERRED | WP-16; domain append-only versioning verified. |

## Blocking defects found and resolved

1. The model catalog had no escalation flag or "at most one escalation" invariant. Resolved: `Model::is_escalation`, `default_models` marks flash default and pro escalation, and `validate_catalog` enforces exactly one default and at most one escalation.
2. There was no deterministic description fallback when the LLM is unavailable. Resolved: `Agent::template_description` returns an always-valid 120–600 character description, with a creation test.

## Reproduced evidence

- `scripts/harness-lint.sh` → 16 checks.
- `scripts/sow-lint.sh` → 31 checks.
- `.tools/check-doc-links.sh` → 143 links.
- `cargo fmt --all --check`, `cargo clippy -D warnings` → clean.
- `cargo test -p ailab-domain` → 52 passed.
- SSRF allowlist negatives and inventory invariants verified.

## Reviewer statement

The WP-07 artifacts satisfy the inventory contract. The deferred criteria are downstream verifications owned by WP-12 and WP-16; the domain contract they build on is delivered. Approved for WP closure.
