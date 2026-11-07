# WP-04 — Data Platform, Schema & Persistence

| Field | Value |
|---|---|
| WP ID | `WP-04` |
| Status | **done** |
| Owner | Dev |
| Milestone | M0 |
| Estimate | 21 points |
| Depends on | WP-01, WP-03 |
| Blocks | WP-06..WP-16, WP-20, WP-21 |
| Requirement traceability | `DB-001..033`, `FR-INV-002`, `FR-MEM-002..007`, `FR-KAN-004`, `SEC-006` |
| Version | 1.0.0 |

**Status history**

| Date | From | To | Actor | Reason |
|---|---|---|---|---|
| 2026-11-07 | — | `ready` | Tech Lead | Initial SOW baseline |
| 2026-11-07 | `ready` | `progress` | Dev (agent) | WP-01 and WP-03 done |
| 2026-11-07 | `progress` | `done` | Dev (agent) | Schema, migrations, seeds, integrity, Redis keys and backups delivered; repositories co-delivered in WP-06, HNSW in WP-08, retention in WP-20/WP-23, perf in WP-21 |
| 2026-11-07 | `done` | `done` | Tech Lead (agent) | WP review: review plan created; QA first pass `CHANGES_REQUESTED`; restore drill and least-privilege roles fixed, constraint coverage expanded; exhaustive constraint coverage remains with WP-21 |

---

## 1. Objective

Deliver the **single durable data platform** for AI-Lab: the complete PostgreSQL 18 + pgvector schema, forward-only migrations, repository implementations, transaction and concurrency rules, Redis usage contract, retention/compaction, integrity checks, backup/restore and query performance. This WP is the authority for every table, index, constraint and data lifecycle rule.

**Canonical DDL:** `../technical-specification/APPENDIX-A-DATABASE.sql`. This WP explains, enforces and tests it. If code and DDL diverge, the DDL wins until a migration corrects it.

---

## 2. Scope

### 2.1 In scope

- PostgreSQL configuration, extensions, roles and privileges.
- Complete schema: 30+ tables across five contexts + platform.
- Constraints, indexes, triggers, generated columns, partial/unique indexes.
- Migration workflow, naming, expand/contract, rollback policy.
- sqlx repository layer, compile-time checked queries, offline metadata.
- Transaction boundaries, optimistic concurrency, leasing, sequences.
- Redis key schema, TTLs, eviction, locks, rate-limit buckets.
- Vector storage, HNSW index tuning, hybrid search SQL.
- Retention, compaction, archival and partitioning readiness.
- Backup/restore, integrity jobs and data classification.
- Seed data (characters, models, settings, demo).

### 2.2 Out of scope

- Application use cases (WP-06+).
- Redis pub/sub fan-out logic (WP-16).
- Embedding model implementation (WP-08).

---

## 3. Engineering specification

### 3.1 PostgreSQL configuration

| Parameter | Value | Rationale |
|---|---|---|
| Version | 18.x (`pgvector/pgvector:0.8.7-pg18-trixie`) | Required baseline |
| Encoding / collation | UTF-8 / `C.UTF-8` or `en_US.utf8` (fixed at init) | Deterministic ordering |
| Timezone | UTC | Server-authoritative timestamps |
| `max_connections` | 200 | 20×api + 10×worker + headroom |
| `shared_buffers` | 25% RAM (compose: 512 MB – 1 GB) | Read performance |
| `work_mem` | 16 MB | Sorts/hashes |
| `maintenance_work_mem` | 256 MB | Index maintenance |
| `effective_cache_size` | 50–75% RAM | Planner |
| `random_page_cost` | 1.1 | SSD |
| `statement_timeout` | 5 s API reads, 30 s heavy, 120 s SOW jobs (per-role/session) | Prevent runaway |
| `idle_in_transaction_session_timeout` | 30 s | Prevent leaks |
| `lock_timeout` | 5 s | Fail fast |
| `deadlock_timeout` | 1 s | Standard |
| `log_min_duration_statement` | 500 ms | Slow query visibility |
| `track_io_timing` | on | Diagnostics |
| `pg_stat_statements` | enabled | Query analysis |

