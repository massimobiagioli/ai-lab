@performance @reliability @acceptance @p1 @nfr-004
Feature: Performance, reliability and acceptance gap closure

  @E2E-236 @suite-platform
  Scenario: E2E-236 Latency and duration budgets are met
    Given the defined load profile
    When the API serves reads and writes and a run executes
    Then read p95 is at most 120 ms and p99 at most 300 ms
    Then write p95 is at most 250 ms and p99 at most 600 ms
    Then DB query p95 is at most 50 ms and non-LLM run overhead is at most 2 seconds
    Then a medium SOW completes within 10 minutes p95

  @E2E-237 @suite-platform
  Scenario: E2E-237 Load, soak and office stress show no leak
    Given the k6 browse, board drag, idea burst, WS fan-out and memory search scenarios
    When they run at twice the expected load for 60 minutes
    Then memory growth stays under 10 percent and the error rate stays under 0.1 percent
    Then the office sustains 16 agents and 200 animated props at 60 FPS
    Then WS event to render stays at or below 250 ms p95

  @E2E-238 @suite-platform
  Scenario: E2E-238 Rolling upgrades and clock rules are safe
    Given a running version and a backward-compatible migration
    When a rolling upgrade is performed
    Then the old and new versions interoperate for one version
    Then all timestamps are server-generated and clients are never authoritative
    Then a simulated deadlock is retried up to three times with jitter

  @E2E-239 @suite-platform
  Scenario: E2E-239 SLOs, chaos and restore evidence agree
    Given the SLO dashboards, alert rules and restore drill
    When the chaos suite kills a worker, the database and Redis
    Then recovery matches the failure matrix and RTO and RPO targets
    Then no alert fires falsely and no data is lost
    Then the restore drill evidence is attached to AC-012 and AC-013

  @E2E-240 @suite-platform
  Scenario: E2E-240 Documentation and release acceptance are complete
    Given the documentation manifest
    When the docs build and link check run
    Then every required document exists, links resolve and screenshots match the deterministic seed
    Then the acceptance report has evidence for AC-001 to AC-015
    Then no milestone regressed, no P0 or P1 is open, and the release candidate is signed off by a non-author
