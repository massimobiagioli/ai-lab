# AI-Lab Security

| Field | Value |
|---|---|
| Document | `docs/security.md` |
| Version | 1.0.0 |
| Status | Accepted |
| Owner | QA |
| Related | `docs/security/abuse-cases.md`, `docs/security/asvs-l2.md`, `docs/security/audit-matrix.md`, `docs/runbooks/security-incident.md` |
| Implemented primitives | `crates/infrastructure/src/{crypto,exposure,redact,security_headers}.rs` |

AI-Lab is secure by default and secure by construction. The default bind is loopback, secrets are encrypted at rest and write-only, tools and sandboxes are deny-by-default, and every sensitive action is auditable. Security findings block merge and release.

---

## 1. Trust boundaries

| Boundary | Inside | Outside | Control |
|---|---|---|---|
| B1 Browser ↔ web/api | SPA, session token (remote only) | Public network in remote mode | TLS, bearer token, CSP, origin check, rate limits |
| B2 api ↔ contexts | Domain and application | — | In-process authorization, validated DTOs |
| B3 api/worker ↔ Postgres/Redis | Trusted runtime | — | Private network, least-privilege roles, no host exposure |
| B4 worker ↔ LLM/SCM | Orchestrator | Internet | TLS, scoped credentials, egress allowlist, timeouts |
| B5 worker ↔ runner/sandbox | Orchestrator | Untrusted agent code | Runner token, container isolation, no host mounts |
| B6 GitHub ↔ api | Webhook endpoint | Internet | HMAC, replay window, idempotency |
| B7 Operator ↔ secrets | Master key | App | Env or secret file, never logged, never in the UI |

## 2. STRIDE summary

| Threat | Example | Control | Verification |
|---|---|---|---|
| Spoofing | Forged webhook; stolen token | HMAC SHA-256, 5-minute replay window, constant-time compare | Signature test; token tests |
| Tampering | Agent edits policy; prompt injection | Forbidden paths, server-side tool policy, untrusted wrapping | Red-team fixtures |
| Repudiation | Agent denies an action | `card_events`, `run_steps`, `audit_log`, commit trailers | Audit completeness test |
| Information disclosure | Key in logs; cross-scope memory leak | AES-256-GCM, write-only keys, redaction, SQL scope filters | Leak tests; leakage fixture |
| Denial of service | Loops, cost exhaustion, floods | Bounded iterations, timeouts, budgets, rate limits | Load/chaos tests |
| Elevation of privilege | Self-review or self-merge | Separation of duties, server-side policy | `422 SEPARATION_OF_DUTIES` tests |
| Supply chain | Malicious dependency or image | Lockfiles, deny, audit, pinned digests, SBOM, signing | CI security jobs |
| Code execution | Shell escape from sandbox | Non-root, cap drop, read-only, no host mount, egress proxy | Escape suite |
| Exfiltration | Tool call to an unknown host | Egress allowlist, DNS via proxy | Egress denial test |

The full abuse-case catalogue is in `docs/security/abuse-cases.md`.

## 3. Exposure modes

| Mode | Bind | Auth | TLS | Use |
|---|---|---|---|---|
| Local (default) | `127.0.0.1:8080` | None (single operator) | No | Local use |
| Remote token | `0.0.0.0` or LAN IP | `AILAB_API_TOKEN` ≥ 32 bytes on all `/api` and `/ws` except `/health/live` | Required | Trusted network |
| Reverse-proxied | Loopback behind Caddy | Token enforced by the app | Terminated at the proxy | Advanced |

Rules:

1. If `AILAB_BIND` is not loopback and `AILAB_API_TOKEN` is missing or shorter than 32 bytes, the API refuses to start with exit code 78. Implemented in `crates/infrastructure/src/exposure.rs` and wired into `crates/api/src/main.rs` (`check-exposure`).
2. The remote token lives in memory and `sessionStorage` only; never `localStorage`, a URL or a log.
3. Tokens and webhook signatures are compared in constant time.
4. Mutations require `Content-Type: application/json` and an allowed `Origin`.
5. No auth cookies in v1; CSRF is mitigated by origin + JSON and the absence of cookies.

## 4. Cryptography and secrets

- **Master key:** 32 random bytes, base64, from `AILAB_MASTER_KEY` or a secret file. Never logged, returned, stored in the database or included in backups. Startup validates the key if present.
- **Field encryption:** AES-256-GCM with a 96-bit random nonce per encryption. AAD is `{table}:{column}:{row_id}:{key_version}`; moving a ciphertext to another row or column fails authentication (tested).
- **Stored shape:** `*_ciphertext`, `*_nonce`, `key_version`, `*_last4`. Only `last4` is readable.
- **Rotation:** envelope encryption with `key_version`; the rotation job re-encrypts in batches, verifies decrypt and retires the old version. The job and DB integration land with WP-07; the primitive is delivered.
- **Token generation:** 32 bytes from the OS CSPRNG, base64url, displayed once; idempotency keys are stored hashed; runner tokens are per-run and short-lived.
- **Redaction:** `crates/infrastructure/src/redact.rs` removes provider, SCM, cloud, bearer, master-key and password shapes from log lines. Prompt and memory redaction is extended by WP-11.

