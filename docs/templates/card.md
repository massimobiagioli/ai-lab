# {CARD_KEY} — {Imperative title, ≤ 80 chars}

| Field | Value |
|---|---|
| WP | `WP-{NN}` |
| Column | `TODO` |
| Priority | `must|should|could` |
| Estimate | `{1|2|3|5|8}` |
| Assignee | `{agent or human}` |
| Reviewer | `{agent or human, not the author}` |
| Due date | `{YYYY-MM-DDThh:mm:ssZ}` |
| Source | `human|agent` |
| Budget (USD) | `{amount > 0}` |
| Dependencies | `{card keys or —}` |

## Story

As a {persona},
I want {capability},
so that {business outcome}.

### Context

{2–6 sentences of context, links and constraints.}

## Acceptance criteria

Given {precondition}
When {action}
Then {observable outcome}
And {observable outcome}

## Deliverables

- {artifact path or named artifact} — {what "done" looks like}

## Out of scope

- {explicit exclusion}

## Risks / dependencies

- {risk} → {mitigation}

## Definition of Ready (all required before READY)

- [ ] Canonical story complete.
- [ ] At least one Given/When/Then criterion.
- [ ] At least one concrete deliverable.
- [ ] Estimate in {1,2,3,5,8}; priority set and not `wont`.
- [ ] Assignee set and role-compatible.
- [ ] Due date set and in the future.
- [ ] Dependencies resolved or waived by the Tech Lead with a reason.
- [ ] WP is `progress`; budget greater than zero; source recorded.
- [ ] `needs_refinement = false`.

## Definition of Done (all required before DONE)

- [ ] PR merged; merge SHA stored.
- [ ] Required CI checks green; new-code coverage ≥ 80% or approved waiver.
- [ ] Every acceptance criterion evidenced.
- [ ] Docs and changelog updated.
- [ ] No P0/P1; security scans clean; no secret in the diff.
- [ ] QA verdict `APPROVED`; reviewer is not the author.
- [ ] Card log has the final summary, decisions and residual risks.
- [ ] CV evidence appended.

## Log

| Date | Actor | Note |
|---|---|---|
| {YYYY-MM-DD} | {actor} | {note} |
