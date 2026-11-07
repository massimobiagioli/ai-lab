# AI-Lab — Appendix D — Delivery WBS, Acceptance & Operations

Document: `AILAB-SPEC-001 / APPENDIX-D`
Status: **NORMATIVE**

---

## D.1 Work Breakdown Structure (executable backlog)

Each row becomes one or more Kanban cards. `Est` is Fibonacci story points. `Role` is the primary owner. Dependencies are task IDs. Every task inherits the global DoD (§6.4.4) and must cite the requirement IDs it satisfies.

### M0 — Foundations

| ID | Task | Role | Est | Depends on | Deliverables |
|---|---|---|---|---|---|
| M0-01 | Initialize monorepo, Rust workspace, `rust-toolchain.toml`, `.tool-versions`, `.editorconfig`, `.gitignore` | Dev | 2 | — | `Cargo.toml`, `crates/*` stubs, toolchain files |
| M0-02 | Angular 22 app shell: standalone, zoneless, routing, layout, design tokens | Dev | 3 | M0-01 | `web/`, pixel tokens, shell renders |
| M0-03 | Docker Compose core: postgres+pgvector, redis, api, worker, web, migrate | Dev | 3 | M0-01 | `infra/compose/*`, `make up` |
| M0-04 | Dockerfiles (api, worker, web, runner) multi-stage, non-root, healthchecks | Dev | 3 | M0-03 | images build < budgets |
| M0-05 | sqlx migrations baseline + Appendix A applied + seed characters | Dev | 3 | M0-03 | `migrations/`, `make migrate` |
| M0-06 | Config loading + validation + fail-fast + secrets encryption utility | Dev | 2 | M0-01 | `config` module, unit tests |
| M0-07 | Observability skeleton: tracing, OTel, Prometheus metrics, health endpoints | Dev | 3 | M0-03 | `/health/*`, dashboards config |
| M0-08 | CI pipeline: fmt, clippy, tests, frontend lint/test/build, security scans | Dev | 3 | M0-01..07 | `.github/workflows/ci.yml` green |
| M0-09 | `Makefile` + `scripts/doctor.sh` + README quickstart | Dev | 2 | M0-03 | `make up`, `make doctor` |
| M0-10 | ADRs 0001–0008 | Analyst | 2 | — | `.docs/adr/*` |

### M1 — Inventory

| ID | Task | Role | Est | Depends on | Deliverables |
|---|---|---|---|---|---|
| M1-01 | Provider/model domain + repos + encryption + API | Dev | 5 | M0-05/06 | CRUD, key never returned, tests |
| M1-02 | DeepSeek provider seed on creation (2 models, costs, limits) | Dev | 2 | M1-01 | seeded models |
| M1-03 | Character catalog API + validation | Dev | 2 | M0-05 | `GET /characters` (internal), tests |
| M1-04 | Agent aggregate: role rules, one Tech Lead, lifecycle | Dev | 5 | M1-03 | CRUD, pause/resume/retire, tests |
| M1-05 | CV v1 schema + read API + seed CV | Dev | 3 | M1-04 | `GET /agents/{id}/cv` |
| M1-06 | Project aggregate + GitHub credential verification | Dev | 5 | M1-01 | project CRUD, `POST /sync` |
| M1-07 | Inventory UI: agents, projects, providers + wizard | Dev | 8 | M1-04/06 | routes, forms, validation |
| M1-08 | Audit log for inventory mutations | Dev | 2 | M1-04 | audit rows, API filter |
| M1-09 | Inventory tests (unit, integration, e2e) | QA | 3 | M1-07 | coverage ≥ 80% |

### M2 — Kanban core

