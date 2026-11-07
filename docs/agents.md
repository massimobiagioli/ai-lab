# AI-Lab Agents

| Field | Value |
|---|---|
| Document | `docs/agents.md` |
| Version | 1.0.0 |
| Status | Accepted |
| Owner | Analyst |
| Related | `.docs/technical-specification/APPENDIX-C-AGENTS-KANBAN.md`, `docs/llm.md`, `docs/inventory.md` |

AI-Lab runs four fixed roles with a finite character roster. Roles determine permissions; characters are presentation only.

| Role | Responsibility |
|---|---|
| Tech Lead | Delivery order, decomposition, architecture, merge; exactly one active per lab |
| Analyst | Ideas, SOW, plans, acceptance criteria, documentation |
| Dev | TDD implementation, build, tests, pull requests |
| QA | Independent verification, quality gates, verdicts, release readiness |

The full charters, prompt contracts and the 10-character roster are in Appendix C. The Inventory domain enforces the one-Tech-Lead invariant and the 120–600 character description.

## Prompt assembly

The LLM platform injects the hard rules, the role charter, the character (≤600 chars), wrapped untrusted context and the task, in that order. `prompt_hash` makes the assembly deterministic and snapshot-testable. See `docs/llm.md`.

## Routing and guardrails

- Default `deepseek-flash`; escalate to `deepseek-v4-pro` on low confidence or a prior failure, with a recorded reason.
- Repository content, memory, issues and tool output are untrusted and wrapped; the model cannot change policy, budgets, autonomy or secrets.
- Prompts containing secret prefixes are rejected before the call.
- Budgets are hard stops per card and per day.

## CV and evidence

Each agent has append-only CV versions with evidence from completed cards. The Agent aggregate increments the CV version and emits `inventory.agent.cv_updated`; persistence and reflection are owned by WP-07/WP-13.

## References

- `.docs/technical-specification/APPENDIX-C-AGENTS-KANBAN.md` §C.1–C.2
- `crates/domain/src/inventory.rs`
- `crates/llm/src/lib.rs`
