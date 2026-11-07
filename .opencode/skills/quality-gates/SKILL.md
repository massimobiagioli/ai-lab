---
name: Quality Gates
description: Use before declaring any task or work package done, or when changing CI, tests, coverage, load, chaos, fuzz or acceptance evidence.
---

# Quality Gates

## When to use

- A task or WP is about to be marked `done`.
- CI, test suites, coverage thresholds, load, chaos or fuzz change.
- A release candidate must be verified.
- A flaky test must be quarantined or fixed.

## Preconditions

- Read `.docs/sow/WP-21-TESTING.md`, `.docs/sow/WP-00-GOVERNANCE.md` §3.8 and `.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §19.
- Know which gates apply to the changed area.
- Have the deterministic seed and fakes available.

## Workflow

1. Run the fast local gate:
   - `make check`
   - `scripts/harness-lint.sh`
2. Run the relevant suites:
   - `cargo nextest run --workspace`
   - `npm --prefix web test -- --run`
   - `npx --prefix web playwright test`
3. Run contract and invariant suites:
   - `make openapi`
   - `make asyncapi`
   - `schemathesis run docs/api/openapi.json`
   - Kanban 36-case matrix
   - memory leakage fixture
4. Run security gates:
   - `gitleaks detect --source . --no-git`
   - `cargo deny check`
   - `npm --prefix web audit --audit-level=high`
   - Trivy on changed images
5. Run performance and resilience gates when affected:
   - `make load`
   - soak test
   - chaos scenario
   - office FPS soak
6. Verify coverage thresholds: backend line at least 80 percent, domain at least 90 percent, frontend at least 80 percent, new code at least 80 percent.
7. Check that no test calls the real DeepSeek or GitHub API.
8. Quarantine a flaky test within 24 hours with an owner and a card; fix or delete within 5 days.
9. Collect evidence and attach it to the WP or card.

## Checklist

- [ ] Fast gate green.
- [ ] Unit, integration and repository suites green.
- [ ] Frontend unit, E2E and visual suites green.
- [ ] Contract and invariant suites green.
- [ ] Security gates green.
- [ ] Coverage thresholds met.
- [ ] No real external API in tests.
- [ ] Load, soak and chaos green where affected.
- [ ] Flaky tests quarantined with an owner.
- [ ] Evidence attached.
- [ ] No P0 or P1 open.

## Anti-patterns

- Declaring done from a partial run.
- Retrying a flaky test until it passes.
- Lowering coverage to unblock a merge.
- Skipping security scans for speed.
- Using production credentials in tests.
- Attaching a screenshot from a different commit.
- Ignoring a performance regression below the threshold.

## References

- `../../../.docs/sow/WP-21-TESTING.md`
- `../../../.docs/sow/WP-00-GOVERNANCE.md` §3.8
- `../../../.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §19
- `../../../.docs/technical-specification/APPENDIX-D-DELIVERY.md` §D.2

## Supporting files

- None.
