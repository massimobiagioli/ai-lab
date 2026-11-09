# WP-22 — Documentation, Release & Handover QA report

Date: 2026-11-09. Reviewer: qa-review-agent. Author: Analyst/opencode-dev (separation of duties).

## Verdict

**NOT APPROVED.** The documentation package and handover are delivered, but the release evidence is missing (P15) and ten acceptance criteria still have gaps.

## Review

| Item | Result |
|---|---|
| Scope vs WP | Docs manifest, acceptance report, demo script, handover and review plan delivered; release/SBOM/signing is P15. |
| Acceptance criteria | AC-013/AC-014 evidenced; AC-001/002/003/004/005/006/008/009/010/011/012 partial; AC-007/AC-015 not evidenced. |
| Security | No secrets in the new docs; disclosure policy and support boundaries stated; `gitleaks`/`cargo deny` green. |
| Tests | Doc link check 171; harness, SOW lint and `make check` green; E2E 6/6, k6 0 % failed, chaos green. |
| Docs | Manifest audited (19 artifacts, 25 ADRs, 11 runbooks); `testing.md`, `acceptance-report.md`, `handover.md` added; product index updated. |
| Gates | Documentation gates green; release, SBOM, signing and the acceptance sign-off are open. |

## Blockers

- P15: SBOM per image, cosign signatures, release 1.0, changelog and rollback evidence.
- AC-001: timed clean-clone log and video.
- AC-005: office FPS report and screenshots.
- AC-007: live run trace and merge SHA.
- AC-008: nDCG report.
- AC-012: seven dashboards and the synthetic alert (WP-24 T07).
- AC-015: demo recording and timings.

## Next

Re-review after P15 and the named evidence; only then mark WP-22 `done`.
