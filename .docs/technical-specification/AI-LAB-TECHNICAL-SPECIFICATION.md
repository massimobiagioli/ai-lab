# AI-Lab — Technical Specification (Capitolato Tecnico)

| Field | Value |
|---|---|
| Document ID | `AILAB-SPEC-001` |
| Title | AI-Lab — Autonomous Software Delivery Lab |
| Version | 1.0.0 |
| Status | **APPROVED — BINDING** |
| Date | 2026-11-07 |
| Owner | Product & Engineering (single decision authority) |
| Audience | Implementing AI agents, engineering leads, reviewers, operators |
| Normative language | RFC 2119 / RFC 8174 (`MUST`, `MUST NOT`, `SHOULD`, `MAY`) |
| Related documents | Appendices A–D (same folder), `idea.txt`, `spunto1.png` |

> **Instruction to implementing agents.** This document is the single source of truth. Where this document is silent, choose the option that best satisfies the stated SLOs, security posture and UX principles, then record the decision as an ADR. Do **not** ask the customer questions. Do **not** substitute technologies. Every requirement carries a stable ID (`FR-*`, `NFR-*`, `SEC-*`, `UX-*`, `OPS-*`); every implementation task and test MUST trace back to one or more IDs.

---

## 0. How to read this document

1. **Normative sections** are §1–§26. Appendices are normative where they define schemas, contracts, prompts, state machines or acceptance criteria.
2. **MUST / MUST NOT** are hard requirements and are individually testable. **SHOULD** deviations require an ADR with a measured justification. **MAY** is optional.
3. **All versions are pinned** in `.tool-versions`, `Cargo.lock`, `package-lock.json`, and image digests. "Latest" means the newest stable release available at bootstrap time, never a pre-release, never an EOL line.
4. **Identifiers** (`FR-KAN-004`) are immutable. Implementation tasks, PR titles, test names and ADRs MUST cite them.
5. **JSON examples** are illustrative; JSON Schemas and OpenAPI/AsyncAPI documents in the appendices are authoritative.

---

## 1. Product vision

**AI-Lab turns a raw idea into a maintained software product through a managed team of AI agents, and makes running that team feel like playing a 16-bit office sim.**

The product owns the complete delivery lifecycle:

```
IDEA → SOW → PLAN/WBS → KANBAN TASKS → EXECUTION (agents + tools)
     → REVIEW → MERGE → RELEASE → MAINTENANCE → LEARNING (CV + memory)
```

The user writes intent in natural language. The **Tech Lead** agent converts it into a canonical, estimable, assignable Kanban card. The **Analyst** produces and maintains the SOW and the work breakdown. **Dev** agents implement. **QA** agents verify. Every action is visible in real time in a pixel-art virtual office and in a Trello-like board. Every agent has a persistent identity, a "character", an evolving CV and three tiers of memory.

### 1.1 Differentiators versus `pixel-agents` and similar tools

| Dimension | pixel-agents (reference competitor) | AI-Lab (this product) |
|---|---|---|
| Scope | Visualizes external coding agents | **Owns the full lifecycle**: SOW → plan → tasks → code → review → merge → maintenance |
| Source of truth | External terminal sessions | **Postgres domain model**; UI is a projection, never the source |
| Orchestration | Observe + launch | **Scheduler, leases, budgets, retries, escalation, autonomous merge policy** |
| Agent identity | Ephemeral characters | **Persistent agents with role, character, evolving CV, memory** |
| Work management | None | **Agile Kanban with enforced invariants and DoR/DoD** |
| Knowledge | None | **Three-tier memory (lab/project/agent) with vector retrieval + Redis cache** |
| Repo integration | None | **GitHub: branches, PRs, checks, reviews, merge, webhooks** |
| UI | Canvas office | Canvas office **plus** dashboard, inventory, Kanban, memory explorer, all interlinked |

### 1.2 Success metrics (product-level)

| KPI | Target | Measurement |
|---|---|---|
| Idea → approved SOW | ≤ 10 min for a medium idea (≤ 800 words) | `sow.approved_at - idea.received_at` p95 |
| Idea → first merged PR | ≤ 60 min in demo scenario | end-to-end synthetic benchmark |
| Card cycle time (Ready → Done) | p50 ≤ 45 min, p95 ≤ 4 h | `card_events` |
| Kanban invariant violations | 0 per 10,000 transitions | invariant test harness |
| Office UI frame rate | ≥ 58 FPS at 1440p, ≥ 30 FPS on integrated GPU | Playwright + `requestAnimationFrame` probe |
| API latency (read) | p95 ≤ 120 ms, p99 ≤ 300 ms | Prometheus histogram |
| WebSocket event fan-out | p95 ≤ 250 ms from domain event to client render | trace spans |
| Crash-free sessions | ≥ 99.9% | frontend telemetry |
| Autonomous task success | ≥ 85% of Ready cards reach Done without human intervention (autonomy A2) | delivery ledger |
| Cost per completed card | ≤ USD 1.50 median on DeepSeek Flash | `llm_requests` ledger |

---

## 2. Scope

### 2.1 In scope (v1.0)

- `FR-INV`: Inventory of providers, models, agents, projects, labs.
- `FR-LAB`: Virtual lab with team composition and three-tier memory/knowledge base.
- `FR-KAN`: Agile Kanban with six columns and enforced semantics.
- `FR-SOW`: SOW generation, plan/WBS generation, task materialization.
- `FR-EXE`: Agent execution runtime with tools, sandbox, budgets, retries, escalation.
- `FR-GH`: GitHub integration (PAT-based in v1, App-ready adapter).
- `FR-MEM`: Persistent memory, CV evolution, retrieval, compaction.
- `FR-UI`: Dashboard, inventory, lab (pixel office), Kanban, memory explorer, settings.
- `FR-RT`: Real-time updates over WebSocket.
- `FR-MAINT`: Maintenance automations (dependency, health, stale work, memory hygiene).

### 2.2 Out of scope (v1.0) — explicitly excluded

- Multi-user authentication, RBAC, SSO, billing. (The product is a **single-operator, local-first** application. Network exposure is a deliberate operator action.)
- Non-DeepSeek LLM providers in the shipped UI (the provider port MUST exist; DeepSeek is the only adapter wired).
- Non-GitHub SCM providers (the SCM port MUST exist; GitHub is the only adapter wired).
- Mobile-native apps. The web UI MUST be responsive and usable on a tablet; phone support is read-mostly.
- On-premise Kubernetes operator. Docker Compose is the supported deployment.
- Agent-to-agent free-form chat beyond the structured handoff protocol defined here.
- Arbitrary code execution outside the sandbox.
- Payment, invoicing, time tracking.

### 2.3 Assumptions

- The operator has Docker Engine ≥ 27 and Docker Compose ≥ 2.30.
- The operator supplies a DeepSeek API key and a GitHub fine-grained PAT.
- Outbound HTTPS is available to `api.deepseek.com`, `api.github.com`, `github.com`.
- The host has ≥ 4 vCPU, ≥ 8 GB RAM, ≥ 20 GB free disk.
- English is the canonical language of generated artifacts, code, commits and UI copy. The UI MAY be localized later; v1 ships English only.

---

## 3. Personas and primary journeys

| Persona | Goal | Key surface |
|---|---|---|
| **Operator** (human, no login) | Turn ideas into shipped software; monitor and steer agents | Dashboard, Kanban, Lab |
| **Tech Lead** (agent, mandatory) | Capture intent, refine cards, plan, assign, unblock, merge | Kanban, orchestrator |
| **Analyst** (agent) | Write SOW, WBS, acceptance criteria, risks | SOW workspace |
| **Dev** (agent) | Implement cards in isolated branches; open PRs | Runner, GitHub |
| **QA** (agent) | Review code, run tests, approve or request changes | Review column, GitHub |
| **Observer** (human, optional) | Watch the office for delight and situational awareness | Lab (read-only) |

### 3.1 Journey J1 — Idea to backlog

1. Operator opens the app. Dashboard loads with no login (`UX-040`).
2. Operator types: "Build a CLI that converts CSV to Parquet with schema inference."
3. Tech Lead agent receives the prompt via `POST /api/v1/ideas`, enriches it with project context and memory, and creates one or more `TODO` cards using the canonical story template (`FR-KAN-010`).
4. The new cards animate into the Kanban and are announced in the office (speech bubble + activity feed).
5. Operator sees the cards within 3 s p95.

### 3.2 Journey J2 — Idea to SOW and plan

1. Operator creates a project, connects a GitHub repository, creates a lab, assigns a team.
2. Operator submits an idea to the lab and requests a SOW.
3. Analyst agent produces a structured SOW (JSON + Markdown) and a WBS with tasks, estimates, dependencies and milestones.
4. Operator approves (single click, no login). Cards are materialized into `TODO`.
5. Every SOW section is versioned and diffable.

### 3.3 Journey J3 — Autonomous execution

1. Tech Lead refines `TODO` → `READY` (assignee + due date + DoR satisfied).
2. Scheduler leases a `READY` card to the assignee.
3. Agent claims → `DOING`, creates a branch, implements, runs tests in a sandbox, opens a PR, appends a log entry, moves to `REVIEW`.
4. QA reviews the PR. On approval, Tech Lead merges (policy-dependent). Card → `DONE`.
5. CV and memory are updated by the reflection job. Office characters visibly celebrate.

### 3.4 Journey J4 — Watch and steer

1. Operator opens **Lab**. Pixel office renders agents at desks, walking, reading, blocked.
2. Clicking an agent opens a side panel: live status, current card, last 20 log lines, CV, memory snippets, controls (pause, resume, reassign, message).
3. Clicking the whiteboard opens the Kanban filtered to the project; clicking the server rack opens system health.
4. Operator drags a card between columns; invalid moves are rejected with an explicit reason (`FR-KAN-020`).

---

## 4. System context and containers

### 4.1 Context diagram

```
                         ┌──────────────────────────────────────────┐
                         │               OPERATOR (browser)          │
                         │  Dashboard · Lab · Kanban · Inventory     │
                         └───────────────────┬──────────────────────┘
                                             │ HTTPS + WSS
                    ┌────────────────────────▼─────────────────────────┐
                    │                 AI-Lab Web (Angular 22)          │
                    │  static SPA served by Nginx; reverse-proxies API │
                    └────────────────────────┬─────────────────────────┘
                                             │ /api/v1 (REST) · /api/v1/ws
          ┌──────────────────────────────────▼───────────────────────────────────┐
          │                        AI-Lab API (Rust / axum)                       │
          │  Auth · REST · WS hub · validation · OpenAPI · rate limiting          │
          └───────┬───────────────────┬───────────────────┬──────────────────────┘
                  │                   │                   │
        ┌─────────▼────────┐  ┌───────▼────────┐  ┌───────▼─────────┐
        │  PostgreSQL 18   │  │   Redis 8.8    │  │ Orchestrator    │
        │  + pgvector      │  │ cache · pubsub │  │ (Rust worker)   │
        │  system of record│  │ locks · rate   │  │ scheduler·runs  │
        └─────────┬────────┘  └───────┬────────┘  └───┬─────────┬───┘
                  │                   │               │         │
                  │                   │        ┌──────▼─────┐ ┌─▼──────────────┐
                  │                   │        │ DeepSeek   │ │ GitHub API     │
                  │                   │        │ API        │ │ (PAT / App)    │
                  │                   │        └────────────┘ └────────────────┘
                  │                   │
                  │           ┌───────▼──────────────────────────────┐
                  │           │ Sandbox Runner (ephemeral containers)│
                  │           │ git worktree · build · test          │
                  │           └──────────────────────────────────────┘
                  │
          ┌───────▼──────────────────────────────────────────────────────────┐
          │ Object storage (local volume): artifacts, SOW exports, logs,     │
          │ sprite packs, recordings. Addressed by content hash.            │
          └──────────────────────────────────────────────────────────────────┘
```

### 4.2 Container responsibilities

| Container | Tech | Responsibility | Scaling |
|---|---|---|---|
| `web` | Nginx 1.29-alpine + Angular 22 static build | Serve SPA, gzip/brotli, proxy `/api` and `/ws`, security headers | stateless, N replicas |
| `api` | Rust, axum 0.8, tokio 1.54 | REST, WS, auth, validation, projections, enqueue | stateless, N replicas |
| `worker` | Rust, tokio | Scheduler, agent runtime, tool execution, reflection, maintenance | N replicas; Postgres leases |
| `postgres` | `pgvector/pgvector:0.8.7-pg18-trixie` | System of record, vectors, outbox | single primary (v1), streaming replica optional |
| `redis` | `redis:8.8-trixie` | Cache, rate limit, distributed locks, pub/sub, presence, working memory | single instance (v1) |
| `runner` | Rust + Docker Engine API | Ephemeral sandboxed containers for build/test | N replicas, node-local |
| `migrate` | `sqlx-cli` | One-shot migrations at boot | run-once |

