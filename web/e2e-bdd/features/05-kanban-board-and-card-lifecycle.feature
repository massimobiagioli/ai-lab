@kanban @p0 @fr-kan-001
Feature: Kanban board and card lifecycle

  @E2E-035
  Scenario: E2E-035 The board has exactly six fixed ordered columns
    Given an active project
    When I call GET /api/v1/projects/{project_id}/board
    Then the columns are exactly TODO, READY, DOING, REVIEW, DONE, ICEBOX in that order
    Then the column names are not configurable

  @E2E-036
  Scenario: E2E-036 Card keys are allocated once and never reused
    Given project prefix "AIL" with no cards
    When I create three cards
    Then the keys are AIL-1, AIL-2 and AIL-3
    Then deleting or completing a card does not free its key

  @E2E-037
  Scenario: E2E-037 Card creation validates required fields
    Given the project board
    When I create a card without a title or without a source
    Then the response status is 422
    Then the violations name the missing fields

  @E2E-038
  Scenario: E2E-038 A canonical story with criteria and deliverables is accepted
    Given a card payload with Title, As a/I want/so that, Context, Given/When/Then criteria and deliverables
    When I call POST /api/v1/projects/{project_id}/cards
    Then the response status is 201
    Then the stored story preserves every section
    Then needs_refinement is false

  @E2E-039
  Scenario: E2E-039 A TODO card may be intentionally incomplete
    Given a human-created card with only a title and a partial story
    When I save it with needs_refinement true
    Then the response status is 201
    Then the card remains in TODO
    Then the flag is visible on the card front

  @E2E-040
  Scenario: E2E-040 Card reads return the card or a not-found error
    Given an existing card id
    When I call GET /api/v1/cards/{card_id}
    Then the response status is 200 and matches the stored card
    When I call GET /api/v1/cards/{unknown_id}
    Then the response status is 404 with code NOT_FOUND

  @E2E-041
  Scenario: E2E-041 Updates are protected by optimistic concurrency
    Given a card at version 3
    When two clients both PATCH the card using expected version 3
    Then the first update succeeds and the version becomes 4
    Then the second update returns 409 VERSION_CONFLICT with the current card

  @E2E-042
  Scenario: E2E-042 Card lists use cursor pagination
    Given 120 cards in a project
    When I call GET /api/v1/projects/{project_id}/cards?limit=50
    Then the response contains data, next_cursor and total_estimate
    Then following next_cursor twice returns the remaining cards without duplicates
    Then a limit above 200 is rejected or clamped according to the contract

  @E2E-043
  Scenario: E2E-043 Card filters combine assignee, label, priority, overdue and source
    Given cards with different assignees, labels, priorities and due dates
    When I filter by assignee and priority=must and source=agent
    Then only matching cards are returned
    Then combining overdue=true returns only cards past their due date

  @E2E-044
  Scenario: E2E-044 Sorting is restricted to an allowlist
    Given the card list endpoint
    When I request sort=title or sort=due_date
    Then the response status is 200 and the order matches
    When I request an unknown sort field or an injection-shaped value
    Then the response status is 422 and no dynamic SQL is executed

  @E2E-045
  Scenario: E2E-045 Comments can be added and listed
    Given an existing card
    When I call POST /api/v1/cards/{card_id}/comments with a non-empty body
    Then the response status is 201
    When I call GET /api/v1/cards/{card_id}/comments
    Then the new comment is returned in chronological order

  @E2E-046
  Scenario: E2E-046 Card events are immutable and ordered
    Given a card that has been created and moved
    When I call GET /api/v1/cards/{card_id}/events
    Then the events are returned in increasing seq order
    Then each event has from, to, actor_type, actor_id and occurred_at
    Then the events cannot be updated or deleted through any endpoint

  @E2E-047
  Scenario: E2E-047 Cards can be assigned and reassigned
    Given a READY card and two agents with compatible roles
    When I call POST /api/v1/cards/{card_id}/assign for the first agent
    Then the assignee is recorded and an event is emitted
    When I reassign to the second agent
    Then the assignee changes and both assignments are auditable

  @E2E-048
  Scenario: E2E-048 Blocked reasons require an allowed category and text
    Given a card being moved to ICEBOX
    When I omit the reason or use category "misc"
    Then the response status is 422
    When I use dependency, clarification, external, risk, budget or policy with text
    Then the move is accepted and the reason is stored

  @E2E-049
  Scenario: E2E-049 Overdue cards are flagged but never moved automatically
    Given a card whose due date is in the past
    When the board is loaded
    Then the card has overdue true and a visible badge
    Then the card remains in its current column
    Then an agent speech nudge is emitted, not a transition

  @E2E-050
  Scenario: E2E-050 Concurrent card creation allocates unique keys
    Given 100 concurrent create requests for one project
    When all requests complete
    Then exactly 100 distinct keys exist
    Then the sequence has no duplicates and no gaps for committed cards

  @E2E-051
  Scenario: E2E-051 A 500-card board loads within budget
    Given a project with 500 cards
    When I load the board through the API and the UI
    Then the board renders all cards
    Then the p95 load time is at most 400 ms

  @E2E-052
  Scenario: E2E-052 An invalid drag-and-drop is rolled back with the exact violation
    Given the Kanban board with a TODO card missing acceptance criteria
    When I drag the card to READY and drop it
    Then the card animates back to TODO
    Then a toast shows the first violation verbatim
    Then no optimistic state remains after the 422 response

  @E2E-053
  Scenario: E2E-053 Same-column transitions are always denied
    Given a card in DOING
    When I request a transition from DOING to DOING
    Then the response status is 422 INVALID_TRANSITION
    Then no card_event is written

  @E2E-054
  Scenario: E2E-054 Reopening a DONE card creates a new card with a parent link
    Given a DONE card
    When I call POST /api/v1/cards/{card_id}/reopen with a reason
    Then a new card is created in TODO
    Then its parent_id points to the original card
    Then the original card remains DONE with its full audit trail
    Then a direct DONE to TODO transition is denied

  @E2E-055
  Scenario: E2E-055 All 36 ordered column pairs match the transition matrix
    Given every ordered pair of the six columns
    When I attempt each transition with valid guards where the matrix allows it
    Then exactly the 12 allowed transitions succeed
    Then the 24 denied transitions return 422 INVALID_TRANSITION
    Then same-column pairs are denied

  @E2E-056
  Scenario: E2E-056 Invalid transitions return a machine-readable violation payload
    Given a card that cannot move from TODO to DONE
    When I call POST /api/v1/cards/{card_id}/transition
    Then the response status is 422
    Then the body is RFC 9457 Problem Details with code INVALID_TRANSITION
    Then violations is a non-empty array with rule, field and message
    Then trace_id is present