| ID | Task | Role | Est | Depends on | Deliverables |
|---|---|---|---|---|---|
| M2-01 | Board/card aggregates, key allocation, versioning | Dev | 5 | M1-06 | repos + services |
| M2-02 | Transition engine + guards + error payloads | Dev | 8 | M2-01 | 36-case matrix tests |
| M2-03 | Card events (immutable) + projection + audit | Dev | 3 | M2-01 | `card_events`, tests |
| M2-04 | TODO creation from idea (Tech Lead) + IdeaCards schema | Dev | 5 | M2-02, M5-01 | `POST /ideas`, 1–20 cards |
| M2-05 | Kanban UI: 6 columns, DnD, keyboard, card front/back | Dev | 8 | M2-02 | routes + components |
| M2-06 | Real-time WebSocket hub + subscribe + replay | Dev | 5 | M2-03 | WS endpoint, backfill |
| M2-07 | Kanban real-time reconciliation + optimistic UI | Dev | 3 | M2-05/06 | ≤300 ms sync |
| M2-08 | Filters, URL state, empty states | Dev | 2 | M2-05 | URL-addressable filters |
| M2-09 | Kanban e2e + invariant test harness | QA | 5 | M2-02/07 | Playwright + matrix |

### M3 — Virtual Office

| ID | Task | Role | Est | Depends on | Deliverables |
|---|---|---|---|---|---|
| M3-01 | Office engine skeleton: canvas, fixed timestep, camera, layers | Dev | 5 | M0-02 | 60 FPS demo scene |
| M3-02 | Sprite pipeline + manifest + asset linter + loading screen | Dev | 5 | M3-01 | atlas loader, progress |
| M3-03 | Tile map, furniture, y-sort, hit testing, tooltips | Dev | 5 | M3-02 | clickable room |
| M3-04 | A* pathfinding + walkability + collision | Dev | 5 | M3-03 | deterministic paths |
| M3-05 | Agent state machine + animation mapping + speech bubbles | Dev | 5 | M3-04, M2-06 | truthful states |
| M3-06 | Agent Inspector drawer (Live/CV/Memory/Controls) | Dev | 5 | M3-05, M1-05 | deep links |
| M3-07 | Object interactions: whiteboard, rack, bookshelf, cabinet, door | Dev | 3 | M3-03 | drawers wired |
| M3-08 | Responsive + reduced motion + a11y fallback list | Dev | 3 | M3-06 | WCAG 2.2 AA |
| M3-09 | Dashboard with KPIs, office preview, activity feed | Dev | 5 | M3-05, M2-06 | `/` route |
| M3-10 | Office visual regression + FPS soak | QA | 3 | M3-09 | screenshots, RAF probe |

### M4 — SOW & Plan

| ID | Task | Role | Est | Depends on | Deliverables |
|---|---|---|---|---|---|
| M4-01 | SOW aggregate + versioning + JSON schema validation | Dev | 5 | M2-01 | `sows`, `sow_versions` |
| M4-02 | SOW generation pipeline (Analyst) + streaming | Dev | 8 | M4-01, M5-01 | async job, MD render |
| M4-03 | SOW approval + freeze + audit | Dev | 3 | M4-02 | `POST /approve` |
| M4-04 | Plan/WBS generation + DAG validation (cycle detection) | Dev | 5 | M4-03 | `plans`, `plan_tasks` |
| M4-05 | Materialization → TODO cards + parent links + preview | Dev | 5 | M4-04, M2-02 | `POST /materialize` |
| M4-06 | SOW workspace UI: versions, diff, export MD/PDF | Dev | 5 | M4-02 | `/projects/{id}/sow` |
| M4-07 | Plan UI: WBS tree, DAG view, timeline | Dev | 5 | M4-04 | plan route |
| M4-08 | SOW/plan e2e + schema conformance + traceability tests | QA | 3 | M4-05/07 | J2 green |

### M5 — Execution

