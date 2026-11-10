@lab @inventory @p0 @fr-lab-001
Feature: Labs, agents and CV

  @E2E-015 @suite-core
  Scenario: E2E-015 A lab cannot be created without a Tech Lead
    Given I submit a new lab with name, slug and settings but no tech_lead agent
    When I call POST /api/v1/labs
    Then the response status is 422
    Then the error code is TECH_LEAD_REQUIRED

  @E2E-016 @suite-core
  Scenario: E2E-016 Create a lab with exactly one Tech Lead
    Given I submit a lab with one veteran_lead agent in the tech_lead role
    When I call POST /api/v1/labs
    Then the response status is 201
    Then the lab is returned with autonomy level A2 by default
    Then the Tech Lead is active and belongs to the lab

  @E2E-017 @suite-core
  Scenario: E2E-017 A second active Tech Lead is rejected
    Given a lab already has one active tech_lead
    When I add another agent in the tech_lead role
    Then the response status is 422
    Then the error code is TECH_LEAD_EXISTS

  @E2E-018 @suite-core
  Scenario: E2E-018 Add Analyst, Dev and QA agents with valid characters
    Given an existing lab
    When I add systems_analyst, scrappy_dev and pixel_artisan agents with unique names
    Then each response status is 201
    Then each agent has one character from the fixed roster
    Then the lab team now contains exactly one Tech Lead and the three added agents

  @E2E-019 @suite-core
  Scenario: E2E-019 An unknown character id is rejected
    Given the agent creation wizard
    When I submit character_id "unknown_hero"
    Then the response status is 422
    Then the error code is VALIDATION_FAILED and the field is character_id

  @E2E-020 @suite-core
  Scenario: E2E-020 Agent description enforces the 120 to 600 character range
    Given the agent creation form
    When I submit a description of 119 characters
    Then the response status is 422
    When I submit a description of 601 characters
    Then the response status is 422
    When I submit a description of 120 characters or 600 characters
    Then the response status is 201 for each valid boundary

  @E2E-021 @suite-core
  Scenario: E2E-021 Pause and resume an agent
    Given an active agent that is idle
    When I call POST /api/v1/agents/{agent_id}/pause
    Then the agent status becomes paused
    When I call POST /api/v1/agents/{agent_id}/resume
    Then the agent status returns to idle
    Then both changes emit agent.status_changed events

  @E2E-022 @suite-core
  Scenario: E2E-022 A retired agent keeps history and receives no new cards
    Given an active agent that has completed at least one card
    When I call DELETE /api/v1/agents/{agent_id} with typed confirmation
    Then the agent is retired, not deleted
    Then its CV versions and evidence remain readable
    Then READY cards can no longer be assigned to it
    Then the office still renders its desk with a plaque

  @E2E-023 @suite-core
  Scenario: E2E-023 Agent concurrency capacity is bounded
    Given the agent create or edit form
    When I set max_concurrent_cards to 0 or to 5
    Then the response status is 422
    When I set it to 1 through 4
    Then the response status is 201 or 200

  @E2E-024 @suite-core
  Scenario: E2E-024 CV versions are append-only and gain evidence
    Given an agent with CV version 1 and no delivered cards
    When a run assigned to the agent completes a card
    Then a new CV version is appended without mutating version 1
    Then the new version contains evidence pointing to the card and run
    Then inventory.agent.cv_updated is emitted

  @E2E-025 @suite-core
  Scenario: E2E-025 CV deltas are bounded and proficiency is clamped
    Given an agent skill at proficiency 5
    When a reflection report proposes a delta of +1
    Then the stored proficiency remains 5
    Then given a delta of -1 the proficiency decreases by at most one
    Then no single run changes a skill by more than one point

  @E2E-026 @suite-core
  Scenario: E2E-026 Lab settings are validated as a whole
    Given the lab settings editor
    When I submit a per-card budget greater than the per-day budget
    Then the response status is 422 with a budget violation
    When I submit a valid settings object with WIP limits, autonomy, scheduler, office and maintenance
    Then the response status is 200
    Then archiving the lab afterwards is terminal and requires typed confirmation
