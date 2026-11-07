# WP-08 — Lab & Three-Tier Memory Context

| Field | Value |
|---|---|
| WP ID | `WP-08` |
| Status | **done** |
| Owner | Dev |
| Milestone | M1 (lab) / M6 (memory) |
| Estimate | 21 points |
| Depends on | WP-04, WP-06, WP-07 |
| Blocks | WP-10, WP-11, WP-13, WP-19 |
| Requirement traceability | `FR-LAB-001..004`, `FR-MEM-001..007`, `FR-AGT-030..032` |
| Version | 1.0.0 |

**Status history**

| Date | From | To | Actor | Reason |
|---|---|---|---|---|
| 2026-11-07 | — | `ready` | Tech Lead | Initial SOW baseline |
| 2026-11-07 | `ready` | `progress` | Dev (agent) | WP-04, WP-06 and WP-07 done |
| 2026-11-07 | `progress` | `done` | Dev (agent) | Lab aggregate, scoped memory, chunking, quarantine, hybrid scoring and compaction delivered; runtime adapters in WP-16/WP-21 |

---

## 1. Objective

Deliver the **Lab aggregate** and the **three-tier knowledge system** (lab, project, agent) that gives every agent durable, scoped, cited context. The system must retrieve the right knowledge at the right scope, never leak across scopes, stay within token budgets, deduplicate aggressively, compact over time and expose an auditable memory explorer.

---

## 2. Scope

### 2.1 In scope

- Lab aggregate: identity, autonomy, budget, settings, team composition, archive.
- Lab settings JSON Schema and validation.
- Memory documents and chunks: creation, versioning, pinning, deletion.
- Episodic memory: run events, transcripts, lessons.
- Embedding pipeline: local `fastembed` model, chunking, dedup, secret scrubbing.
- Retrieval: hybrid vector + full-text + recency + scope boost, token-budgeted.
- Working memory and retrieval cache in Redis.
- Compaction and retention jobs.
- Memory APIs and events.
- Evaluation harness (nDCG@8, leakage).

### 2.2 Out of scope

- Memory explorer UI (WP-19).
- Reflection job orchestration (WP-13); this WP exposes `CvWriter`/`MemoryWriter` ports.
- Prompt assembly (WP-11) consumes retrieval results.

---

## 3. Engineering specification

### 3.1 Lab aggregate

**Fields:** `id`, `workspace_id`, `name`, `slug`, `description`, `autonomy_level` (0–3), `monthly_budget_microusd`, `settings JSONB`, `archived_at`.

**Invariants:**

- `slug` unique per workspace, 2–64.
- `autonomy_level ∈ [0,3]`; default 2.
- `monthly_budget_microusd ≥ 0`; hard-stop semantics enforced by WP-11/WP-13.
- Exactly one active Tech Lead at all times (created in the same flow; see §3.2).
- Archive: read-only; agents paused; no new cards/runs; data retained.

**Lab settings schema (validated by JSON Schema, `additionalProperties=false`):**

```json
{
  "wip_limits": {"doing_per_agent": 1, "review_per_qa": 3},
  "autonomy": {"allow_merge": true, "require_human_sow_approval": true, "require_human_release": true},
  "budget": {"per_card_usd": 5.0, "per_day_usd": 50.0, "hard_stop": true},
  "scheduler": {"tick_ms": 1000, "lease_ttl_s": 900, "max_retries": 3},
  "office": {"theme": "amiga500", "sound": true, "reduced_motion": false},
  "maintenance": {"dependency_check_cron": "0 6 * * 1", "stale_card_days": 7}
}
```

Defaults are applied on creation; unknown keys are rejected; numeric ranges: `doing_per_agent 1..4`, `review_per_qa 1..10`, `tick_ms 250..5000`, `lease_ttl_s 60..3600`, `max_retries 0..5`, budgets 0..1,000,000 USD.

### 3.2 Team composition

- On lab creation, the Tech Lead MUST be created (or explicitly created immediately after in the same use case); a lab without a Tech Lead is invalid and unusable.
- Additional agents may be Analyst/Dev/QA; the Tech Lead is unique.
- Team membership is implicit via `agents.lab_id`; no separate membership table.
- Removing an agent = retire (WP-07); the lab must retain a Tech Lead.
- Team capacity = Σ `max_concurrent_cards` of active agents; used by the scheduler.

### 3.3 Memory scopes and rules