| ID | Task | Role | Est | Depends on | Deliverables |
|---|---|---|---|---|---|
| M5-01 | LlmProvider port + DeepSeek adapter (stream, tools, retries, circuit breaker) | Dev | 8 | M1-01 | adapter + wiremock tests |
| M5-02 | Prompt assembler (ordered sections, hash, budgets) | Dev | 5 | M5-01 | deterministic prompts |
| M5-03 | Token/cost ledger + budgets + hard stops | Dev | 3 | M5-01 | `llm_requests`, `cost_ledger` |
| M5-04 | Runner service + sandbox lifecycle + resource limits | Dev | 8 | M0-04 | ephemeral containers |
| M5-05 | Tool registry + policy engine + allowlists | Dev | 5 | M5-04 | typed tools, denial |
| M5-06 | Scheduler + leases + heartbeats + retries + dead letter | Dev | 8 | M2-01 | robust dispatch |
| M5-07 | Run pipeline: context → plan → loop → verify → commit → PR | Dev | 13 | M5-02..06 | end-to-end run |
| M5-08 | GitHub adapter: branches, commits, PR, checks, reviews, merge | Dev | 8 | M1-06 | SCM port + tests |
| M5-09 | QA review pipeline + merge policy + separation of duties | Dev | 5 | M5-07/08 | REVIEW→DONE |
| M5-10 | Run UI: live steps, logs, controls, dead letters | Dev | 5 | M5-07 | run inspector |
| M5-11 | Execution e2e on sample repo (branch→merge) | QA | 5 | M5-09 | J3 green, autonomy A2 |

### M6 — Memory & CV

| ID | Task | Role | Est | Depends on | Deliverables |
|---|---|---|---|---|---|
| M6-01 | Embeddings (fastembed, 384d) + chunker + ingestion | Dev | 5 | M4-01 | `memory_chunks` |
| M6-02 | Hybrid retrieval (vector + FTS + scope filter + recency) | Dev | 5 | M6-01 | `/memory/search` |
| M6-03 | Redis working memory + retrieval cache + invalidation | Dev | 3 | M6-02 | TTLs, namespacing |
| M6-04 | Memory API + explorer UI | Dev | 5 | M6-02 | documents, search |
| M6-05 | Reflection job + CV versioning + bounded deltas | Dev | 5 | M5-07, M6-01 | `agent_cv_versions` |
| M6-06 | Dedup + secret scrub before embedding/persist | Dev | 3 | M6-01 | pipeline + tests |
| M6-07 | Compaction + retention jobs | Dev | 3 | M6-05 | maintenance jobs |
| M6-08 | Retrieval quality eval (nDCG@8 ≥ 0.8) + cross-scope leakage tests | QA | 3 | M6-02 | eval report |

### M7 — Hardening

| ID | Task | Role | Est | Depends on | Deliverables |
|---|---|---|---|---|---|
| M7-01 | Threat model review + fixes | QA | 5 | M5-11 | `.docs/product/security.md` |
| M7-02 | Sandbox escape tests + egress policy verification | QA | 5 | M5-04 | security test suite |
| M7-03 | Load tests (k6) + performance tuning to budgets | Dev | 5 | M5-11 | results vs §17 |
| M7-04 | Chaos: kill worker/DB/Redis; recovery verification | Dev | 5 | M5-06 | chaos report |
| M7-05 | Backup/restore drill + runbooks | Dev | 3 | M0-05 | RTO verified |
| M7-06 | A11y audit + fixes | QA | 3 | M3-08 | WCAG report |
| M7-07 | Docs completion (manifest §21) + link check | Analyst | 5 | all | docs build green |
| M7-08 | Dependency/EOL audit + SBOM + image signing | Dev | 2 | M0-04 | SBOM artifacts |
| M7-09 | Final e2e regression + flaky quarantine | QA | 3 | all | green pipeline |

### M8 — Release 1.0

| ID | Task | Role | Est | Depends on | Deliverables |
|---|---|---|---|---|---|
| M8-01 | Deterministic demo seed (`make seed-demo`) | Dev | 3 | M5-11 | 4 agents, 12 cards, 30 events |
| M8-02 | Polish pass: transitions, sounds, micro-interactions | Dev | 5 | M3-10 | final UX |
| M8-03 | Demo script + recording + screenshots/GIFs | Analyst | 2 | M8-01 | `.docs/product/demo-script.md` |
| M8-04 | Release notes, versioning, images, compose `1.0.0` | Dev | 2 | all | tag + artifacts |
| M8-05 | Golden-path acceptance run (AC-001..015) | QA | 3 | all | signed acceptance report |

