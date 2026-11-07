---
name: Documentation Sync
description: Use when creating or updating docs, skills, commands, ADRs, diagrams or AGENTS.md, or when validating links, indexes and screenshots.
---

# Documentation Sync

## When to use

- A document under `.docs/` is created, updated or deleted.
- A skill or command is created, renamed or removed.
- An ADR is created or superseded.
- A diagram or screenshot must be regenerated.
- `AGENTS.md` must reflect the current documentation and skills.

## Preconditions

- Read `.docs/CONSTITUTION.md` §9 and `.docs/sow/WP-22-DOCS-RELEASE.md`.
- Know the documentation manifest and the current indexes in `AGENTS.md`.
- Use the deterministic seed for screenshots and diagrams.

## Workflow

1. Identify every document, skill, command or ADR affected by the change.
2. Update the canonical file first; never duplicate content across files.
3. Update the `AGENTS.md` documentation index and skills index in the same change.
4. Update `.docs/adr/registry.md` when an ADR is added or superseded.
5. Regenerate diagrams and screenshots from source; do not hand-edit exports.
6. Run validation:
   - `scripts/harness-lint.sh`
   - `make sow-lint`
   - docs build and link check
   - visual screenshot comparison
7. Confirm every relative link resolves.
8. Confirm that no unresolved placeholder marker remains in harness files.
9. Confirm `AGENTS.md` contains only agent instructions and references `README.md` without duplicating it.
10. Record the sync in the WP progress line.

## Checklist

- [ ] Canonical file updated.
- [ ] `AGENTS.md` documentation index updated.
- [ ] `AGENTS.md` skills index updated.
- [ ] ADR registry updated.
- [ ] Diagrams regenerated from source.
- [ ] Screenshots regenerated from the deterministic seed.
- [ ] `scripts/harness-lint.sh` green.
- [ ] `make sow-lint` green.
- [ ] Docs build and link check green.
- [ ] No stale or duplicate content.
- [ ] `README.md` referenced, not duplicated.

## Anti-patterns

- Updating a doc without updating the index.
- Adding a skill without the `AGENTS.md` entry.
- Adding an ADR without the registry row.
- Hand-editing an exported SVG or a screenshot.
- Duplicating the README inside `AGENTS.md`.
- Leaving a broken relative link.
- Leaving a placeholder in a generated document.

## References

- `../../../.docs/CONSTITUTION.md` §9
- `../../../.docs/sow/WP-22-DOCS-RELEASE.md`
- `../../../.docs/sow/SOW.md`
- `../../../.docs/adr/registry.md`
- `../../../AGENTS.md`

## Supporting files

- None.
