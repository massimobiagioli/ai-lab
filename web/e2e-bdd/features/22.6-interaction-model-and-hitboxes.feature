@office @interaction @p0 @ux-010
Feature: Interaction model and hitboxes

  @E2E-297
  Scenario: E2E-297 Clicking an agent opens the Live inspector
    Given the office is open
    When I click an agent
    Then the right-side inspector opens on the Live tab
    Then it shows status, current card and the last 20 run steps
    Then the selection is visible in the office

  @E2E-298
  Scenario: E2E-298 Clicking a desk opens the inspector with desk metadata
    Given the office is open
    When I click a desk
    Then the same inspector opens
    Then the desk metadata is shown
    Then the selected agent matches the desk

  @E2E-299
  Scenario: E2E-299 Inspector tabs are keyboard accessible
    Given the inspector is open
    When I use the keyboard on Live, CV, Memory and Controls
    Then roving tabindex and Arrow, Home and End work
    Then Enter and Space activate the tab
    Then focus is visible on every tab

  @E2E-300
  Scenario: E2E-300 The inspector can be resized and closed safely
    Given the inspector is open
    When I resize it and press Escape
    Then it slides out and closes
    Then focus returns to the triggering object
    Then the office remains interactive

  @E2E-301
  Scenario: E2E-301 Agent controls are functional and audited
    Given the inspector Controls tab
    When I pause, resume, reassign, message, set autonomy or retire an agent
    Then each action succeeds or reports a clear refusal
    Then each state change emits an event and an audit row
    Then the office reflects the change immediately

  @E2E-302
  Scenario: E2E-302 The whiteboard opens Kanban in split view
    Given the office is open
    When I click the whiteboard
    Then the Kanban drawer opens filtered to the project
    Then the office remains visible in split view
    Then the filter is reflected in the URL or state

  @E2E-303
  Scenario: E2E-303 The server rack opens System Health
    Given the office is open
    When I click the server rack
    Then the System Health drawer opens
    Then it shows provider, database, Redis, queue, error rate and budget
    Then the values match the API health endpoints

  @E2E-304
  Scenario: E2E-304 The bookshelf opens the Memory explorer
    Given the office is open
    When I click the bookshelf
    Then the Memory explorer drawer opens
    Then it supports scope switching, search, recent documents and add note
    Then no cross-scope result is shown

  @E2E-305
  Scenario: E2E-305 The filing cabinet opens failed runs and replay
    Given the office is open
    When I click the filing cabinet
    Then the failed-runs and dead-letter drawer opens
    Then each failed run shows its error and cost
    Then a replay action creates a new attempt without mutating the old run

  @E2E-306
  Scenario: E2E-306 The remaining objects open their documented surfaces
    Given the office is open
    When I click the meeting table, door, coffee machine, plant and pet
    Then the meeting table opens the handoffs timeline
    Then the door opens retired agents
    Then the coffee machine opens idle agents
    Then the plant and pet only play a cosmetic effect

  @E2E-307
  Scenario: E2E-307 Hover feedback is immediate and pixel-styled
    Given the office is open
    When I hover an interactive object
    Then a pixel tooltip and outline appear within 100 ms
    Then the tooltip does not cover the object entirely
    Then leaving the object removes the feedback

  @E2E-308
  Scenario: E2E-308 Keyboard navigation reaches every office object
    Given the office is open without a mouse
    When I press Tab, Enter, Escape and the arrow keys
    Then Tab cycles through interactive objects in a stable order
    Then Enter opens the focused object
    Then Escape closes the active drawer
    Then the arrow keys pan the camera