---

## D.2 Acceptance test catalog

Every acceptance test is automated unless marked *(manual)*. Test IDs are stable and referenced by cards.

| ID | Test | Method | Pass condition |
|---|---|---|---|
| AT-001 | Bootstrap from clean clone | CI job + timer | `make up` ≤ 10 min; health green |
| AT-002 | Create provider/lab/team/project via UI | Playwright J1 | all entities exist; Tech Lead enforced |
| AT-003 | Idea → TODO cards | API + Playwright | ≤ 3 s p95; schema valid; 1–20 cards |
| AT-004 | Kanban matrix (36 cases) | Rust integration | allowed/denied exactly per C.3 |
| AT-005 | DoR/DoD guards | Rust integration | each missing field yields the right violation |
| AT-006 | Versions/conflicts | API | stale write → 409 with current version |
| AT-007 | Office render + interactions | Playwright | agents clickable; panels open; FPS ≥ 58 |
| AT-008 | Reduced motion + screen reader | Playwright + axe | no motion; list exposes statuses |
| AT-009 | SOW generation + approval | API + UI | schema valid; versioned; export works |
| AT-010 | Plan acyclicity | Property test | cycle rejected; DAG accepted |
| AT-011 | Materialization | API | cards created with parent/SOW links |
| AT-012 | Full run branch→PR→review→merge | E2E sample repo | card DONE; SHA stored; audit complete |
| AT-013 | Separation of duties | API | self-review/self-merge denied |
| AT-014 | Budget hard stop | Integration | run stops; lab paused; event emitted |
| AT-015 | Retry/backoff/dead letter | Chaos | max 3 attempts; ICEBOX + escalation |
| AT-016 | Worker crash recovery | Chaos | lease expires; run resumes/requeues |
| AT-017 | Three-tier memory isolation | Integration | no cross-scope hits; leakage test fails build |
| AT-018 | Retrieval quality | Eval harness | nDCG@8 ≥ 0.8 on golden set |
| AT-019 | CV evolution | Integration | delta ∈ [−1,1]; version immutable; evidence link |
| AT-020 | Secret never exposed | Security tests | API/logs/events contain no plaintext key |
| AT-021 | Sandbox isolation | Security tests | no host FS, no network, no caps, non-root |
| AT-022 | Webhook signature + idempotency | Integration | bad sig 401; duplicate 202 once |
| AT-023 | Prompt injection | Red-team fixtures | no policy bypass; event logged |
| AT-024 | API contract conformance | Schemathesis | 0 failures |
| AT-025 | Load budgets | k6 | §17 thresholds |
| AT-026 | Soak 60 min | k6 | no leak > 10%; errors < 0.1% |
| AT-027 | Backup/restore | Scripted drill | restore ≤ RTO; checksums match |
| AT-028 | Docs build + links | CI | 0 broken links; screenshots fresh |
| AT-029 | Golden demo | Manual + recording | ≤ 5 min; expected visuals |
| AT-030 | EOL/dependency audit | CI | no EOL, no critical CVE |

---

## D.3 ADR minimum set

Each ADR follows: **Context → Decision → Consequences → Alternatives → Status**. Store in `.docs/adr/NNNN-kebab-title.md`.