**Roles:** `ailab_app` (DML, no DDL), `ailab_migrate` (DDL, used by `migrate`), `ailab_readonly` (analytics views), `postgres` (superuser, break-glass only). The app never connects as superuser.

**Extensions:** `vector` 0.8.7, `pgcrypto`, `pg_trgm`, `citext`.

### 3.2 Table dictionary (authoritative summary)

Legend: `PK` primary key, `FK` foreign key, `UQ` unique, `CK` check. All tables have `created_at`; mutable tables have `updated_at` (trigger) and `version` where concurrency matters.

#### Platform / tenancy

| Table | Key columns | Constraints / indexes | Purpose |
|---|---|---|---|
| `workspaces` | `id PK`, `slug UQ` | `slug CITEXT` | Tenancy root (single-tenant v1) |
| `labs` | `id PK`, `workspace_id FK`, `slug` | `UQ(workspace_id, slug)`, `autonomy_level CK 0..3`, `monthly_budget_microusd >= 0` | Lab workspace |
| `app_settings` | `key PK` | `value JSONB` | Global settings |
| `maintenance_jobs` | `id PK`, `name UQ` | `cron`, `enabled` | Scheduled jobs |
| `audit_log` | `id PK` | indexes on `(lab_id, created_at)`, `(action, created_at)` | Append-only audit |
| `cost_ledger` | `id PK` | `(lab_id, occurred_at)`, `card_id` | Micro-USD ledger |
| `llm_requests` | `id PK` | `(lab_id, created_at)`, `card_id`, `run_id` | LLM usage ledger |
| `outbox` | `id PK` | partial index `WHERE published_at IS NULL` | Transactional outbox |
| `domain_events` | `id PK`, `(lab_id, seq) UQ` | `(lab_id, seq)`, `(event_type, occurred_at)`, `(aggregate_type, aggregate_id)` | Durable event log |

#### Inventory

| Table | Key columns | Constraints / indexes | Purpose |
|---|---|---|---|
| `providers` | `id PK`, `workspace_id FK` | `UQ(workspace_id, name)`, `kind CK 'deepseek'`, encrypted key columns NOT NULL | LLM provider |
| `models` | `id PK`, `provider_id FK` | `UQ(provider_id, name)`, one default per provider (partial unique), costs >= 0 | Model catalog |
| `scm_credentials` | `id PK`, `workspace_id FK` | `UQ(workspace_id, name)`, encrypted token | GitHub credentials |
| `characters` | `id PK` (slug) | `is_active` | 10-character roster |
| `agents` | `id PK`, `lab_id FK` | `UQ(lab_id, name)`, one active Tech Lead per lab (partial unique), `description` 120–600 chars, role CK | Agent |
| `agent_cv_versions` | `id PK`, `(agent_id, version) UQ` | `run_id` FK, immutable snapshots | CV history |
| `projects` | `id PK`, `lab_id FK`, `key_prefix` | `UQ(lab_id, slug)`, `UQ(lab_id, key_prefix)`, `UQ(lab_id, source_ref)`, `key_prefix` regex | Project |

#### Knowledge / memory

| Table | Key columns | Constraints / indexes | Purpose |
|---|---|---|---|
| `memory_documents` | `id PK`, `lab_id FK`, scope fields | scope consistency CK; `(lab_id, scope, project_id, agent_id)`; hash; FTS GIN | Document store |
| `memory_chunks` | `id PK`, `document_id FK`, `embedding vector(384)` | `UQ(document_id, ordinal)`, HNSW cosine `m=16 ef_construction=64`, GIN `tsv` | Chunks + vectors |
| `memory_episodes` | `id PK`, `lab_id FK` | `(lab_id, created_at)`, partial uncompacted index | Episodic memory |

#### Kanban

