---
name: Release Management
description: Use for versioning, changelog, release candidate verification, SBOM, image signing, publishing, rollback and post-release review.
---

# Release Management

## When to use

- A version is being cut or a release candidate is being verified.
- The changelog, release notes or migration guide must be produced.
- Images must be built, scanned, signed and published.
- A rollback or post-release review is required.

## Preconditions

- Read `.docs/sow/WP-22-DOCS-RELEASE.md` and `.docs/technical-specification/APPENDIX-D-DELIVERY.md` §D.5 and §D.6.
- All work packages required for the release are `done`.
- No P0 or P1 is open.
- The acceptance report evidence is available.

## Workflow

1. Freeze: confirm all required WPs are `done` and all acceptance criteria are evidenced.
2. Choose the semantic version: `MAJOR.MINOR.PATCH`. Breaking contract changes require a major version and a new API version.
3. Generate the changelog from Conventional Commits with `git-cliff` and curate breaking changes and migration notes.
4. Build reproducibly:
   - `cargo build --release --locked`
   - `npm --prefix web ci`
   - `npm --prefix web run build`
   - `docker compose build`
5. Verify:
   - full CI, E2E, load, soak, security scans, restore drill;
   - SBOM per image;
   - cosign signature;
   - checksums.
6. Tag `vX.Y.Z` and publish the release with notes, SBOMs, checksums, compose file and upgrade guide.
7. Deploy to staging, run the golden demo, monitor for 24 hours.
8. Verify rollback: previous tags retained, migrations are expand/contract for one version, rollback runbook tested.
9. Schedule and complete 7-day and 30-day post-release reviews.
10. Record the release in the changelog and the acceptance report.

## Checklist

- [ ] All required WPs `done`.
- [ ] No P0 or P1 open.
- [ ] Version chosen and documented.
- [ ] Changelog curated with breaking changes.
- [ ] Images reproducible and pinned.
- [ ] SBOM, signature and checksums present.
- [ ] Full gate suite green.
- [ ] Acceptance report signed.
- [ ] Golden demo recorded.
- [ ] Rollback verified.
- [ ] Post-release reviews scheduled.

## Anti-patterns

- Releasing with an open P0 or P1.
- Using floating image tags.
- Publishing without a signed SBOM.
- Skipping the restore drill.
- Writing release notes by hand without the changelog.
- Changing migrations without a rollback path.
- Tagging a commit that is not the one that passed CI.

## References

- `../../../.docs/sow/WP-22-DOCS-RELEASE.md`
- `../../../.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §21, §22, §23
- `../../../.docs/technical-specification/APPENDIX-D-DELIVERY.md` §D.5, §D.6
- `../../../.docs/adr/registry.md`

## Supporting files

- None.