> **Decision:** Postgres is the only durable system of record. Redis is **never** authoritative for domain state. Object storage is a local volume in v1, abstracted behind a `BlobStore` port.

---

## 5. Technology baseline and version policy

### 5.1 Pinned baseline (v1.0 bootstrap)

| Layer | Choice | Version policy |
|---|---|---|
| Backend language | Rust | **1.99.x stable**, edition 2024, MSRV = toolchain in `.tool-versions` |
| Async runtime | Tokio | 1.54.x |
| HTTP | axum | 0.8.x |
| Middleware | tower / tower-http | 0.5.x / 0.7.x |
| DB access | sqlx (compile-time checked) | 0.9.x |
| Migrations | sqlx-cli | 0.9.x |
| Serialization | serde / serde_json | 1.0.x |
| OpenAPI | utoipa + utoipa-swagger-ui | 6.0.x |
| Redis client | redis | 1.8.x |
| Vector | pgvector crate | 0.4.x |
| Embeddings | fastembed-rs (ONNX, local) | latest stable; model `BAAI/bge-small-en-v1.5` (384 dim) |
| Telemetry | opentelemetry + tracing | 0.33.x / latest stable |
| Container API | bollard | latest stable |
| Frontend framework | Angular | **22.2.x** |
| Language | TypeScript | **7.0.x** |
| Build | Angular CLI / esbuild | 22.2.x |
| State | Angular Signals + RxJS interop | as shipped |
| Rendering | Canvas 2D (primary), WebGL2 (optional acceleration) | native |
| Styling | SCSS + CSS custom properties | native |
| DB | PostgreSQL | **18.x** |
| DB extension | pgvector | **0.8.7** |
| Cache | Redis | **8.8.x** |
| Web server | Nginx | 1.29-alpine |
| OS build images | Debian 13 "trixie" slim | pinned by digest |
| Test (Rust) | cargo-nextest, proptest, insta, wiremock | latest stable |
| Test (FE) | Vitest, Angular TestBed, Playwright | latest stable |
| Load | k6 | latest stable |
| Containers | Docker Engine ≥ 27, Compose ≥ 2.30 | operator-provided |

### 5.2 Version policy (binding)

- `VER-001`: Every dependency MUST be pinned by lockfile. Container images MUST be pinned by tag **and** digest in production compose.
- `VER-002`: No EOL runtime, framework or database line. No deprecated APIs when a stable replacement exists.
- `VER-003`: Automated dependency updates (Renovate or equivalent) MUST run weekly; security patches within 48 h.
- `VER-004`: Major upgrades require an ADR, a full green pipeline and a rollback plan.
- `VER-005`: Angular MUST use standalone components, signals, `provideZonelessChangeDetection()` (zoneless), `@if/@for/@switch` control flow, and the current router. No NgModules for new code.
- `VER-006`: Rust MUST compile with `#![forbid(unsafe_code)]` in all first-party crates. Any third-party `unsafe` is reviewed and documented.
- `VER-007`: SQL MUST be compile-time checked (`sqlx::query!` / `query_as!`) or covered by an offline `.sqlx` cache in CI.
- `VER-008`: TypeScript `strict: true`, `noUncheckedIndexedAccess: true`, `exactOptionalPropertyTypes: true`. No `any` in first-party code except at validated parsing boundaries, where it MUST be narrowed immediately.

### 5.3 Deliberate exclusions

- No ORM (Diesel/SeaORM). Raw SQL with sqlx gives control and performance.
- No GraphQL. REST + WebSocket is simpler to secure and version.
- No message broker (Kafka/RabbitMQ). Postgres outbox + `SKIP LOCKED` + Redis pub/sub is sufficient for v1 and removes operational burden.
- No client-side database. The Angular app is a projection with a normalized store.

---

## 6. Domain model

The domain is split into five bounded contexts. Each context owns its tables and publishes domain events. Cross-context reads go through application services, never direct table joins from another context's repository.

```
┌─────────────┐   ┌─────────────┐   ┌─────────────┐   ┌─────────────┐   ┌─────────────┐
│  INVENTORY  │   │     LAB     │   │  KNOWLEDGE  │   │   KANBAN    │   │  DELIVERY   │
│ providers   │   │ labs/teams  │   │ memory/KB   │   │ boards/cards│   │ sow/plan/run│
│ models      │◄──┤ membership  │──►│ vectors     │◄──┤ transitions │──►│ execution   │
│ agents/CV   │   │ settings    │   │ CV evidence │   │ logs        │   │ merge       │
│ projects    │   └─────────────┘   └─────────────┘   └─────────────┘   └─────────────┘
└─────────────┘                                                                  │
        ▲                                                                         │
        └───────────────────────────── domain events ─────────────────────────────┘
```

### 6.1 Inventory context (`INV`)

#### 6.1.1 Provider

`FR-INV-001`: A **Provider** is a configured LLM vendor endpoint. Fields: `id`, `name`, `kind` (`deepseek` in v1), `base_url`, `api_key_ciphertext`, `api_key_last4`, `status` (`active|disabled|degraded`), `rate_limit_rpm`, `rate_limit_tpm`, `timeout_ms`, `created_at`, `updated_at`.

- `FR-INV-002`: API keys MUST be encrypted at rest with AES-256-GCM using a master key supplied via `AILAB_MASTER_KEY` (32 bytes, base64). Keys MUST never be returned by any API after creation; only `last4` is readable.
- `FR-INV-003`: Provider health is probed every 60 s with a 1-token completion; failures drive `degraded` status and expose `GET /api/v1/health/providers`.
- `FR-INV-004`: Rate limits are enforced by a Redis token bucket per provider and per lab. Exhaustion MUST queue, not fail.

#### 6.1.2 Model

`FR-INV-010`: A **Model** belongs to a provider and declares: `id`, `provider_id`, `name` (API model identifier, e.g. `deepseek-flash`, `deepseek-v4-pro`), `display_name`, `context_window`, `max_output_tokens`, `supports_tools`, `supports_reasoning`, `supports_vision`, `input_cost_per_mtok`, `output_cost_per_mtok`, `default_temperature`, `default_reasoning_effort`, `status`.

- `FR-INV-011`: v1 MUST seed exactly two models: `deepseek-flash` (default for Analyst, Dev, QA, Tech Lead) and `deepseek-v4-pro` (escalation only, see `FR-LLM-003`).
- `FR-INV-012`: Cost figures are configuration, not hardcoded; every LLM request records `input_tokens`, `output_tokens`, `cached_tokens`, `cost_usd` computed from the model row at request time.

#### 6.1.3 Agent

`FR-INV-020`: An **Agent** is a persistent employee of a lab. Fields: `id`, `name` (unique within lab), `role`, `character_id`, `lab_id`, `description` (detailed, human-readable capability statement), `cv` (current version reference), `model_preference`, `autonomy_level`, `status` (`idle|working|blocked|paused|offline`), `max_concurrent_cards` (default 1), `avatar_seed`, `created_at`, `updated_at`.

`FR-INV-021`: Roles are fixed in v1: `tech_lead` (**exactly one active per lab, mandatory**), `analyst`, `dev`, `qa`. Additional dev/qa/analyst agents are allowed. Creating a lab without a Tech Lead MUST fail validation.

`FR-INV-022`: Each agent MUST have exactly one character from the finite roster in Appendix C.1. The character is a visual/behavioral persona only; it MUST NOT affect role permissions.

`FR-INV-023`: The **CV** is a versioned structured document (JSONB) containing: `skills[]` (`name`, `category`, `proficiency` 1–5, `evidence[]` (`card_id`, `run_id`, `summary`, `at`), `last_used_at`), `delivered_cards`, `review_stats`, `failure_modes[]`, `preferred_tools[]`, `languages[]`, `frameworks[]`, `notes`. Every completed card MUST append evidence. CV history is append-only; "current" is the highest version.

`FR-INV-024`: The agent **description** is generated at creation from role charter + character + model, and is editable. It MUST be 120–600 characters and describe what the agent can do, not who it "is".

`FR-INV-025`: Agent lifecycle: `created → active → paused → retired`. Retired agents keep CV and history, cannot receive new cards, and remain visible in the office as an empty desk with a plaque.

#### 6.1.4 Project

`FR-INV-030`: A **Project** has: `id`, `name`, `slug` (unique), `description`, `lab_id`, `source_kind` (`github`), `source_ref` (repo owner/name), `default_branch` (default `main`), `credential_id`, `status` (`active|archived`), `settings` (JSONB), timestamps.

- `FR-INV-031`: GitHub credentials reference a `SCM credential` row, not raw tokens in project rows.
- `FR-INV-032`: A project MAY be linked to multiple labs? **No.** v1: one project belongs to one lab; one lab may own many projects. Moving a project between labs requires an audited transfer operation.

### 6.2 Lab context (`LAB`)

`FR-LAB-001`: A **Lab** is the top-level workspace: `id`, `name`, `slug`, `description`, `settings`, `autonomy_level` (`A0|A1|A2|A3`), `monthly_budget_usd`, `created_at`.

`FR-LAB-002`: A lab owns: exactly one Tech Lead, zero or more Analyst/Dev/QA agents, zero or more projects, and three-tier memory/knowledge bases.

`FR-LAB-003`: Lab settings (JSONB, validated by JSON Schema):
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

`FR-LAB-004`: **Autonomy levels**:
- `A0` — Human approves every transition and every merge.
- `A1` — Agents may move cards up to `REVIEW`; human approves merges.
- `A2` — Agents may merge when QA approves, CI is green, no P0/P1 issues, within budget. **Default.**
- `A3` — Agents may additionally create projects, choose dependencies and release patch versions within budget. Requires explicit lab opt-in and a hard monthly budget.

### 6.3 Knowledge & Memory context (`MEM`)

`FR-MEM-001`: Memory exists at exactly three scopes: **lab**, **project**, **agent**. There is no global scope in v1.

`FR-MEM-002`: Each scope has a **knowledge base** composed of **documents** and **chunks**:
- A document is Markdown/plain text with `id`, `scope`, `scope_id`, `kind` (`sow|plan|adr|note|transcript|artifact|policy|skill`), `title`, `content`, `content_hash`, `source_ref`, `created_by`, `version`, timestamps.
- A chunk is a contiguous slice (target 800 tokens, 15% overlap) with `embedding vector(384)`, `document_id`, `ordinal`, `text`, `token_count`, `metadata`.
- Retrieval uses **hybrid search**: pgvector HNSW cosine (weight 0.7) + Postgres `tsvector` BM25-ish rank (weight 0.3), with recency decay and scope boost. Top-k default 8; reranking is deterministic (no extra LLM call) in v1.

`FR-MEM-003`: Memory tiers:
- **Episodic** — events and run traces (append-only, retention 180 days, compacted summaries retained).
- **Semantic** — facts, decisions, domain knowledge (documents + vectors).
- **Procedural** — agent CV, tool preferences, learned playbooks.

`FR-MEM-004`: Redis is the **working memory and cache**: last N messages per run (N=40), retrieval cache (TTL 300 s), agent presence, rate-limit buckets, locks. Cache keys are namespaced `ailab:{env}:{scope}:{id}:{kind}`.

`FR-MEM-005`: Every retrieval MUST be logged with query, scopes, chunk IDs and scores for audit and evaluation. PII/secret scrubbing runs before embedding and before sending to the LLM.

`FR-MEM-006`: Memory compaction runs daily: episodes older than 30 days are summarized into semantic documents; raw episodes older than 180 days are deleted unless pinned.

`FR-MEM-007`: Cross-scope leakage is a **P0 security defect**. Retrieval MUST be scope-filtered at the SQL level, never in application post-processing alone.

### 6.4 Agile Kanban context (`KAN`)

`FR-KAN-001`: Every project has exactly one board with exactly six columns, ordered: `TODO`, `READY`, `DOING`, `REVIEW`, `DONE`, `ICEBOX`. Column semantics are fixed; names are not configurable in v1.

`FR-KAN-002`: A card is the unit of work. Required fields: `id`, `key` (`{PROJECT_KEY}-{seq}`), `title`, `story` (canonical user story), `deliverables[]`, `acceptance_criteria[]` (Gherkin), `priority` (`must|should|could|wont`), `estimate` (Fibonacci 1,2,3,5,8,13), `labels[]`, `column`, `assignee_id`, `reviewer_id`, `due_date`, `blocked_reason`, `source` (`human|agent`), `created_by_agent_id`, `parent_id`, `sow_id`, `budget_usd`, `risk`, `version`, timestamps.

