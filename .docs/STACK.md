# AI-Lab Stack

| Field | Value |
|---|---|
| Document | `.docs/STACK.md` |
| Version | 1.0.0 |
| Status | Accepted |
| Owner | Dev |
| Source | `.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §5 |

This document is the single stack reference for implementation. Versions are pinned; floating versions and EOL lines are forbidden. If this document and the technical specification disagree, the specification wins and this document is corrected.

---

## 1. Pinned baseline

| Layer | Technology | Version | Pin location |
|---|---|---|---|
| Backend language | Rust | 1.99.1, edition 2024 | `rust-toolchain.toml` |
| Async runtime | Tokio | 1.54.x | `Cargo.lock` |
| HTTP framework | axum | 0.8.x | `Cargo.lock` |
| Middleware | tower / tower-http | 0.5.x / 0.7.x | `Cargo.lock` |
| Database access | sqlx (compile-time checked) | 0.9.x | `Cargo.lock` |
| Migrations | sqlx-cli | 0.9.x | `Cargo.lock` / image |
| Serialization | serde / serde_json | 1.0.x | `Cargo.lock` |
| OpenAPI | utoipa + Swagger UI | 6.0.x | `Cargo.lock` |
| Redis client | redis | 1.8.x | `Cargo.lock` |
| Vector types | pgvector | 0.4.x | `Cargo.lock` |
| Embeddings | fastembed-rs + `BAAI/bge-small-en-v1.5` | 384 dimensions | `Cargo.lock` |
| Telemetry | OpenTelemetry + tracing | 0.33.x | `Cargo.lock` |
| Container API | bollard | latest stable | `Cargo.lock` |
| Frontend framework | Angular | 22.2.x | `package-lock.json` |
| Frontend language | TypeScript | 6.0.x (ADR-0024; Angular 22.2 peer range `>=6.0 <6.1`) | `package-lock.json` |
| Frontend build | Angular CLI / esbuild | 22.2.x | `package-lock.json` |
| State | Angular Signals + RxJS interop | as shipped | `package-lock.json` |
| Rendering | Canvas 2D; optional WebGL2 backend | native | code |
| Styling | SCSS + CSS custom properties | native | code |
| Database | PostgreSQL | 18.x | Compose image |
| Database extension | pgvector | 0.8.7 | Compose image |
| Cache | Redis | 8.8.x | Compose image |
| Web server | Nginx | 1.29-alpine | Compose image |
| Runtime Node.js | Node.js | 24.x LTS (Krypton) | `.tool-versions` / image |
| Package manager | npm | 11.x | `package.json` `packageManager` |
| Build images | Debian | 13 "trixie" slim | digest-pinned |
| Containers | Docker Engine | ≥ 27 | operator |
| Orchestration | Docker Compose | ≥ 2.30 | operator |
| LLM provider | DeepSeek | `deepseek-flash`, `deepseek-v4-pro` | provider config |
| SCM provider | GitHub | REST v3 | adapter config |

**Version policy.** Patch and minor updates land within 7 days of a green CI run. Major updates require an ADR, full regression and a rollback plan. Security patches land within 48 hours. No dependency may be on an EOL line.

---

## 2. Logical architecture

```
Browser (Angular 22, zoneless, signals)
   │ HTTPS + WSS
   ▼
web (Nginx + static SPA) ──► api (Rust/axum)
                                 │
                                 ├── PostgreSQL 18 + pgvector (system of record)
                                 ├── Redis 8.8 (cache, rate limits, locks, pub/sub)
                                 └── outbox
                                        │
                                        ▼
                                  worker (scheduler, orchestrator, relay)
                                   │        │
                                   │        ├── DeepSeek API
                                   │        ├── GitHub API
                                   │        └── runner (sandbox control plane)
                                   │                 │
                                   │                 └── docker-socket-proxy
                                   │
                                   └── reflection + maintenance jobs
```

AI-Lab v1 is a **modular monolith deployed as runtime services** (`web`, `api`, `worker`, `runner`, `migrate`). Bounded contexts are contract-first: Inventory, Lab, Knowledge/Memory, Kanban, Delivery, Platform. No context reads or writes another context's tables. See `.docs/adr/adr-0002.md`.

---

## 3. Repository layout

```text
Cargo.toml                 Rust workspace
rust-toolchain.toml        Rust pin
.tool-versions             Node/Python pins
crates/
  domain/                  entities, value objects, invariants, events
  application/             use cases, ports, transactions
  infrastructure/          sqlx repositories, Redis, blob store, clock, IDs
  llm/                     LlmProvider port + DeepSeek adapter
  scm/                     ScmProvider port + GitHub adapter
  memory/                  embeddings, chunking, retrieval, compaction
  orchestrator/            scheduler, leases, run pipeline, sandbox client
  api/                     axum router, DTOs, OpenAPI, WebSocket hub
  worker/                  orchestration, relay, reflection, maintenance
  telemetry/               tracing, metrics, OpenTelemetry
  testkit/                 fakes, fixtures, builders
  cli/                     admin CLI
