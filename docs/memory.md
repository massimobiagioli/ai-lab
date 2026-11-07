# AI-Lab Lab & Memory

| Field | Value |
|---|---|
| Document | `docs/memory.md` |
| Version | 1.0.0 |
| Status | Accepted |
| Owner | Dev |
| Related | `docs/inventory.md`, `docs/security.md`, `crates/domain/src/{lab,memory}.rs` |

The Lab context owns the lab aggregate, settings and the team invariant. The Knowledge/Memory context owns scoped documents, chunking, hybrid retrieval, deduplication, secret quarantine and compaction.

## 1. Lab

- A lab belongs to a workspace; `slug` is validated.
- A lab cannot be created without exactly one active Tech Lead (`TECH_LEAD_REQUIRED` / `TECH_LEAD_EXISTS`).
- Team members must belong to the lab.
- Settings: WIP limits, per-card and per-day budgets, hard stop. Per-card budget cannot exceed per-day.
- Archive is terminal and guarded.

## 2. Memory scopes

Exactly three scopes: lab, project, agent. Scope ids must match the scope; there is no global scope. Retrieval filters by `lab_id` and scope **before** scoring.

## 3. Chunking

- Word-based, deterministic: 800 words per chunk with 120-word overlap (15%).
- The same content always produces the same boundaries and token counts.
- Embeddings are 384-dimensional.

## 4. Secret quarantine

A document containing a known secret shape (`sk-`, `ghp_`, `github_pat_`, `AKIA`, `xoxb-`, `AILAB_MASTER_KEY=`, `password=`, PEM header) is rejected with `MEMORY_QUARANTINED` and is not embedded.

## 5. Hybrid retrieval

For each candidate chunk in the requested scope:

```
weighted = 0.7 * max(cosine, 0) + 0.3 * lexical_rank
score    = weighted * exp(-age_days / 30) * scope_boost
```

- `lexical_rank` is the fraction of query terms present in the chunk.
- Scope boost: project 1.0, agent 0.9, lab 0.8.
- Results are sorted by score; a candidate with cosine ≥ 0.92 against an already selected chunk is skipped (near-duplicate dedupe).
- The formula is tested to 1e-9 on a fixed fixture.

## 6. Compaction

- Episodes older than 30 days are compacted unless pinned.
- Raw episodes older than 180 days are deleted unless pinned.
- The policy is a pure function; the scheduled jobs are wired in WP-20/WP-23.

## 7. Tests

`cargo test -p ailab-domain` covers lab team/settings/archive, scope consistency, secret quarantine, deterministic chunking, scope leakage, the hybrid formula, near-duplicate dedupe and compaction with pins.

## 8. Deferred to consumer WPs

- Real fastembed pipeline, batching and retries: WP-08 follow-up.
- HNSW/hybrid SQL function, token budgeting, citations API: WP-08 follow-up/WP-16.
- Redis working memory and cache invalidation: WP-16.
- Memory API, OpenAPI and events: WP-16.
- Evaluation harness, nDCG@8 and 50k-chunk performance: WP-21.

## 9. References

- `.docs/sow/WP-08-LAB-MEMORY.md`
- `.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §6.2–§6.3
- `crates/domain/src/lab.rs`, `crates/domain/src/memory.rs`
