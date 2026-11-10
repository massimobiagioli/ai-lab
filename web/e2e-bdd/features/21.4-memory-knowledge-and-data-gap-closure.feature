@memory @kanban @data @p1 @fr-dlv-007
Feature: Memory, knowledge and data gap closure

  @E2E-220 @suite-platform
  Scenario: E2E-220 Reflection updates CV and proposes memory safely
    Given a completed or failed run
    When the reflection job runs
    Then it produces a ReflectionReport with outcome, skills demonstrated and missing, failure modes, lessons and memory candidates
    Then skill deltas stay within minus one to plus one per run and proficiency stays within one to five
    Then a memory candidate is written only after cosine deduplication at 0.92 or higher and secret scanning

  @E2E-221 @suite-platform
  Scenario: E2E-221 Decisions are promoted to project memory
    Given a run decision with rationale, alternatives and links
    When it is promoted
    Then a project-scoped memory document is created as an ADR candidate
    Then it is linked to the card and run for traceability

  @E2E-222 @suite-platform
  Scenario: E2E-222 Memory tiers and Redis working memory behave as specified
    Given episodic, semantic and procedural memory
    When I inspect retention
    Then episodes are append-only with the 180-day raw retention and summarized compaction
    Then semantic knowledge is stored as documents and vectors
    Then procedural knowledge is the agent CV, tools and playbooks
    Then Redis keeps the last 40 messages per run, a retrieval cache with TTL 300 seconds and namespaced keys
    Then a Redis cache miss never breaks correctness

  @E2E-223 @suite-platform
  Scenario: E2E-223 Retrieval bounds and data scrubbing are enforced
    Given a memory search without an explicit k
    When it runs
    Then the default is 8 and the maximum is 50
    Then secret and PII scrubbing runs before embedding and before sending context to the LLM
    Then a scrubbed secret never appears in a chunk or a prompt

  @E2E-224 @suite-kanban
  Scenario: E2E-224 Kanban dependencies and card splitting are enforced
    Given a READY candidate with an open dependency
    When the Tech Lead attempts to move it to READY without a waiver
    Then the response status is 422 with DOR_DEPENDENCY_OPEN
    Then a Tech Lead waiver with a reason allows the move
    Then a card estimated 13, or a card larger than 8 that should be split, is refused or split before READY

  @E2E-225 @suite-kanban
  Scenario: E2E-225 Work is card-bound and card data is durable
    Given any agent action that mutates the repository
    When the action is attempted outside a card and run
    Then it is rejected
    Then a card persists parent_id, sow_id, budget_usd, risk and version
    Then due dates are stored in UTC and rendered in the operator timezone with Intl
    Then archiving a project blocks new cards, while moving a project between labs requires an audited transfer
