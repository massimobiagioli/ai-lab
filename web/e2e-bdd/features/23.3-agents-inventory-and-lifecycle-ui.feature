@inventory @agents @ui @p0 @ux-060
Feature: Agents inventory and lifecycle UI

  @E2E-383 @suite-management
  Scenario: E2E-383 The agents list shows the information needed to decide
    Given a lab with several agents
    When the agents list renders
    Then each row shows name, role, character avatar, status, capacity, CV version and last activity
    Then the columns are aligned and readable
    Then the primary create action is visible

  @E2E-384 @suite-management
  Scenario: E2E-384 Agents can be searched by name, role and character
    Given agents with distinct names, roles and characters
    When I search by each field
    Then only the matching agents remain
    Then the search is case-insensitive and debounced
    Then the result count updates without a full reload

  @E2E-385 @suite-management
  Scenario: E2E-385 Agents can be filtered by role and status
    Given agents in every role and status
    When I filter by role and by status
    Then the list contains only matching agents
    Then the active filters are visible
    Then clearing a filter restores the matching rows

  @E2E-386 @suite-management
  Scenario: E2E-386 Agents can be sorted deterministically
    Given a list of agents
    When I sort by name, role, status or last activity
    Then the order is stable and correct
    Then the active sort is indicated with aria-sort
    Then reloading preserves the sort

  @E2E-387 @suite-management
  Scenario: E2E-387 Agent pagination is clear and reversible
    Given more agents than one page
    When I move to the next and previous page
    Then the boundary controls are enabled and disabled correctly
    Then the page cursor is reflected in the URL
    Then returning to the first page restores the first rows

  @E2E-388 @suite-management
  Scenario: E2E-388 The agent detail drawer explains the agent
    Given an agent row
    When I open its detail
    Then the drawer shows description, role, character, model preference, autonomy, capacity and CV summary
    Then the edit and lifecycle actions are clear
    Then the drawer traps focus and closes with Escape

  @E2E-389 @suite-management
  Scenario: E2E-389 The agent creation wizard follows the documented steps
    Given the create-agent action
    When I open the wizard
    Then the steps are role, character, name, model, autonomy and review
    Then Back and Next preserve the entered data
    Then the review step summarises every choice before creation

  @E2E-390 @suite-management
  Scenario: E2E-390 The character picker is visual and valid
    Given the character step
    When I inspect the roster
    Then each character shows an animated preview and its display name
    Then selecting one highlights it
    Then an unknown character id cannot be submitted

  @E2E-391 @suite-management
  Scenario: E2E-391 Agent names are unique and validated inline
    Given the name step
    When I submit an empty name or a duplicate name within the lab
    Then an inline error explains the problem and keeps my input
    Then a valid unique name advances the wizard

  @E2E-392 @suite-management
  Scenario: E2E-392 The agent description teaches the limit
    Given the description field
    When I type fewer than 120 or more than 600 characters
    Then a live counter and an inline message explain the range
    Then the submit is blocked with a clear reason
    Then a valid description is accepted

  @E2E-393 @suite-management
  Scenario: E2E-393 Agent capacity, model and autonomy are constrained
    Given the model and autonomy steps
    When I set capacity outside 1 to 4 or choose an unsupported model
    Then the control rejects the value with an explanation
    Then valid choices are preserved in the review

  @E2E-394 @suite-management
  Scenario: E2E-394 A created agent appears everywhere immediately
    Given a completed agent wizard
    When I submit it
    Then the agent appears in the list and the lab team
    Then a success toast confirms the creation
    Then an inventory event and audit row are recorded
    Then view in office resolves the new agent

  @E2E-395 @suite-management
  Scenario: E2E-395 Editing an agent is safe and reversible
    Given an existing agent
    When I edit its description, model, autonomy or capacity and save
    Then the detail and list reflect the change
    Then the form validates before saving
    Then cancel discards the change without altering the agent

  @E2E-396 @suite-management
  Scenario: E2E-396 Pause, resume and retire are clear and safe
    Given an active agent
    When I pause, resume and retire it from the detail or list
    Then each action shows its consequence and requires confirmation where needed
    Then a retired agent stays visible with its CV and desk
    Then a retired agent cannot receive new cards
    Then every action is audited
