@journey @p0 @ac-003
Feature: Golden end-to-end journeys

  @E2E-189 @suite-platform
  Scenario: E2E-189 J1 Idea to backlog
    Given a project and a lab with a Tech Lead
    When the operator submits an idea in natural language
    Then canonical TODO cards are created within 3 seconds p95
    Then the cards animate into the Kanban
    Then the office announces them with a speech bubble and an activity event
    Then each card has story, criteria, deliverables, estimate and priority

  @E2E-190 @suite-platform
  Scenario: E2E-190 J2 Idea to approved SOW and materialized plan
    Given a project, a GitHub connection, a lab and a team
    When the operator requests a SOW and approves it
    Then the Analyst produces a structured SOW and an acyclic WBS
    Then every section is versioned and diffable
    Then materialization creates TODO cards linked to the SOW and plan

  @E2E-191 @suite-platform
  Scenario: E2E-191 J3 Autonomous execution end to end
    Given a READY card with DoR satisfied and autonomy A2
    When the scheduler leases it to the assignee
    Then the agent creates a branch, implements, runs tests in the sandbox and opens a PR
    Then the card moves to REVIEW
    Then QA verifies every criterion independently
    Then the Tech Lead merges per policy and the card reaches DONE
    Then the merge SHA, audit trail and cost accounting are recorded
    Then CV and memory are updated by reflection

  @E2E-192 @suite-platform
  Scenario: E2E-192 J4 Watch and steer
    Given the office is open
    When the operator clicks agents, the whiteboard, the server rack and the bookshelf
    Then the office shows live status, current card, log lines, CV and memory
    Then an invalid drag is rejected with the exact reason
    Then pause, resume, reassign and message controls work and are audited

  @E2E-193 @suite-platform
  Scenario: E2E-193 The deterministic demo reproduces the golden path
    Given make up and make seed-demo
    When I follow .docs/product/demo-script.md
    Then the seed reproduces one lab, four agents, one project, one SOW, 12 cards and 30 events
    Then the golden path completes in at most five minutes
    Then the expected visual output is reproducible

  @E2E-194 @suite-platform
  Scenario: E2E-194 The full setup is possible through the UI
    Given a fresh installation
    When the operator uses only the UI
    Then they create a provider, a lab, a Tech Lead, an Analyst, a Dev, a QA and a GitHub-connected project
    Then every step validates inline and reaches a ready state
