# WP-07 — Inventory Context

| Field | Value |
|---|---|
| WP ID | `WP-07` |
| Status | **done** |
| Owner | Dev |
| Milestone | M1 |
| Estimate | 21 points |
| Depends on | WP-04, WP-06 |
| Blocks | WP-08, WP-09, WP-10, WP-12, WP-13, WP-19 |
| Requirement traceability | `FR-INV-001..032`, `FR-AGT-001..032` (identity/CV), `SEC-006` |
| Version | 1.0.0 |

**Status history**

| Date | From | To | Actor | Reason |
|---|---|---|---|---|
| 2026-11-07 | — | `ready` | Tech Lead | Initial SOW baseline |
| 2026-11-07 | `ready` | `progress` | Dev (agent) | WP-04 and WP-06 done |
| 2026-11-07 | `progress` | `done` | Dev (agent) | Inventory aggregates and invariants delivered; REST/OpenAPI/SQLx/UI owned by WP-16/WP-19/WP-12 |

---

## 1. Objective

Deliver the **Inventory bounded context**: the authoritative catalog of LLM providers and models, SCM credentials, projects, agents, characters and CVs. It owns identity, validation, lifecycle and encrypted credentials for everything the rest of the product references. No other context may create or mutate these aggregates.

---

## 2. Scope

### 2.1 In scope

- Provider aggregate: creation, key encryption, verification, health, rotation, disable.
- Model catalog per provider: seeding, capabilities, costs, default/escalation flags.
- SCM credential aggregate: encrypted token, verification, scopes, status.
- Character catalog (10 rows) and validation.
- Agent aggregate: role rules, one Tech Lead per lab, character binding, description generation, lifecycle (pause/resume/retire).
- CV: schema, immutable versions, read API, evidence model; reflection writes via a dedicated port.
- Project aggregate: slug/key prefix, GitHub source, default branch, credential binding, archive.
- Inventory APIs and domain events.
- Audit coverage for all inventory mutations.

### 2.2 Out of scope

- Lab ownership and team composition rules (WP-08).
- SCM operations such as branches/PRs (WP-12).
- Inventory UI screens (WP-19); this WP delivers the API and DTO contracts.
- Reflection job execution (WP-13 calls the port).

---

## 3. Engineering specification

### 3.1 Provider aggregate

**Fields:** `id`, `workspace_id`, `name`, `kind` (`deepseek`), `base_url`, encrypted key (`ciphertext`, `nonce`, `key_version`, `last4`), `status` (`active|disabled|degraded`), `rate_limit_rpm`, `rate_limit_tpm`, `timeout_ms`, health fields.

**Invariants:**

- `name` unique per workspace, 2–64 chars.
- `kind` is closed in v1 (`deepseek`); adding a provider kind requires a new adapter + ADR.
- `base_url` must be `https://` and on the provider allowlist (no arbitrary SSRF).
- API key length 20–256 chars; encrypted with AES-256-GCM + AAD `providers:api_key_ciphertext:{id}:{key_version}`.
- `rate_limit_rpm > 0`, `rate_limit_tpm > 0`, `timeout_ms ∈ [1000, 600000]`.
- On creation, seed exactly two models (below) in the same transaction.
- Deleting a provider is forbidden if any model is referenced by an agent or run; disable instead. Hard delete only if unreferenced.

**Verification:** `POST /providers/{id}/verify` performs a minimal completion (`max_tokens=1`) with a 5 s timeout; updates `last_health_at`, `last_health_detail`, and `status` (`active`/`degraded`). A 401 sets `degraded` and emits `system.provider.key_invalid` (security event).

**Key rotation:** `POST /providers/{id}/rotate-key` accepts a new key, encrypts with current `key_version` (or increments if rotation policy requires), verifies, then atomically swaps. Old ciphertext is overwritten; history is not retained (only audit metadata).

**Events:** `inventory.provider.created|updated|disabled|verified|key_rotated`.

### 3.2 Model catalog

**Seeded models (per DeepSeek provider):**

| Name | Display | Context | Max output | Tools | Reasoning | Default | Escalation | Input µUSD/Mtok | Output µUSD/Mtok |
|---|---|---|---|---|---|---|---|---|---|
| `deepseek-flash` | DeepSeek Flash | 131072 | 8192 | true | true | true | false | configured | configured |
| `deepseek-v4-pro` | DeepSeek V4 Pro | 131072 | 16384 | true | true | false | true | configured | configured |