| ADR | Title | Decision |
|---|---|---|
| 0001 | Monorepo with Rust workspace + Angular app | Single repo, independent build pipelines |
| 0002 | Hexagonal architecture and bounded contexts | Ports/adapters, five contexts |
| 0003 | Postgres as sole system of record | Redis is cache/ephemeral only |
| 0004 | Outbox + `SKIP LOCKED` instead of a broker | At-least-once, simpler ops |
| 0005 | DeepSeek as first LLM provider | OpenAI-compatible adapter behind a port |
| 0006 | Local embeddings with fastembed | No second vendor; 384-d vectors |
| 0007 | Canvas 2D office, internal 384×216 | Pixel-perfect, deterministic, 60 FPS |
| 0008 | Angular zoneless + signals | No zone.js; hot loop outside CD |
| 0009 | Kanban invariants enforced server-side | UI is a projection; 422 with violations |
| 0010 | Fixed agent roles and finite character roster | Predictable permissions; personas are cosmetic |
| 0011 | Separation of duties | Reviewer ≠ author; merger ≠ author |
| 0012 | Sandboxed ephemeral runner containers | No host mounts; allowlisted egress |
| 0013 | Login-less local-first, token for remote | Single-operator product; secure by default |
| 0014 | WebSocket + resumable seq | Ordering, replay, backpressure |
| 0015 | Structured outputs + JSON Schema validation | Reliability over free-form text |
| 0016 | Three-tier memory with SQL-level scoping | Prevent leakage; hybrid retrieval |
| 0017 | Budgets as first-class limits | Cost safety and predictability |
| 0018 | Autonomy levels A0–A3 | Explicit, auditable delegation |
| 0019 | Conventional Commits + trace trailers | Traceability from commit to run |
| 0020 | Docker Compose deployment (no K8s in v1) | Operational simplicity |
| 0021 | MVU-style office engine decoupled from Angular | Testability and performance |
| 0022 | Version pinning + Renovate + SBOM | No EOL, supply-chain safety |

---

## D.4 Environment variables (complete)

| Variable | Required | Default | Description |
|---|---|---|---|
| `AILAB_ENV` | no | `production` | `production|development` |
| `AILAB_BIND` | no | `127.0.0.1` | Bind address; non-loopback requires token |
| `AILAB_PORT` | no | `8080` | HTTP port (web proxy) |
| `AILAB_DATABASE_URL` | **yes** | — | `postgres://user:pass@host:5432/ailab` |
| `AILAB_APP_DB_PASSWORD` | no | `ailab_app` | Password for the application role (CR-0007) |
| `AILAB_MIGRATE_DB_PASSWORD` | no | `ailab_migrate` | Password for the migration role (CR-0007) |
| `AILAB_READONLY_DB_PASSWORD` | no | `ailab_readonly` | Password for the read-only role (CR-0007) |
| `AILAB_REDIS_URL` | **yes** | — | `redis://host:6379/0` |
| `AILAB_MASTER_KEY` | **yes** | — | base64 32 bytes; AES-256-GCM |
| `AILAB_API_TOKEN` | conditional | — | Required if non-loopback; ≥32 bytes |
| `AILAB_LOG_LEVEL` | no | `info` | `trace|debug|info|warn|error` |
| `AILAB_LOG_FORMAT` | no | `json` | `json|pretty` |
| `AILAB_OTEL_ENDPOINT` | no | — | OTLP HTTP endpoint |
| `AILAB_METRICS_ENABLED` | no | `true` | Prometheus `/metrics` |
| `AILAB_ARTIFACTS_DIR` | no | `/var/lib/ailab/artifacts` | Artifact volume |
| `AILAB_WORKSPACES_DIR` | no | `/var/lib/ailab/workspaces` | Git worktrees |
| `AILAB_RUNNER_IMAGE` | no | `ailab/runner:1.0.0` | Sandbox image |
| `AILAB_RUNNER_CPU` | no | `2` | CPU limit |
| `AILAB_RUNNER_MEMORY_MB` | no | `4096` | Memory limit |
| `AILAB_RUNNER_TIMEOUT_S` | no | `2700` | Run wall clock |
| `AILAB_RUNNER_TOKEN` | **yes** (runner) | — | Shared secret for the worker→runner API; ≥ 32 bytes (CR-0004) |
| `AILAB_RUNNER_URL` | no | `http://runner:8081` | Worker readiness probe target (CR-0006) |
| `AILAB_RUNNER_BIND` | no | `0.0.0.0:8081` | Runner readiness listener bind address (CR-0006) |
| `AILAB_WORKER_CONCURRENCY` | no | `4` | Runs per worker |
| `AILAB_SCHEDULER_TICK_MS` | no | `1000` | Scheduler tick |
| `AILAB_LEASE_TTL_S` | no | `900` | Lease TTL |
| `AILAB_LLM_DEFAULT_MODEL` | no | `deepseek-flash` | Default model |
| `AILAB_LLM_ESCALATION_MODEL` | no | `deepseek-v4-pro` | Escalation model |
| `AILAB_GITHUB_API_URL` | no | `https://api.github.com` | GitHub API |
| `AILAB_GITHUB_WEBHOOK_SECRET` | no | — | Webhook HMAC secret |
| `AILAB_ALLOWED_ORIGINS` | no | — | Comma-separated CORS allowlist |
| `AILAB_CORS_ENABLED` | no | `false` | Enable cross-origin |
| `AILAB_RATE_LIMIT_READ_RPM` | no | `120` | Read rate limit/IP |
| `AILAB_RATE_LIMIT_WRITE_RPM` | no | `30` | Write rate limit/IP |
| `AILAB_MAX_REQUEST_BYTES` | no | `2097152` | JSON body limit |
| `AILAB_PG_POOL_MAX` | no | `20` | Pool per replica |
| `AILAB_REDIS_MAXMEMORY` | no | `512mb` | Redis limit |
| `AILAB_EMBEDDING_MODEL` | no | `BAAI/bge-small-en-v1.5` | Local embeddings |
| `AILAB_MEMORY_TOP_K` | no | `8` | Retrieval top-k |
| `AILAB_MEMORY_CACHE_TTL_S` | no | `300` | Retrieval cache TTL |
| `AILAB_PROMPT_RETENTION_DAYS` | no | `90` | Prompt/response retention |
| `AILAB_EPISODE_RETENTION_DAYS` | no | `180` | Raw episode retention |
| `AILAB_BUDGET_DEFAULT_PER_CARD_USD` | no | `5.0` | Card budget |
| `AILAB_BUDGET_DEFAULT_PER_DAY_USD` | no | `50.0` | Daily budget |
| `AILAB_SEED_DEMO` | no | `false` | Seed demo data on boot |

