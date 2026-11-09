@api @observability @p0 @api-002
Feature: API contracts, metrics and traces

  @E2E-173
  Scenario: E2E-173 Errors use RFC 9457 Problem Details
    Given any failing request
    When the error response is inspected
    Then it contains type, title, status, detail, instance, code, violations and trace_id
    Then the code is a stable catalog value such as INVALID_TRANSITION or VALIDATION_FAILED

  @E2E-174
  Scenario: E2E-174 Correlation identifiers are propagated
    Given I send a valid X-Request-Id
    When the request is processed
    Then the response echoes X-Request-Id
    Then a traceparent-compatible trace id is attached to logs and error payloads

  @E2E-175
  Scenario: E2E-175 Rate limits are enforced by class
    Given the read, write and idea/SOW generation limits
    When I exceed 120 reads, 30 mutations or 5 generations per minute
    Then the matching request receives 429 with Retry-After
    Then requests below the limit are unaffected

  @E2E-176
  Scenario: E2E-176 The OpenAPI document matches the served API
    Given the published .docs/product/api/openapi.json
    When schemathesis runs against the API
    Then every documented path and schema conforms
    Then an additive change is accepted while a removal requires /v2

  @E2E-177
  Scenario: E2E-177 List endpoints follow the pagination contract
    Given a list endpoint
    When I request limit and cursor and sort and filters
    Then the response is data, next_cursor and total_estimate
    Then limits are capped at 200 and unknown filters are rejected

  @E2E-178
  Scenario: E2E-178 Metrics and structured logs are exposed and redacted
    Given the api and worker are running
    When I call GET /metrics
    Then Prometheus metrics include HTTP, DB pool, Redis, queue depth, lease age, run, LLM, cost, WS, event lag and office FPS
    Then every request log line is JSON with method, path, status and duration but no secrets

  @E2E-179
  Scenario: E2E-179 Traces link the whole pipeline
    Given a run that reaches an LLM and GitHub call
    When the trace is inspected
    Then API, use case, DB, outbox, worker, LLM and GitHub spans share one trace_id
    Then errors and run pipelines are sampled at 100 percent

  @E2E-180
  Scenario: E2E-180 Health and system status are independently observable
    Given the api, worker, provider and SCM
    When I call /health/live, /health/ready, /health/providers and /health/scm
    Then each reports its own component status
    Then a system.health event is emitted every 15 seconds
