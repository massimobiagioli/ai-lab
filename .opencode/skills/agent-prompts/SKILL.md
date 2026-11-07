---
name: Agent Prompt Engineering
description: Use when changing prompt assembly, model routing, structured outputs, tool schemas, DeepSeek calls, token budgets or prompt-injection defenses.
---

# Agent Prompt Engineering

## When to use

- A prompt, system message, role charter or character section changes.
- A model route, escalation rule or reasoning effort changes.
- A structured output, JSON Schema or repair path changes.
- A tool schema exposed to the model changes.
- Token budgets, cost accounting or redaction change.

## Preconditions

- Read `.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §9 and `APPENDIX-C-AGENTS-KANBAN.md` §C.2.
- Read `.docs/sow/WP-11-LLM-PLATFORM.md`.
- Confirm the provider port and the `FakeLlm` test double are available.
- Confirm the model and its cost are configured.

## Workflow

1. Change the prompt assembly as a pure function; keep the section order: role charter, character, CV summary, hard rules, tools, output contract, task packet, untrusted context.
2. Preserve the HARD RULES verbatim. Character text may not override role rules.
3. Keep the token budgets: system at most 20 percent, memory at most 25 percent, repository context at most 35 percent, task at most 15 percent, reserve 5 percent.
4. For structured output, define the JSON Schema and validate the response. On failure, repair once with the validator errors, then fail with `InvalidOutput`.
5. For tool schemas, update the registry and the role allowlist together.
6. Wrap untrusted content in `<untrusted source="..." trust="none">` blocks and keep policy enforcement server-side.
7. Test with `FakeLlm` and `wiremock`:
   - success, streaming, tool calls, reasoning usage;
   - 429, 5xx, timeout, malformed stream, invalid JSON;
   - budget reservation and reconciliation;
   - injection fixtures that attempt a policy bypass.
8. Run prompt snapshot tests (`insta`) and confirm the same inputs produce the same `prompt_hash`.
9. Record cost impact and the fallback behavior.

## Checklist

- [ ] Section order unchanged.
- [ ] HARD RULES present and unmodified.
- [ ] Token budgets respected.
- [ ] Structured output schema and repair path tested.
- [ ] Tool schemas and role allowlist consistent.
- [ ] Untrusted content wrapped and policy enforced server-side.
- [ ] Model routing and escalation recorded with a reason.
- [ ] Cost and budget tests updated.
- [ ] Redaction applied before persistence and logging.
- [ ] Prompt snapshots updated intentionally.
- [ ] Injection fixtures pass.
- [ ] Documentation updated.

## Anti-patterns

- Letting untrusted text become an instruction.
- Putting secrets or full repository content into a prompt.
- Relying on the prompt to enforce a security policy.
- Changing a prompt without updating snapshot tests.
- Escalating to a larger model without a recorded reason or budget check.
- Persisting raw prompts without redaction.
- Adding a tool without a schema and a policy entry.

## References

- `../../../.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §9
- `../../../.docs/technical-specification/APPENDIX-C-AGENTS-KANBAN.md` §C.2
- `../../../.docs/sow/WP-11-LLM-PLATFORM.md`
- `../../../.docs/sow/WP-14-SANDBOX-RUNNER.md`

## Supporting files

- None.