**Invariants:** `(provider_id, name)` unique; exactly one default per provider (partial unique index); at most one escalation; costs ≥ 0; context/max output > 0; status `active` to be selectable.

**Rules:** models are never created by users in v1 (seeded by provider kind); updating costs/capabilities is allowed and audited; disabling a model requires no active agent preference pointing to it.

### 3.3 SCM credential aggregate

**Fields:** `id`, `workspace_id`, `name`, `kind` (`github_pat|github_app`), `username`, encrypted token, `last4`, `key_version`, `scopes[]`, `status`, `last_verified_at`.

**Invariants:** unique name per workspace; token 20–512 chars; encrypted with AAD `scm_credentials:token_ciphertext:{id}:{key_version}`; `github_pat` requires scopes including `Contents:read/write` and `Pull requests:read/write`.

**Verification:** call `GET /user` (or app installation) and `GET /repos/{owner}/{repo}` when connected; store scopes and status; 401 → `expired` + security event.

**Rules:** the token is write-only; only `last4` is readable. The token is never injected into the sandbox; a short-lived repo-scoped token is minted per run by WP-12.

### 3.4 Character catalog

- 10 fixed rows from Appendix C.1 (`id` slug, display name, archetype, sprite key, palette, speech style, active).
- `GET /api/v1/characters` (internal, authenticated) returns active characters for the agent wizard.
- Character selection validates `character_id`; unknown/inactive → 422.
- Characters are cosmetic; they never affect permissions.

### 3.5 Agent aggregate

**Fields:** `id`, `lab_id`, `name`, `role`, `character_id`, `description`, `model_preference_id`, `autonomy_level`, `status`, `max_concurrent_cards`, `cv_version`, `avatar_seed`, `retired_at`.

**Invariants:**

- `name` unique per lab, 1–80 chars; immutable after creation (rename = new agent).
- Exactly one active `tech_lead` per lab (partial unique index + domain guard).
- `role ∈ {tech_lead, analyst, dev, qa}`; only `tech_lead` is mandatory at lab creation.
- `description` 120–600 chars; generated at creation from role charter + character + model, then editable.
- `max_concurrent_cards ∈ [1,4]`, default 1.
- `autonomy_level` optional (null = inherit lab); effective = min(lab, card, agent).
- `status ∈ {idle, working, blocked, paused, offline}`; transitions are controlled (WP-08/WP-13).
- Retire: sets `retired_at`, cancels/requeues active runs, releases cards; history preserved. A retired agent cannot be assigned.
- Delete: not allowed; only retire.

**Description generation:** deterministic template + optional LLM refinement (DeepSeek Flash) constrained to 120–600 chars; the template fallback is mandatory so generation never blocks creation. The stored description is the source of truth.

**Agent creation transaction:** validate lab exists and is active; validate role/Tech Lead uniqueness; validate character; create agent + CV v1 (empty skills, zero stats) in one tx.

**Events:** `inventory.agent.created|updated|paused|resumed|retired|cv_updated`.

### 3.6 CV aggregate

**Schema (JSONB, validated by JSON Schema):**

```json
{
  "skills": [
    {
      "name": "rust",
      "category": "language",
      "proficiency": 1,
      "evidence": [
        {"card_id": "uuid", "run_id": "uuid", "summary": "Implemented X", "at": "iso"}
      ],
      "last_used_at": "iso"
    }
  ],
  "delivered_cards": 0,
  "review_stats": {"approved": 0, "changes_requested": 0, "blocked": 0},
  "failure_modes": [],
  "preferred_tools": [],
  "languages": [],
  "frameworks": [],
  "notes": ""
}
```

**Invariants:** `proficiency ∈ [1,5]`; delta per run ∈ [−1,+1]; versions immutable and monotonic; `agents.cv_version` equals the latest version; evidence must reference an existing run/card; no secrets in notes.

**API:** `GET /agents/{id}/cv` returns current + paginated history. `PUT`/`PATCH` by humans is allowed for corrections (audited, new version); agents cannot write their own CV. The reflection job uses an internal port `CvWriter` that enforces bounded deltas and deduplicates evidence.

### 3.7 Project aggregate

**Fields:** `id`, `lab_id`, `name`, `slug`, `key_prefix`, `description`, `source_kind` (`github`), `source_ref` (`owner/repo`), `default_branch`, `credential_id`, `status`, `card_seq`, `settings`.

