@maintenance @metrics @p1 @fr-maint-001
Feature: Maintenance, budgets and flow metrics

  @E2E-181
  Scenario: E2E-181 Card and daily budgets are enforced
    Given a lab with a per-card and per-day budget
    When a run charges tokens
    Then the ledger records the cost per card, agent and lab
    Then the run stops before exceeding either budget
    Then the remaining budget is exposed as a metric

  @E2E-182
  Scenario: E2E-182 Budget exhaustion pauses the lab and can resume
    Given the lab budget is exhausted
    When a run reaches the limit
    Then system.budget.exhausted is emitted
    Then active runs checkpoint safely and the lab is paused
    When the operator increases the budget
    Then the lab resumes and queued work continues

  @E2E-183
  Scenario: E2E-183 Flow metrics are derived from immutable events
    Given the demo seed
    When I call GET /api/v1/labs/{lab_id}/metrics
    Then lead time, cycle time p50 and p95, throughput, WIP and blocked counts are returned
    Then the values agree with the card_events trail

  @E2E-184
  Scenario: E2E-184 Spend is visible at every level with a trend
    Given completed runs with recorded costs
    When I view spend by lab, project, card and agent
    Then each level shows its spend
    Then a 30-day trend and a forecast are available

  @E2E-185
  Scenario: E2E-185 The maintenance job catalog is seeded and idempotent
    Given the migrated database
    When the worker ticks
    Then the job catalog includes data_integrity, memory_compaction, cv_reflection_backfill, dependency_check, stale_card_scan, overdue_notify and provider_health
    Then each job runs idempotently and records a structured outcome

  @E2E-186
  Scenario: E2E-186 Maintenance jobs create work without auto-moving cards
    Given a dependency finding and a card stale for more than seven days
    When the dependency_check and stale_card_scan jobs run
    Then a TODO card is created for the dependency finding
    Then the stale card is flagged with a refinement nudge
    Then no card is moved automatically

  @E2E-187
  Scenario: E2E-187 Memory hygiene and CV backfill are bounded and observable
    Given episodes eligible for compaction and runs missing CV evidence
    When memory_compaction and cv_reflection_backfill run
    Then compaction respects pins and retention
    Then the CV backfill is bounded to 500 runs
    Then both jobs emit completion events and metrics

  @E2E-188
  Scenario: E2E-188 Provider health probing drives status and events
    Given a provider endpoint that starts failing
    When the provider_health probe runs on its 60-second schedule
    Then the provider status becomes degraded
    Then a health event is emitted and surfaced in the UI
