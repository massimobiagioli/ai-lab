@inventory @providers @security @ui @p0 @ux-060
Feature: Providers inventory and write-only key UI

  @E2E-407
  Scenario: E2E-407 The providers list is readable and operational
    Given several providers
    When the list renders
    Then each row shows name, kind, base URL host, status, last4, last health and rate limits
    Then the status and last4 are immediately legible
    Then the create action is obvious

  @E2E-408
  Scenario: E2E-408 The provider creation wizard covers the contract
    Given the create-provider action
    When I open the wizard
    Then the fields cover name, kind, base URL, key, rate limits and timeout
    Then the key is a write-only input
    Then I cannot submit an invalid provider

  @E2E-409
  Scenario: E2E-409 The API key is never echoed or recoverable
    Given the provider key field and a saved provider
    When I type, save and reopen the provider
    Then the full key is never shown in the DOM, network response, toast or console
    Then only the last four characters are displayed
    Then the field is cleared after submit

  @E2E-410
  Scenario: E2E-410 Provider fields are validated inline
    Given the provider form
    When I submit a non-HTTPS URL, a duplicate name, a non-positive limit or a key outside the allowed length
    Then an inline error explains the rule
    Then a valid provider is accepted
    Then the validation happens before any external call

  @E2E-411
  Scenario: E2E-411 Provider verification shows actionable states
    Given a provider
    When I verify it and the upstream returns success, degraded, timeout or unauthorized
    Then each state is visible with a clear message
    Then the provider status updates accordingly
    Then a failure offers a retry or an edit action

  @E2E-412
  Scenario: E2E-412 Key rotation is hidden, confirmed and audited
    Given an existing provider
    When I rotate its key
    Then the new key is entered through a hidden prompt
    Then the old last4 is replaced only after a successful save
    Then the audit row records the rotation without the key

  @E2E-413
  Scenario: E2E-413 Disabling and enabling a provider is deliberate
    Given an active provider
    When I disable and then enable it
    Then the status changes and is visible in the list and detail
    Then a confirmation explains the impact on runs
    Then the action is audited

  @E2E-414
  Scenario: E2E-414 Provider health detail is understandable
    Given a provider with a health history
    When I open its detail
    Then the last health time and detail are shown in plain language
    Then the difference between degraded and disabled is explained
    Then the next action is clear

  @E2E-415
  Scenario: E2E-415 Provider errors are actionable, not alarming
    Given an invalid key, a 401, a degraded endpoint or a timeout
    When the error is displayed
    Then the message states what happened and what to do next
    Then it uses the precise operational tone with no exclamation marks
    Then the provider form remains editable

  @E2E-416
  Scenario: E2E-416 No secret leaks through any management surface
    Given a provider key was entered
    When I inspect the DOM, network, logs, toasts, errors and traces
    Then no plaintext key appears
    Then the redactor covers the key shape
    Then the browser history and URL contain no key
