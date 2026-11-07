# WP-05 Independent Review

| Field | Value |
|---|---|
| Reviewer | review-agent (independent subagent) |
| Author | QA (agent) |
| Date | 2026-11-07 |
| Scope | Threat model, abuse cases, crypto, exposure guard, redaction, security headers, supply chain, audit matrix, ASVS |
| Verdict | `APPROVED` |

Two review passes returned `CHANGES_REQUESTED`; the author fixed the defects and the third pass approved.

## Final acceptance results

| AC | Result | Evidence |
|---|---|---|
| AC1 non-loopback without token exits 78 | PASS | `check-exposure` matrix: 78 for `0.0.0.0` without/short token, 0 for loopback and 32-byte token. Unauthenticated 401 is WP-16. |
| AC2 secret never returned/logged | DEFERRED | Write-only DTOs and outputs are WP-07/WP-11; the redaction primitive covers prefixes, Bearer/Basic, JWT, PEM, headers and JSON pairs. |
| AC3 AAD-bound AES-256-GCM | PASS | `wrong_aad_fails_authentication`, `tampered_ciphertext_fails`, `nonces_are_unique`. |
| AC4 key rotation | DEFERRED | Rotation job is WP-07; versioned envelope primitive delivered. |
| AC5 CSP and security headers | PASS | `security_headers.rs` and nginx/Caddy carry the hardened set; API middleware is WP-16. |
| AC6 cross-scope memory | DEFERRED | WP-08. |
| AC7 prompt injection | DEFERRED | WP-11 fixtures; policy and abuse cases delivered. |
| AC8 webhook security | DEFERRED | WP-12 implementation. |
| AC9 sandbox escape | DEFERRED | WP-14 suite. |
| AC10 supply chain | PASS (local) | `.tools/test-supply-chain.sh` 2/2; release scan/SBOM/signature is WP-22. |
| AC11 audit coverage | DEFERRED | Owning WPs; 21-action matrix delivered. |
| AC12 data deletion | DEFERRED | WP-16. |

## Blocking defects found and resolved

1. The redactor bypassed secrets in JSON, URLs, sensitive headers, JWTs and PEM blocks. Resolved with regex rules and regression tests.
2. JSON-serialized `cookie`/`set-cookie`/`authorization` (Basic) still leaked. Resolved by explicit JSON key/value and Basic rules.
3. Security headers were incomplete/weakened and `security.md` documented the old CSP. Resolved in code, nginx, Caddy and the product doc.

## Reproduced evidence

- `scripts/harness-lint.sh` → 16 checks.
- `scripts/sow-lint.sh` → 31 checks.
- `.tools/check-doc-links.sh` → 143 links.
- `cargo fmt --all --check`, `cargo clippy -D warnings` → clean.
- `cargo test -p ailab-infrastructure` → 31 passed.
- `check-exposure` matrix → 78/78/0/0.
- `.tools/test-supply-chain.sh` → 2 negative fixtures rejected.

## Reviewer statement

The WP-05 artifacts satisfy the security contract. The deferred criteria are downstream verifications owned by WP-07, WP-08, WP-11, WP-12, WP-14, WP-16 and WP-22; the artifacts they verify are delivered. Approved for WP closure.
