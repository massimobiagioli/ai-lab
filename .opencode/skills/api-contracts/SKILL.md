---
name: API & Event Contracts
description: Use when changing REST endpoints, DTOs, OpenAPI, WebSocket events, AsyncAPI or their compatibility, versioning and tests.
---

# API & Event Contracts

## When to use

- A REST route, request, response, error code or pagination behavior changes.
- A WebSocket event, topic, envelope or sequence behavior changes.
- OpenAPI or AsyncAPI must be regenerated or checked for compatibility.
- A client depends on a contract that is about to change.

## Preconditions

- Read `.docs/technical-specification/APPENDIX-B-CONTRACTS.md` and `.docs/sow/WP-16-API-REALTIME.md`.
- Know the current contract version and the consumers.
- Confirm the change is additive within `/api/v1`; breaking changes require `/v2` and an ADR.

## Workflow

1. Write the contract first: update the DTO, OpenAPI annotation or event schema.
2. Write a failing contract test:
   - schemathesis against the OpenAPI document;
   - a unit test for the error code and shape;
   - a WebSocket test for subscribe, replay, gap resync and backpressure.
3. Implement the handler, event or fan-out change.
4. Regenerate the artifacts:
   - `make openapi`
   - `make asyncapi`
   - `npm --prefix web run generate:api`
5. Verify determinism: running generation twice produces no diff.
6. Run compatibility checks:
   - `openapi-diff` or the project compatibility job;
   - AsyncAPI schema validation;
   - client type check.
7. Run the contract suite:
   - `cargo nextest run -p ailab-api`
   - `npx --prefix web playwright test`
   - `schemathesis run docs/api/openapi.json`
8. Update the error catalog and the event catalog if a new code or event was added.
9. Record the change in the WP progress line and, if architectural, in an ADR.

## Checklist

- [ ] Contract updated before implementation.
- [ ] Request and response schemas validated.
- [ ] Unknown fields rejected on writes.
- [ ] Error uses RFC 9457 with a stable code and `trace_id`.
- [ ] Pagination uses a stable cursor.
- [ ] Idempotency covered where required.
- [ ] WebSocket sequence, replay and backpressure covered.
- [ ] OpenAPI and AsyncAPI regenerated deterministically.
- [ ] Compatibility check green.
- [ ] Generated client types updated.
- [ ] Contract tests green.
- [ ] Documentation and examples updated.

## Anti-patterns

- Changing a field name or removing a field within `/api/v1`.
- Returning a different error shape per endpoint.
- Hand-editing generated OpenAPI or client types.
- Adding an event without catalog and AsyncAPI updates.
- Using offset pagination on a large table.
- Making a POST idempotent only in the client.

## References

- `../../../.docs/technical-specification/APPENDIX-B-CONTRACTS.md`
- `../../../.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §15
- `../../../.docs/sow/WP-16-API-REALTIME.md`
- `../../../.docs/sow/WP-17-FRONTEND-PLATFORM.md`

## Supporting files

- None.