`FR-KAN-003`: Cards are the **only** way work is executed. Agents MUST NOT perform project work outside a card.

`FR-KAN-004`: Every transition writes an immutable `card_event` with `from`, `to`, `actor_type`, `actor_id`, `reason`, `metadata`, `at`, `seq`. The card's `column` is a projection of the latest event.

`FR-KAN-005`: All mutations are optimistic-concurrency controlled by `version`; stale writes return `409 CONFLICT` with the current card.

`FR-KAN-006`: Card keys are allocated per project via a Postgres sequence and are never reused.

#### 6.4.1 Column invariants

| Column | Entry invariants (all MUST hold) | Exit rules |
|---|---|---|
| `TODO` | Title present. Story present or explicitly flagged `needs_refinement`. Source recorded. | To `READY` only after DoR; to `ICEBOX` with reason. |
| `READY` | Canonical story complete; ≥ 1 acceptance criterion; ≥ 1 deliverable; estimate set; priority set; `assignee_id` set; `due_date` set and in the future; DoR satisfied; dependencies resolved or waived. | To `DOING` when leased/claimed; to `ICEBOX` with reason; back to `TODO` if DoR broken (audited). |
| `DOING` | Exactly one assignee; due date; run (attempt) exists or is being created; log non-empty within 15 min of entry. | To `REVIEW` only when PR opened, self-checks green, log has summary and decisions; to `ICEBOX` with reason; to `READY` on explicit abandon. |
| `REVIEW` | `reviewer_id` set and ≠ assignee; PR link present; CI status known. | To `DONE` only when approved + merged + CI green + DoD satisfied; to `DOING` on changes requested; to `ICEBOX` with reason. |
| `DONE` | Merge commit SHA recorded; DoD satisfied; QA approval recorded. | Terminal. Reopen creates a **new** card linked to the old one; old card keeps audit trail. |
| `ICEBOX` | `blocked_reason` non-empty and categorized (`dependency|clarification|external|risk|budget|policy`). | To `TODO` when reason resolved (reason resolution recorded); to `READY` only via `TODO` re-refinement. |

`FR-KAN-020`: Invalid transitions MUST be rejected with `422 INVALID_TRANSITION` and a machine-readable `violations[]` array. The UI MUST surface the first violation verbatim.

`FR-KAN-021`: WIP limits: `DOING` ≤ `max_concurrent_cards` per agent (default 1); `REVIEW` ≤ 3 per QA agent. Exceeding a limit blocks the transition with `WIP_LIMIT_EXCEEDED`.

`FR-KAN-022`: Due dates are UTC timestamps; the UI renders local time. Overdue cards get `overdue: true`, a red badge and an `agent.speech` nudge; they are not automatically moved.

`FR-KAN-023`: Cards created by agents MUST include `source=agent` and a rationale in the log. Agents may create only `TODO` cards; they may never create `READY` directly.

#### 6.4.2 Canonical user story

`FR-KAN-010`: The canonical form is:
```
Title: <imperative, ≤ 80 chars>

As a <persona>,
I want <capability>,
so that <business outcome>.

Context:
<2–6 sentences of context, links, constraints>

Acceptance criteria:
Given <precondition>
When <action>
Then <observable outcome>
And <observable outcome>

Deliverables:
- <artifact path or named artifact> — <what "done" looks like>
- ...

Out of scope:
- <explicit exclusion>

Risks / dependencies:
- <risk> → <mitigation>
```
`FR-KAN-011`: The Tech Lead MUST refine every human idea into one or more cards in this form before moving to `READY`. Refinement may be iterative; `TODO` cards MAY be incomplete.

`FR-KAN-012`: Acceptance criteria MUST be executable or observable. "Works well" is invalid. Each criterion MUST map to at least one automated or manual test in the PR.

`FR-KAN-013`: Deliverables MUST be concrete: file paths, endpoints, migrations, UI routes, docs, tests, charts. They drive the DoD checklist.

`FR-KAN-014`: Estimates use Fibonacci; 13 means "must split". Cards > 8 SHOULD be split by the Tech Lead before `READY`.

#### 6.4.3 Definition of Ready (DoR)

A card is `READY` iff: canonical story complete; ≥1 acceptance criterion; ≥1 deliverable; estimate ∈ {1,2,3,5,8}; priority set; assignee set; due date set; dependencies listed and either resolved or explicitly waived by Tech Lead with reason; no `needs_refinement` flag; budget > 0.

#### 6.4.4 Definition of Done (DoD)

A card is `DONE` iff: PR merged to the project default branch; all required CI checks green; test coverage for new code ≥ 80% (or justified waiver); acceptance criteria demonstrably met; docs updated (README/ADR/API as applicable); changelog entry; no P0/P1 defects; QA approval recorded; card log contains final summary and decisions; merge SHA stored; CV evidence appended.

### 6.5 Delivery context (`DLV`)

`FR-DLV-001`: **Idea** → **SOW** → **Plan/WBS** → **Cards** → **Runs** → **Deliveries** is a first-class pipeline with persisted artifacts and versions.

`FR-DLV-002`: A **SOW** (Statement of Work) is a versioned structured document with mandatory sections: `executive_summary`, `objectives`, `scope_in`, `scope_out`, `stakeholders`, `functional_requirements[]`, `non_functional_requirements[]`, `architecture`, `milestones[]`, `deliverables[]`, `acceptance_criteria[]`, `assumptions[]`, `risks[]`, `timeline`, `budget`, `governance`, `change_control`. It MUST be rendered to Markdown and stored as an artifact.

`FR-DLV-003`: A **Plan** (WBS) is a DAG of tasks with: `id`, `title`, `description`, `estimate`, `depends_on[]`, `role`, `milestone`, `deliverables[]`, `acceptance_criteria[]`. The DAG MUST be acyclic; cycles are rejected at validation.

`FR-DLV-004`: Materialization converts WBS leaves into Kanban cards in `TODO` (or `READY` if DoR is already satisfied and the operator enabled auto-ready), preserving parent links.

`FR-DLV-005`: A **Run** is one attempt by one agent to execute one card. Fields: `id`, `card_id`, `agent_id`, `attempt`, `status` (`queued|running|succeeded|failed|cancelled|timed_out`), `started_at`, `ended_at`, `lease_id`, `sandbox_id`, `tokens_in`, `tokens_out`, `cost_usd`, `error`, `result_summary`.

`FR-DLV-006`: A **RunStep** is an ordered, typed record within a run: `plan`, `tool_call`, `llm_request`, `command`, `test`, `commit`, `pr`, `review`, `decision`, `roadblock`, `handoff`. Steps are streamed to the UI.

`FR-DLV-007`: A **Decision** is an explicit, reversible-or-not record with rationale, alternatives, and links. Decisions from runs are promoted to project memory as ADR candidates.

---

## 7. Agent operating model

### 7.1 Role charters

| Role | Mandatory | Owns | May do | Must not do |
|---|---|---|---|---|
| `tech_lead` | 1 per lab | Backlog quality, assignment, unblocking, merge policy, escalation | Refine cards, set due dates, reassign, merge, create `TODO`, pause agents | Write production code except trivial fixes; approve its own code |
| `analyst` | 0..n | SOW, WBS, requirements, acceptance criteria, risk register | Create `TODO`, edit SOW/plan, request clarification | Merge; execute code changes |
| `dev` | 0..n | Implementation, tests, PR, self-review | Claim `READY`, write code/tests/docs, open PR, move to `REVIEW` | Merge; approve own PR; edit SOW |
| `qa` | 0..n | Independent verification, review, regression | Claim `REVIEW`, run tests, approve/request changes, file defects as `TODO` | Merge; modify the implementation under review (except test-only branches) |

`FR-AGT-001`: Separation of duties is enforced server-side: the reviewer of a card MUST NOT be its assignee. The merger MUST NOT be the PR author. Violations return `422 SEPARATION_OF_DUTIES`.

### 7.2 Agent anatomy

Every agent is composed of six layers:

1. **Charter** — immutable role boundaries and output contract (Appendix C.2).
2. **Character** — persona, tone, visual archetype, speech style. Affects communication only, never permissions.
3. **CV** — evolving structured skills and evidence, injected as a compact summary.
4. **Memory** — retrieved lab/project/agent knowledge, scoped and cited.
5. **Task packet** — the card, its story, DoR/DoD, dependencies, recent events, repository map.
6. **Tool belt** — the exact tool schemas the agent may call, filtered by role and run policy.

`FR-AGT-002`: The system prompt is **assembled deterministically** from these layers with a fixed section order and explicit delimiters. It MUST be reproducible from persisted inputs and stored hash (`prompt_hash`) with each LLM request.

`FR-AGT-003`: Character text is capped at 400 tokens and MUST NOT override role rules. If a conflict is detected, role rules win and the conflict is logged.

### 7.3 Agent state machine

```
OFFLINE ──start──► IDLE ──lease──► WORKING ──blocked──► BLOCKED ──resolved──► WORKING
   ▲                 │                 │                                      │
   │                 │ pause           │ complete/fail                        │ pause
   │                 ▼                 ▼                                      ▼
   └────────────── PAUSED ◄────────────┘◄─────────────────────────────────────┘
```

- `FR-AGT-010`: `WORKING` requires an active run and a lease. A worker crash MUST release the lease after TTL (default 900 s) and mark the run `timed_out`.
- `FR-AGT-011`: Heartbeats every 15 s extend the lease. Three missed heartbeats cancel the run and requeue with backoff.
- `FR-AGT-012`: State changes emit `agent.status_changed` and update office animation within 250 ms p95.

### 7.4 Autonomy and guardrails

`FR-AGT-020`: Each run carries an effective autonomy level = min(lab level, card level, agent level).

`FR-AGT-021`: Hard stops (always enforced, regardless of autonomy): budget exceeded; secret detected in diff; forbidden path touched (`.github/workflows`, `CODEOWNERS`, production credentials); force-push; destructive SQL; dependency with critical CVE; request to disable tests/CI; PR > 1,500 changed lines or > 40 files (requires human confirmation).

`FR-AGT-022`: Agents MUST NOT modify their own charter, CV, autonomy, budgets, or the scheduler configuration. CV updates are performed by a separate reflection job.

`FR-AGT-023`: Every agent action that mutates the repository MUST be traceable to `run_id` via commit trailers:
```
AI-Lab-Run: <run_uuid>
AI-Lab-Card: <CARD-KEY>
AI-Lab-Agent: <agent_uuid>
```

### 7.5 CV evolution (reflection)

`FR-AGT-030`: After every run, a reflection job (model `deepseek-flash`, temperature 0.2, JSON output) produces a `ReflectionReport`:
```json
{
  "card_id": "uuid",
  "outcome": "success|partial|failure",
  "skills_demonstrated": [{"name": "rust", "delta": 1, "evidence": "..."}],
  "skills_missing": ["...", "..."],
  "failure_modes": ["..."],
  "lessons": ["..."],
  "memory_candidates": [{"scope": "project", "kind": "note", "title": "...", "content": "..."}]
}
```
`FR-AGT-031`: Deltas are bounded (−1..+1 per run, proficiency clamped 1..5). CV versions are immutable snapshots; `agent.cv_version` increments atomically.

`FR-AGT-032`: Memory candidates are written only if they pass deduplication (cosine ≥ 0.92 against existing chunks) and secret scanning.

---

## 8. Orchestration engine

### 8.1 Scheduling model

- `FR-ORC-001`: The scheduler ticks every 1,000 ms per worker. Each tick: expire leases → reconcile runs → enqueue eligible cards → dispatch up to `WORKER_CONCURRENCY` runs.
- `FR-ORC-002`: Eligibility: card in `READY`, assignee active and not paused, agent has capacity, lab budget available, no open blocker, provider healthy, dependencies satisfied.
- `FR-ORC-003`: Leasing uses `SELECT ... FOR UPDATE SKIP LOCKED` in a short transaction, writing a `run` row and a lease row. Leases are unique per card; a second lease attempt returns "already leased".
- `FR-ORC-004`: Retries: max 3 per card (configurable). Backoff 30 s → 2 min → 10 min with ±20% jitter. Exhaustion moves the card to `ICEBOX` with `blocked_reason.category=risk` and notifies the Tech Lead.
- `FR-ORC-005`: Failures are classified: `transient` (network/rate limit/timeout) → retry; `deterministic` (bad code/test failure) → return to `DOING` with feedback, count attempt; `policy` (guardrail) → `ICEBOX` + escalation; `budget` → pause lab and escalate.
- `FR-ORC-006`: Dead-letter runs are visible in `GET /api/v1/labs/{id}/runs?status=failed` and in the office as a red filing cabinet.

