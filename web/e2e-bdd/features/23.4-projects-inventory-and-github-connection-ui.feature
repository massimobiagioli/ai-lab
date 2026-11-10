@inventory @projects @ui @p0 @ux-060
Feature: Projects inventory and GitHub connection UI

  @E2E-397 @suite-management
  Scenario: E2E-397 The projects list explains each project at a glance
    Given several projects
    When the list renders
    Then each row shows name, slug, key prefix, repository, default branch, status and lab
    Then the visual hierarchy makes the project name the anchor
    Then the create action is obvious

  @E2E-398 @suite-management
  Scenario: E2E-398 Projects can be searched, filtered and sorted
    Given projects with distinct names, statuses and labs
    When I search, filter by status or lab, and sort
    Then the list updates deterministically
    Then the active state is visible and reflected in the URL
    Then an empty filter result offers a clear reset

  @E2E-399 @suite-management
  Scenario: E2E-399 The project creation wizard is complete
    Given the create-project action
    When I open the wizard
    Then the steps cover name, slug, key prefix, lab, SCM credential, repository and default branch
    Then the review step summarises the choices
    Then I cannot submit an incomplete project

  @E2E-400 @suite-management
  Scenario: E2E-400 Slug and key prefix are validated inline
    Given the slug and prefix fields
    When I enter a duplicate slug, a duplicate prefix, a malformed prefix or an invalid character
    Then an inline error explains the rule
    Then a valid slug and prefix are accepted
    Then the uniqueness check is scoped to the lab

  @E2E-401 @suite-management
  Scenario: E2E-401 The SCM credential is selected and verified
    Given the repository step
    When I choose an SCM credential
    Then the token is never displayed
    Then the connection can be verified
    Then an invalid or expired credential produces an actionable error

  @E2E-402 @suite-management
  Scenario: E2E-402 The project detail is a control panel
    Given a project detail
    When it renders
    Then it shows repository metadata, sync status, last sync, board, SOW and plan links
    Then the primary next action is clear
    Then the page does not overwhelm with secondary actions

  @E2E-403 @suite-management
  Scenario: E2E-403 Project sync reports progress and recovers
    Given a connected project
    When I trigger sync
    Then progress and result are visible
    Then a failure shows the reason and a retry
    Then a successful sync updates the repository metadata

  @E2E-404 @suite-management
  Scenario: E2E-404 Archiving a project is deliberate and reversible in the UI
    Given an active project
    When I archive it with the typed confirmation
    Then it leaves the active list and remains available in the archived filter
    Then new cards are blocked with a clear message
    Then the action is audited

  @E2E-405 @suite-management
  Scenario: E2E-405 Repository and credential errors never become dead ends
    Given an archived, empty or inaccessible repository or a bad credential
    When the project step validates it
    Then the error explains the exact problem and how to fix it
    Then the user can change the repository or credential without losing the form
    Then no silent failure occurs

  @E2E-406 @suite-management
  Scenario: E2E-406 A project links to its board, SOW and plan
    Given a project detail
    When I follow the board, SOW and plan links
    Then the matching route opens with the project context
    Then returning to the project preserves the shell state
    Then the links are keyboard reachable
