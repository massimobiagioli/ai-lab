# AI-Lab Abuse Cases

| Field | Value |
|---|---|
| Document | `docs/security/abuse-cases.md` |
| Version | 1.0.0 |
| Status | Accepted |
| Owner | QA |
| Related | `docs/security.md` |

At least 20 abuse cases, each mapped to a control and a verification owner. A case without a control is a gap; a control without a test is a claim.

| ID | Abuse case | Threat | Control | Verification / owner |
|---|---|---|---|---|
| AC-01 | Forged GitHub webhook | Spoofing | HMAC SHA-256 + constant-time compare | WP-12 integration test |
| AC-02 | Replayed webhook older than 5 minutes | Spoofing | Replay window + delivery idempotency | WP-12 integration test |
| AC-03 | Stolen remote bearer token | Spoofing | High-entropy token, constant-time compare, TLS required | WP-05 exposure tests |
| AC-04 | Secret returned by an API | Information disclosure | Write-only DTOs, `last4` only | WP-07 leak tests |
| AC-05 | Secret written to a log or trace | Information disclosure | Central redactor + leak tests | WP-05 redaction tests |
| AC-06 | Secret in a commit or fixture | Information disclosure | gitleaks hook + CI | `.tools/test-supply-chain.sh`, CI |
| AC-07 | Prompt injection in a README | Tampering | Untrusted wrapping + server-side policy | WP-11 red-team fixtures |
| AC-08 | Prompt injection in a memory document | Tampering | Scope filter + wrapping + policy | WP-11/WP-08 fixtures |
| AC-09 | Prompt injection in a tool result | Tampering | Tool output treated as data; policy server-side | WP-11 fixtures |
| AC-10 | Cross-scope memory retrieval | Information disclosure | SQL scope filter before ranking | WP-08 leakage fixture |
| AC-11 | Agent reviews or merges its own work | Elevation of privilege | Separation of duties | WP-09/WP-15 tests |
| AC-12 | Agent edits autonomy, budget or policy | Elevation of privilege | Server-side validation + human approval | WP-15 policy tests |
| AC-13 | Sandbox reads host filesystem | Code execution | No host mounts, read-only, non-root | WP-14 escape suite |
| AC-14 | Sandbox reaches the Docker socket | Code execution | No socket mount; proxy allowlist | WP-14 escape suite |
| AC-15 | Sandbox scans the internal network | Exfiltration | `network=none` by default | WP-14 escape suite |
| AC-16 | Sandbox escalates privileges | Elevation of privilege | Cap drop, no-new-privileges | WP-14 escape suite |
| AC-17 | Fork bomb or disk fill in a sandbox | Denial of service | PIDs, CPU, memory, disk limits | WP-14 escape suite |
| AC-18 | Sandbox contacts an unknown host | Exfiltration | Egress proxy allowlist + DNS via proxy | WP-14 egress test |
| AC-19 | XSS via a card title or comment | Tampering | Output encoding + CSP without inline scripts | WP-17/WP-19 a11y/security tests |
| AC-20 | CSRF via a form post | Tampering | Origin check + JSON content type + no cookies | WP-16 tests |
| AC-21 | Rate-limit bypass or flood | Denial of service | Redis token bucket, fail closed | WP-16 load tests |
| AC-22 | Path traversal in an artifact export | Information disclosure | Allowlisted paths + canonicalization | WP-16 tests |
| AC-23 | Dependency confusion or typosquat | Supply chain | Lockfiles, pinned registries, `cargo-deny` | `.tools/test-supply-chain.sh`, CI |
| AC-24 | Malicious or unsigned image | Supply chain | Digest pins, Trivy, SBOM, cosign | `images.yml`, WP-22 |
| AC-25 | Audit log tampering | Repudiation | Append-only audit, no update/delete API | WP-05/WP-06 tests |
| AC-26 | Backup exfiltration of plaintext secrets | Information disclosure | Encrypted fields; master key not in backup | WP-04/WP-20 drill |
| AC-27 | Rotation leaves rows on an old key | Information disclosure | Batch rotation + verify + version check | WP-07 rotation test |
| AC-28 | Remote bind without a token | Spoofing | Exit 78 at startup | `ailab-api check-exposure` test |
