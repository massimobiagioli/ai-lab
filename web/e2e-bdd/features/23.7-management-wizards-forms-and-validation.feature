@inventory @lab @ui @forms @p1 @ux-061
Feature: Management wizards, forms and validation

  @E2E-431 @suite-management
  Scenario: E2E-431 Wizard progress is visible and honest
    Given any management wizard
    When I advance through the steps
    Then the current step, completed steps and remaining steps are visible
    Then the progress indicator is not clickable ahead of validation
    Then the final review is reachable only when the steps are valid

  @E2E-432 @suite-management
  Scenario: E2E-432 Back and cancel preserve or discard deliberately
    Given a partially completed wizard
    When I go back
    Then my entries are preserved
    When I cancel with unsaved changes
    Then I am asked to confirm the discard
    Then confirming returns me to the list unchanged

  @E2E-433 @suite-management
  Scenario: E2E-433 Validation is inline and summarised
    Given a form with an invalid field
    When I blur the field and submit
    Then an inline message is linked with aria-describedby
    Then a summary lists the errors and focuses the first invalid field
    Then the submit is blocked with a clear reason

  @E2E-434 @suite-management
  Scenario: E2E-434 Required and optional fields are unambiguous
    Given every management form
    When I inspect the labels
    Then required fields are marked in text, not colour alone
    Then optional fields are clearly optional
    Then placeholders never replace a label

  @E2E-435 @suite-management
  Scenario: E2E-435 Every wizard is completable by keyboard
    Given a management wizard
    When I use only the keyboard
    Then I can move through every step, select, type and submit
    Then Enter submits and Escape cancels with confirmation
    Then focus is never trapped outside the active surface

  @E2E-436 @suite-management
  Scenario: E2E-436 Drawers and dialogs manage focus correctly
    Given a drawer or dialog
    When it opens and closes
    Then focus moves inside it, is trapped, and returns to the trigger
    Then the surface has aria-modal and an accessible name
    Then Escape closes it without losing data unexpectedly

  @E2E-437 @suite-management
  Scenario: E2E-437 Leaving a dirty form is guarded
    Given a form with unsaved changes
    When I navigate away or close the drawer
    Then a confirmation explains the unsaved change
    Then cancelling keeps me in the form
    Then confirming discards and navigates

  @E2E-438 @suite-management
  Scenario: E2E-438 Submit loading prevents duplicates
    Given a valid form
    When I submit it
    Then the primary action enters a loading state and is disabled
    Then a second submit is ignored
    Then success or error resolves the state with a toast

  @E2E-439 @suite-management
  Scenario: E2E-439 Server violations map to the fields
    Given a server 422 response with violations
    When the form receives it
    Then each violation is attached to its field
    Then a summary names the problem in plain language
    Then the entered data is preserved

  @E2E-440 @suite-management
  Scenario: E2E-440 Long content never breaks the layout
    Given very long names, descriptions, URLs and labels
    When the list, detail and form render
    Then text wraps or truncates with an accessible full value
    Then no action is pushed off screen
    Then the layout remains aligned at every breakpoint
