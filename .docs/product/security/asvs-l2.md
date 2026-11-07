# OWASP ASVS L2 Checklist

| Field | Value |
|---|---|
| Document | `.docs/product/security/asvs-l2.md` |
| Version | 1.0.0 |
| Status | In progress |
| Owner | QA |
| Related | `.docs/product/security.md` |

Status: `met`, `partial` (control exists, test or wiring pending) or `deferred` (owned by a later WP). QA signs the checklist before release.

| Chapter | Requirement | Status | Evidence / owner |
|---|---|---|---|
| V1 Architecture | Trust boundaries and threat model documented | met | `.docs/product/security.md`, abuse cases |
| V2 Authentication | No default credentials; high-entropy tokens | met | Exposure/token tests |
| V2 Authentication | Remote bind requires a token | met | `ailab-api check-exposure` exit 78 |
| V3 Session | No auth cookies; token in memory/sessionStorage | met | UI contract (WP-17) |
| V4 Access control | Separation of duties | partial | DB/card guard; WP-09/WP-15 tests |
| V4 Access control | Deny-by-default tool policy | deferred | WP-11/WP-14 |
| V5 Validation | Boundary validation, deny unknown fields | deferred | WP-16 DTOs |
| V5 Validation | Output encoding and CSP | partial | Nginx CSP; WP-17 tests |
| V6 Cryptography | AES-256-GCM with unique nonce and AAD | met | `crypto.rs` tests |
| V6 Cryptography | Key rotation with versions | partial | Primitive; rotation job WP-07 |
| V7 Errors/logging | Secrets never logged | met | `redact.rs` tests |
| V7 Errors/logging | RFC 9457 errors, no stack traces | deferred | WP-16 |
| V8 Data protection | Secrets encrypted at rest, write-only | partial | Crypto + schema; WP-07 |
| V8 Data protection | Retention and deletion | partial | Policy in `.docs/product/database.md`; WP-16/WP-23 |
| V9 Communication | TLS for remote mode | met | Caddy TLS profile |
| V9 Communication | Webhook HMAC and replay window | deferred | WP-12 |
| V10 Malicious code | Sandbox isolation | deferred | WP-14 |
| V11 Business logic | Budget and autonomy hard stops | deferred | WP-11/WP-15 |
| V12 Files | Safe artifact paths | deferred | WP-16 |
| V13 API | Rate limits and body limits | partial | Nginx/limits; WP-16 |
| V14 Config | Pinned dependencies, no EOL, SBOM | met | `.tools/test-supply-chain.sh`, `images.yml` |

A `deferred` item is not a pass. It is tracked by its owner WP and must be `met` before the release gate.
