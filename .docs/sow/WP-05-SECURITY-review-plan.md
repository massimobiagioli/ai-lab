# WP-05 — Security Architecture & Secrets Management Review Plan

| Field | Value |
|---|---|
| WP | `WP-05` |
| Contract | `.docs/sow/WP-05-SECURITY.md` |
| Review branch | `review/wp-05-security` |
| Reviewer | Chief Software Engineer (agent) |
| QA reviewer | qa-review-agent (independent subagent) |
| Status | `in_review` |

## 1. Contract completeness

- [x] Header fields, objective, scope, engineering specification, deliverables, tasks, acceptance criteria, DoD, risks, references and status history are present and consistent.
- [x] Requirement traceability cited in §9 maps to spec §16/§18/§20, Appendix A (encrypted columns, audit) and Appendix B §B.4.

## 2. Specification conformance

- [x] Specification §16 (security), §18 (operations), §20 (delivery) — evidence: `.docs/product/security.md`, `.docs/product/security/*`, `crates/infrastructure/src/{crypto,exposure,redact,security_headers}.rs`.
- [x] Appendix B §B.4 (error/security contracts) — evidence: `.docs/product/contracts.md`, `redact.rs`.
- [x] ADR-0011 separation of duties, ADR-0013 login-less exposure, ADR-0012 sandbox, ADR-0023 runner token — evidence: registry.

## 3. Deliverables

- [x] `.docs/product/security.md` threat model/trust boundaries/exposure/controls/classification/IR — evidence: file.
- [x] `.docs/product/security/abuse-cases.md` 28 abuse cases — evidence: AC-01..AC-28.
- [x] Secret encryption AES-256-GCM + AAD + key versions + tests — evidence: `crates/infrastructure/src/crypto.rs`, tests.
- [x] Master key handling and exposure guard (exit 78) — evidence: `exposure.rs`, `crates/api/src/main.rs`; reproduced exit 78.
- [x] Constant-time token compare — evidence: `exposure.rs` `subtle::ConstantTimeEq`.
- [x] Security headers (CSP, nosniff, frame-deny, referrer, HSTS) — evidence: `security_headers.rs`, `infra/nginx/nginx.conf`.
- [x] Redaction library + tests — evidence: `redact.rs`.
- [x] Prompt-injection policy — evidence: `security.md`; fixtures WP-11.
- [x] Webhook HMAC/replay/idempotency contract — evidence: `contracts.md`; implementation WP-12.
- [x] Sandbox security requirements — evidence: WP-14 contract.
- [x] Supply-chain gates — evidence: `.tools/test-supply-chain.sh`, `deny.toml`, `renovate.json`.
- [x] Audit coverage matrix — evidence: `.docs/product/security/audit-matrix.md`.
- [x] Data classification/retention — evidence: `security.md`, `database.md`.
- [x] Vulnerability SLA / incident runbook / ADR exception template — evidence: `.docs/product/runbooks/security-incident.md`.
- [x] OWASP ASVS L2 checklist — evidence: `.docs/product/security/asvs-l2.md` (partial/deferred tracked).

## 4. Acceptance criteria

- [x] AC1 non-loopback bind without token exits 78 with a clear message — evidence: `AILAB_BIND=0.0.0.0 ailab-api check-exposure` → exit 78; loopback/token → 0. Unauthenticated 401 for `/api`/`/ws` is WP-16.
- [x] AC2 provider key never returned/logged — DEFERRED to WP-07 (write-only DTOs) and WP-11 (outputs); redaction primitive and tests delivered.
- [x] AC3 AES-256-GCM ciphertext bound via AAD; tampering fails — evidence: `crypto.rs` `wrong_aad_fails_authentication`, `tampered_ciphertext_fails`.
- [x] AC4 key rotation re-encrypts all secrets — DEFERRED to WP-07 (rotation job); primitive versioned ciphertext delivered.
- [x] AC5 CSP/security headers on every response — evidence: `security_headers.rs` + nginx; API middleware WP-16.
- [x] AC6 cross-scope memory retrieval returns zero forbidden chunks — DEFERRED to WP-08 (SQL scope filter + eval).
- [x] AC7 prompt-injection fixtures cannot bypass — DEFERRED to WP-11 (red-team fixtures).
- [x] AC8 invalid webhook signature 401, duplicate once, replay rejected — DEFERRED to WP-12.
- [x] AC9 sandbox escape suite — DEFERRED to WP-14.
- [x] AC10 no high/critical supply chain; SBOM/signature — evidence: `.tools/test-supply-chain.sh`; release scan/SBOM WP-22.
- [x] AC11 every sensitive action audited — DEFERRED to owning WPs; audit matrix delivered (21 actions).
- [x] AC12 data deletion removes rows/vectors/chunks/artifacts — DEFERRED to WP-16.

## 5. Implementation and tests

- [x] Crypto: round-trip, AAD binding, tamper, nonce uniqueness, key-length validation — tests green.
- [x] Exposure: loopback/token matrix, constant-time compare — tests green.
- [x] Redaction: GitHub/Bearer/master-key/password shapes — tests green.
- [x] Security headers: CSP without inline/eval, nosniff, frame-deny — tests green.
- [x] No real external API in tests.
- [x] Coverage thresholds met (security primitives crate).

## 6. Security and quality gates

- [x] Secret encryption, exposure guard, redaction and headers verified.
- [x] `scripts/harness-lint.sh` green (16 checks).
- [x] `scripts/sow-lint.sh` green (31 checks).
- [x] `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p ailab-infrastructure` green.
- [x] `.tools/test-supply-chain.sh` green.

## 7. Documentation, ADRs and indexes

- [x] `.docs/product/security.md` and `security/*` updated.
- [x] `.docs/adr/registry.md` consistent (ADR-0011/0012/0013/0023).
- [x] `AGENTS.md` indexes updated for this review plan.
- [x] `.docs/sow/SOW.md` and WP-05 status/progress updated.

## 8. QA review

- [ ] QA subagent verdict: `APPROVED`.
- [ ] QA evidence: reproduced gates (harness 16, sow-lint 31, doc links 143, crypto/exposure/redact/headers tests, exit-78 matrix, supply-chain 2/2).

## 9. Deferred items and residual risks

- [x] AC2/AC4 secret DTOs and rotation — owner: Dev, WP-07.
- [x] AC5/AC12 API headers and deletion — owner: Dev, WP-16.
- [x] AC6 memory scope — owner: Dev, WP-08.
- [x] AC7 prompt injection — owner: QA, WP-11.
- [x] AC8 webhooks — owner: Dev, WP-12.
- [x] AC9 sandbox escape — owner: QA, WP-14.
- [x] AC10 release scan/SBOM/signature and AC11 per-action audit — owners: WP-22 and owning WPs.
- [x] No unexplained deferral remains.

## 10. Decisions

- [x] ADR-0011, ADR-0012, ADR-0013 and ADR-0023 recorded; ASVS L2 checklist tracks partial/deferred items with owners.