web/                       Angular application
migrations/                sqlx migrations
infra/                     Dockerfiles, Compose profiles, proxies, observability
scripts/                   doctor, sow-lint, harness-lint, wait-for, generators
.docs/product/                      project documentation and ADRs
.opencode/                 skills and commands
```

The exact tree and ownership are defined in `.docs/sow/WP-02-REPOSITORY-DEVEX.md` and `.docs/sow/WP-01-ARCHITECTURE.md`.

---

## 4. Primary commands

| Command | Purpose |
|---|---|
| `make up` | Start the core stack (Postgres, Redis, migrate, api, worker, runner, web) |
| `make down` | Stop the stack, preserving volumes |
| `make reset` | Stop, remove volumes, start, migrate, seed demo |
| `make logs` | Follow logs for core services |
| `make doctor` | Validate Docker, ports, disk, tools, keys and connectivity |
| `make fmt` | Format Rust and frontend code |
| `make lint` | Clippy with `-D warnings` and ESLint |
| `make check` | Fast local gate: format check, lint, typecheck, unit tests, SOW lint, docs links |
| `make test` | Rust and frontend unit/integration suites |
| `make e2e` | Playwright end-to-end suite |
| `make bdd` | Gherkin BDD E2E suite; all, `SUITE=id` or `UP_TO=id`, preflight-gated and make-only |
| `make bdd-list` | List the six BDD suites and their scenario counts |
| `make bdd-set-api-key` | Load the live DeepSeek API key through a hidden prompt |
| `make load` | k6 load scenarios |
| `make seed-demo` | Deterministic demo dataset |
| `make migrate` | Apply sqlx migrations |
| `make openapi` | Regenerate OpenAPI and client types |
| `make asyncapi` | Regenerate AsyncAPI |
| `make sbom` | Generate SBOMs for images |
| `make sow-lint` | Validate SOW statuses, links, checklists and dependencies |
| `scripts/harness-lint.sh` | Validate the agent harness (docs, skills, commands, ADRs, `AGENTS.md`) |
| `make backup` | Create a database dump and artifact manifest |
| `make restore-drill` | Restore into a scratch database and verify |
| `make clean` | Remove build artifacts, never source |

Every command is idempotent and non-interactive. Failures exit non-zero with an actionable message.

---

## 5. Runtime profiles and ports

| Profile | Services | Host exposure |
|---|---|---|
| `core` (default) | postgres, redis, migrate, api, worker, runner, docker-socket-proxy, web | `127.0.0.1:8080` (web only) |
| `dev` | core + hot reload and dev ports | `127.0.0.1:3000`, `5432`, `6379` |
| `observability` | otel-collector, prometheus, grafana | `127.0.0.1:3001` (Grafana) |
| `tls` | caddy | `80`, `443` |
| `egress` | egress-proxy | none |
| `demo` | core + seeded demo job | `127.0.0.1:8080` |

Postgres and Redis are never published in production. Sandbox containers never join `edge` or `backbone`. See `.docs/sow/WP-03-INFRASTRUCTURE.md`.

---

## 6. Constraints

- No ORM (Diesel, SeaORM); raw SQL with sqlx.
- No GraphQL; REST + WebSocket only.
- No Kafka or RabbitMQ; Postgres outbox plus `FOR UPDATE SKIP LOCKED` plus Redis pub/sub.
- No Kubernetes operator in v1; Docker Compose is the supported deployment.
- No `unsafe` in first-party crates; `#![forbid(unsafe_code)]` is mandatory.
- No `zone.js`; Angular runs zoneless with signals.
- No `any` in first-party TypeScript outside validated parsing boundaries.
- No third-party frontend scripts; CSP is `self` only.
- No EOL runtime, framework, database or dependency.
- No secrets in environment examples, logs, events, prompts, fixtures or the sandbox.

---

## 7. External dependencies and limits

| Dependency | Purpose | Limit / control |
|---|---|---|
| DeepSeek API | LLM inference | `deepseek-flash` default, `deepseek-v4-pro` escalation; timeouts, retries, circuit breaker, budgets |
| GitHub API | SCM | Fine-grained PAT; 80% rate-limit ceiling; backoff; webhooks with HMAC |
| Docker Engine | Sandbox | Only through `docker-socket-proxy`; no host mounts; no privileged |
| Package registries | Build and optional sandbox egress | Allowlist: crates.io, npmjs.org, pypi.org, github.com, objects.githubusercontent.com |
| SMTP | Not used in v1 | Out of scope |
| External analytics | Not used | Telemetry is local |

---

## 8. Links

- Product specification: `./technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md`
- Database schema: `./technical-specification/APPENDIX-A-DATABASE.sql`
- API and event contracts: `./technical-specification/APPENDIX-B-CONTRACTS.md`
- Agents, prompts and Kanban: `./technical-specification/APPENDIX-C-AGENTS-KANBAN.md`
- Delivery WBS and acceptance: `./technical-specification/APPENDIX-D-DELIVERY.md`
- SOW index: `./sow/SOW.md`
- Architecture decisions: `./adr/registry.md`
- Constitution: `./CONSTITUTION.md`
- Principles: `./PRINCIPLES.md`
- Harness entry point: `../AGENTS.md`