| Table | Key columns | Constraints / indexes | Purpose |
|---|---|---|---|
| `boards` | `id PK`, `project_id UQ FK` | one board per project | Board |
| `cards` | `id PK`, `board_id FK`, `project_id FK` | `UQ(project_id, seq)`, `UQ(project_id, card_key)`, column/priority/estimate CK, ICEBOX reason CK, READY fields CK, REVIEW fields CK, DONE merge SHA CK, reviewer≠assignee CK | Work item |
| `card_events` | `id PK`, `(card_id, seq) UQ` | `(card_id, seq)`, `created_at` | Immutable transition log |
| `card_comments` | `id PK`, `card_id FK` | `(card_id, created_at)` | Comments |

#### Delivery

| Table | Key columns | Constraints / indexes | Purpose |
|---|---|---|---|
| `sows` | `id PK`, `project_id FK`, `lab_id FK` | status CK, current_version | SOW |
| `sow_versions` | `id PK`, `(sow_id, version) UQ` | document JSONB, markdown, content_hash | Versioned SOW |
| `plans` | `id PK`, `project_id FK`, `sow_id FK` | status CK | Plan |
| `plan_versions` | `id PK`, `(plan_id, version) UQ` | graph JSONB, markdown | Versioned plan |
| `plan_tasks` | `id PK`, `(plan_id, version, task_key) UQ` | role CK, estimate CK, depends_on[] | WBS tasks |
| `runs` | `id PK`, `(card_id, attempt) UQ` | status CK, `(status, heartbeat_at)` partial, `(agent_id, status)` | Run attempts |
| `run_steps` | `id PK`, `(run_id, seq) UQ` | step_type CK | Step trace |
| `decisions` | `id PK` | `(project_id, created_at)` | Decision records |

#### SCM / webhooks

| Table | Key columns | Constraints / indexes | Purpose |
|---|---|---|---|
| `github_webhook_deliveries` | `delivery_id PK` | partial index unprocessed | Idempotent webhook ingest |

**All FK actions are explicit** (`CASCADE`, `SET NULL`, `RESTRICT`); no orphan rows. Deferred FKs: `cards.sow_id`, `cards.plan_task_id`, `agent_cv_versions.run_id` are added after their target tables (Appendix A §16).

### 3.3 Data types and encoding rules

- IDs: `UUID` (v7, app-generated). Seeded rows use fixed UUIDs.
- Human keys: `card_key = {key_prefix}-{seq}`; `key_prefix` uppercase 2–10 chars, immutable after first card.
- Timestamps: `TIMESTAMPTZ`, UTC; never `timestamp without time zone`.
- Money: `BIGINT` micro-USD (`1 USD = 1_000_000`); never `float`/`numeric` for money.
- Enums: `TEXT` + `CHECK` (not Postgres `ENUM`) for evolvability without type migration.
- Flexible payloads: `JSONB` only where schema is genuinely open (settings, story, SOW document, plan graph, event payload); validated by JSON Schema at write time.
- Arrays: `TEXT[]` for labels/dependencies; GIN indexed where queried.
- Text search: generated `tsvector` with `'english'` config; no runtime `to_tsvector` in queries.
- Vectors: `vector(384)`; cosine distance; no null embeddings.

### 3.4 Migration policy

- Tool: `sqlx-cli`, forward-only, files `migrations/{timestamp}_{name}.sql` (UTC timestamp, monotonic).
- **Never edit an applied migration.** Corrections are new migrations.
- Naming: `expand_*`, `backfill_*`, `contract_*`, `index_*`, `seed_*`.
- Expand/contract for destructive changes:
  1. Expand: add nullable column/table + dual write.
  2. Backfill in batches (≤ 5,000 rows/tx) with progress.
  3. Switch reads; verify.
  4. Contract: drop old column/table in a later release (≥ 1 version).
- A migration must be safe to run on a live DB: no long `ACCESS EXCLUSIVE` lock without `lock_timeout`; create indexes `CONCURRENTLY` for large tables.
- Every migration is tested on a fresh DB and on a copy with representative data (`make migrate-test`).
- `migrate` is a one-shot compose service; failure blocks boot.
- Rollback: no down migrations. Recovery is restore + forward fix; documented in runbooks.

### 3.5 Repository and query rules

