@health @p0 @fr-orc-001
Feature: Platform health and bootstrap

  @E2E-001
  Scenario: E2E-001 Liveness succeeds without dependencies
    Given the api process is running and the database and Redis are stopped
    When I call GET /api/v1/health/live
    Then the response status is 200
    Then the body reports the process as alive
    Then no database or Redis query is required to serve the request

  @E2E-002
  Scenario: E2E-002 Readiness reports all dependencies available
    Given the database, Redis and migrations are ready
    When I call GET /api/v1/health/ready
    Then the response status is 200
    Then the body reports database, Redis and migrations as ready

  @E2E-003
  Scenario: E2E-003 Readiness fails when Redis is unreachable
    Given Redis is unreachable and the database is healthy
    When I call GET /api/v1/health/ready
    Then the response status is 503
    Then the body names Redis as the unavailable dependency
    Then the error uses the NOT_READY code

  @E2E-004
  Scenario: E2E-004 Provider and SCM health endpoints report component status
    Given a DeepSeek provider and a GitHub credential are configured
    When I call GET /api/v1/health/providers and GET /api/v1/health/scm
    Then both responses have status 200
    Then each body reports the component status and the last probe time

  @E2E-005
  Scenario: E2E-005 Cold bootstrap reaches the running stack
    Given a clean clone and a valid environment file
    When I run make up following only the README
    Then the web UI responds at http://127.0.0.1:8080
    Then every core service reports healthy
    Then the elapsed time is at most 10 minutes
    Then the result is recorded as evidence for AC-001

  @E2E-006
  Scenario: E2E-006 Readiness fails while migrations are pending
    Given the database is reachable but a migration is not applied
    When I call GET /api/v1/health/ready
    Then the response status is 503
    Then the body reports pending migrations