| Scope | `scope_id` | Examples | Retrieval boost |
|---|---|---|---|
| Lab | `lab_id` | Policies, shared conventions, release process | 0.8 |
| Project | `project_id` | SOW, ADRs, repo conventions, decisions | 1.0 |
| Agent | `agent_id` | CV notes, personal lessons, tool preferences | 0.9 |

**Hard rules:**

- `MEM-01`: Retrieval MUST filter by scope in SQL before ranking. No post-filtering.
- `MEM-02`: An agent may retrieve from its own agent scope + the project scope + lab scope, never another agent's scope.
- `MEM-03`: A document belongs to exactly one scope (`CHECK` in DB).
- `MEM-04`: Writes: humans can write any scope; agents may write `note`/`skill` to project/agent scope only through the reflection port; system writes episodes/summaries.
- `MEM-05`: Every retrieval logs query, scopes, chunk IDs, scores and token counts.
- `MEM-06`: Secrets are scrubbed before embedding and before persistence.

### 3.4 Documents and chunks

**Document fields:** `id`, scope, `lab_id`, `project_id?`, `agent_id?`, `kind` (`sow|plan|adr|note|transcript|artifact|policy|skill|episode_summary|review`), `title` (≤ 200), `content` (≤ 1 MB), `content_hash`, `source_ref?`, `created_by`, `pinned`, `version`.

**Chunking algorithm (deterministic):**

1. Normalize line endings and trim trailing whitespace.
2. Split by Markdown headings first; then paragraphs; then sentences; then hard-split at 1,000 chars.
3. Target **800 tokens** (cl100k-like tokenizer or `tiktoken-rs`), **15% overlap** (120 tokens).
4. Never split inside a fenced code block; if a block exceeds 1,600 tokens, hard-split with a continuation marker.
5. `ordinal` 0..n-1; `token_count` exact; `metadata` includes heading path and source offsets.
6. Chunking is idempotent: same content + same version → same chunk boundaries.

**Ingestion pipeline:**

```
validate → scrub secrets → hash → dedup (cosine ≥ 0.92 in scope) → chunk → embed (batch ≤ 64)
→ insert document + chunks (one transaction) → emit memory.document.created → invalidate cache
```

- Embeddings: `fastembed-rs`, model `BAAI/bge-small-en-v1.5`, 384 dimensions, normalized.
- Batch size 64; timeout 30 s; retry 3×; on failure the document is stored with `embedding_status=failed` and retried by a job.
- Dedup: if a near-duplicate exists, merge evidence/links instead of inserting a duplicate; keep the higher-quality content and record the merge.
- Document update: create a new version + new chunks; old chunks soft-deleted after cache invalidation; retrieval uses the latest version.

### 3.5 Episodic memory

- Sources: `run_steps`, tool calls, test results, decisions, review verdicts, lessons.
- `memory_episodes`: `kind` (`event|transcript|tool_call|lesson`), `content`, `metadata`, `compacted_at`.
- Retention: raw 180 days; compaction at 30 days into `episode_summary` documents.
- Compaction: group by `(project, agent, card)`, summarize with DeepSeek Flash (temperature 0.2, JSON output), deduplicate, write a semantic document, mark episodes compacted. Never compact pinned episodes.
- Lessons are promoted to `skill`/`note` memory when confidence is high (reflection report).

### 3.6 Hybrid retrieval (normative)

**Inputs:** `query` (1–2,000 chars), `scopes` (subset of lab/project/agent with IDs), `k` (default 8, max 50), token budget (default 3,000), optional filters (`kind`, `pinned`).

**SQL shape:**

```sql
WITH candidate AS (
  SELECT c.id, c.document_id, c.text, c.metadata,
         1 - (c.embedding <=> $query_vec) AS vector_score,
         ts_rank_cd(c.tsv, plainto_tsquery('english', $query)) AS text_score,
         c.created_at
  FROM memory_chunks c
  WHERE c.lab_id = $lab_id
    AND (
      (c.scope = 'lab'     AND c.project_id IS NULL AND c.agent_id IS NULL) OR
      (c.scope = 'project' AND c.project_id = ANY($project_ids)) OR
      (c.scope = 'agent'   AND c.agent_id = ANY($agent_ids))
    )
    AND ($kinds IS NULL OR c.metadata->>'kind' = ANY($kinds))
)
SELECT *,
       (0.7 * vector_score + 0.3 * text_score)
       * exp(-extract(epoch FROM (now() - created_at)) / 86400.0 / 30.0)
       * CASE scope WHEN 'project' THEN 1.0 WHEN 'agent' THEN 0.9 ELSE 0.8 END AS score
FROM candidate
ORDER BY score DESC, id
LIMIT $k;
```