### 8.2 Run pipeline

```
LEASE → CONTEXT BUILD → PLAN → [LLM ⇄ TOOL]* → VERIFY → COMMIT → PUSH → PR
      → TRANSITION(REVIEW) → QA REVIEW → MERGE POLICY → DONE → REFLECTION
```

- `FR-ORC-010`: `CONTEXT BUILD` assembles: repo map (tree + language stats + recent commits), card, memory retrieval, relevant files (max 120 KB / 30k tokens initial), conventions (AGENTS.md, README, linters), and previous review feedback.
- `FR-ORC-011`: The agent loop is bounded: max 40 tool calls, max 25 iterations, max wall clock 45 min per run, max output tokens per response 8,192.
- `FR-ORC-012`: The agent MUST produce a plan before mutating files. The plan is stored as a `RunStep(plan)` and streamed to the UI.
- `FR-ORC-013`: Before commit, `VERIFY` runs formatter, linter, type check, unit tests and secret scan in the sandbox. Failures loop back to the agent with the raw output (truncated to 20 KB).
- `FR-ORC-014`: Commits MUST be conventional (`feat(scope): ...`, `fix`, `chore`, `docs`, `test`, `refactor`, `perf`, `build`, `ci`). PR description MUST use the template in Appendix C.4 and link the card.

### 8.3 Handoffs and escalation

- `FR-ORC-020`: Agent-to-agent handoff is a structured `Handoff` object: `from`, `to`, `card_id`, `summary`, `state`, `open_questions[]`, `artifacts[]`, `acceptance_status`. Free-form chat is not a handoff.
- `FR-ORC-021`: Escalation ladder: agent → Tech Lead → human operator (UI notification + optional webhook). Escalation MUST include the last 20 log lines, the failing command, and a proposed next action.
- `FR-ORC-022`: Human interventions are recorded as `Decision` rows with `actor_type=human` and MUST be visible to the next agent.

### 8.4 Idempotency and crash safety

- `FR-ORC-030`: All external side effects (GitHub PR, comment, merge, webhook) use idempotency keys derived from `run_id + operation + target`.
- `FR-ORC-031`: On worker restart, in-flight runs are reconciled: if a PR exists and is merged, the card moves to `DONE`; if a branch exists, the run resumes; otherwise it retries.
- `FR-ORC-032`: Outbox rows are published at-least-once. Consumers are idempotent by `event_id`. The UI deduplicates by monotonic `seq`.

---

## 9. LLM provider layer (DeepSeek)

### 9.1 Port and adapters

`FR-LLM-001`: Define a provider-agnostic port:
```rust
#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse, LlmError>;
    async fn stream(&self, req: CompletionRequest) -> Result<CompletionStream, LlmError>;
    async fn health(&self) -> Result<ProviderHealth, LlmError>;
    fn capabilities(&self) -> ModelCapabilities;
}
```
`FR-LLM-002`: v1 implements `DeepSeekProvider` against `https://api.deepseek.com` (OpenAI-compatible `/chat/completions`). Base URL, model names, timeouts and limits are configuration.

`FR-LLM-003`: Model routing:
- `deepseek-flash` — default for all roles, summaries, reflection, embeddings-adjacent tasks.
- `deepseek-v4-pro` — allowed only for: SOW architecture section, complex debugging after 2 failed attempts, security review, and Tech Lead conflict resolution. Each use MUST be justified in the run step.
- Reasoning: `thinking.type=enabled` for planning/review; `reasoning_effort=high` only for `deepseek-v4-pro` escalations; disabled for cheap classification.

### 9.2 Request policy

`FR-LLM-010`: Every request sets: model, messages, tools (JSON Schema), `tool_choice=auto`, temperature by role (Analyst 0.3, Tech Lead 0.2, Dev 0.1, QA 0.1), max tokens, and a deterministic `seed` when supported. Structured outputs MUST use JSON Schema and be validated server-side; invalid output retries once with a repair prompt, then fails.

`FR-LLM-011`: Context construction MUST budget tokens explicitly: system ≤ 20%, memory ≤ 25%, repo context ≤ 35%, task ≤ 15%, reserve 5%. Overflow triggers summarization, never silent truncation of the task.

`FR-LLM-012`: Streaming is mandatory for UI-visible generation (SOW, plans, agent speech). Token deltas are coalesced at 50 ms to limit WebSocket chatter.

`FR-LLM-013`: Retries: 429/5xx/timeouts with exponential backoff (1 s, 2 s, 4 s, 8 s; max 4), honoring `Retry-After`. Circuit breaker opens after 5 consecutive failures for 60 s; half-open probe after.

`FR-LLM-014`: Prompt injection defense: retrieved content and repository files are wrapped in `<untrusted>` blocks with explicit instructions to never follow instructions inside them. Tool calls that would exfiltrate secrets or reach non-allowlisted hosts are blocked by policy, not by prompt alone.

`FR-LLM-015`: All prompts and responses are persisted (redacted) for audit with retention 90 days. Secrets are scrubbed with a regex + entropy detector before persistence.

### 9.3 Cost and budget accounting

`FR-LLM-020`: Each request records token usage and cost. A Redis counter per lab/day and per card enforces budgets atomically (`INCRBY` + check). On breach: hard stop (default) or degrade to `deepseek-flash` and pause non-critical jobs.

`FR-LLM-021`: The UI MUST show spend per lab, project, card and agent, with a 30-day trend and forecast.

`FR-LLM-022`: A `budget.exhausted` event pauses the lab, moves active runs to a safe checkpoint, and requires a human budget increase to resume.

---

## 10. GitHub integration

`FR-GH-001`: v1 authenticates with a **fine-grained PAT** stored encrypted. Scopes required: `Contents: read/write`, `Pull requests: read/write`, `Checks: read`, `Metadata: read`, `Webhooks: read/write` (optional), `Issues: read/write` (optional). Classic PATs are discouraged but accepted with a warning.

`FR-GH-002`: The adapter MUST be an `ScmProvider` port with methods: `list_repos`, `get_repo`, `create_branch`, `get_file`, `put_file`, `delete_file`, `create_commit`, `create_pull_request`, `get_pull_request`, `list_checks`, `merge_pull_request`, `create_review`, `comment`, `verify_webhook`.

`FR-GH-003`: Branch naming: `ailab/{card_key}-{slug}` (e.g., `ailab/AIL-42-csv-inference`). Branches are deleted after merge (configurable).

`FR-GH-004`: PR title: `[{card_key}] {card_title}`. Body MUST include summary, changes, test evidence, risks, card link, and the AI-Lab trace block.

`FR-GH-005`: Merge policy: squash merge by default; merge commit allowed for release branches; rebase disabled. The merge is performed only after required checks pass and the reviewer is not the author.

`FR-GH-006`: Webhooks (optional but recommended): `pull_request`, `check_run`, `push`. Signature verification (`X-Hub-Signature-256`) is mandatory. Webhook events reconcile state and emit domain events. Polling fallback runs every 60 s for connected repos when webhooks are unavailable.

`FR-GH-007`: Rate-limit handling: respect `X-RateLimit-Remaining`; queue and back off; never exceed 80% of the limit. Secondary rate limits trigger a 60 s cooldown.

`FR-GH-008`: Repository safety: refuse to operate on archived, disabled, or empty repos; refuse to force-push; refuse to write to protected branches; detect and respect branch protection.

---

## 11. Virtual Office — "Pixel" UI (`UX`)

### 11.1 Experience principles

1. **Game first, dashboard second.** The office is the emotional core; information density is progressive.
2. **Everything is clickable.** Agents, desks, whiteboard, server rack, bookshelf, filing cabinet, plants, pets.
3. **State is truthful.** Animations derive from real backend events, never from random idle loops alone.
4. **60 FPS or it ships broken.** Pixel-perfect, integer-scaled, no anti-aliasing.
5. **Instant feedback.** Every click has a visual response within 100 ms.
6. **Accessible by construction.** Reduced motion, keyboard navigation, screen-reader summaries, list fallback.

### 11.2 Visual language

- **Style:** 16-bit, Amiga 500 / OCS-inspired office. Chunky pixels, limited palette, dithered gradients, beveled UI chrome, CRT-adjacent but not blurry.
- **Internal resolution:** `384 × 216` logical pixels (16:9), integer-scaled by `floor(min(vw/384, vh/216))`. Never fractional scaling.
- **Palette:** 32 colors, fixed. Core ramp: deep navy `#0B1026`, midnight `#141B3C`, steel `#2E3A6B`, periwinkle `#5B6FB0`, ice `#9FB3E6`, cream `#F2E9D8`, amber `#F2B84B`, orange `#E07A3F`, crimson `#C0392B`, teal `#2A9D8F`, green `#57C785`, magenta `#C45AA0`. Colorblind-safe accents for status.
- **Typography:** pixel bitmap font (e.g., a licensed 8×8/8×12 face) for in-canvas text; UI outside canvas uses a crisp geometric sans with `font-smooth: never` only inside sprites. No emoji in the office.
- **Assets:** sprite sheets, 32×32 tiles, 16×24 or 24×32 characters, 8-direction walk (or 4-direction with mirroring), 4-frame walk cycle, 2-frame idle, 4-frame work loop. All assets have a manifest (Appendix C.5).
- **Sound:** optional chimes on card completion, merge, and blocked state. Master volume, mute, and "respect `prefers-reduced-motion`/autoplay" defaults.

### 11.3 Office layout

```
┌──────────────────────────────────────────────────────────────────────┐
│  WINDOW STRIP · lab name · clock · status LEDs · mute · fullscreen   │
├──────────────────────────────────────────────────────────────────────┤
│  ┌──────────┐  ┌──────────┐  ┌──────────┐        ┌───────────────┐   │
│  │ DEV DESK │  │ DEV DESK │  │ QA DESK  │        │  WHITEBOARD   │   │
│  │  agent   │  │  agent   │  │  agent   │        │  → Kanban     │   │
│  └──────────┘  └──────────┘  └──────────┘        └───────────────┘   │
│                                                                      │
│  ┌──────────┐   ┌────────────────────┐   ┌──────────┐  ┌─────────┐  │
│  │ TL DESK  │   │   MEETING TABLE    │   │ SERVER   │  │BOOKSHELF│  │
│  │  agent   │   │  (handoffs)        │   │ RACK     │  │→ Memory │  │
│  └──────────┘   └────────────────────┘   └──────────┘  └─────────┘  │
│                                                                      │
│  ┌──────────┐   ┌──────────┐   ┌──────────┐   ┌──────────────────┐  │
│  │ COFFEE   │   │ PLANT    │   │ FILING   │   │  DOOR / EXIT     │  │
│  │ (idle)   │   │ (decor)  │   │ (failed) │   │  (retired agents)│  │
│  └──────────┘   └──────────┘   └──────────┘   └──────────────────┘  │
└──────────────────────────────────────────────────────────────────────┘
```

- `UX-001`: The office MUST render every active agent of the selected lab. Agents without a desk (e.g., > 8 agents) sit at hot desks or stand; the layout MUST scale to 16 agents without overlap.
- `UX-002`: Each agent has a persistent desk. Desk plaque shows name, role icon, and status LED (`idle` teal, `working` amber, `blocked` crimson, `paused` gray, `offline` dark).
- `UX-003`: Agents pathfind around furniture using A* on the tile grid with a walkability map. No teleporting except on spawn/crash recovery.
- `UX-004`: Animation states: `IDLE` (subtle breathing), `WALK` (8-direction), `TYPE` (at desk, working code), `READ` (reading, searching), `THINK` (hand on chin, thinking), `TALK` (speech bubble), `BLOCKED` (pacing or red "!" bubble), `CELEBRATE` (on merge), `OFFLINE` (empty chair).
- `UX-005`: Speech bubbles show short, role-appropriate messages (≤ 120 chars), streamed from real events. Bubbles auto-dismiss after 6 s; click opens the detail panel.
- `UX-006`: Ambient life is allowed only when it does not contradict state: coffee trips when idle, plants sway, fish tank bubbles. Ambient actions MUST yield to work actions within 1 tick.

### 11.4 Interaction model

