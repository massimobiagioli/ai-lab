# API and Event Contracts

Owned by WP-16. This directory holds the canonical REST and event contracts.

| File | Purpose |
|---|---|
| `openapi.source.json` | Canonical OpenAPI 3.1 source |
| `openapi.json` | Generated contract (deterministic copy) |
| `asyncapi.source.json` | Canonical AsyncAPI 3.0 source |
| `asyncapi.json` | Generated contract (deterministic copy) |

```bash
make openapi
make asyncapi
```

The generation scripts copy the source to the generated file, so the output is deterministic and CI fails on drift. The HTTP server, DTOs, auth middleware and the utoipa-based generator are delivered in the WP-16 follow-up; the contract is the source of truth until then.

Core REST paths: health, providers, models, projects, board, cards, transition, events backfill and metrics. Core event channels: inventory, kanban, delivery, memory and system health. Errors use RFC 9457 with a stable `code` and `violations[]`.

The contract is validated by `cargo test -p ailab-api` and the `openapi-compat` CI job.