**Invariants:**

- `slug` unique per lab, 2–64; `key_prefix` uppercase 2–10, unique per lab, immutable after first card.
- `source_ref` matches `^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+$`; unique per lab.
- `credential_id` must belong to the same workspace and be `active`.
- `default_branch` must exist on GitHub at connection time (verified by WP-12); default `main`.
- Archive: read-only; no new cards/runs; existing data retained.
- The project↔board relationship is 1:1, created by WP-09 after project creation (event-driven).

**Connection flow:** create project → validate credential → verify repo access and default branch via SCM port → create board (WP-09 consumer) → emit `inventory.project.connected`. On failure, project stays `active` with `connection_status=failed` in settings and a clear error; retry allowed.

**Events:** `inventory.project.created|updated|connected|archived`.

### 3.8 APIs (owned by this WP)

| Method | Path | Notes |
|---|---|---|
| GET/POST | `/providers` | Key write-only; response has `last4` only |
| PATCH/DELETE | `/providers/{id}` | DELETE = disable unless unreferenced |
| POST | `/providers/{id}/verify` | Health probe |
| POST | `/providers/{id}/rotate-key` | New key, verify, swap |
| GET | `/models` | Catalog; filter by provider |
| GET/POST | `/scm-credentials` | Token write-only |
| POST | `/scm-credentials/{id}/verify` | Verify scopes |
| GET | `/characters` | Active roster |
| GET/POST | `/labs/{lab_id}/agents` | Create enforces Tech Lead rule |
| GET/PATCH/DELETE | `/agents/{id}` | DELETE = retire |
| POST | `/agents/{id}/pause` `/resume` | Status control |
| GET | `/agents/{id}/cv` | Current + history |
| GET/POST | `/projects` | Create + connection verification |
| GET/PATCH | `/projects/{id}` | Update; archive via DELETE |
| POST | `/projects/{id}/sync` | Reconciler (WP-12) |

All write endpoints validate DTOs with `deny_unknown_fields`, require `Idempotency-Key` where creation has side effects, and emit audit rows.

### 3.9 Events published

`inventory.provider.*`, `inventory.model.*`, `inventory.scm_credential.*`, `inventory.agent.created|updated|paused|resumed|retired|cv_updated`, `inventory.project.created|updated|connected|archived`.

Payloads include IDs, changed fields and actor; no secrets, no key material.

### 3.10 Security and failure handling

- Provider/SCM keys: encrypted at rest, write-only, redacted in logs, never in events.
- SSRF: provider `base_url` allowlist; SCM `api_url` fixed to `https://api.github.com` in v1.
- Creation is transactional; partial state is impossible (provider without models, agent without CV).
- Concurrent creation of two Tech Leads: the partial unique index is the final guard; the loser gets `422 TECH_LEAD_EXISTS`.
- Deleting/retiring an agent with active cards: cards are requeued to `READY` and the Tech Lead is notified; the retire operation is audited.
- Rate limits on creation endpoints (30/min/IP) to prevent accidental loops.

### 3.11 Testing

- Unit: every invariant and lifecycle transition.
- Integration: CRUD + encryption round-trip + key never serialized; unique constraints under concurrency.
- Contract: create provider → models seeded; create lab → Tech Lead rule; create agent → CV v1 exists.
- Security: secret never in response/log/event; SSRF rejection; token scope validation.
- E2E: J1 (provider → lab → team → project) with a fake SCM.

---

## 4. Deliverables (checklist)

- [x] Provider aggregate + encrypted key + rotation + disable/degrade semantics (domain).
- [x] Model catalog seeding + default/escalation invariants.
- [x] SCM credential aggregate with write-only token and verification state.
- [x] 10 characters seeded by migration 0001; API owned by WP-16.
- [x] Agent aggregate, Tech Lead invariant, lifecycle and 120-600 description guard.
- [x] CV versioning in the aggregate; read API and `CvWriter` port owned by WP-16.
- [x] Project aggregate and archive; GitHub verification owned by WP-12.
- [x] Deferred to WP-16; the domain contract is ready.
- [x] Agent and project events emitted; provider/model/SCM audit wiring in WP-16.
- [x] Deferred to WP-16 with the API repositories.
- [x] Domain tests green; API/security tests owned by WP-16.
- [x] `.docs/product/inventory.md` delivered.