- `UX-010`: Click agent → right-side **Agent Inspector** (slide-in, 420 px, resizable) with tabs: **Live** (status, current card, last 20 steps), **CV** (skills with bars, evidence timeline), **Memory** (recent retrievals, pinned notes), **Controls** (pause/resume, reassign card, message, set autonomy, retire).
- `UX-011`: Click desk → same inspector, plus desk metadata.
- `UX-012`: Click whiteboard → Kanban overlay filtered to the project; drawer keeps office visible (split view).
- `UX-013`: Click server rack → System Health drawer: provider status, DB/Redis latency, queue depth, error rate, budget.
- `UX-014`: Click bookshelf → Memory explorer drawer: scope switcher, search, recent documents, "add note".
- `UX-015`: Click filing cabinet → Failed runs / dead letters with replay action.
- `UX-016`: Hovering any interactive object shows a pixel tooltip + outline. Keyboard: `Tab` cycles interactive objects, `Enter` opens, `Esc` closes, arrow keys pan the camera.
- `UX-017`: Camera: fixed full-room view by default; optional pan/zoom with wheel and drag when the room exceeds the viewport. Zoom snaps to integer factors (1×, 2×, 3×, 4×). A "fit" button resets.
- `UX-018`: Deep links: `#/lab/{labId}?agent={agentId}&panel=cv` opens the office focused on an agent. Every object exposes a shareable URL.

### 11.5 Rendering architecture

- `UX-020`: Single `<canvas>` at internal resolution, upscaled with `image-rendering: pixelated`. Layers (composited in order): floor → walls → furniture behind → characters (y-sorted) → furniture in front → effects → UI overlays (DOM, not canvas).
- `UX-021`: Simulation is decoupled from rendering: fixed timestep 60 Hz (accumulator, max 3 catch-up steps), render on `requestAnimationFrame`. Background tab pauses simulation but keeps state sync via WebSocket.
- `UX-022`: Sprite atlas: one texture atlas per category (characters, furniture, tiles, FX) with JSON frames. Draw calls batched by atlas. Target ≤ 150 draw calls/frame.
- `UX-023`: Offscreen canvases cache static layers (floor/walls/furniture). Only dynamic layers redraw each frame.
- `UX-024`: Device pixel ratio is respected by scaling the CSS size, never the internal buffer. No blurry upscaling.
- `UX-025`: Memory: peak JS heap ≤ 150 MB; sprite atlas ≤ 8 MB compressed; no per-frame allocations in the hot loop (object pools for particles and bubbles).
- `UX-026`: WebGL2 is an optional backend behind a capability flag; Canvas 2D is the reference and MUST be pixel-identical.
`UX-027`: Asset provenance is mandatory. Every sprite sheet, font, sound and tileset MUST have a redistributable license recorded in `docs/office.md` (source URL, license, author). Assets from the reference product MUST NOT be copied. Original art is produced for AI-Lab or sourced from a permissively licensed pack (CC0 / OFL / commercial license with redistribution rights).

### 11.6 Responsive behavior

- `UX-030`: ≥ 1280 px: sidebar expanded (240 px) + office + optional inspector.
- `UX-031`: 768–1279 px: sidebar collapsed to icons (64 px); inspector overlays.
- `UX-032`: < 768 px: sidebar becomes a bottom sheet; office scales to fit width; inspector becomes a full-screen sheet; Kanban becomes single-column swipe.
- `UX-033`: Orientation changes reflow without reload. Touch targets ≥ 44×44 CSS px.
- `UX-034`: `prefers-reduced-motion`: disable walking animation (instant reposition with fade), ambient motion, and parallax; keep state changes and speech text.
- `UX-035`: Screen-reader mode: a hidden but focusable "Office list" provides an ordered list of agents with status, current card and actions; live region announces transitions.

### 11.7 Dashboard

- `UX-040`: Route `/` shows: lab selector; KPI strip (active agents, cards by column, spend today, success rate); live office preview (small, non-interactive except click-to-enter); activity feed (last 50 events); "New idea" composer; recent SOWs; system health pill.
- `UX-041`: No login. First run shows an onboarding checklist: create provider → add key → create lab → add Tech Lead → connect GitHub → create project → submit idea.
- `UX-042`: Empty states are illustrated in pixel art with a single clear CTA. No dead ends.

### 11.8 Kanban UI

- `UX-050`: Six columns, horizontal scroll on narrow screens, drag-and-drop with keyboard alternative (`M` to move, arrow keys, `Enter`).
- `UX-051`: Card front: key, title, assignee avatar (pixel), priority stripe, estimate chip, due-date badge, labels, blocked icon.
- `UX-052`: Card back: story rendering, acceptance criteria checklist, deliverables, activity log, comments, links (PR, SOW), agent run summary, "why is this blocked" panel.
- `UX-053`: Invalid drop animates back and shows a toast with the exact violation.
- `UX-054`: Filters: assignee, label, priority, overdue, source (human/agent), text search. Filters persist in the URL.
- `UX-055`: Real-time: another client's move animates in ≤ 300 ms. Optimistic UI with rollback on `409/422`.
- `UX-056`: Trello-like simplicity: no swimlanes, no custom fields, no automation builder in v1. Exactly the six columns and the defined card schema.

### 11.9 Inventory and SOW UI

- `UX-060`: Inventory routes: `/inventory/agents`, `/inventory/projects`, `/inventory/providers`. Tables with search, filters, create/edit drawers, and a "view in office" action.
- `UX-061`: Agent creation wizard: role → character (visual picker with animated preview) → name → model → autonomy → review. Validation inline.
- `UX-062`: SOW workspace: `/projects/{id}/sow` with version selector, rendered Markdown, section navigation, diff view between versions, approve button, export (MD/PDF).
- `UX-063`: Plan view: WBS tree + dependency graph (DAG), milestone timeline, "materialize to Kanban" action with preview of cards to create.

---

## 12. Frontend architecture (Angular 22)

### 12.1 Application structure

```
web/
├── src/
│   ├── app/
│   │   ├── core/                 # singletons: api client, ws, auth-less config, error handler, telemetry
│   │   ├── shared/               # pixel UI kit, directives, pipes, icons
│   │   ├── domain/               # typed models generated from OpenAPI + hand-written view models
│   │   ├── features/
│   │   │   ├── dashboard/
│   │   │   ├── office/           # canvas engine, sprites, pathfinding, inspector
│   │   │   ├── kanban/
│   │   │   ├── inventory/
│   │   │   ├── memory/
│   │   │   ├── sow/
│   │   │   └── settings/
│   │   ├── layout/               # shell, sidebar, topbar, drawers
│   │   └── app.routes.ts
│   ├── styles/                   # tokens, reset, pixel utilities
│   └── main.ts
├── e2e/
└── public/assets/sprites/
```

- `FE-001`: Standalone components only. Route-level lazy loading via `loadComponent`/`loadChildren`. No `NgModule`.
- `FE-002`: Zoneless change detection. All state via signals; RxJS only at boundaries (WebSocket, HTTP) with `toSignal`.
- `FE-003`: A typed `ApiClient` generated from OpenAPI (`openapi-typescript`) plus a thin wrapper for errors, retries and correlation IDs. No manual DTO duplication.
- `FE-004`: Global state is organized in feature stores (signal-based, e.g. `signalStore` pattern) with explicit `load/refresh/mutate` actions. No global mutable singletons except `WsService`, `ApiClient`, `Telemetry`.
- `FE-005`: The WebSocket service maintains a single connection, exponential reconnect (1→30 s), heartbeat ping every 20 s, and a resumable cursor (`last_seq`). On reconnect it calls `GET /api/v1/events?lab_id={labId}&after={seq}&limit=1000` to backfill.
- `FE-006`: The office engine is framework-agnostic TypeScript (`OfficeEngine`) with no Angular imports; Angular hosts it and feeds it a signal-derived snapshot. This keeps the hot loop testable in isolation.
- `FE-007`: Rendering MUST NOT trigger Angular change detection per frame. The engine writes to canvas directly; UI overlays update at most 4 Hz or on discrete events.
- `FE-008`: Bundle budgets: initial JS ≤ 350 KB gzip, office chunk ≤ 180 KB gzip, total CSS ≤ 60 KB gzip. Lighthouse performance ≥ 90 on desktop.
- `FE-009`: Strict CSP: no inline scripts, no `eval`, no third-party scripts. `connect-src 'self'`. Assets self-hosted.
- `FE-010`: Accessibility: WCAG 2.2 AA for all DOM UI; canvas has an accessible fallback list; focus visible; reduced motion honored; contrast ≥ 4.5:1 for text.
- `FE-011`: Error handling: global `ErrorHandler` reports to `/api/v1/telemetry/frontend` with correlation ID, route, build hash, and sanitized stack. User sees a pixel-styled toast + retry, never a blank screen.
- `FE-012`: The app MUST work offline-read-only for the last loaded snapshot (service worker optional in v1; if implemented, cache shell + last snapshot, never mutations).
- `FE-013`: All dates rendered with `Intl.DateTimeFormat` in the operator's timezone; the API always returns UTC ISO-8601 with `Z`.

### 12.2 Office engine internals

- `FE-020`: Entity model: `OfficeEntity { id, kind, tile, sprite, state, z, hitbox, meta }`. Characters are entities with `AgentStateMachine`.
- `FE-021`: Pathfinding: A* on a 64×64 max grid, 8-directional with corner-cutting prevention, binary heap, max 4,096 expansions per query. Paths are cached per agent and invalidated on layout change.
- `FE-022`: State machine transitions are driven by server events (`agent.status_changed`, `run.step`, `card.moved`). Local prediction is allowed for movement only, with server reconciliation.
- `FE-023`: Animation: frame timing in ticks (60/s), no wall-clock drift. Sprite variants chosen by direction and state. Speech bubble text rendered on canvas with word wrap at 28 chars/line, max 3 lines.
- `FE-024`: Hit testing: pixel-accurate via per-sprite hitboxes in tile coordinates, plus a DOM overlay of invisible buttons for accessibility and testability (`data-testid="agent-{id}"`).
- `FE-025`: Determinism: given the same event sequence and seed, the office reaches the same positions. Used for visual regression tests.
- `FE-026`: Asset loading: manifest → parallel fetch with progress → decode into atlases → start simulation only when all critical atlases are ready. A pixel loading screen with progress bar is mandatory.

### 12.3 Design system

- `FE-030`: Tokens: color, spacing (4 px base), radii (0/2/4 px only), shadows (hard pixel offsets), z-index scale, motion durations (80/160/240 ms; no easing curves except `steps()` for pixel effects).
- `FE-031`: Components: `PixelButton`, `PixelPanel`, `PixelTabs`, `PixelBadge`, `PixelTable`, `PixelDrawer`, `PixelToast`, `PixelModal`, `PixelInput`, `PixelSelect`, `PixelProgress`, `PixelAvatar`, `PixelTooltip`. Each has states (default/hover/active/disabled/loading/error) and keyboard behavior.
- `FE-032`: Icons: 16×16 pixel icon set, single color + accent, no antialiasing, exported as SVG with `shape-rendering="crispEdges"`.
- `FE-033`: No rounded "SaaS" look. Bevels, 1 px borders, hard shadows, scanline texture only at ≤ 4% opacity (and off under reduced motion).
- `FE-034`: Copy tone: concise, direct, slightly playful in the office, strictly precise in operational panels. No exclamation marks in errors.

---

## 13. Backend architecture (Rust)

### 13.1 Workspace layout

```
crates/
├── domain/            # entities, value objects, invariants, domain events (no I/O)
├── application/       # use cases, ports, transactions, authorization
├── infrastructure/    # sqlx repos, redis, blob store, clock, idgen
├── llm/               # LlmProvider port + DeepSeek adapter + prompt assembly
├── scm/               # ScmProvider port + GitHub adapter
├── memory/            # embeddings, chunking, retrieval, compaction
├── orchestrator/      # scheduler, leases, run pipeline, sandbox client
├── api/               # axum router, handlers, DTOs, OpenAPI, WS hub
├── worker/            # binary: orchestrator + maintenance jobs
├── telemetry/         # tracing, metrics, OTel setup
├── testkit/           # fixtures, fakes, testcontainers helpers
└── cli/               # `ailab` admin CLI (seed, migrate, doctor, replay)
```

- `BE-001`: Dependency rule: `domain` depends on nothing but `std` + small value crates; `application` depends on `domain`; adapters depend on ports; `api`/`worker` are composition roots. No adapter types leak into domain.
- `BE-002`: Domain invariants are enforced in constructors and methods returning `Result<_, DomainError>`. Invalid states are unrepresentable where practical (newtypes for IDs, validated emails/URLs/slugs).
- `BE-003`: All IDs are UUIDv7 (time-ordered) generated application-side. Human-facing keys are separate (`AIL-42`).
- `BE-004`: Timestamps are `TIMESTAMPTZ` in Postgres, `chrono::DateTime<Utc>` in Rust, ISO-8601 in JSON.
- `BE-005`: Money is integer micro-USD (`i64`) to avoid float drift; display formatting is frontend-only.
- `BE-006`: Errors implement `std::error::Error`; API maps them to RFC 9457 Problem Details. No panics in request paths; `catch_unwind` at the worker boundary; `panic=abort` is forbidden.
- `BE-007`: Every use case is transactional where it mutates more than one aggregate. Domain events are written to the outbox in the same transaction.
- `BE-008`: Concurrency: optimistic locking via `version`; pessimistic `SELECT ... FOR UPDATE` only for leases and sequence allocation. Deadlocks are retried up to 3 times with jitter.
- `BE-009`: Background work is idempotent and observable; every job has a name, timeout, and structured outcome.

