---
name: ADR Authoring
description: Use when an architectural or irreversible decision is made, when an accepted decision is superseded, or when the ADR registry must be updated.
---

# ADR Authoring

## When to use

- A decision changes architecture, technology, contracts, data ownership, security posture or deployment.
- A previous ADR is superseded or deprecated.
- A WP task creates a decision that future agents must follow.

Do not use an ADR for a reversible implementation detail that is already covered by the spec or a skill.

## Preconditions

- Read `.docs/adr/registry.md` and the ADRs related to the decision.
- Read the relevant technical specification section and WP.
- Confirm the decision is not already accepted and does not contradict a higher source.

## Workflow

1. Choose the next four-digit ID: `adr-0023`, `adr-0024`, and so on. Never reuse or renumber.
2. Create `.docs/adr/adr-nnnn.md` with:
   - title `# ADR-nnnn — <Title>`;
   - table with `Status`, `Date`, `Deciders`, `Supersedes`, `Superseded by`;
   - `## Context`, `## Decision`, `## Consequences`, `## Alternatives considered`, `## References`.
3. Set status `accepted` only after Tech Lead approval; otherwise `proposed`.
4. If superseding an old ADR, update the old ADR's `Superseded by` field and its status to `superseded`; do not delete it.
5. Add the new row to `.docs/adr/registry.md` in ascending order with a correct relative link.
6. Reference the ADR from the relevant WP progress log and, when it changes navigation, from `AGENTS.md` through the registry.
7. Run the ADR checks:
   - every file has the required fields;
   - every file appears in the registry;
   - every registry row points to an existing file;
   - no broken links.
8. Record the decision in the WP progress line.

## Checklist

- [ ] ID is the next sequential number.
- [ ] Context explains the forces and constraints.
- [ ] Decision is stated in the imperative and is unambiguous.
- [ ] Consequences include trade-offs, not only benefits.
- [ ] Alternatives and rejection reasons are recorded.
- [ ] Status and date are correct.
- [ ] Supersession fields are consistent in both ADRs.
- [ ] Registry updated with a working link.
- [ ] References point to existing files.
- [ ] No contradiction with the technical specification.

## Anti-patterns

- Editing an accepted ADR's decision instead of superseding it.
- Reusing or renumbering ADR IDs.
- Writing a decision without alternatives.
- Adding a registry row without a file, or a file without a row.
- Using an ADR for a trivial naming choice.
- Hiding a breaking change in a code comment instead of an ADR.

## References

- `../../../.docs/adr/registry.md`
- `../../../.docs/sow/WP-01-ARCHITECTURE.md` §3.12
- `../../../.docs/technical-specification/APPENDIX-D-DELIVERY.md` §D.3
- `../../../.docs/CONSTITUTION.md` §5, §10

## Supporting files

- None.
