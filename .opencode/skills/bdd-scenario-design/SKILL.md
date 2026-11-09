---
name: BDD Scenario Design
description: Use when writing, refining or reviewing AI-Lab Gherkin scenarios, mapping requirement IDs to E2E IDs, ordering them from simple to complex, or closing specification gaps in the BDD checklist.
---

# BDD Scenario Design

## When to use

- Add, change or review scenarios in `.docs/e2e/BDD.md`.
- Map `FR-*`, `UX-*`, `SEC-*`, `OPS-*`, `NFR-*` and `AC-*` requirements to `E2E-XXX` scenarios.
- Decide whether a scenario is automated, `@manual` or `@live`.
- Close a gap found by a specification cross-check.
- Review a scenario for observability, determinism and single-behaviour scope.

Use `bdd-step-engineering` to implement the steps and `bdd-coverage-traceability` to prove coverage. Use `bdd-e2e` for the end-to-end workflow.

## Preconditions

- Read the relevant specification sections, Appendix B and Appendix C, and the matching product document.
- Read `.docs/e2e/BDD.md` and confirm the current highest `E2E-XXX` id.
- Read `.docs/e2e/req.md`; it is the manual setup checklist, not a scenario source.
- Know the fake harness, the deterministic seed and the checklist marker rules.
- Know the current area, priority and traceability tags.

## Workflow

1. Identify the use case and the normative requirement IDs it verifies.
2. Write one observable behaviour per scenario with `Given`, `When`, `Then` and optional `And`.
3. Make the `Then` assertions observable: a response, a persisted field, an emitted event, a UI projection, an audit row or a metric. Never assert private methods, CSS classes or implementation details.
4. Add tags:
   - area tag such as `@kanban`, `@memory`, `@office`, `@security`;
   - priority `@p0`, `@p1` or `@p2`;
   - traceability tags such as `@fr-kan-020`, `@ux-016`, `@sec-004`, `@ac-007`;
   - `@manual` with a reason when it cannot be automated;
   - `@live` when it needs a real DeepSeek or GitHub credential; the DeepSeek key is loaded with `make bdd-set-api-key`, never from a file or an argument.
5. Place the scenario in the matching area and keep the simple-to-complex order. New gaps use the next free id and are never renumbered.
6. Use a deterministic fixture or the demo seed; never depend on wall-clock time, execution order or a real external service.
7. For a `@manual` scenario, record the owner, reason, required human evidence and review date in the manual-scenario map.
8. Update the coverage summary and the progress tracker when an area count changes. Add a gap-analysis row when a scenario closes a specification gap.
9. Run `.tools/check-bdd-coverage.mjs` and `.tools/gen-bdd-features.mjs --check`.

## Checklist

- [ ] Scenario id is unique and is the next free id.
- [ ] Scenario has exactly one observable behaviour.
- [ ] `Then` assertions are observable and deterministic.
- [ ] Tags include area, priority and at least one requirement id.
- [ ] `@manual` or `@live` has a reason and an owner.
- [ ] No hardcoded URL, port, token, secret or environment default.
- [ ] Data comes from the seed or an isolated fixture.
- [ ] The scenario keeps the simple-to-complex order.
- [ ] Coverage summary and progress tracker are updated.
- [ ] Generator drift and coverage checks pass.

## Anti-patterns

- Packing several behaviours into one scenario to reduce the count.
- Asserting a database column name, a private function or a CSS class instead of observable behaviour.
- Vague outcomes such as "works well" or "looks correct".
- Renumbering or deleting an id to make a gap disappear.
- Adding a scenario without a requirement tag.
- Depending on the real DeepSeek or GitHub API in a default scenario.
- Hardcoding hostnames, ports, versions or secrets in feature text.
- Duplicating a scenario instead of adding a parameterised example.

## References

- `../../../.docs/e2e/BDD.md`
- `../../../.docs/e2e/req.md`
- `../../../.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md`
- `../../../.docs/technical-specification/APPENDIX-B-CONTRACTS.md`
- `../../../.docs/technical-specification/APPENDIX-C-AGENTS-KANBAN.md`
- `../../../.docs/product/testing.md`
- `../bdd-e2e/SKILL.md`

## Supporting files

- `../../../.prompts/bdd-suite-test-create.md`