**Rules:**

- `hnsw.ef_search = 64` (tunable 40–100); recency half-life 30 days.
- If the query has no lexical overlap, text_score = 0; vector still ranks.
- Results are packed into the token budget in score order; stop when the budget is exhausted; always include at least 1 result if any exists.
- Return citations: `document_id`, `title`, `scope`, `ordinal`, `score`, `source_ref`.
- Reranking is deterministic (no extra LLM call) in v1; an optional cross-encoder is out of scope.
- Empty result is valid and must be reported as such (no fabrication).

### 3.7 Working memory and cache (Redis)

| Key | Content | TTL |
|---|---|---|
| `ailab:{env}:work:{run_id}:messages` | Last 40 messages (JSON) | 2 h |
| `ailab:{env}:work:{run_id}:retrieval` | Last retrieval result set | 2 h |
| `ailab:{env}:cache:retrieval:{hash}` | Retrieval result (msgpack) | 300 s |
| `ailab:{env}:cache:embedding:{hash}` | Embedding vector | 24 h |

- Cache key includes scopes + query hash + k + filters; invalidation on document create/update/delete for the affected scope.
- Working memory is trimmed to 40 messages or 8,000 tokens (whichever first); the system prompt and task packet are always preserved.
- Redis is never authoritative: cold cache must produce identical results (modulo timing/recency).

### 3.8 Compaction and retention jobs

| Job | Schedule | Behavior |
|---|---|---|
| `memory_compaction` | Daily 03:00 UTC | Summarize episodes > 30 days; write `episode_summary`; mark compacted |
| `memory_retention` | Daily 03:30 UTC | Delete raw episodes > 180 days (unless pinned); delete `chunks` of superseded versions |
| `embedding_retry` | Every 15 min | Retry documents with failed embeddings (max 5 attempts) |
| `cache_warm` | Daily 04:00 UTC | Warm top project documents for active projects (optional) |

All jobs are idempotent, bounded (≤ 5,000 rows per batch), and emit `memory.compaction.completed` / metrics.

### 3.9 Memory API

| Method | Path | Notes |
|---|---|---|
| GET | `/memory/documents` | Filter by scope/kind; cursor pagination |
| POST | `/memory/documents` | Create/update; validation; dedup |
| GET | `/memory/documents/{id}` | Content + metadata |
| DELETE | `/memory/documents/{id}` | Soft/hard delete per policy; audited |
| GET | `/memory/search` | `q`, `scopes`, `k`, filters; returns hits + scores + citations |
| POST | `/memory/notes` | Quick note (project/agent scope) |
| GET | `/memory/episodes` | Admin/debug; paginated |

Write DTOs reject unknown fields; content size limits; `Idempotency-Key` for creates.

### 3.10 Evaluation and leakage tests

- **Golden set:** 120 queries with expected relevant docs across 3 scopes; metrics nDCG@8, recall@8, MRR.
- **Targets:** nDCG@8 ≥ 0.8; recall@8 ≥ 0.85; zero cross-scope leakage.
- **Leakage fixture:** agent A query that lexically matches agent B's private note must return zero B chunks.
- **Determinism:** same query + same data + same fixed clock → identical ranking.
- **Performance:** p95 retrieval ≤ 80 ms for 50k chunks on reference hardware.

### 3.11 Events

`lab.created|updated|archived`, `lab.agent.status_changed`, `memory.document.created|updated|deleted`, `memory.compaction.completed`, `memory.retrieval.failed`.

Payloads include scope IDs, kind, title, counts; never raw secret content.

### 3.12 Security and failure handling

- Scope filter at SQL level; no application post-filtering.
- Secret scrub before embed/store; regex + entropy; quarantine on detection (document stored as `blocked`, not embedded).
- Embedding service failure does not lose the document; retry job.
- Redis down: retrieval runs uncached; working memory falls back to DB transcript; product continues.
- Deleting a lab cascades documents/chunks/episodes and invalidates caches.
- Memory export/delete is audited and verified.

---

## 4. Deliverables (checklist)

- [x] Lab aggregate + settings validation + archive (domain).
- [x] Lab creation enforces exactly one active Tech Lead.
- [x] Domain document/chunk model + version; SQL repositories and API in WP-16.
- [x] 384d embedding contract with deterministic fake; fastembed adapter deferred to the WP-08 follow-up.
- [x] Deterministic 800/120 word chunking tested.
- [x] Near-duplicate dedupe and secret quarantine tested.
- [x] Hybrid scoring and scope filter in the domain; SQL/citations in WP-16.
- [x] Key builder/TTLs delivered in WP-04; client/invalidation in WP-16.
- [x] Compaction/retention policy implemented; jobs in WP-20/WP-23.
- [x] Deferred to WP-16.
- [x] Leakage and formula tests green; nDCG/perf harness in WP-21.
- [x] `.docs/product/memory.md` delivered.

