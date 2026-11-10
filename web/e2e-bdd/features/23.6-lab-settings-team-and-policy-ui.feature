@lab @settings @ui @p0 @ux-060
Feature: Lab settings, team and policy UI

  @E2E-417 @suite-management
  Scenario: E2E-417 The settings page is grouped into clear sections
    Given the lab settings route
    When the page renders
    Then the sections cover lab, autonomy, budget, scheduler, office and danger zone
    Then each section has a clear heading and description
    Then the primary save action is obvious

  @E2E-418 @suite-management
  Scenario: E2E-418 Lab identity is editable and validated
    Given the lab section
    When I change the name, slug or description
    Then the fields validate inline
    Then saving updates the lab and its shell label
    Then cancel discards the change

  @E2E-419 @suite-management
  Scenario: E2E-419 Autonomy levels are explained before selection
    Given the autonomy section
    When I inspect A0, A1, A2 and A3
    Then each level explains what agents may do and what requires a human
    Then the current level is selected
    Then a change is confirmed with its impact

  @E2E-420 @suite-management
  Scenario: E2E-420 A3 requires an explicit opt-in and a hard budget
    Given the autonomy section
    When I select A3 without an explicit opt-in or a hard monthly budget
    Then the save is blocked with a clear explanation
    Then the opt-in and budget fields are revealed
    Then a valid A3 configuration saves and is audited

  @E2E-421 @suite-management
  Scenario: E2E-421 Budget limits are coherent
    Given the budget section
    When I set a per-card budget greater than the per-day budget or a negative value
    Then an inline error explains the relationship
    Then a valid pair with a hard stop saves
    Then the effective limits are visible after saving

  @E2E-422 @suite-management
  Scenario: E2E-422 Scheduler settings are bounded and understandable
    Given the scheduler section
    When I edit the tick, lease TTL or retry count outside the allowed range
    Then the control rejects the value with an explanation
    Then valid values save
    Then the defaults are one click away

  @E2E-423 @suite-management
  Scenario: E2E-423 Office settings preview their effect
    Given the office section
    When I change the theme, sound or reduced motion
    Then a preview reflects the choice immediately
    Then saving persists it to the lab
    Then reduced motion removes non-essential animation

  @E2E-424 @suite-management
  Scenario: E2E-424 WIP limits are validated as a pair
    Given the WIP section
    When I set the DOING limit or the REVIEW limit outside its range
    Then an inline error explains the limit
    Then valid limits save
    Then the current limits are visible in the scheduler context

  @E2E-425 @suite-management
  Scenario: E2E-425 The team always has exactly one active Tech Lead
    Given the team section
    When I add, remove or reassign roles
    Then the one-Tech-Lead rule is enforced with a clear message
    Then an empty Tech Lead cannot be saved
    Then a valid team composition saves

  @E2E-426 @suite-management
  Scenario: E2E-426 Removing the last Tech Lead is blocked
    Given a lab with one Tech Lead
    When I try to remove or retire that agent
    Then the action is refused with an explanation
    Then the team remains valid
    Then the UI suggests adding a replacement

  @E2E-427 @suite-management
  Scenario: E2E-427 Adding an agent from settings reuses the wizard
    Given the team section
    When I add an agent
    Then the same validated agent wizard opens
    Then the role is compatible with the lab
    Then the new agent appears in the team and inventory

  @E2E-428 @suite-management
  Scenario: E2E-428 Saving settings is optimistic and recoverable
    Given a valid settings change
    When I save
    Then the UI applies the change immediately
    Then a server validation error rolls it back with the field error
    Then a success toast confirms the save

  @E2E-429 @suite-management
  Scenario: E2E-429 Resetting settings requires confirmation
    Given changed settings
    When I reset to defaults
    Then a confirmation explains what will be restored
    Then cancelling leaves the values unchanged
    Then confirming restores and audits the defaults

  @E2E-430 @suite-management
  Scenario: E2E-430 The danger zone is explicit and safe
    Given the danger zone
    When I archive, export or delete the lab
    Then each action explains its consequence
    Then archive and delete require typed confirmation
    Then the result is audited and the UI returns to a safe state