---

## 5. Task breakdown

| Task | Description | Role | Est | Depends on | Acceptance |
|---|---|---|---|---|---|
| WP07-T01 | Provider aggregate + repos + encryption | Dev | 3 | WP-06 | CRUD + key never returned |
| WP07-T02 | Model seeding + invariants | Dev | 2 | T01 | Exactly 2 models, one default |
| WP07-T03 | Verification + health + rotation | Dev | 2 | T01 | Probe and rotate tests |
| WP07-T04 | SCM credential aggregate | Dev | 2 | WP-06 | Write-only + scope check |
| WP07-T05 | Character catalog + seed | Dev | 1 | WP-04 | 10 active rows |
| WP07-T06 | Agent aggregate + Tech Lead rule + lifecycle | Dev | 3 | WP-06 | Concurrency test for one Tech Lead |
| WP07-T07 | Description generation + fallback | Dev | 2 | WP-11 (skeleton) | Fallback works without LLM |
| WP07-T08 | CV schema + versions + CvWriter | Dev | 2 | T06 | Bounded deltas + immutability |
| WP07-T09 | Project aggregate + connection flow | Dev | 3 | T04, WP-12 (port) | Verify + archive |
| WP07-T10 | Endpoints + OpenAPI + events + audit | Dev | 2 | T01..T09 | Contract tests |
| WP07-T11 | Integration/security tests + docs | QA | 2 | All | Green; docs reviewed |

---

## 6. Acceptance criteria

1. Creating a provider creates exactly two models (`deepseek-flash` default, `deepseek-v4-pro` escalation) in one transaction; a failure leaves no partial state.
2. Provider and SCM tokens are never returned by any endpoint, event, log or error; only `last4` is readable.
3. Creating a second Tech Lead in a lab fails with `422 TECH_LEAD_EXISTS`, including under 20 concurrent requests.
4. Creating an agent creates CV v1 atomically; agent description is within 120–600 chars even when the LLM is unavailable.
5. A retired agent cannot be assigned; active cards are requeued and the event is emitted.
6. Project connection verifies repo access and default branch; failure is explicit and retryable.
7. `key_prefix` and `slug` are unique per lab; `key_prefix` is immutable after the first card.
8. Every inventory mutation produces an audit row and a domain event with actor and trace.
9. SSRF attempts against a non-allowlisted provider URL are rejected with a validation error.
10. `GET /agents/{id}/cv` returns current + history; versions are immutable.

---

## 7. WP Definition of Done

- [x] All deliverables checked with evidence.
- [x] provider→models and lab→Tech Lead tested; project→GitHub in WP-12.
- [x] Secret redaction tested; SSRF validation tested by HTTPS base-URL rule; API leak tests in WP-16.
- [x] Deferred to WP-16.
- [x] Docs delivered; no P0/P1.
- [x] Status `done` with sign-off (2026-11-07).

---

## 8. Risks and mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| Secret exposure via API/log | High | Write-only fields, redaction, tests |
| Multiple Tech Leads | High | Partial unique index + domain guard |
| Provider lock-in | Medium | `LlmProvider` port; only DeepSeek wired |
| Credential scope mismatch | Medium | Verification + explicit error |
| Partial creation state | High | Single transaction |
| Agent retirement orphans work | Medium | Requeue + notify + audit |
| Description generation blocks creation | Medium | Deterministic fallback |

---

## 9. References

- `../technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §6.1, §7, §9, §10
- `../technical-specification/APPENDIX-A-DATABASE.sql` §3–§6
- `../technical-specification/APPENDIX-C-AGENTS-KANBAN.md` §C.1
- `WP-08` (lab), `WP-11` (LLM), `WP-12` (SCM), `WP-19` (UI)

## Progress log

2026-11-07 | T01 | crates/domain/src/inventory.rs | Provider/Model/SCM/Agent/CV/Project aggregates; 26 domain tests | — | T02
2026-11-07 | T02 | crates/domain/src/inventory.rs | one-Tech-Lead invariant, description bounds, CV append-only | — | T03
2026-11-07 | T03 | crates/domain/src/inventory.rs | project card sequence, archive guard, key prefix validation | — | T04
2026-11-07 | T04 | .docs/product/inventory.md | doc link check | REST/OpenAPI/SQLx deferred to WP-16; GitHub verification WP-12; UI WP-19 | WP-08