### 13.2 API layer

- `BE-010`: axum with typed extractors, `tower-http` middleware: request ID, tracing, CORS (same-origin by default), compression, timeout (30 s default; 120 s for SOW generation), body limit (2 MB default; 10 MB for imports).
- `BE-011`: Validation at the boundary with `validator`/JSON Schema; reject unknown fields (`deny_unknown_fields`) on write DTOs.
- `BE-012`: Pagination: cursor-based (`?limit=50&cursor=...`), max 200. Responses include `next_cursor`. No offset pagination for large tables.
- `BE-013`: Filtering and sorting are allowlisted per endpoint; no dynamic SQL string interpolation.
- `BE-014`: `Idempotency-Key` required for all POST that create resources with external side effects; replay within 24 h returns the original response.
- `BE-015`: OpenAPI 3.1 served at `/api/v1/openapi.json`, Swagger UI at `/api/v1/docs` (disabled in production by default via config).
- `BE-016`: WebSocket at `/api/v1/ws` with a subscribe protocol (`{"type":"subscribe","topics":["lab:uuid","card:uuid"]}`), server ack, and per-connection backpressure (drop-oldest for ephemeral typing events, never drop state events).
- `BE-017`: Health: `/health/live` (process), `/health/ready` (DB + Redis + migrations), `/health/providers` (LLM), `/health/scm` (GitHub). Detailed health is authenticated when exposed remotely.

### 13.3 Observability hooks

- `BE-020`: Structured logs (JSON) with `trace_id`, `span_id`, `lab_id`, `project_id`, `card_id`, `agent_id`, `run_id`, `request_id`. No secrets, no full prompts in logs.
- `BE-021`: Metrics (Prometheus): HTTP duration/count by route/status; DB pool; Redis latency; queue depth; lease age; run duration; LLM tokens/cost/latency; provider errors; WS connections; event lag; budget remaining.
- `BE-022`: Traces (OTLP): API request → use case → DB → outbox → worker → LLM → GitHub, linked by `trace_id`. Sampling: 100% for errors, 10% baseline, 100% for run pipelines (configurable).

---

## 14. Data architecture

### 14.1 PostgreSQL

- `DB-001`: Postgres 18, UTF-8, `timestamptz`, `uuid` PKs, `jsonb` only for genuinely schemaless payloads (validated by JSON Schema at write time). Relational data stays relational.
- `DB-002`: Extensions: `pgvector`, `pgcrypto` (gen_random_uuid), `pg_trgm` (search), `citext` (slugs/emails if added).
- `DB-003`: Every table has `created_at`, `updated_at` (trigger-maintained), and `version` where mutable. Soft delete only for agents/projects (`retired_at`, `archived_at`); hard delete for transient rows.
- `DB-004`: Foreign keys enforced with explicit `ON DELETE` behavior. No orphan rows.
- `DB-005`: Indexes: every FK; every filter/sort column used by the API; partial indexes for hot statuses; GIN on `jsonb` only where queried; HNSW on embeddings (`m=16`, `ef_construction=64`); GIN `tsvector` for full-text.
- `DB-006`: Migrations are forward-only, numbered, idempotent where possible, and reviewed. Destructive changes require a two-step expand/contract migration.
- `DB-007`: Row-level scoping by `workspace_id`/`lab_id` is enforced in application repositories and verified by tests; v1 is single-tenant but the column exists for forward compatibility.
- `DB-008`: Connection pool: `max_connections = 20 × api replicas + 10 × worker replicas`, `acquire_timeout=5s`, `idle_timeout=10m`, `max_lifetime=30m`. PgBouncer is out of scope in v1.
- `DB-009`: Statement timeout: 5 s for API reads, 30 s for heavy reports, 120 s for SOW persistence jobs. Long analytics run in the worker.
- `DB-010`: Backups: `pg_dump` daily + WAL archiving optional; retention 14 days locally; restore drill monthly.

### 14.2 Redis

- `DB-020`: Usage only for: retrieval/response cache, working memory (run context), rate limiting, distributed locks (leases use Postgres, not Redis), presence, pub/sub fan-out, and ephemeral counters. TTLs on **every** key; no unbounded keyspace.
- `DB-021`: Key schema: `ailab:{env}:{domain}:{id}:{kind}`. Values are JSON or msgpack with a version field.
- `DB-022`: Pub/sub channels: `ailab:{env}:events` (domain event fan-out), `ailab:{env}:presence`. Pub/sub is lossy; durable history is Postgres.
- `DB-023`: Eviction policy `allkeys-lru`, maxmemory configured by operator (default 512 MB). Cache misses must never break correctness.
- `DB-024`: Locks: `SET key value NX PX ttl` with owner token and Lua-based safe release. Lock TTL always < operation timeout.

### 14.3 Outbox and events

- `DB-030`: `outbox` rows: `id`, `aggregate_type`, `aggregate_id`, `event_type`, `payload jsonb`, `headers jsonb`, `created_at`, `published_at`, `attempts`, `last_error`.
- `DB-031`: A relay (worker) polls unpublished rows in order per aggregate, publishes to Redis and WebSocket, and marks published. At-least-once; consumers deduplicate by `event_id`.
- `DB-032`: Domain event naming: `{context}.{aggregate}.{past_tense}` (e.g., `kanban.card.moved`, `delivery.run.completed`, `inventory.agent.cv_updated`).
- `DB-033`: Every event has `event_id`, `occurred_at`, `seq` (per lab monotonic), `lab_id`, `actor`, `payload`, `schema_version`.

---

## 15. API and real-time contracts

### 15.1 REST surface (summary)

| Method | Path | Purpose |
|---|---|---|
| GET | `/api/v1/health/*` | liveness, readiness, provider/SCM health |
| GET/POST | `/api/v1/labs` | list/create labs |
| GET/PATCH/DELETE | `/api/v1/labs/{id}` | read/update/archive lab |
| GET/POST | `/api/v1/labs/{id}/agents` | list/add agents |
| PATCH/DELETE | `/api/v1/agents/{id}` | update/retire agent |
| POST | `/api/v1/agents/{id}/pause` `/resume` | control |
| GET | `/api/v1/agents/{id}/cv` | current + history |
| GET/POST | `/api/v1/providers` | provider CRUD (key write-only) |
| POST | `/api/v1/providers/{id}/verify` | probe |
| GET | `/api/v1/models` | model catalog |
| GET/POST | `/api/v1/projects` | projects |
| POST | `/api/v1/projects/{id}/sync` | reconcile GitHub |
| POST | `/api/v1/ideas` | submit natural-language idea |
| GET/POST | `/api/v1/projects/{id}/sows` | list/create SOW |
| GET | `/api/v1/sows/{id}` (`/versions`, `/diff`) | read/diff |
| POST | `/api/v1/sows/{id}/approve` | approve + materialize |
| GET/POST | `/api/v1/projects/{id}/plans` | plans |
| POST | `/api/v1/plans/{id}/materialize` | create cards |
| GET | `/api/v1/projects/{id}/board` | board with cards |
| POST | `/api/v1/projects/{id}/cards` | create card |
| GET/PATCH | `/api/v1/cards/{id}` | read/update |
| POST | `/api/v1/cards/{id}/transition` | move column (validated) |
| POST | `/api/v1/cards/{id}/comments` | comment |
| GET | `/api/v1/cards/{id}/events` | audit log |
| POST | `/api/v1/cards/{id}/assign` | assign/reassign |
| POST | `/api/v1/cards/{id}/runs` | manual run/retry |
| GET | `/api/v1/runs/{id}` (`/steps`) | run detail |
| GET | `/api/v1/labs/{id}/runs` | run list |
| GET/POST | `/api/v1/memory/documents` | documents |
| GET | `/api/v1/memory/search` | hybrid retrieval |
| POST | `/api/v1/memory/notes` | quick note |
| GET | `/api/v1/events` | event backfill (`after` cursor) |
| GET | `/api/v1/labs/{id}/metrics` | KPIs, spend, trends |
| POST | `/api/v1/telemetry/frontend` | frontend errors |
| GET | `/api/v1/settings` / PATCH | global settings |

- `API-001`: All responses are JSON (`application/json; charset=utf-8`) except artifacts (`text/markdown`, `application/pdf`) and event streams.
- `API-002`: Errors use RFC 9457:
```json
{
  "type": "https://ai-lab.dev/errors/invalid-transition",
  "title": "Invalid Kanban transition",
  "status": 422,
  "detail": "Card AIL-42 cannot move from TODO to DONE.",
  "instance": "/api/v1/cards/019.../transition",
  "code": "INVALID_TRANSITION",
  "violations": [
    {"rule": "DOR_INCOMPLETE", "field": "acceptance_criteria", "message": "At least one acceptance criterion is required."}
  ],
  "trace_id": "..."
}
```
- `API-003`: Correlation: server returns `X-Request-Id` (echoing client value if valid) and `traceparent` compatibility.
- `API-004`: Versioning: URL major version `/api/v1`. Additive changes are non-breaking; removals require `/v2`.
- `API-005`: CORS: same-origin by default. If `AILAB_ALLOWED_ORIGINS` is set, exact-match allowlist, credentials false, no wildcard with credentials.
- `API-006`: Rate limiting: 120 req/min per IP for reads, 30 req/min for mutations, 5 req/min for idea/SOW generation. `429` includes `Retry-After`.
- `API-007`: Request size: JSON ≤ 2 MB; memory import ≤ 10 MB; artifacts streamed separately.
- `API-008`: All list endpoints support `?limit&cursor&sort&filter[...]`, returning `{"data": [...], "next_cursor": "...", "total_estimate": 123}`.

### 15.2 WebSocket

- `WS-001`: Connect: `GET /api/v1/ws?since={seq}` (upgrade). Server sends `hello` with `server_time`, `seq`, `topics`.
- `WS-002`: Client sends `subscribe`/`unsubscribe` with topic patterns: `lab:{id}`, `card:{id}`, `run:{id}`, `system`. Server authorizes topics (single-tenant, but validates existence).
- `WS-003`: Server message envelope:
```json
{
  "type": "event",
  "event": "kanban.card.moved",
  "event_id": "uuid",
  "seq": 10422,
  "lab_id": "uuid",
  "occurred_at": "2026-11-07T10:00:00Z",
  "payload": { "...": "..." }
}
```
- `WS-004`: Ephemeral events (`agent.typing`, `office.cursor`) are coalesced and never persisted. State events are persisted and replayable.
- `WS-005`: Backpressure: per-connection buffer 256 messages; when full, drop ephemeral first, then close with code `1013` and let the client resume from `last_seq`.
- `WS-006`: Heartbeat: server ping every 20 s; client must pong within 10 s; otherwise close.
- `WS-007`: Reconnect: client exponential backoff with jitter, max 30 s; on reconnect uses `?since=` and then `GET /api/v1/events?after=` if the gap exceeds buffer.
- `WS-008`: `system.health` event every 15 s with provider, DB, Redis, queue depth and budget summary.

### 15.3 Event catalog (core)

| Event | Producer | Key payload |
|---|---|---|
| `inventory.agent.created` | INV | `agent_id`, `role`, `character_id` |
| `inventory.agent.cv_updated` | INV | `agent_id`, `version`, `skills_delta` |
| `lab.agent.status_changed` | LAB | `agent_id`, `from`, `to`, `reason` |
| `kanban.card.created` | KAN | `card_id`, `key`, `column`, `source` |
| `kanban.card.moved` | KAN | `card_id`, `from`, `to`, `actor`, `reason` |
| `kanban.card.blocked` | KAN | `card_id`, `category`, `reason` |
| `kanban.card.updated` | KAN | `card_id`, `changed_fields` |
| `delivery.run.queued` | DLV | `run_id`, `card_id`, `agent_id` |
| `delivery.run.started` | DLV | `run_id`, `lease_id` |
| `delivery.run.step` | DLV | `run_id`, `step_type`, `summary` |
| `delivery.run.completed` | DLV | `run_id`, `status`, `cost_usd` |
| `delivery.pr.opened` | DLV | `card_id`, `pr_url`, `branch` |
| `delivery.review.submitted` | DLV | `card_id`, `reviewer_id`, `verdict` |
| `delivery.merge.completed` | DLV | `card_id`, `sha`, `merged_by` |
| `memory.document.created` | MEM | `scope`, `scope_id`, `document_id` |
| `memory.compaction.completed` | MEM | `scope`, `documents_created`, `episodes_removed` |
| `system.budget.exhausted` | SYS | `lab_id`, `scope`, `limit_usd`, `spent_usd` |
| `system.health.changed` | SYS | `component`, `status`, `detail` |