**Validation rules:** unknown `AILAB_*` variables are logged as warnings; malformed values abort boot with exit code 78 and a message naming the variable. Secret variables are never echoed.

---

## D.5 Documentation manifest (release gate)

| Artifact | Path | Owner | Acceptance |
|---|---|---|---|
| README + quickstart | `README.md` | Dev | Fresh clone → running in ≤10 min; GIF embedded |
| Architecture (C4 + flows) | `.docs/product/architecture.md` | Analyst | Context/container/component + trust boundaries |
| Domain model | `.docs/product/domain-model.md` | Analyst | Aggregates, invariants, events match code |
| API reference | `.docs/product/api/openapi.json` + rendered | Dev | Contract tests pass; examples valid |
| Event reference | `.docs/product/api/asyncapi.json` | Dev | All emitted events documented |
| Database dictionary + ERD | `.docs/product/database.md` | Dev | Every table/column/index explained |
| Kanban semantics | `.docs/product/kanban.md` | Analyst | Matrix + DoR/DoD + examples |
| Agents & prompts | `.docs/product/agents.md` | Analyst | Charters, roster, prompt order, guardrails |
| Office architecture | `.docs/product/office.md` | Dev | Engine, assets, adding sprites, perf budget |
| Security | `.docs/product/security.md` | QA | Threat model, sandbox, exposure modes |
| Operations | `.docs/product/operations.md` | Dev | Install, backup, restore, upgrade, scale |
| Runbooks (6) | `.docs/product/runbooks/*` | Dev | Each has symptoms, diagnosis, action, rollback |
| Testing | `.docs/product/testing.md` | QA | Commands, fixtures, seed, flaky policy |
| Contributing | `CONTRIBUTING.md` | Dev | Conventions, review checklist, DoD |
| ADRs (≥22) | `.docs/adr/*` | All | Each decision recorded and linked |
| Diagrams source + SVG | `.docs/product/diagrams/*` | Analyst | Regenerable; reviewed |
| CHANGELOG / SECURITY / LICENSE | repo root | Dev | SemVer, Apache-2.0, disclosure policy |
| Demo script | `.docs/product/demo-script.md` | Analyst | ≤5 min, timings, expected visuals |
| Glossary | `.docs/product/glossary.md` | Analyst | Terms match domain language |

