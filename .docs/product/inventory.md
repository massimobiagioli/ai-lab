# AI-Lab Inventory

| Field | Value |
|---|---|
| Document | `.docs/product/inventory.md` |
| Version | 1.0.0 |
| Status | Accepted |
| Owner | Dev |
| Related | `.docs/product/domain-model.md`, `.docs/product/security.md`, `crates/domain/src/inventory.rs` |

The Inventory context owns providers, models, SCM credentials, characters, agents, CVs and projects. The domain aggregates and invariants are implemented in `crates/domain/src/inventory.rs`; the REST/OpenAPI layer lands with WP-16 and the GitHub verification with WP-12.

## 1. Aggregates

| Aggregate | Invariants |
|---|---|
| `Provider` | Name non-empty; base URL HTTPS; positive rate limit and timeout; key stored as encrypted material with only `last4` readable; disable/enable/degrade/rotate |
| `Model` | Name non-empty; positive windows; non-negative costs; exactly two seeded models (`deepseek-flash` default, `deepseek-v4-pro` escalation) |
| `ScmCredential` | Workspace-scoped; write-only encrypted token; `last4` only; verification state |
| `Agent` | Description 120–600 chars; role in the fixed set; capacity 1–4; pause/resume/retire; append-only CV versions; exactly one active Tech Lead per lab |
| `Project` | Unique slug and key prefix per lab; prefix `[A-Z][A-Z0-9]{1,9}`; `owner/repo`; project-local card sequence; archive blocks new cards |

## 2. Lifecycles

- **Provider:** active → degraded/disabled → active; key rotation increments `key_version`.
- **Agent:** idle → working/blocked → paused → idle → retired. Retired agents keep CV history and cannot receive cards.
- **Project:** active → archived. Card keys are allocated once per card and never reused.
- **CV:** version 1 on creation, then append-only versions with evidence.

## 3. Security

Provider and SCM secrets are stored as encrypted material and never returned; only `last4` is readable. `EncryptedKey` implements a redacting `Debug`. The AES-256-GCM primitive and AAD are in `crates/infrastructure/src/crypto.rs` (WP-05).

## 4. Tests

`scripts/run-rust.sh cargo test -p ailab-domain` covers provider lifecycle and validation, model seeding, SCM write-only credentials, agent description/lifecycle/CV, the one-Tech-Lead invariant, and project card sequences.

## 5. Deferred to consumer WPs

- REST endpoints, DTOs and OpenAPI: WP-16.
- SQLx repositories and `.sqlx` metadata: WP-16/WP-07 follow-up.
- GitHub connection verification: WP-12.
- Provider/model/SCM audit rows and events: WP-16.
- Inventory UI: WP-19.

## 6. References

- `.docs/sow/WP-07-INVENTORY.md`
- `.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §6.1
- `.docs/technical-specification/APPENDIX-A-DATABASE.sql`