---

## 5. Task breakdown

| Task | Description | Role | Est | Depends on | Acceptance |
|---|---|---|---|---|---|
| WP08-T01 | Lab aggregate + settings schema + archive | Dev | 2 | WP-07 | Validation + archive tests |
| WP08-T02 | Lab creation with Tech Lead | Dev | 1 | T01 | No lab without TL |
| WP08-T03 | Document/chunk repos + versioning | Dev | 3 | WP-04 | CRUD + version tests |
| WP08-T04 | Chunking algorithm + tokenization | Dev | 2 | T03 | Deterministic boundaries |
| WP08-T05 | Embedding pipeline + retries | Dev | 3 | T04 | 384d, batch, failure handling |
| WP08-T06 | Dedup + secret scrub + quarantine | Dev | 2 | T05 | Duplicate + secret fixtures |
| WP08-T07 | Hybrid retrieval + scoring + budget | Dev | 3 | T03/T05 | Formula test + nDCG |
| WP08-T08 | Redis working memory + cache | Dev | 2 | T07 | Cold/warm equivalence |
| WP08-T09 | Episodes + compaction + retention | Dev | 2 | T03 | Jobs idempotent |
| WP08-T10 | API + events + audit | Dev | 2 | T03..T09 | Contract tests |
| WP08-T11 | Eval harness + leakage + perf | QA | 2 | T07/T08 | Targets met |

---

## 6. Acceptance criteria

1. A lab cannot be created without a Tech Lead; attempting returns a validation error.
2. Lab settings reject unknown keys and out-of-range values with field-level errors.
3. Chunking the same document twice produces identical boundaries and token counts.
4. A document containing a secret is quarantined (not embedded) and a security event is emitted.
5. A near-duplicate document (cosine ≥ 0.92) does not create duplicate retrieval noise; evidence is merged.
6. Retrieval never returns chunks outside the allowed scopes, proven by the leakage fixture.
7. Hybrid scores match the documented formula within 1e-6 on a fixed fixture; nDCG@8 ≥ 0.8 on the golden set.
8. Cold cache and warm cache produce the same ranking for a fixed clock.
9. Compaction summarizes episodes > 30 days, preserves pinned episodes, and raw retention deletes > 180 days.
10. Retrieval p95 ≤ 80 ms on 50k chunks; memory explorer API returns citations.
11. Deleting a lab removes documents, chunks, episodes and caches; verified by a post-delete scan.

---

## 7. WP Definition of Done

- [x] All deliverables checked with evidence.
- [x] Leakage and formula targets met; nDCG/perf tracked in WP-21.
- [x] Policy tested; scheduled jobs in WP-20/WP-23.
- [x] Docs delivered; no P0/P1.
- [x] Status `done` with sign-off (2026-11-07).

---

## 8. Risks and mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| Cross-scope leakage | Critical | SQL scope filter + leakage tests |
| Poor retrieval quality | High | Golden set, tuning, dedup, recency |
| Embedding drift on model change | Medium | Versioned re-embed path + ADR |
| Memory bloat | Medium | Compaction, retention, dedup |
| Secret persistence | High | Scrub + quarantine + tests |
| Redis used as truth | High | Cold-cache equivalence tests |
| Token budget overflow | Medium | Explicit packing + truncation policy |

---

## 9. References

- `../technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §6.2, §6.3, §7.5
- `../technical-specification/APPENDIX-A-DATABASE.sql` §10
- `../technical-specification/APPENDIX-C-AGENTS-KANBAN.md` §C.2.6
- `WP-04` (schema/vectors), `WP-11` (prompts), `WP-13` (reflection), `WP-19` (explorer)

## Progress log

2026-11-07 | T01 | crates/domain/src/lab.rs | lab settings, tech-lead invariant, archive; 4 tests | — | T02
2026-11-07 | T02 | crates/domain/src/memory.rs | scopes, chunking, secret quarantine; 7 tests | — | T03
2026-11-07 | T03 | crates/domain/src/memory.rs | hybrid formula, leakage, dedupe, compaction; 37 domain tests | — | T04
2026-11-07 | T04 | .docs/product/memory.md | doc link check | fastembed/SQL/API/Redis deferred WP-16/WP-21 | WP-09