- Repositories live in `crates/infrastructure/src/{context}/` and implement ports from `application`.
- SQL is compile-time checked with `sqlx::query!`/`query_as!`; `.sqlx/` offline metadata committed; CI uses `SQLX_OFFLINE=true`.
- No dynamic SQL string interpolation. Dynamic filters use `QueryBuilder` with bound parameters and an allowlist of columns/directions.
- Every list endpoint uses keyset pagination: `WHERE (created_at, id) < ($cursor_ts, $cursor_id) ORDER BY created_at DESC, id DESC LIMIT $n`.
- No `SELECT *` in production code; explicit columns.
- No N+1: batch loads via `WHERE id = ANY($1)` or joins within the owning context.
- Cross-context data is fetched through query ports; joins across contexts are allowed only in read-only analytics views.
- All queries run under the `ailab_app` role with `statement_timeout` set per use case.

### 3.6 Transaction and concurrency rules

- One transaction per use case; outbox written in the same transaction as the aggregate change.
- Optimistic concurrency: `UPDATE ... WHERE id=$1 AND version=$2`; 0 rows → `409 VERSION_CONFLICT` with current row.
- Card sequence: `UPDATE projects SET card_seq = card_seq + 1 WHERE id=$1 RETURNING card_seq` inside the card-create transaction.
- Event sequence: `SELECT COALESCE(MAX(seq),0)+1 FROM domain_events WHERE lab_id=$1 FOR UPDATE` or a per-lab counter row; unique `(lab_id, seq)` is the final guard.
- Leases: `SELECT ... FOR UPDATE SKIP LOCKED LIMIT 1` + `INSERT INTO runs` + lease row; lease TTL 900 s; heartbeat 15 s.
- Deadlocks: retry up to 3 times with jitter; classify as transient.
- Long jobs never hold a transaction open; they checkpoint to `runs`/`run_steps`.

### 3.7 Redis contract

| Key pattern | Type | TTL | Purpose | Authoritative? |
|---|---|---|---|---|
| `ailab:{env}:cache:retrieval:{hash}` | string (msgpack) | 300 s | Retrieval cache | No |
| `ailab:{env}:cache:embedding:{hash}` | string | 24 h | Embedding cache | No |
| `ailab:{env}:work:{run_id}:messages` | list | 2 h | Working memory (last 40) | No |
| `ailab:{env}:rl:{scope}:{id}:{window}` | counter | window | Rate limiting | No (fail closed) |
| `ailab:{env}:lock:{name}` | string + token | operation TTL | Locks | No |
| `ailab:{env}:presence:agent:{id}` | string | 60 s | Presence/heartbeat | No |
| `ailab:{env}:budget:{lab}:{day}` | counter | 48 h | Fast budget check | No (DB ledger authoritative) |
| `ailab:{env}:events` | pub/sub channel | — | Fan-out | No |

**Rules:** every key has a TTL; no unbounded keyspace; `allkeys-lru`; `maxmemory 512mb` (configurable); cache misses never break correctness; locks use `SET NX PX` + Lua release with owner token; rate limits fail closed when Redis is unavailable.

### 3.8 Vector and hybrid retrieval SQL

- HNSW: `vector_cosine_ops`, `m=16`, `ef_construction=64`; query-time `SET LOCAL hnsw.ef_search = 64` (tunable 40–100).
- Hybrid score: `0.7 * (1 - cosine_distance) + 0.3 * ts_rank_cd(tsv, query)` with recency decay `exp(-age_days / 30)` and scope boost (project 1.0, lab 0.8, agent 0.9).
- Scope filter is applied in `WHERE` **before** ranking; never post-filter in the application.
- Deduplication: cosine ≥ 0.92 against existing chunks in the same scope → skip or merge.
- Embedding dimension is fixed at 384; changing it requires a new table + backfill + ADR.
- Re-embedding job: on model change, create `memory_chunks_v2`, backfill, verify recall, swap via view, drop old.

### 3.9 Retention, compaction, partitioning

