# AI-Lab Constitution

| Field | Value |
|---|---|
| Document | `.docs/CONSTITUTION.md` |
| Version | 1.0.0 |
| Status | Accepted |
| Owner | Tech Lead |
| Last review | 2026-11-07 |

This document defines the non-negotiable rules of AI-Lab. It is intentionally high-level. Implementation detail lives in `.docs/technical-specification/`, `.docs/sow/`, `.docs/PRINCIPLES.md`, `.docs/STACK.md` and the accepted ADRs.

---

## 1. Mission

AI-Lab turns a raw idea into a maintained software product through a managed team of AI agents, and makes running that team feel like a 16-bit office sim. The product owns the full lifecycle: idea → SOW → plan/WBS → Kanban tasks → execution → review → merge → release → maintenance → learning.

The **virtual office is the experience**. The **Kanban board and the domain model are the truth**. The **agents are accountable workers**, not chat toys.

## 2. Product boundaries

- AI-Lab is a **single-operator, local-first** application. It binds to `127.0.0.1` by default.
- Remote exposure is an explicit operator action and requires a bearer token and TLS.
- v1 supports **DeepSeek only** behind an `LlmProvider` port and **GitHub only** behind an `ScmProvider` port.
- v1 is a **modular monolith deployed as a small set of runtime services**, not a distributed microservice mesh. Context boundaries are contract-first so they can be extracted later without changing consumers.
- Out of scope: multi-user RBAC/SSO, billing, non-GitHub SCM, non-DeepSeek providers in the shipped UI, Kubernetes operators, native mobile apps, arbitrary code execution outside the sandbox.

## 3. Non-negotiables

1. **Security first.** No secret is ever exposed through an API, log, event, error, prompt or sandbox. Secrets are encrypted at rest with AES-256-GCM and are write-only.
2. **No invalid state.** The Kanban engine rejects invalid transitions, missing DoR/DoD, WIP violations and separation-of-duties conflicts server-side. The UI is a projection, never the authority.
3. **No self-review, no self-merge.** Reviewer ≠ author; merger ≠ author. This applies to humans and agents.
4. **Evidence over claims.** "Done" requires a merged PR, a test report, a benchmark, a screenshot or a run ID. Self-reported success is not evidence.
5. **Single source of truth.** Each fact has one canonical location. Documentation, code and contracts must not disagree.
6. **Documentation is a release gate.** Missing, broken or stale documentation blocks release.
7. **No obsolete versions.** Every runtime, framework and dependency is pinned and supported; EOL lines are forbidden.
8. **No unrequested scope.** New scope requires a change request; small, reversible decisions may be made locally and recorded.
9. **Traceability.** Every mutation has an actor, a correlation ID and, for agentic work, a `run_id`. Every commit carries AI-Lab trailers.
10. **Accessibility is part of quality.** WCAG 2.2 AA for DOM surfaces; reduced motion and screen-reader fallbacks for the office.

## 4. Decision rights

| Decision | Authority | Consultation | Record |
|---|---|---|---|
| Delivery order, task decomposition | Tech Lead | Analyst, QA | SOW / WP |
| Architecture and technology | Tech Lead | Analyst, Dev, QA | ADR |
| SOW and plan content | Analyst | Tech Lead, QA | SOW / plan |
| Implementation approach | Dev | Tech Lead | Card log / PR |
| Independent verification | QA | Tech Lead | Review verdict |
| Budget, autonomy, exposure, release | Human Operator | Tech Lead | CR + audit log |
| Security exception | QA + Human Operator | Tech Lead | ADR with expiry |

## 5. Change control

A change request is required for scope changes, new dependencies, architecture changes, contract breaks, reopening a `done` work package, budget/autonomy changes and milestone re-baselining. The CR records context, proposed change, impact (scope, schedule, cost, risk), alternatives, decision, approver and date. Emergency security fixes may bypass the normal pull order but must pass security gates and be retro-documented within 24 hours.

## 6. Quality contract

- Work follows **Agile/Kanban** with WIP = 1 per agent and pull-based selection.
- Work follows **TDD**: Red → Green → Refactor. No production behavior without a failing test first.
- Cards satisfy **Definition of Ready** before `READY` and **Definition of Done** before `DONE`.
- The required CI gates in `.docs/sow/WP-00-GOVERNANCE.md` §3.8 must be green.
- Coverage thresholds: backend line ≥ 80%, domain ≥ 90%, frontend ≥ 80%, new code ≥ 80%.
- Flaky tests are quarantined within 24 hours and fixed or deleted within 5 days.

## 7. Security contract

- Threat model before building any external surface.
- Least privilege for services, DB roles, agents, tools and sandboxes.
- Deny-by-default tool policy; allowlisted commands, paths, hosts and egress.
- Ephemeral non-root sandboxes with no host mounts, no Docker socket, read-only root filesystem and dropped capabilities.
- Prompt injection is treated as untrusted data; policy is enforced server-side, not by prompt.
- Supply chain: lockfiles, `cargo-deny`, `npm audit`, Trivy, SBOM, cosign signatures, no EOL dependencies.
- Security incidents are P0/P1, follow the incident runbook and block release.

## 8. Definition of Done (project level)

A unit of work is done only when:

1. Its acceptance criteria are demonstrably met.
2. Tests exist and pass; coverage thresholds are met.
3. Linters, type checks and security scans are green.
4. Documentation, ADRs and `AGENTS.md` indexes are updated where affected.
5. The change is reviewed by someone other than the author and merged.
6. No P0/P1 defect is open.
7. Evidence is linked from the WP or card.
8. The relevant status is updated with an append-only history row.

## 9. Source-of-truth order

When sources disagree, the higher source wins and the lower source must be corrected:

1. `.docs/technical-specification/`
2. `.docs/sow/SOW.md` and the target `WP-XX-*.md`
3. `.docs/CONSTITUTION.md`, `.docs/PRINCIPLES.md`, `.docs/STACK.md`
4. `.docs/adr/` (later accepted ADR supersedes earlier)
5. `.opencode/skills/`
6. `AGENTS.md`

Implementation must stop and escalate on an unresolved conflict. Silent divergence is a defect.

## 10. Amendment

This constitution changes only through an accepted ADR and a change request approved by the Human Operator. History is append-only; superseded text remains available in the ADR record.
