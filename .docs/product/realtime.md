# AI-Lab Realtime

| Field | Value |
|---|---|
| Document | `.docs/product/realtime.md` |
| Version | 1.0.0 |
| Status | Accepted |
| Owner | Dev |
| Related | `.docs/product/api/README.md`, `crates/api/src/lib.rs`, `.docs/product/eventing.md` |

The realtime contract is a WebSocket stream of persisted events with resumable sequencing and an HTTP backfill fallback.

## 1. Envelope

```json
{"type":"event","event":"kanban.card.moved","event_id":"…","seq":42,"lab_id":"…","occurred_at":"…","payload":{}}
```

The client sends `subscribe`/`unsubscribe` with topics such as `lab:{id}`, `card:{id}`, `run:{id}`, `system`. The server sends `hello` with `server_time`, `seq` and `topics`.

## 2. Resume and backfill

- Clients reconnect with their last applied `seq`.
- `events_after(events, since)` filters `seq > since`, removes duplicate `event_id` values and sorts by `seq`.
- A gap larger than the in-memory buffer falls back to `GET /api/v1/events?since={seq}`.
- State events are persisted and replayable; ephemeral events are coalesced and never persisted.

## 3. Backpressure and heartbeat

- Per-connection buffer: 256 messages; drop ephemeral first, then close with `1013` and let the client resume.
- Server ping every 20 s; the client pongs within 10 s.
- `system.health.changed` is emitted every 15 s.

## 4. Tests

`scripts/run-rust.sh cargo test -p ailab-api` covers the envelope round-trip, `events_after` filtering/dedupe/order, pagination, rate limits and origin/JSON guards.

## 5. Deferred to the WP-16 follow-up

- Axum WS hub, Redis fan-out and outbox relay wiring.
- DTOs, auth middleware and the Swagger UI.
- Realtime performance suite (event → render p95 ≤ 250 ms) in WP-21.

## 6. References

- `.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §15.2
- `.docs/technical-specification/APPENDIX-B-CONTRACTS.md` §B.2
- `crates/api/src/lib.rs`