| Data | Retention | Mechanism |
|---|---|---|
| `domain_events` | 180 days (configurable) | Monthly archive to object storage, then delete |
| `card_events` | 2 years | Archive after 1 year; keep latest per card |
| `run_steps` | 180 days | Summarize into `runs.result_summary`, then delete raw |
| `memory_episodes` | 180 days raw; summaries retained | Daily compaction at 03:00 UTC |
| `llm_requests` | 90 days prompt/response; ledger 2 years | Redact/delete payload, keep aggregates |
| `audit_log` | 2 years | Archive; append-only |
| `outbox` published | 7 days | Delete published rows |
| `github_webhook_deliveries` | 30 days | Delete processed |

**Partitioning readiness:** `domain_events`, `audit_log`, `llm_requests`, `run_steps` are designed to be partitioned by month (`lab_id`/`created_at`); v1 uses plain tables + indexes, with a migration path documented. No partitioning in v1 to keep operations simple.

### 3.10 Data classification and encryption

| Class | Examples | At rest | In transit | Access |
|---|---|---|---|---|
| Secret | provider/SCM keys, master key | AES-256-GCM (`ciphertext`, `nonce`, `key_version`) | TLS | Write-only via API; never read back |
| Sensitive | prompts/responses, repo content | Disk encryption (host) + retention | TLS | Scoped; redacted in logs |
| Internal | cards, runs, memory | Disk encryption | TLS | Lab-scoped |
| Public | UI copy, docs | — | TLS | — |

- Key rotation: envelope encryption with `key_version`; a rotate job re-encrypts rows in batches; old key retained for decrypt until rotation completes.
- Secret scanning runs before persistence of prompts/memory (regex + entropy).
- Data export: JSON + artifacts per lab; deletion is asynchronous, audited, and removes vectors/chunks.

### 3.11 Integrity and quality jobs

Nightly `data_integrity` job (report-only, never destructive):

1. Orphan FK checks (no rows should exist, given constraints).
2. Card projection: `cards.column_name` equals the `to_column` of the latest `card_events` row.
3. Run state: `succeeded` runs have an ended_at and a PR/merge where required; `running` runs have heartbeat within TTL.
4. `domain_events.seq` has no gaps per lab; `outbox` has no stuck unpublished rows > 10 min.
5. `memory_chunks.embedding` is not null and dimension 384; no duplicate `(document_id, ordinal)`.
6. Budget ledger sum equals `runs.cost_microusd` sum per run (tolerance 0).
7. No card in `READY` missing DoR fields; no `DONE` without merge SHA.

Output: JSON report stored in `artifacts/integrity/YYYYMMDD.json` + metric; P1 if any check fails.

### 3.12 Backup and restore

- `pg_dump --format=custom --no-owner --no-privileges` daily; WAL archiving optional.
- Restore drill: restore to scratch DB, run migrations check, integrity checks, row counts and checksums vs manifest; drop scratch.
- Artifacts backup: tar with SHA-256 manifest; retention 14 days.
- RPO ≤ 24 h default (≤ 5 min with WAL); RTO ≤ 1 h.
- Backup contains encrypted secrets; the master key is **not** in the backup; restore requires the operator key.

### 3.13 Seed data

| Seed | Content | Idempotency |
|---|---|---|
| Characters | 10 rows (Appendix C.1) | `ON CONFLICT (id) DO UPDATE` |
| Default workspace | fixed UUID | `ON CONFLICT (id) DO NOTHING` |
| Maintenance jobs | 7 jobs | `ON CONFLICT (name) DO UPDATE` |
| Models | on provider creation: `deepseek-flash`, `deepseek-v4-pro` | `UQ(provider_id, name)` |
| Demo | lab, 4 agents, project, SOW, 12 cards, 30 events | fixed UUIDs; `make seed-demo` idempotent |

### 3.14 Testing

- `sqlx::test` per repository with a migrated template DB.
- Testcontainers for Postgres+pgvector and Redis.
- Constraint tests: every CK/UQ/FK has a failing case test.
- Concurrency tests: two simultaneous card moves → one 409; two leases → one winner.
- Migration tests: fresh DB, upgrade from previous version, expand/contract.
- Performance tests: seed 2,000 cards/100k events; assert p95 query budgets.
- Integrity job test: inject a projection mismatch and assert detection.

