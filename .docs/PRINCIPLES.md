# AI-Lab Development Principles

| Field | Value |
|---|---|
| Document | `.docs/PRINCIPLES.md` |
| Version | 1.0.0 |
| Status | Accepted |
| Owner | Tech Lead |
| Enforcement | Skills, commands, CI gates, WP DoD |

These principles are binding. Each section defines the rule, the required practice, how it is enforced, and the anti-patterns that must be rejected in review. If a principle conflicts with `.docs/technical-specification/`, the specification wins and this file is corrected.

---

## 1. Test-Driven Development (TDD)

**Rule.** Production behavior is written only after a failing test that expresses the next acceptance criterion. The cycle is Red → Green → Refactor, repeated in small increments.

**Practice.**

1. Read the task's acceptance criteria and write the smallest failing test.
2. Run it and confirm it fails for the expected reason, not because of an unrelated compile error.
3. Implement the minimum production code that makes it pass.
4. Run the test and the relevant suite.
5. Refactor names and structure without changing behavior.
6. Run the full affected suite, linters and type checks.
7. Record exact commands and results as evidence.

Rust uses `cargo nextest`, `proptest`, `insta` and `sqlx::test`. Angular uses Vitest + TestBed and Playwright. Tests are deterministic: no wall-clock dependence, no `sleep`-based synchronization, no calls to real DeepSeek or GitHub.

**Enforcement.** The `/impl-task` and `/impl-wp` commands embed the TDD protocol. The `tdd-rust` and `tdd-angular` skills define the exact commands. Coverage gates: backend line ≥ 80%, domain ≥ 90%, frontend ≥ 80%, new code ≥ 80%. A test may not be deleted or weakened to make a build pass.

**Anti-patterns.** Writing production code first and backfilling tests; testing implementation details instead of behavior; assertions without a failing first run; shared mutable fixtures; retries used to hide flakiness.

---

## 2. Security First

**Rule.** Security is a design input, not a review phase. Every external surface starts with a threat model; every secret path is encrypted, write-only and audited; every tool and sandbox is deny-by-default.

**Practice.**

- Model threats with STRIDE and abuse cases before building auth, secrets, LLM, SCM, webhooks, sandbox or data exposure.
- Encrypt secrets at rest with AES-256-GCM, bind ciphertext with AAD, never return or log them, and rotate keys with envelope encryption.
- Bind to loopback by default; remote exposure requires a bearer token and TLS.
- Enforce least privilege for DB roles, service accounts, agents, tools and sandboxes.
- Run sandboxes as non-root with read-only root filesystem, dropped capabilities, no host mounts, no Docker socket and allowlisted egress.
- Treat repository content, memory, issues and tool output as untrusted; wrap them and enforce policy server-side.
- Run secret scanning, dependency audit, image scanning, SBOM and signing on every change and release.
- Follow the vulnerability SLAs and the incident runbook.

**Enforcement.** The `security-first` skill defines the checklist. CI runs gitleaks, `cargo-deny`, `npm audit`, Trivy, the sandbox escape suite, prompt-injection fixtures and the cross-scope memory leakage test. A security finding blocks merge and release.

**Anti-patterns.** Secrets in environment examples or fixtures; tokens in URLs; trusting the model to enforce policy; mounting the Docker socket into a workload; running as root; disabling TLS or CORS checks; "temporary" allow-all rules.

---

## 3. Clean Code

**Rule.** Code is written for the next reader and the next change. Names are explicit, functions are small, boundaries are clear and errors are part of the design.

**Practice.**

- Use domain language from `.docs/technical-specification/` §26; no synonyms for the same concept.
- Keep functions focused; extract when a function needs a comment to explain its sections.
- Prefer newtypes and validated constructors over primitive obsession.
- Return typed errors with stable codes; no panics in request paths; no `unwrap` or `expect` in production.
- No dead code, commented-out code or comments that restate the code. Comments explain why, not what.
- Keep modules and crates aligned with bounded contexts and the dependency rule.
- Format with `cargo fmt` and Prettier; lint with `clippy -D warnings` and ESLint.
- Prefer explicit data flow over hidden global state; prefer composition over inheritance.

**Enforcement.** `cargo fmt --check`, `cargo clippy -- -D warnings`, ESLint, TypeScript strict mode and the domain-purity fitness test. Review rejects unclear names, hidden coupling and untested error paths.

