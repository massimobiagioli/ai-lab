@reliability @p1 @rel-001
Feature: Reliability, backup and recovery

  @E2E-195
  Scenario: E2E-195 Backup and restore drill meet the recovery objectives
    Given a populated database and artifacts
    When make backup runs and then make restore-drill restores into a scratch database
    Then the dump and manifest contain every table and artifact reference
    Then the restored row counts match the source
    Then the restore completes within the RTO of one hour
    Then no master key is present in the backup

  @E2E-196
  Scenario: E2E-196 Nightly integrity checks are report-only
    Given a database with no orphans or invariant violations
    When the data_integrity job runs
    Then it reports zero findings
    Then given an injected orphan or projection mismatch
    Then it reports the finding as P1
    Then it never performs a destructive automatic repair

  @E2E-197
  Scenario: E2E-197 Graceful shutdown drains and releases resources
    Given in-flight requests and active runs
    When the service receives a shutdown signal
    Then it stops accepting new work
    Then it drains for at most 20 seconds
    Then it checkpoints or finishes runs, releases leases and flushes telemetry

  @E2E-198
  Scenario: E2E-198 Dependency degradation is explicit and safe
    Given Redis is unavailable
    When the app is configured for degraded mode
    Then a read-only banner is shown and correctness is preserved
    Then given Redis fail-closed rate limiting the API returns an explicit error rather than allowing unbounded traffic
    Then a provider outage queues work and surfaces a visible status instead of losing data

  @E2E-199
  Scenario: E2E-199 A provider outage stalls delivery but does not corrupt state
    Given the LLM provider is down
    When a run attempts a completion
    Then the run is queued or failed as transient according to policy
    Then the card and audit state remain consistent
    Then the provider health UI shows the outage

  @E2E-200
  Scenario: E2E-200 No milestone regresses a previously accepted journey
    Given the full E2E-001 to E2E-199 suite
    When CI runs on the release candidate
    Then every deterministic scenario passes or is explicitly quarantined with an owner and a card
    Then no P0 or P1 is open
    Then the evidence is attached to the release acceptance record