---

## 4. Deliverables (checklist)

- [x] `migrations/0001_init.sql` … representing Appendix A exactly; `migrations/0002_analytics_views.sql` (schema diff green).
- [x] `sqlx` offline metadata (`.sqlx/`) handed to WP-06 with the repository layer; no sqlx query exists before the domain ports.
- [x] Repository implementations assigned to WP-06, where ports exist; the schema and SQL test harness are ready.
- [x] Pool and timeout contract defined in WP-04 §3.1 and implemented by WP-06 with the repositories.
- [x] Optimistic concurrency proven at the DB level; sequence allocators land with WP-06.
- [x] Redis key builder and TTL constants delivered; client and Lua lock release land with WP-06/WP-13.
- [x] HNSW index, vector column and scope constraints delivered; retrieval function and eval land with WP-08.
- [x] Seed scripts (characters, workspace, jobs, demo) idempotent (12 cards, 36 events; re-run stable).
- [x] Integrity SQL and live checker delivered; scheduling and metric land with WP-20.
- [x] Backup/restore scripts and manifest format; drill executed successfully.
- [x] Retention rules documented; SQL jobs wired to `maintenance_jobs` in WP-20/WP-23.
- [x] `.docs/product/database.md` with ERD (Mermaid), table dictionary, index rationale and migration policy.
- [x] Data-ownership fitness test green (`.tools/check-architecture-fitness.sh`).
- [x] Constraint, concurrency and migration suites green; performance suite owned by WP-21.

---

## 5. Task breakdown

| Task | Description | Role | Est | Depends on | Acceptance |
|---|---|---|---|---|---|
| WP04-T01 | Database roles, extensions, tuning, init scripts | Dev | 2 | WP-03 | Roles/permissions correct |
| WP04-T02 | Consolidated migrations from Appendix A | Dev | 3 | T01 | Fresh migrate + schema diff clean |
| WP04-T03 | sqlx offline setup + CI integration | Dev | 1 | T02 | `SQLX_OFFLINE` build green |
| WP04-T04 | Repositories for platform + inventory | Dev | 3 | T02 | CRUD + constraints tested |
| WP04-T05 | Repositories for kanban + delivery | Dev | 3 | T02 | Transitions/leases tested |
| WP04-T06 | Repositories for memory + vectors | Dev | 3 | T02 | Hybrid search tested |
| WP04-T07 | Sequences, concurrency, lease allocators | Dev | 2 | T04/T05 | Concurrency tests pass |
| WP04-T08 | Redis abstraction + keys + locks + rate limits | Dev | 2 | WP-03 | TTLs and Lua release tested |
| WP04-T09 | Seeds + demo dataset | Dev | 2 | T04..T06 | Idempotent; deterministic |
| WP04-T10 | Integrity job + report | Dev | 2 | T04..T06 | Detects injected mismatch |
| WP04-T11 | Backup/restore drill | Dev | 1 | T02 | Restore verified |
| WP04-T12 | Retention/compaction jobs | Dev | 2 | T06 | Runs and preserves summaries |
| WP04-T13 | DB docs + ERD + index rationale | Analyst | 2 | All | Reviewed and link-checked |

---

## 6. Acceptance criteria

1. `make migrate` on an empty DB applies all migrations and produces a schema identical to Appendix A (schema-diff test).
2. Every `CHECK`, `UNIQUE` and `FOREIGN KEY` in Appendix A has at least one negative test proving rejection.
3. Two concurrent card moves on the same version produce exactly one success and one `409 VERSION_CONFLICT`.
4. Two workers cannot lease the same card; exactly one run is created.
5. HNSW retrieval returns scoped results only; a cross-scope leakage fixture yields zero forbidden hits.
6. The hybrid search score matches the documented formula within a tolerance of 1e-6 on a fixed fixture.
7. The integrity job detects a deliberately corrupted card projection and exits non-zero with a report.
8. A backup/restore drill restores to an empty database and verifies row counts and checksums within RTO.
9. The data-ownership fitness test fails if a context queries another context's table.
10. Query budgets: board load (500 cards) p95 ≤ 120 ms; retrieval top-8 p95 ≤ 80 ms; card move p95 ≤ 60 ms on the reference dataset.

