# Architecture Decision Record Registry

| Field | Value |
|---|---|
| Document | `.docs/adr/registry.md` |
| Version | 1.0.0 |
| Status | Accepted |
| Owner | Tech Lead |

This registry is the index of all architecture decisions. Every ADR is immutable once accepted. A new decision that changes an accepted one creates a new ADR and marks the old one as `superseded`. The `Status` column and the ADR file must agree.

## Status lifecycle

`proposed` → `accepted` → `superseded` or `deprecated`. Rejected proposals remain archived with status `deprecated`; they are never deleted.

## Registry

| ID | Title | Status | Date | Supersedes | Superseded by | File |
|---|---|---|---|---|---|---|
| ADR-0001 | Monorepo with Rust workspace and Angular app | accepted | 2026-11-07 | — | — | [adr-0001.md](./adr-0001.md) |
| ADR-0002 | Modular monolith with bounded contexts | accepted | 2026-11-07 | — | — | [adr-0002.md](./adr-0002.md) |
| ADR-0003 | PostgreSQL as the sole durable system of record | accepted | 2026-11-07 | — | — | [adr-0003.md](./adr-0003.md) |
| ADR-0004 | Transactional outbox with SKIP LOCKED instead of a broker | accepted | 2026-11-07 | — | — | [adr-0004.md](./adr-0004.md) |
| ADR-0005 | DeepSeek as the first LLM provider behind a port | accepted | 2026-11-07 | — | — | [adr-0005.md](./adr-0005.md) |
| ADR-0006 | Local embeddings with fastembed | accepted | 2026-11-07 | — | — | [adr-0006.md](./adr-0006.md) |
| ADR-0007 | Canvas 2D pixel office at 384x216 internal resolution | accepted | 2026-11-07 | — | — | [adr-0007.md](./adr-0007.md) |
| ADR-0008 | Angular 22 zoneless with signals | accepted | 2026-11-07 | — | — | [adr-0008.md](./adr-0008.md) |
| ADR-0009 | Server-enforced Kanban invariants | accepted | 2026-11-07 | — | — | [adr-0009.md](./adr-0009.md) |
| ADR-0010 | Fixed agent roles and a finite character roster | accepted | 2026-11-07 | — | — | [adr-0010.md](./adr-0010.md) |
| ADR-0011 | Separation of duties for review and merge | accepted | 2026-11-07 | — | — | [adr-0011.md](./adr-0011.md) |
| ADR-0012 | Ephemeral sandboxed runner containers | accepted | 2026-11-07 | — | — | [adr-0012.md](./adr-0012.md) |
| ADR-0013 | Login-less local-first with a token for remote exposure | accepted | 2026-11-07 | — | — | [adr-0013.md](./adr-0013.md) |
| ADR-0014 | WebSocket with resumable sequence and event backfill | accepted | 2026-11-07 | — | — | [adr-0014.md](./adr-0014.md) |
| ADR-0015 | Structured outputs validated against JSON Schema | accepted | 2026-11-07 | — | — | [adr-0015.md](./adr-0015.md) |
| ADR-0016 | Three-tier memory with SQL-level scope isolation | accepted | 2026-11-07 | — | — | [adr-0016.md](./adr-0016.md) |
| ADR-0017 | Budgets as first-class hard limits | accepted | 2026-11-07 | — | — | [adr-0017.md](./adr-0017.md) |
| ADR-0018 | Autonomy levels A0 through A3 | accepted | 2026-11-07 | — | — | [adr-0018.md](./adr-0018.md) |
| ADR-0019 | Conventional Commits with AI-Lab trace trailers | accepted | 2026-11-07 | — | — | [adr-0019.md](./adr-0019.md) |
| ADR-0020 | Docker Compose as the supported deployment | accepted | 2026-11-07 | — | — | [adr-0020.md](./adr-0020.md) |
| ADR-0021 | Office engine decoupled from Angular | accepted | 2026-11-07 | — | — | [adr-0021.md](./adr-0021.md) |
| ADR-0022 | Version pinning, Renovate, SBOM and no EOL dependencies | accepted | 2026-11-07 | — | — | [adr-0022.md](./adr-0022.md) |
| ADR-0023 | Runner control-plane API | accepted | 2026-11-07 | — | — | [adr-0023.md](./adr-0023.md) |
| ADR-0024 | TypeScript 6.0.x for Angular 22.2 compatibility | accepted | 2026-11-07 | — | — | [adr-0024.md](./adr-0024.md) |
| ADR-0025 | Orchestrator as the delivery-card coordinator | accepted | 2026-11-07 | — | — | [adr-0025-orchestrator-card-coordinator.md](./adr-0025-orchestrator-card-coordinator.md) |
| ADR-0026 | BDD E2E suite with Playwright-BDD and a make-only gate | accepted | 2026-11-09 | — | — | [adr-0026-bdd-e2e-suite.md](./adr-0026-bdd-e2e-suite.md) |
| ADR-0027 | Docker-only development toolchain | accepted | 2026-11-10 | — | — | [adr-0027-docker-only-toolchain.md](./adr-0027-docker-only-toolchain.md) |

## Rules

1. Every file matching `.docs/adr/adr-*.md` appears exactly once in this table.
2. Every row points to an existing file.
3. New ADRs use the next four-digit number and are added in ascending order.
4. Architectural decisions in code or docs without an ADR are a review defect.
5. The registry is referenced by `AGENTS.md` and is part of the harness validation.
