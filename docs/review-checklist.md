# AI-Lab Review Checklist

Use this checklist for every pull request and every work-package review. A reviewer cites the principle and the evidence when requesting changes.

## Correctness

- [ ] The change matches the card and the WP contract.
- [ ] Acceptance criteria are evidenced by tests, commands, screenshots or run IDs.
- [ ] Domain invariants are enforced server-side; invalid states are rejected.
- [ ] Optimistic concurrency and idempotency are handled where relevant.

## Scope and design

- [ ] No unrequested scope; out-of-scope items are documented.
- [ ] One canonical location per fact; no duplicated DTOs or schemas.
- [ ] Context boundaries are respected; no cross-context table access.
- [ ] Contracts (REST, WS, events, runner) are versioned and documented.

## Tests

- [ ] A failing test existed before the production change (TDD).
- [ ] Unit, integration and contract tests cover the change.
- [ ] No real DeepSeek or GitHub calls in tests.
- [ ] Coverage thresholds met; flaky tests are not hidden.

## Security

- [ ] No secret in the diff, logs, events, prompts or fixtures.
- [ ] Input validation at the boundary; deny-by-default policy.
- [ ] Sandbox and exposure controls are unchanged or strengthened.
- [ ] Security scans are green.

## Performance and reliability

- [ ] Budgets from the specification are respected.
- [ ] Failure modes and retries are defined.
- [ ] Observability (logs, metrics, traces) is added where needed.

## Documentation and release

- [ ] Docs, ADRs and indexes are updated.
- [ ] Changelog entry exists for a user-visible change.
- [ ] Rollback or migration path is documented.
- [ ] No P0/P1 open; reviewer ≠ author; merger ≠ author.
