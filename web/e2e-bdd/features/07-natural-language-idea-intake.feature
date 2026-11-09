@kanban @ideas @p0 @fr-kan-010
Feature: Natural-language idea intake

  @E2E-077
  Scenario: E2E-077 An idea creates canonical TODO cards within the latency budget
    Given a lab with a Tech Lead and a project
    When I submit an idea with at least one actionable capability
    Then the response status is 202
    Then one or more TODO cards appear
    Then the p95 time from submission to visible cards is at most 3 seconds

  @E2E-078
  Scenario: E2E-078 Generated cards follow the canonical story template
    Given a submitted idea
    When the Tech Lead creates cards
    Then every card has a title of at most 80 characters
    Then every card has a story with persona, capability, outcome and context
    Then every card has at least one acceptance criterion and one deliverable
    Then every card has a Fibonacci estimate and a priority in must, should or could

  @E2E-079
  Scenario: E2E-079 Agents can create only TODO cards
    Given an agent acting through the kanban tool
    When it attempts to create a card in READY or DONE
    Then the attempt is rejected with POLICY_DENIED
    Then the card is created in TODO only when the request targets TODO

  @E2E-080
  Scenario: E2E-080 Idea submission is idempotent by key
    Given an idea submitted with Idempotency-Key "idea-42"
    When the same request is replayed within 24 hours
    Then the original response is returned
    Then no duplicate cards are created

  @E2E-081
  Scenario: E2E-081 Idea generation is rate limited
    Given the idea generation rate limit of five per minute is exhausted
    When I submit a sixth idea in the same minute
    Then the response status is 429
    Then the Retry-After header is present

  @E2E-082
  Scenario: E2E-082 Idea text is validated
    Given the idea composer
    When I submit empty text or text longer than 20000 characters
    Then the response status is 422
    Then the error code is VALIDATION_FAILED
