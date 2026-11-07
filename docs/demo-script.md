# AI-Lab Golden Demo (≤ 5 minutes)

| Field | Value |
|---|---|
| Document | `docs/demo-script.md` |
| Version | 1.0.0 |
| Status | Accepted |
| Owner | Analyst |
| Related | `docs/operations.md`, `docs/testing.md`, `docs/product-ui.md` |

Prerequisites: `make up && make seed-demo`; open `http://127.0.0.1:8080`; the stack is healthy.

| Time | Action | Expected |
|---|---|---|
| 0:00 | Dashboard loads | Pixel office preview, flow KPIs (lead/cycle p50, WIP, throughput, blocked), flow-health badge |
| 0:30 | Open the office | Four agents at their desks; idle/working states match the domain; no invented activity |
| 1:00 | Open the board | Six fixed columns; 12 demo cards across TODO/READY/DOING/REVIEW/DONE |
| 1:30 | Move a READY card | Guarded transition; an invalid move returns the violation; the audit event is visible |
| 2:00 | Open a card inspector | Story, acceptance criteria, deliverables, assignee, reviewer, due date |
| 2:30 | Start a run | Run appears in DOING; plan/tool steps stream; cost and budget are visible |
| 3:00 | Review and merge | QA verdict, green checks and separation of duties; squash merge records the SHA; the card is DONE |
| 3:30 | Open memory | Scoped search returns citations from the demo SOW; no cross-scope hit |
| 4:00 | Open the SOW/plan | Approved SOW and acyclic WBS; materialized cards link back to the SOW |
| 4:30 | Metrics and audit | Flow metrics, cost ledger and audit trail agree with the board |

Expected timings are targets; the CI seed and the deterministic data make the run reproducible.

**Current state.** The backend contracts, database seed, Compose stack, dashboard and office slices are delivered. The full board/SOW/memory/settings pages and Playwright journeys are in the WP-19/WP-21 follow-ups; the demo steps above describe the target golden path and must be re-recorded when those pages land.

## References

- `.docs/technical-specification/APPENDIX-D-DELIVERY.md` §D.6
- `scripts/seed-demo.sql`