## 5. Authentication and authorization

- Single-operator product; no multi-user RBAC in v1.
- Remote mode uses a bearer token; the app is the authority, the proxy never bypasses it.
- Separation of duties: reviewer ≠ author, merger ≠ author. Conflicts return `422 SEPARATION_OF_DUTIES`.
- Agents act only inside a card and role; tool policy is enforced server-side, never by prompt.
- The audit matrix in `docs/security/audit-matrix.md` lists every sensitive action and its audit row.

## 6. Web security

- CSP: `default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self' ws: wss:; object-src 'none'; base-uri 'self'; frame-ancestors 'none'`.
- Headers: `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`, `Referrer-Policy: strict-origin-when-cross-origin`, HSTS in TLS mode.
- Nginx applies the headers today; the API middleware reuses `security_headers.rs` from WP-16.
- CORS is disabled by default; an exact-match allowlist is available for advanced deployments.
- Input validation at the boundary, deny-unknown-fields on writes, output encoding, no `eval`, no inline scripts.
- Rate limits: 120 reads/min, 30 writes/min, 5 idea/SOW generations/min; Redis fail-closed.
- No third-party frontend scripts; the SPA is same-origin.

## 7. Prompt injection and untrusted content

- Repository content, issues, comments, memory and tool output are untrusted data. They are wrapped in delimiters and never treated as instructions.
- The model cannot change tool policy, budgets, autonomy, forbidden paths or secrets; those are server-side.
- Every tool call is validated against the role allowlist and the egress allowlist.
- Red-team fixtures (README, issue, comment, memory, tool output) are required before WP-11 closes. They cannot cause a forbidden write, secret access, non-allowlisted egress or policy bypass.

## 8. Sandbox security

The sandbox is implemented in WP-14. Requirements: ephemeral container, non-root UID 10001, read-only root filesystem, `cap_drop ALL`, `no-new-privileges`, no host mounts, no Docker socket, `network=none` by default, resource limits, and egress only through the proxy allowlist. The escape suite covers host filesystem, Docker socket, network scan, privilege escalation, fork bomb and disk fill.

## 9. Webhooks

- HMAC SHA-256 with the webhook secret; constant-time compare.
- Reject signatures older than 5 minutes; deduplicate by `X-GitHub-Delivery`; duplicate deliveries are acknowledged without reprocessing.
- Bad signature returns 401; replay returns 401; duplicate returns 202 once.
- Implemented with WP-12 against the contract in `docs/contracts.md` and `github_webhook_deliveries`.

## 10. Supply chain

- Lockfiles committed; images pinned; Renovate weekly with major review; TypeScript major blocked by ADR-0024.
- `cargo-deny`, `npm audit`, gitleaks, Trivy and SBOM/signing in CI.
- `.tools/test-supply-chain.sh` proves that a vulnerable npm dependency and a disallowed license are rejected.
- No high or critical finding may ship.

## 11. Audit and data protection

- Every sensitive action writes an audit row with actor, action, target, before/after where safe, IP/user agent and `trace_id`.
- Data classes: secret (encrypted, write-only), sensitive (prompts, repo content), internal (cards, runs, memory), public (docs).
- Retention follows `docs/database.md` §7. Deletion removes DB rows, vectors, chunks and artifacts; export/delete is owned by WP-16.
- Backups contain encrypted fields but never the master key.

## 12. Vulnerability management and incidents

| Severity | Remediation | Mitigation |
|---|---|---|
| Critical | 72 h | 24 h |
| High | 7 days | 48 h |
| Medium | 30 days | 14 days |
| Low | 90 days | — |

Security incidents are P0/P1 and block release. The incident runbook is `docs/runbooks/security-incident.md`: detect → contain → eradicate → recover → review within 48 h. A tabletop exercise is required before WP-05 closes at M7.

## 13. Testing and evidence

```bash
cargo test -p ailab-infrastructure
.tools/test-supply-chain.sh
gitleaks detect --source . --no-git
make check
```

The infrastructure suite covers AES-GCM round-trip, wrong AAD, tampering, nonce uniqueness, key-length validation, exposure guard, constant-time token compare, redaction and security headers. The ASVS L2 checklist is in `docs/security/asvs-l2.md`.

## 14. References

- `.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §16, §18, §20
- `.docs/technical-specification/APPENDIX-A-DATABASE.sql`
- `.docs/technical-specification/APPENDIX-B-CONTRACTS.md` §B.4
- `docs/security/abuse-cases.md`, `docs/security/asvs-l2.md`, `docs/security/audit-matrix.md`
- `docs/runbooks/security-incident.md`