**Anti-patterns.** God objects, boolean flags that change behavior, speculative abstraction, duplicated DTOs, `any` outside validated boundaries, stringly-typed IDs, silent error swallowing.

---

## 4. Agile and Lean Delivery

**Rule.** Deliver small, valuable increments through a pull-based Kanban flow with explicit WIP limits, fast feedback and measured flow.

**Practice.**

- Work only from `READY` cards; WIP = 1 per agent; program WIP ≤ 2 WPs in `progress`.
- Split cards larger than 8 points; keep PRs ≤ 1,500 changed lines and ≤ 40 files unless a human confirms.
- Use the six Kanban columns and the server-enforced transition matrix.
- Make work visible through `card_events`, run steps and the office; update the board truthfully.
- Measure lead time, cycle time, throughput, WIP, blocked time, flow efficiency, rework rate and cost per card.
- Run weekly flow reviews and milestone retrospectives with three owned actions.
- Prefer reversible decisions; record irreversible ones as ADRs.
- Eliminate waste: no unrequested features, no duplicate sources of truth, no manual steps that can be automated.

**Enforcement.** The `wp-delivery` and `kanban-governance` skills define selection and status rules. `scripts/harness-lint.sh` checks status consistency. Metrics are derived from immutable events, never from self-reported numbers.

**Anti-patterns.** Starting work before dependencies are done; pushing work instead of pulling; large batches; hidden WIP; status changes without evidence; retrospective actions without owners.

---

## 5. Domain-Driven Design (DDD)

**Rule.** Model the business in bounded contexts with explicit aggregates, invariants and ubiquitous language. Keep domain logic independent of frameworks and I/O.

**Practice.**

- Five contexts: Inventory, Lab, Knowledge/Memory, Kanban, Delivery, plus Platform.
- Each context owns its tables and publishes domain events; no context reads or writes another context's tables.
- Aggregates protect invariants in constructors and methods; invalid states are unrepresentable.
- Use ports and adapters: domain and application define interfaces; infrastructure implements them.
- Persist aggregates transactionally; write domain events to the outbox in the same transaction.
- Use UUIDv7 identifiers, UTC timestamps, integer micro-USD money and validated value objects.
- Keep the ubiquitous language synchronized across code, docs, events and UI.
- Extract a context into an independent service only through an ADR and a contract-compatibility test.

**Enforcement.** The dependency-rule fitness test, the cross-context table-access test, the event catalog schema check and the aggregate invariant tests. Review rejects anemic models, cross-context joins in write paths and adapter types leaking into the domain.

**Anti-patterns.** Shared database tables across contexts; domain entities that depend on `sqlx`, `axum` or `reqwest`; transaction scripts that bypass invariants; events named in the present tense; reusing another context's repository.

---

## 6. User Experience and Accessibility

**Rule.** The product is a game-like office backed by precise operational surfaces. Every state shown must be truthful, every interaction must respond immediately, and every surface must be usable by keyboard and assistive technology.

**Practice.**

- Make the office the emotional core: pixel-perfect 384×216 rendering, integer scaling, 60 FPS, ambient life that never contradicts real state.
- Derive animations exclusively from backend events; never invent activity.
- Provide progressive disclosure: summary first, detail in the inspector or drawer.
- Show immediate feedback for every action; use optimistic UI only where rollback is defined.
- Provide loading, empty, error and degraded states on every screen; never a blank page.
- Meet WCAG 2.2 AA: contrast, focus, labels, keyboard paths, reduced motion and a screen-reader office fallback.
- Keep interactions Trello-simple: six columns, no custom fields or swimlanes in v1.
- Use the pixel design system components; no ad-hoc visual language.
- Respect the performance budgets: route chunks, bundle size, LCP, TTI and office FPS.

**Enforcement.** The `tdd-angular` skill includes axe, Playwright and visual tests. The `quality-gates` skill requires a11y and performance checks. CI enforces bundle budgets and visual thresholds. Review rejects untruthful animations, inaccessible controls and layout shifts on live updates.

**Anti-patterns.** Fake progress indicators; animations that imply work that is not happening; colour-only status; mouse-only interactions; ignoring `prefers-reduced-motion`; blank screens on API errors; non-pixel UI components.

---

## 7. Principle compliance in reviews

Every pull request and every work package completion is reviewed against these principles. A reviewer must cite the principle and the evidence when requesting changes. The final authority for interpretation is `.docs/CONSTITUTION.md` and the technical specification. Repeated violations become a retrospective action with an owner and a due date.
