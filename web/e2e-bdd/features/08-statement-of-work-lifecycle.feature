@delivery @sow @p0 @fr-dlv-002
Feature: Statement of Work lifecycle

  @E2E-083
  Scenario: E2E-083 Creating a SOW starts asynchronous generation
    Given an approved idea and a project
    When I call POST /api/v1/projects/{project_id}/sows
    Then the response status is 202
    Then a generation run is visible with streaming progress
    Then the SOW persists as a structured document plus Markdown artifact

  @E2E-084
  Scenario: E2E-084 A generated SOW contains every mandatory section
    Given a completed SOW generation
    When I read the SOW
    Then it contains executive_summary, objectives, scope_in, scope_out, stakeholders
    Then it contains functional_requirements, non_functional_requirements and architecture
    Then it contains milestones, deliverables, acceptance_criteria and assumptions
    Then it contains risks, timeline, budget, governance and change_control

  @E2E-085
  Scenario: E2E-085 A SOW missing or with an empty mandatory section is rejected
    Given a SOW payload with a missing or empty objectives section
    When I validate or approve it
    Then the response status is 422
    Then the violation names the missing section

  @E2E-086
  Scenario: E2E-086 A SOW with a short executive summary is rejected
    Given a SOW whose executive_summary is under 120 characters
    When I validate it
    Then the response status is 422
    Then the violation names executive_summary

  @E2E-087
  Scenario: E2E-087 Duplicate requirement identifiers are rejected
    Given a SOW with two requirements both named FR-001
    When I validate it
    Then the response status is 422
    Then the violation reports the duplicate id

  @E2E-088
  Scenario: E2E-088 Requirements need priority and testable criteria
    Given a SOW with a functional requirement missing a priority or acceptance criterion
    Then validation returns 422
    Then given a non-functional requirement without a numeric target or measurement method
    Then validation returns 422

  @E2E-089
  Scenario: E2E-089 A SOW moves from draft to in review to approved
    Given a draft SOW
    When I submit it for review
    Then the status becomes in_review
    When I approve it with an approver
    Then the status becomes approved and the approver is recorded

  @E2E-090
  Scenario: E2E-090 Approval freezes the current version
    Given an approved SOW at version 1
    When I attempt to edit version 1 in place
    Then the edit is rejected
    Then version 1 remains immutable and readable

  @E2E-091
  Scenario: E2E-091 Revision requires a change note and creates a new draft version
    Given an approved SOW at version 1
    When I revise it with an empty change note
    Then the response status is 422
    When I revise it with a non-empty change note
    Then a new immutable version 2 enters draft
    Then version 1 is unchanged

  @E2E-092
  Scenario: E2E-092 Rejection requires a reason and returns to draft
    Given a SOW in review
    When I reject it without a reason
    Then the response status is 422
    When I reject it with a reason
    Then the SOW returns to draft and the reason is recorded

  @E2E-093
  Scenario: E2E-093 Superseding requires a newer approved version
    Given an approved SOW version 1
    When I supersede it with an unapproved draft
    Then the operation is rejected
    When a newer approved version exists
    Then superseding succeeds and the old version is marked superseded

  @E2E-094
  Scenario: E2E-094 The SOW diff and export are deterministic
    Given two SOW versions
    When I call GET /api/v1/sows/{sow_id}/diff
    Then a unified diff is returned
    When I export the SOW as Markdown and as PDF
    Then the Markdown rendering is deterministic and its SHA-256 equals the content hash
    Then both artifacts download with the correct content type