Full AsyncAPI schema: Appendix B.2.

---

## 16. Security and privacy

### 16.1 Threat model (STRIDE, abbreviated)

| Threat | Vector | Control |
|---|---|---|
| Spoofing | Forged webhooks | HMAC verification (`X-Hub-Signature-256`), replay window 5 min |
| Tampering | Prompt injection via repo/memory | `<untrusted>` wrapping, tool policy, diff scanning, no secret access |
| Repudiation | Agent actions | Immutable `card_events`, `run_steps`, audit log, commit trailers |
| Information disclosure | API keys/prompts | AES-256-GCM at rest, redaction, no keys in logs, CSP |
| Denial of service | Runaway loops / cost | Bounded iterations, timeouts, budgets, circuit breaker, rate limits |
| Elevation of privilege | Agent edits policy/CI | Forbidden paths, server-side policy, separation of duties |
| Supply chain | Malicious dependency | Lockfiles, `cargo-deny`, `npm audit`, SBOM, pinned digests |
| Code execution | Agent shell | Ephemeral sandbox, non-root, read-only root FS, network allowlist, no host mounts |
| Data exfiltration | Tool calls to unknown hosts | Egress allowlist, DNS pinning, no raw sockets |
| Secret leakage in diff | Hardcoded keys | Pre-commit secret scan (gitleaks-like), block + escalate |

### 16.2 Authentication and exposure

- `SEC-001`: The product is **login-less by design** for the local operator. It MUST bind to `127.0.0.1` by default.
- `SEC-002`: If `AILAB_BIND=0.0.0.0` or a remote host is configured, the server MUST require a bearer token (`AILAB_API_TOKEN`, ≥ 32 bytes) for all `/api` and `/ws` requests except `/health/live`. The UI prompts once and stores the token in memory + `sessionStorage` (never `localStorage`).
- `SEC-003`: TLS is terminated by a reverse proxy (Caddy/Traefik/Nginx) supplied in an optional compose profile. The app never generates self-signed certs silently.
- `SEC-004`: CSRF: same-origin + `SameSite=Strict` cookies are not used; bearer tokens in headers for remote mode. Mutations require `Content-Type: application/json` and an `Origin` check.
- `SEC-005`: Security headers: `Content-Security-Policy`, `X-Content-Type-Options: nosniff`, `Referrer-Policy: no-referrer`, `Permissions-Policy` minimal, `Cross-Origin-Opener-Policy: same-origin`, HSTS when TLS.
- `SEC-006`: Secrets at rest encrypted with `AILAB_MASTER_KEY`; key rotation supported via envelope encryption (`key_version` column + re-encrypt job).
- `SEC-007`: Audit log for: provider key create/rotate/delete, agent create/retire, autonomy/budget changes, merges, project connect/disconnect, manual overrides, memory deletion, settings changes. Audit rows are append-only and exportable.

### 16.3 Sandbox specification

- `SEC-010`: Every run executes in an ephemeral container:
  - image: `ailab/runner:<version>` (non-root UID 10001, no shell in production image except the controlled runner entrypoint);
  - root filesystem read-only; workspace mounted at `/workspace` (tmpfs or volume, size-limited 2 GB);
  - CPU 2, memory 4 GB, pids 512, no privileged, `cap_drop: ALL`, `no-new-privileges`;
  - network: `none` by default for build/test; an explicit `egress` profile with an allowlist (package registries) when dependency installation is required;
  - timeout 45 min; kill on lease loss; logs streamed and capped at 20 MB.
- `SEC-011`: The runner never mounts the Docker socket into the workload container. The orchestrator talks to the Docker API from a dedicated runner service with a narrow, audited API surface.
- `SEC-012`: Files written outside `/workspace` are discarded. Host paths are never mounted.
- `SEC-013`: The agent's shell tool accepts an allowlist of commands/binaries (git, cargo, npm/pnpm, node, python, pytest, make, just, rg, ls, cat, sed, jq). Arbitrary `curl | sh` is blocked. Package installation requires the `egress` policy and is logged.
- `SEC-014`: No production credentials are ever available inside the sandbox. Only a short-lived, repo-scoped GitHub token is injected when needed, with minimal permissions.

### 16.4 Privacy

- `SEC-020`: Telemetry is local by default (Postgres + logs). No third-party analytics.
- `SEC-021`: Prompt/response retention is configurable (default 90 days); deletion is honored within 24 h including embeddings.
- `SEC-022`: The operator can export all data (JSON + artifacts) and delete a lab with a typed confirmation. Deletion is asynchronous and audited.

---

## 17. Performance and capacity

### 17.1 Budgets (binding)

| Area | Budget | Test |
|---|---|---|
| API read p95 / p99 | 120 ms / 300 ms | k6 + Prometheus |
| API write p95 / p99 | 250 ms / 600 ms | k6 |
| Board load (500 cards) | p95 ≤ 400 ms | k6 + Playwright |
| WS event → UI render | p95 ≤ 250 ms | tracing + Playwright |
| Office FPS | ≥ 58 desktop, ≥ 30 low-end | RAF probe |
| Initial load (cold, 4G) | LCP ≤ 2.5 s, TTI ≤ 3.5 s | Lighthouse CI |
| Idea → Todo cards | p95 ≤ 3 s | synthetic |
| SOW medium idea | p95 ≤ 10 min | synthetic |
| Run overhead (non-LLM) | ≤ 2 s per run | metrics |
| DB query p95 | ≤ 50 ms | pg_stat_statements |
| Memory per API replica | ≤ 256 MB RSS steady | cAdvisor |

### 17.2 Capacity model (v1 targets)

- 1 lab, 10 agents, 5 projects, 2,000 cards, 100k events, 50k memory chunks.
- 20 concurrent runs cluster-wide (configurable); 4 runs per worker by default.
- 100 concurrent WebSocket clients; 500 events/s fan-out burst.
- Postgres ≤ 20 GB for the above; Redis ≤ 512 MB.

### 17.3 Load and soak

- `PERF-001`: k6 scenarios: browse (read-heavy), board drag (write), idea burst, WS fan-out, memory search. Thresholds enforced in CI nightly.
- `PERF-002`: 60-minute soak at 2× expected load with no memory growth > 10% and no error rate > 0.1%.
- `PERF-003`: Office stress test with 16 agents + 200 animated props at 60 FPS.

---

## 18. Reliability and disaster recovery

- `REL-001`: API and worker are stateless; restart is safe. In-flight runs reconcile from persisted state (`FR-ORC-031`).
- `REL-002`: Readiness fails if migrations are pending, DB is unreachable, or Redis is unreachable (Redis is required for rate limiting/locks; a degraded read-only mode MAY be enabled by config).
- `REL-003`: Graceful shutdown: stop accepting requests, drain ≤ 20 s, finish or checkpoint runs, release leases, flush telemetry.
- `REL-004`: RPO ≤ 24 h (daily backup) or ≤ 5 min with WAL archiving (recommended). RTO ≤ 1 h. A `make restore-drill` command MUST exist.
- `REL-005`: Upgrades: rolling for stateless services; migrations are backward-compatible for one version (expand/contract).
- `REL-006`: Data integrity job nightly: FK orphan checks, invariant checks (card column vs latest event, run status vs steps), and a repair report (no automatic destructive repair).
- `REL-007`: Clock skew: all timestamps server-generated; clients never authoritative.

---

## 19. Testing strategy

| Layer | Tool | Scope | Gate |
|---|---|---|---|
| Domain unit | `cargo test` + proptest | invariants, state machine, parsers | 100% of transitions, mutation score ≥ 70% |
| Application | `cargo test` + fakes | use cases, transactions, authorization | critical paths |
| Repository | `sqlx::test` + testcontainers | SQL, constraints, indexes, RLS | all repos |
| Contract | OpenAPI validation + schemathesis | request/response conformance | CI |
| LLM adapter | `wiremock` | retries, streaming, errors, budgets | CI |
| SCM adapter | `wiremock` | PR lifecycle, rate limits, webhooks | CI |
| Orchestrator | integration + chaos | leases, crash recovery, retries, dead letter | nightly |
| Frontend unit | Vitest + TestBed | stores, guards, pipes, components | ≥ 80% |
| Office engine | Vitest | pathfinding, state machine, determinism | golden tests |
| E2E | Playwright | journeys J1–J4, a11y, responsive | every PR |
| Visual | Playwright screenshots | office + Kanban + dashboard | threshold 0.1% |
| Load | k6 | §17 | nightly |
| Security | `cargo-audit`, `cargo-deny`, `npm audit`, gitleaks, Trivy | deps, secrets, images | every PR |
| Fuzz | `cargo-fuzz` | JSON/event parsers, prompt assembly | weekly |
| Chaos | kill workers/DB/Redis | recovery | weekly |

- `TEST-001`: The Kanban invariant suite MUST exhaustively test all 36 ordered column pairs and assert allowed/denied outcomes against Appendix C.3.
- `TEST-002`: A deterministic **demo seed** MUST exist (`make seed-demo`) producing a lab, 4 agents, a project, a SOW, 12 cards across columns, and 30 events. E2E and screenshots use it.
- `TEST-003`: No test may call the real DeepSeek or GitHub APIs. Fakes and recorded fixtures only. A separate, opt-in `make smoke-live` is allowed for maintainers.
- `TEST-004`: Flaky tests are quarantined within 24 h and fixed or deleted within 5 days. A flaky test blocks release.
- `TEST-005`: Coverage is measured with `cargo-llvm-cov` and Vitest coverage; thresholds: backend line ≥ 80%, frontend line ≥ 80%, domain ≥ 90%.

---

## 20. DevOps, containers and configuration

### 20.1 Repository layout (monorepo)

```
ai-lab/
├── Cargo.toml                 # workspace
├── rust-toolchain.toml
├── .tool-versions
├── crates/                    # §13.1
├── web/                       # Angular app
├── infra/
│   ├── docker/
│   │   ├── api.Dockerfile
│   │   ├── worker.Dockerfile
│   │   ├── runner.Dockerfile
│   │   ├── web.Dockerfile
│   │   └── nginx.conf
│   ├── compose/
│   │   ├── docker-compose.yml
│   │   ├── docker-compose.dev.yml
│   │   └── docker-compose.observability.yml
│   └── grafana/ prometheus/ otel/
├── migrations/                # sqlx
├── docs/
│   ├── adr/
│   ├── diagrams/
│   ├── runbooks/
│   └── api/
├── scripts/
├── .github/workflows/
├── Makefile
└── README.md
```

### 20.2 Docker requirements

- `OPS-001`: Multi-stage builds. Build images are pinned by digest. Runtime images are distroless or Debian slim with no package manager, no shell (except runner), non-root user.
- `OPS-002`: Rust builds use `cargo-chef` layer caching; final binary stripped, LTO thin, `panic=abort` **not** used (unwinding preserved for graceful errors), `codegen-units=1` for release.
- `OPS-003`: Image size budgets: api ≤ 120 MB, worker ≤ 140 MB, web ≤ 40 MB, runner ≤ 600 MB (toolchains included).
- `OPS-004`: Every image has `HEALTHCHECK`, OCI labels (source, revision, version, created), and a signed SBOM (Syft) published as an artifact.
- `OPS-005`: Compose profiles: `core` (db, redis, api, worker, web), `runner`, `observability`, `tls`. `make up` starts core + runner; `make up-obs` adds Prometheus/Grafana/OTel collector.
- `OPS-006`: Volumes: `pgdata`, `redisdata`, `artifacts`, `workspaces`, `caddy_data`. Named volumes only; no bind mounts in production.
- `OPS-007`: Resource limits in compose for every service (`cpus`, `mem_limit`, `pids_limit`). No service may be unconstrained.
- `OPS-008`: Logging: Docker json-file with rotation (`max-size=10m`, `max-file=3`) or OTLP. No unbounded logs.
- `OPS-009`: One-command bootstrap: `make up` → migrations → seed provider placeholders → UI at `http://localhost:8080`. `make doctor` verifies Docker, ports, keys, disk, connectivity.

### 20.3 Configuration

