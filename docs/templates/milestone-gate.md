# Milestone {M0..M8} Gate — {Name}

| Field | Value |
|---|---|
| Milestone | `{M0..M8}` |
| Date | {YYYY-MM-DD} |
| Tech Lead | {actor} |
| QA | {actor} |
| Human Operator | {actor or pending} |

A milestone is `done` only when every box below is checked and every item links to evidence.

## 1. Scope

| WP | Status | Evidence |
|---|---|---|
| {WP-XX} | {done} | {link} |

## 2. Demo

- [ ] Demo recording or live demo completed in ≤ 30 minutes.
- [ ] Golden path reproduced from the deterministic seed.
- [ ] Every demonstrated state matches persisted domain truth.
- [ ] Recording link: {link}

## 3. Acceptance evidence

- [ ] Every acceptance test for the milestone is green.
- [ ] Test report link: {link}
- [ ] No regression against a prior milestone acceptance test.
- [ ] No open P0/P1.

## 4. Quality gates

- [ ] `scripts/harness-lint.sh` green.
- [ ] `scripts/sow-lint.sh` green.
- [ ] Required CI checks green on the milestone commit.
- [ ] Security scans clean.
- [ ] Coverage thresholds met.

## 5. Documentation

- [ ] Documentation manifest entries for the milestone are current.
- [ ] Docs build and link check green.
- [ ] Screenshots and GIFs regenerated from the seed.
- [ ] ADRs recorded for every material decision.
- [ ] Changelog and release notes drafted.

## 6. Retrospective

- [ ] Retrospective held.
- [ ] Exactly three actions with owners, due dates and `TODO` cards.
- [ ] Retro record link: {link}

## 7. Sign-off

| Role | Name | Decision | Date |
|---|---|---|---|
| Tech Lead | {name} | {accept|reject} | {YYYY-MM-DD} |
| QA | {name} | {accept|reject} | {YYYY-MM-DD} |
| Human Operator | {name} | {accept|reject|pending} | {YYYY-MM-DD} |

## 8. Residual risks and follow-ups

| Item | Owner | Due | Card |
|---|---|---|---|
| {item} | {owner} | {date} | {key} |
