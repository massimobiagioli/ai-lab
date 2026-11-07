# AI-Lab Contracts

| Field | Value |
|---|---|
| Document | `docs/contracts.md` |
| Version | 1.0.0 |
| Status | Accepted |
| Owner | Dev |
| Source of truth | `.docs/technical-specification/APPENDIX-B-CONTRACTS.md`, spec §15 |
| Related | `docs/architecture.md`, `docs/domain-model.md`, `.docs/sow/WP-16-API-REALTIME.md` |

This document is the inter-service and edge contract matrix. The normative schemas are `docs/api/openapi.json` and `docs/api/asyncapi.json` once WP-16 lands; until then this document and Appendix B are the contract. A contract change requires the owning WP to bump its version and notify consumers in the same change.

---

## 1. Communication matrix

| From → To | Mechanism | Contract | Idempotency | Timeout |
|---|---|---|---|---|
| Browser → `web`/`api` | HTTPS REST | OpenAPI 3.1 | `Idempotency-Key` on POST | 30 s (120 s SOW) |
| Browser ↔ `api` | WSS | AsyncAPI 3.0 | `event_id` + `seq` | heartbeat 20 s |
| `api` → contexts | in-process application services | Rust traits | transaction | — |
| Any context → `outbox` | same DB transaction | `outbox` row | `event_id` | — |
| `relay` → Redis | pub/sub | event envelope | `event_id` dedupe | 5 s |
| `relay` → WS clients | in-process fan-out | AsyncAPI | monotonic `seq` | 250 ms p95 |
| `worker` → `postgres` | sqlx | SQL | lease and unique constraints | 5 s read / 30 s write |
| `worker` → `runner` | private HTTP + shared token | Runner API §2 | `run_id` | 30 s create / stream |
| `worker` → DeepSeek | HTTPS | OpenAI-compatible | request ID | 120 s |
| `worker` → GitHub | HTTPS REST v3 | GitHub API | idempotency key | 30 s |
| GitHub → `api` | webhook | HMAC | `X-GitHub-Delivery` | 5 s ack |

**No direct `api` ↔ `worker` HTTP calls.** Coordination is exclusively through Postgres state/outbox and Redis ephemeral fan-out.

## 2. Runner control-plane API (normative)

Private HTTP on the `runner` network, authenticated by `AILAB_RUNNER_TOKEN` (≥ 32 bytes), bound to the internal interface only.

| Method | Path | Purpose | Request | Response |
|---|---|---|---|---|
| POST | `/v1/sandboxes` | Create a sandbox for a run | `{run_id, image, worktree_ref, cpu, memory_mb, timeout_s, egress}` | `{sandbox_id, container_id, created_at}` |
| GET | `/v1/sandboxes/{id}` | Status | — | `{status, exit_code, started_at, ended_at}` |
| GET | `/v1/sandboxes/{id}/logs?follow=true` | Stream logs (SSE) | — | log chunks, 20 MB cap |
| POST | `/v1/sandboxes/{id}/exec` | Run an allowlisted command | `{argv, cwd, timeout_s, env_allowlist}` | `{exit_code, stdout, stderr, duration_ms}` |
| DELETE | `/v1/sandboxes/{id}` | Kill and remove | `{reason}` | `204` |
| GET | `/health/live`, `/health/ready` | Liveness/readiness | — | `200`/`503` |

Rules:

1. Creation is idempotent by `run_id`; a second create returns the existing sandbox.
2. The runner never mounts the Docker socket into a sandbox.
3. Every call is logged with `run_id` and `trace_id`.
4. `image` must be a digest-pinned allowlisted image; `egress` must be a named policy.
5. `exec` rejects any command not in the role tool policy.

The consumer is WP-13; the implementation is WP-14. A contract-compatibility test is required before either WP closes. The contract is recorded as [ADR-0023](../.docs/adr/adr-0023.md).

## 3. Edge REST contract

- Base path `/api/v1`; JSON only except markdown/PDF artifacts and event streams.
- Errors use RFC 9457 with `type`, `title`, `status`, `detail`, `instance`, `code`, `violations[]`, `trace_id`.
- Correlation: `X-Request-Id` and `traceparent`.
- Versioning: additive changes are non-breaking; removals require `/v2`.
- CORS: same-origin by default; exact-match allowlist only when configured; no wildcard with credentials.
- Rate limits: 120 req/min reads, 30 req/min mutations, 5 req/min idea/SOW generation; `429` includes `Retry-After`.
- Request size: JSON ≤ 2 MB; memory import ≤ 10 MB.
- Lists: `?limit&cursor&sort&filter[...]` returning `{data, next_cursor, total_estimate}`.

The full endpoint list is in the specification §15.1 and Appendix B.1; WP-16 owns the OpenAPI file.

## 4. WebSocket contract

- Connect `GET /api/v1/ws?since={seq}`; server sends `hello` with `server_time`, `seq`, `topics`.
- Topics: `lab:{id}`, `card:{id}`, `run:{id}`, `system`.
- Envelope: `{type, event, event_id, seq, lab_id, occurred_at, payload}`.
- Ephemeral events are coalesced and never persisted; state events are persisted and replayable.
- Backpressure: 256-message buffer; drop ephemeral first, then close `1013` and resume from `last_seq`.
- Heartbeat: server ping every 20 s; client pong within 10 s.
- Reconnect: exponential backoff with jitter, max 30 s; gap larger than the buffer falls back to `GET /api/v1/events?after=`.

## 5. Event envelope and catalog

Envelope fields: `event_id`, `event`, `seq`, `lab_id`, `occurred_at`, `actor_type`, `actor_id`, `payload`, `schema_version`.

The core catalog is in `docs/domain-model.md` §8. Producers publish to the outbox in the same transaction as the state change; the relay delivers at-least-once; consumers deduplicate by `event_id` and apply by `seq`.

## 6. Idempotency and retries

| Operation | Key | Retry |
|---|---|---|
| Create card / SOW / plan | `Idempotency-Key` header | client-controlled |
| Sandbox create | `run_id` | safe |
| LLM request | request ID | bounded retries with backoff |
| GitHub write | idempotency key | bounded retries |
| Webhook | `X-GitHub-Delivery` | duplicate returns 202 once |
| Outbox delivery | `event_id` | at-least-once, consumer dedupe |

## 7. Compatibility and change control

1. The owning WP bumps the contract version and updates OpenAPI/AsyncAPI in the same change.
2. Consumers add a compatibility test; `openapi-compat` fails on drift.
3. Removing or renaming a field requires `/v2` or an expand/contract migration.
4. The runner API is versioned by path (`/v1`); a breaking change requires `/v2` and a coordinated rollout.

## 8. References

- `.docs/technical-specification/APPENDIX-B-CONTRACTS.md`
- `.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §15
- `.docs/sow/WP-13-ORCHESTRATION.md`, `.docs/sow/WP-14-SANDBOX-RUNNER.md`, `.docs/sow/WP-16-API-REALTIME.md`
- `docs/architecture.md`, `docs/domain-model.md`
