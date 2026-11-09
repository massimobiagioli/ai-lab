@inventory @p0 @fr-inv-001
Feature: LLM providers and model catalog

  @E2E-007
  Scenario: E2E-007 Create a DeepSeek provider with a write-only key
    Given I am on the providers inventory page
    When I submit name "DeepSeek", kind "deepseek", an HTTPS base URL and a valid API key
    Then the response status is 201
    Then the body returns the provider id and only the last four key characters
    Then the full API key is never present in the response

  @E2E-008
  Scenario: E2E-008 Reject a provider whose base URL is not HTTPS
    Given I have the provider create form open
    When I submit a base URL with the http scheme
    Then the response status is 422
    Then the error code is VALIDATION_FAILED
    Then the violation field is base_url

  @E2E-009
  Scenario: E2E-009 A stored API key is never readable
    Given a provider was created with a known API key
    When I call GET /api/v1/providers
    When I call GET /api/v1/providers/{provider_id}
    Then no response contains the plaintext key
    Then each provider row exposes only api_key_last4

  @E2E-010
  Scenario: E2E-010 Provider verification probes the endpoint and updates status
    Given a provider whose fake endpoint returns a one-token completion
    When I call POST /api/v1/providers/{provider_id}/verify
    Then the response status is 200 and the result is healthy
    Then the provider status becomes active
    Then the last probe timestamp is recorded

  @E2E-011
  Scenario: E2E-011 Key rotation increments the envelope key version
    Given an active provider with a known last4
    When I call POST /api/v1/providers/{provider_id}/rotate-key with a new key
    Then the response status is 200
    Then key_version is incremented
    Then the new last4 reflects the new key
    Then the old ciphertext can no longer be decrypted with the active version

  @E2E-012
  Scenario: E2E-012 The model catalog seeds exactly two models
    Given the database is freshly migrated
    When I call GET /api/v1/models
    Then the response contains exactly deepseek-flash and deepseek-v4-pro
    Then deepseek-flash is marked as the default model
    Then deepseek-v4-pro is marked as the escalation model

  @E2E-013
  Scenario: E2E-013 Model pricing configuration drives recorded cost
    Given a model row with fixed input and output prices
    When an LLM request records tokens_in and tokens_out
    Then the ledger cost equals (tokens_in * input_price + tokens_out * output_price) / 1_000_000 micro-USD
    Then the cost is returned as an integer with no floating-point drift

  @E2E-014
  Scenario: E2E-014 Provider rate limiting queues work instead of failing it
    Given the provider rate limit is exhausted for the current minute
    When a run requests a completion
    Then the request is queued rather than rejected
    Then the job is dispatched after the rate-limit window resets
    Then a 429 with Retry-After is returned only to interactive callers that opt out of queueing
