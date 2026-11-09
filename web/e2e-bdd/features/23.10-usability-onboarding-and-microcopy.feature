@inventory @lab @usability @p0 @ux-041
Feature: Usability, onboarding and microcopy

  @E2E-459
  Scenario: E2E-459 The first-run onboarding checklist is complete
    Given a fresh installation
    When the dashboard renders
    Then the checklist covers provider, key, lab, Tech Lead, GitHub, project and idea
    Then each step shows its state
    Then the next incomplete step is the primary action

  @E2E-460
  Scenario: E2E-460 Every onboarding step deep-links to its task
    Given an onboarding step
    When I open it
    Then the exact form or route opens
    Then completing the task returns me to the checklist with the step marked done
    Then I never have to hunt for the next step

  @E2E-461
  Scenario: E2E-461 Microcopy is concise, precise and human
    Given management forms, empty states and errors
    When I read the copy
    Then it states what happened and what to do next
    Then errors use the operational tone with no exclamation marks
    Then no marketing fluff appears in operational panels

  @E2E-462
  Scenario: E2E-462 Empty states have one unmistakable next action
    Given every management empty state
    When it renders
    Then it explains the missing entity in one sentence
    Then exactly one primary CTA is offered
    Then the illustration supports the action instead of competing with it

  @E2E-463
  Scenario: E2E-463 Toasts confirm without stealing the flow
    Given a successful or failed management action
    When the toast appears
    Then it is announced to assistive technology
    Then it auto-dismisses at a readable pace
    Then it never covers the primary action or the focused field

  @E2E-464
  Scenario: E2E-464 Destructive confirmations explain the consequence
    Given archive, delete, retire or reset
    When the confirmation opens
    Then it names the entity and the consequence
    Then the destructive action is visually distinct
    Then the safe action is the easy default

  @E2E-465
  Scenario: E2E-465 The UI teaches the domain without a manual
    Given autonomy, budget, DoR, DoD and provider health
    When I inspect their labels and help text
    Then each term is explained in place with a short sentence
    Then the explanation links to the deeper document where useful
    Then no undefined acronym is left unexplained

  @E2E-466
  Scenario: E2E-466 The management experience feels calm under failure
    Given a server error, a validation failure and a timeout
    When each occurs
    Then the UI stays composed, explains the state and offers a next step
    Then no raw stack, JSON blob or internal code is shown to the operator
    Then the retry path is always available