All configuration is environment variables with the `AILAB_` prefix, validated at boot; the process MUST fail fast with a clear message on invalid config. Secrets are never logged. A complete table is in Appendix D.4. Core variables:

```
AILAB_ENV=production|development
AILAB_BIND=127.0.0.1
AILAB_PORT=8080
AILAB_DATABASE_URL=postgres://...
AILAB_REDIS_URL=redis://...
AILAB_MASTER_KEY=<base64 32 bytes>
AILAB_API_TOKEN=<optional, required if bind != loopback>
AILAB_LOG_LEVEL=info
AILAB_OTEL_ENDPOINT=<optional>
AILAB_ARTIFACTS_DIR=/var/lib/ailab/artifacts
AILAB_WORKSPACES_DIR=/var/lib/ailab/workspaces
AILAB_RUNNER_IMAGE=ailab/runner:1.0.0
AILAB_WORKER_CONCURRENCY=4
AILAB_LLM_DEFAULT_MODEL=deepseek-flash
AILAB_LLM_ESCALATION_MODEL=deepseek-v4-pro
AILAB_GITHUB_API_URL=https://api.github.com
AILAB_ALLOWED_ORIGINS=
```

### 20.4 CI/CD

- `OPS-010`: GitHub Actions pipelines: `ci` (fmt, clippy `-D warnings`, test, coverage, openapi diff, frontend lint/test/build, e2e, security scans), `images` (build, scan, sign, SBOM), `nightly` (load, chaos, fuzz), `release` (tag, changelog, images, docs).
- `OPS-011`: Required checks: fmt, clippy, unit, integration, e2e, a11y, visual, security, openapi compatibility. Branch protection on `main`.
- `OPS-012`: Conventional Commits + Changesets/`git-cliff` changelog. SemVer for the product and images.
- `OPS-013`: No secrets in CI logs; OIDC to the registry; least-privilege tokens.
- `OPS-014`: Deployment is `docker compose pull && docker compose up -d`; a `Makefile` target performs backup → migrate → up → smoke.

---

## 21. Documentation deliverables (required from implementing agents)

The implementation MUST produce and keep current the following. This is part of DoD for the relevant cards, not an afterthought.

1. **README.md** — what it is, 5-minute quickstart, screenshots/GIF, architecture at a glance, links.
2. **docs/architecture.md** — C4 context/container/component, data flow, trust boundaries.
3. **docs/domain-model.md** — bounded contexts, aggregates, invariants, event flows.
4. **docs/adr/NNNN-*.md** — one ADR per material decision, using the template in Appendix D.3. Minimum ADR set listed in Appendix D.3.
5. **docs/api/openapi.json** + rendered reference; **docs/api/asyncapi.json** + event catalog.
6. **docs/database.md** — ERD (Mermaid), table dictionary, migration policy, index rationale.
7. **docs/kanban.md** — column semantics, transition matrix, DoR/DoD, examples.
8. **docs/agents.md** — role charters, character roster, CV schema, prompt assembly, guardrails.
9. **docs/office.md** — rendering architecture, asset pipeline, adding new sprites/characters, performance budget.
10. **docs/security.md** — threat model, sandbox spec, secret handling, exposure modes.
11. **docs/operations.md** — install, configure, backup/restore, upgrade, scaling, troubleshooting.
12. **docs/runbooks/** — `provider-outage.md`, `db-restore.md`, `stuck-runs.md`, `budget-exhausted.md`, `github-rate-limit.md`, `disk-full.md`.
13. **docs/testing.md** — how to run every suite, fixtures, deterministic seed, flaky policy.
14. **docs/contributing.md** — conventions, commit style, review checklist, definition of done.
15. **docs/diagrams/** — source files (Mermaid/Excalidraw) plus exported SVGs; diagrams are code-reviewed.
16. **CHANGELOG.md**, **SECURITY.md**, **CODE_OF_CONDUCT.md**, **LICENSE** (Apache-2.0).
17. **docs/demo-script.md** — the 5-minute golden-path demo with expected timings.
18. **docs/glossary.md** — domain vocabulary (Appendix D.5 as a starting point).

`DOC-001`: Every public function/type in Rust has rustdoc; every exported Angular component/service has TSDoc; every endpoint has an OpenAPI description and example.
`DOC-002`: Docs are built in CI (`mdbook` or VitePress) and deployed as static artifacts. Broken links fail the build.
`DOC-003`: Screenshots/GIFs are generated from the deterministic seed and refreshed on UI changes.

---

## 22. Delivery plan (milestones)

Each milestone is independently demoable and gated by acceptance criteria in Appendix D.1.

| Milestone | Scope | Exit criteria (summary) |
|---|---|---|
| **M0 — Foundations** | Repo, Rust workspace, Angular shell, Docker Compose, Postgres+pgvector, Redis, migrations, CI, observability skeleton | `make up` works; health green; CI green; ADRs 0001–0006 |
| **M1 — Inventory** | Providers/models/agents/projects CRUD, encryption, characters, CV v1 | Create DeepSeek provider, lab, 4 agents, project; API + UI; tests |
| **M2 — Kanban core** | Board, cards, transitions, invariants, logs, real-time | All 36 transition cases pass; J1 journey works |
| **M3 — Office** | Canvas engine, sprites, pathfinding, inspector, interactions, responsive, a11y | 60 FPS; J4 works; visual + a11y tests green |
| **M4 — SOW & plan** | Idea intake, SOW generation, approval, WBS, materialization | J2 works; SOW schema validated; diff/export |
| **M5 — Execution** | DeepSeek adapter, runner sandbox, agent loop, tools, GitHub PR, QA review, merge | J3 works end-to-end on a sample repo; autonomy A2 |
| **M6 — Memory & CV** | 3-tier memory, embeddings, hybrid search, reflection, compaction | Retrieval eval ≥ 0.8 nDCG@8; no cross-scope leakage |
| **M7 — Hardening** | Security, load, chaos, reliability, docs, runbooks, budgets | SLOs met; security review; restore drill; docs complete |
| **M8 — Release 1.0** | Polish, demo seed, images, SBOM, release notes | Golden demo ≤ 5 min; all gates green |

`DLV-010`: Milestones are delivered in order. Each milestone ends with a tagged pre-release (`v0.x.0`), a demo recording, and updated docs.
`DLV-011`: No milestone may regress a prior acceptance test (enforced in CI).

---

## 23. Acceptance criteria (global)

The product is accepted when **all** of the following hold:

1. `AC-001` A new operator can go from `git clone` to a running stack with `make up` in ≤ 10 minutes on the reference host, following only the README.
2. `AC-002` The operator can create a provider, lab, Tech Lead + Analyst + Dev + QA, and a GitHub-connected project entirely through the UI.
3. `AC-003` Submitting an idea creates canonical `TODO` cards within 3 s p95, each with story, acceptance criteria, deliverables, estimate and priority.
4. `AC-004` The Kanban enforces every invariant in §6.4 and Appendix C.3; invalid transitions are rejected with actionable violations.
5. `AC-005` The office renders all agents with truthful state, 60 FPS desktop, clickable objects, deep links, reduced-motion and screen-reader modes.
6. `AC-006` An idea can produce an approved SOW and a materialized WBS with acyclic dependencies.
7. `AC-007` A `READY` card is executed end-to-end: branch → code → tests → PR → QA review → merge → `DONE`, with full audit trail and cost accounting.
8. `AC-008` Three-tier memory works with scoped retrieval, citations, deduplication, compaction and zero cross-scope leakage.
9. `AC-009` CVs evolve from completed work with bounded deltas and immutable history.
10. `AC-010` Autonomy levels and hard stops behave exactly as specified; budget exhaustion stops work safely.
11. `AC-011` Security: no secret is readable via API/logs; sandbox prevents host/network escape; webhooks verified; CSP enforced.
12. `AC-012` SLOs in §17 are met under the defined load; soak and chaos tests pass.
13. `AC-013` Backup/restore drill succeeds within RTO; runbooks are accurate.
14. `AC-014` Documentation manifest (§21) is complete, built, link-checked and screenshot-current.
15. `AC-015` The deterministic demo (`make seed-demo`) reproduces the golden path in ≤ 5 minutes with expected visual output.

---

## 24. Risks and mitigations

| Risk | Impact | Likelihood | Mitigation |
|---|---|---|---|
| LLM output quality below bar | Bad cards/code | Medium | Structured outputs, validation, retries, escalation model, QA gate, human override |
| Prompt injection via repo/memory | Security | Medium | Untrusted blocks, tool policy, egress allowlist, secret scanning, no credential access |
| Runaway cost | Financial | Medium | Per-card/day/lab budgets, hard stops, circuit breaker, cost dashboards |
| Agent loops / stuck runs | Reliability | Medium | Bounded iterations/time, heartbeats, lease TTL, dead-letter, escalation |
| GitHub API limits/outages | Delivery stall | Medium | Backoff, queue, polling fallback, local branch work continues |
| Canvas performance on low-end | UX | Medium | Internal low resolution, batching, offscreen caching, quality auto-tune, FPS telemetry |
| Postgres growth (events/vectors) | Ops | Low/Med | Retention, partitioning-ready schema, compaction, archival |
| Sandbox escape | Security | Low | Ephemeral containers, no host mounts, cap drop, network none, seccomp/AppArmor profile |
| Scope creep (multi-tenant, auth) | Schedule | Medium | Explicit non-goals, ADR gate, milestone discipline |
| Character roster sensitivity | Brand | Low | Professional visual archetypes, neutral copy, documented as fictional personas |
| Version drift / EOL | Security | Medium | Pinned baseline, Renovate, EOL watch, upgrade ADRs |

---

## 25. Non-functional requirements summary

| ID | Requirement |
|---|---|
| `NFR-001` | All services containerized; single-command local bootstrap. |
| `NFR-002` | Rust backend with `#![forbid(unsafe_code)]`; Angular 22 zoneless frontend. |
| `NFR-003` | Postgres 18 + pgvector is the only durable store; Redis is cache/ephemeral only. |
| `NFR-004` | p95 API read ≤ 120 ms; p99 ≤ 300 ms at defined load. |
| `NFR-005` | Office ≥ 58 FPS desktop; ≥ 30 FPS low-end; deterministic given events+seed. |
| `NFR-006` | End-to-end traceability: every mutation has `request_id`, `trace_id`, `run_id` (if agentic), and an audit event. |
| `NFR-007` | Zero cross-scope memory leakage; verified by automated tests. |
| `NFR-008` | Graceful degradation: provider down → queue + visible status; Redis down → read-only mode (config) with explicit banner. |
| `NFR-009` | WCAG 2.2 AA for DOM; accessible fallback for canvas. |
| `NFR-010` | All data exportable; lab deletion complete and audited. |
| `NFR-011` | No obsolete/EOL versions; weekly dependency updates; SBOM per image. |
| `NFR-012` | Documentation is versioned with the code and built in CI. |

---

## 26. Glossary

| Term | Definition |
|---|---|
| **Lab** | Top-level workspace owning agents, projects and memory. |
| **Agent** | Persistent AI employee with role, character, CV and memory. |
| **Character** | Visual/behavioral persona from the fixed roster (Appendix C.1). |
| **CV** | Versioned structured record of an agent's skills and evidence. |
| **Card** | Unit of work on the Kanban with a canonical user story. |
| **Run** | One agent attempt to execute one card. |
| **SOW** | Statement of Work: structured, versioned delivery agreement. |
| **WBS/Plan** | Acyclic task DAG derived from the SOW. |
| **DoR / DoD** | Definition of Ready / Definition of Done. |
| **Autonomy level** | A0–A3 policy governing what agents may do without a human. |
| **Lease** | Time-bounded exclusive claim on a card by a run. |
| **Outbox** | Transactional table of domain events awaiting publication. |
| **Scope** | Memory boundary: lab, project or agent. |
| **Office** | Pixel-art visualization of a lab. |
| **Verification** | Pre-commit checks: format, lint, types, tests, secret scan. |

---

## Appendix index

- **Appendix A** — `APPENDIX-A-DATABASE.sql`: complete DDL, indexes, triggers, seeds.
- **Appendix B** — `APPENDIX-B-CONTRACTS.md`: OpenAPI 3.1 surface, AsyncAPI 3.0 events, JSON Schemas, error codes.
- **Appendix C** — `APPENDIX-C-AGENTS-KANBAN.md`: character roster, role charters, prompt contracts, Kanban templates, transition matrix, asset manifest.
- **Appendix D** — `APPENDIX-D-DELIVERY.md`: WBS, acceptance tests, ADR list, env vars, documentation manifest, demo script.

---

*End of `AILAB-SPEC-001` v1.0.0. This document is binding. Deviations require an approved ADR cited in the implementing PR.*