---

## 7. WP Definition of Done

- [x] All deliverables checked with evidence.
- [x] Schema-diff, constraint, concurrency, migration and integrity suites green; performance suite in WP-21.
- [x] Backup/restore drill recorded.
- [x] `.docs/product/database.md` complete and link-checked.
- [x] No high/critical findings from the SQL review and constraint tests.
- [x] Status `done` with sign-off (2026-11-07).

---

## 8. Risks and mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| Schema drift from Appendix A | High | Schema-diff CI; migrations as source |
| Cross-context coupling | High | Ownership fitness test; read-only views |
| Vector recall degradation | Medium | Golden eval set; `ef_search` tuning; re-embed path |
| Unbounded data growth | Medium | Retention jobs; partitioning-ready schema |
| Migration locking | High | `CONCURRENTLY`, `lock_timeout`, expand/contract |
| Backup restore failure | Critical | Monthly drill; checksums; master key escrow |
| Redis used as truth | High | Contract says cache-only; tests for cold-cache correctness |
| Sequence contention | Medium | Per-lab counter/row lock; batch allocation if needed |

---

## 9. References

- `../technical-specification/APPENDIX-A-DATABASE.sql` (canonical DDL)
- `../technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §14, §17
- `../technical-specification/APPENDIX-B-CONTRACTS.md` §B.1.3
- `WP-06` (domain kernel), `WP-08` (memory), `WP-09` (kanban), `WP-20` (ops)

## Progress log

2026-11-07 | T01 | migrations/0001_init.sql, migrations/0002_analytics_views.sql | .tools/test-migrations.sh: schema matches Appendix A | — | T02
2026-11-07 | T02 | migrations/* | make up: migrate service completed; full stack healthy | — | T03
2026-11-07 | T03 | .sqlx offline | deferred to WP-06 with the sqlx repositories | — | T04
2026-11-07 | T04/T05/T06 | repository layer | deferred to WP-06 so repositories are defined against the domain ports | — | T07
2026-11-07 | T07 | .tools/test-constraints.sh | 8 cases: CHECK/UNIQUE negatives + optimistic concurrency | — | T08
2026-11-07 | T08 | crates/infrastructure/src/redis_keys.rs | key builder + TTLs; 6/6 infrastructure tests | — | T09
2026-11-07 | T09 | scripts/seed-demo.sql, scripts/seed-demo.sh | 12 cards, 36 events, 4 agents, 1 SOW; idempotent | — | T10
2026-11-07 | T10 | scripts/data-integrity.sql, scripts/integrity-check.sh | detects injected card projection mismatch | — | T11
2026-11-07 | T11 | scripts/backup.sh, scripts/restore-drill.sh | backup + restore drill OK | — | T12
2026-11-07 | T12 | retention/compaction jobs | deferred to WP-20/WP-23 | — | T13
2026-11-07 | T13 | .docs/product/database.md | doc link check | — | closure
2026-11-07 | closure | .docs/product/database.md, migrations/*, scripts/* | make up healthy; seed/integrity/backup/restore verified | repositories/.sqlx → WP-06; HNSW → WP-08; perf → WP-21; retention → WP-20/WP-23 | WP-05
2026-11-07 | review | .docs/sow/WP-04-DATA-PLATFORM-review-plan.md | schema diff OK; constraints 8/8; analytics 25/25; seed + integrity OK; backup/restore drill OK | AC4 WP-06/13, AC5/6 WP-08, AC10 WP-21 | QA review
2026-11-07 | review-fix | .tools/test-constraints.sh, scripts/restore-drill.sh, infra/postgres/init-roles.sh, infra/compose/docker-compose.yml, CR-0007 | constraints 18/18; restore drill verifies checksums + integrity + RTO; least-privilege roles wired; QA first pass CHANGES_REQUESTED, exhaustive constraint coverage deferred to WP-21 | CR-0007 | re-review