`DOC-GATE`: Release is blocked if any artifact is missing, the docs build fails, links are broken, or screenshots do not match the current UI.

---

## D.6 Demo script (golden path, ≤ 5 minutes)

**Prerequisites:** `make up && make seed-demo`. Open `http://localhost:8080`.

| Time | Action | Expected |
|---|---|---|
| 0:00 | Dashboard loads | Pixel office preview animates; KPIs populated; 4 agents present |
| 0:25 | Open **Lab** | 16-bit office, agents at desks, ambient motion, 60 FPS |
| 0:45 | Click the Dev agent | Inspector: live status, current card, last steps, CV bars |
| 1:05 | Click whiteboard | Kanban drawer with 6 columns and seeded cards |
| 1:25 | Type an idea: "Add CSV→Parquet conversion" | Tech Lead speech bubble; within 3 s a canonical TODO card appears |
| 1:50 | Open the card | Story, AC, deliverables, estimate, source=agent |
| 2:10 | Move card to READY (assign + due date) | Valid transition; toast confirms DoR |
| 2:25 | Run the agent | Office Dev walks to desk, `type` animation; live run steps stream |
| 3:10 | PR opened → REVIEW | QA agent animates `read`; review verdict appears |
| 3:40 | Merge (autonomy A2) | Celebrate animation, chime, card → DONE, CV evidence appended |
| 4:05 | Click bookshelf | Memory explorer shows the new decision and episode |
| 4:25 | Click server rack | Health: provider green, DB/Redis latency, budget spend |
| 4:40 | Toggle reduced motion | Animation stops; states remain readable; a11y list available |
| 4:55 | End | Dashboard reflects updated KPIs and activity feed |

**Recording:** 1440p, 60 FPS, cursor highlighted, captions. Stored in `.docs/product/media/demo.mp4` and linked from README.

---

## D.7 Operating procedures (runbook index)

| Runbook | Trigger | First action |
|---|---|---|
| `provider-outage.md` | `provider_health` degraded, 5xx burst | Verify key/status page; check circuit breaker; queue continues |
| `db-restore.md` | Data loss/corruption | Stop writers; restore latest dump; verify migrations; integrity job |
| `stuck-runs.md` | Lease age > TTL, no heartbeat | Inspect run steps; cancel; requeue; check runner capacity |
| `budget-exhausted.md` | `system.budget.exhausted` | Confirm spend; raise budget or wait for reset; resume lab |
| `github-rate-limit.md` | 403/429 from GitHub | Check headers; pause queue; wait for reset; reduce polling |
| `disk-full.md` | Disk > 85% | Prune workspaces/artifacts; rotate logs; verify backup |
| `ws-disconnects.md` | Fan-out gaps | Check Redis pub/sub; client backfill; connection limits |
| `migration-failure.md` | Boot fails after upgrade | Roll back image; restore DB; fix migration; re-run expand/contract |

Each runbook: **Symptoms → Impact → Diagnosis commands → Remediation → Verification → Rollback → Post-incident**.

---

## D.8 Traceability matrix (requirement → tests)

| Requirement group | Primary tests |
|---|---|
| `FR-INV-*` | AT-002, AT-019, inventory integration |
| `FR-LAB-*` | AT-002, AT-013 |
| `FR-KAN-*` | AT-003..006, AT-011, AT-013, invariant harness |
| `FR-DLV-*` | AT-009..012, AT-015, AT-016 |
| `FR-LLM-*` | AT-014, AT-023, adapter tests |
| `FR-GH-*` | AT-012, AT-022 |
| `FR-MEM-*` | AT-017, AT-018 |
| `FR-UI-*` / `UX-*` | AT-007, AT-008, AT-029 |
| `SEC-*` | AT-020..023, AT-027 |
| `NFR-*` | AT-025, AT-026, AT-030 |

---

*End of Appendix D.*
