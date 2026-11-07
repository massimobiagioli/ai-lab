# AI-Lab Glossary

| Term | Meaning |
|---|---|
| Lab | Top-level workspace with exactly one active Tech Lead, agents, projects and memory. |
| Agent | Persistent employee with a fixed role, a character and append-only CV versions. |
| Character | Cosmetic/visual persona from the finite 10-character roster; never a permission. |
| CV | Versioned evidence of an agent's skills, delivered cards and review stats. |
| Project | GitHub-connected unit of work; owns a board and a project-local card sequence. |
| Provider / Model | DeepSeek endpoint and its model catalog (`deepseek-flash` default, `deepseek-v4-pro` escalation). |
| SOW | Versioned Statement of Work with 17 mandatory sections and semantic validation. |
| Plan / WBS | Acyclic task graph that materializes leaf tasks into Kanban cards. |
| Card | Unit of work with story, acceptance criteria, deliverables, estimate, budget and column. |
| Run | One attempt by one agent on one card, traced by ordered run steps. |
| Review | Independent QA verification; verdict `APPROVED` or `CHANGES_REQUESTED`. |
| Merge | Squash merge of an approved PR; reviewer/merger differ from the author. |
| Column | `TODO`, `READY`, `DOING`, `REVIEW`, `DONE`, `ICEBOX`. |
| WIP | Cards in `DOING` + `REVIEW`, bounded by limits. |
| Autonomy | `A0` human approves everything; `A1` human merges; `A2` agent merges with QA/CI/budget; `A3` wider scope. |
| Memory scope | Lab, project or agent; there is no global scope. |
| Outbox | Transactional table written with the aggregate; relay publishes at-least-once. |
| Seq | Per-lab monotonic event sequence used for ordering and resume. |
| Budget | Per-card/day hard limit in micro-USD. |
| Evidence | A test, benchmark, screenshot, scan report, ADR or run ID that proves a claim. |

## References

- `.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §26
- `.docs/technical-specification/APPENDIX-D-DELIVERY.md` §D.5
