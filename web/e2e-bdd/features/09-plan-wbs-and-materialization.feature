@delivery @planning @p0 @fr-dlv-003
Feature: Plan, WBS and materialization

  @E2E-095 @suite-delivery
  Scenario: E2E-095 A plan is generated for an approved SOW
    Given an approved SOW
    When I call POST /api/v1/projects/{project_id}/plans
    Then the response status is 202
    Then the plan is a DAG of tasks with keys, roles, estimates, deliverables and criteria
    Then every task maps to exactly one functional requirement

  @E2E-096 @suite-delivery
  Scenario: E2E-096 Empty task lists and duplicate keys are rejected
    Given a plan with no tasks
    Then validation fails
    Then given two tasks with the same task_key
    Then validation fails

  @E2E-097 @suite-delivery
  Scenario: E2E-097 Dangling or self-referential dependencies are rejected
    Given a task that depends on itself
    Then validation fails
    Then given a task that depends on a missing key
    Then validation fails

  @E2E-098 @suite-delivery
  Scenario: E2E-098 Cyclic plans are rejected
    Given a plan where task A depends on B and B depends on A
    When I validate the plan
    Then the response status is 422
    Then the error reports a cycle

  @E2E-099 @suite-delivery
  Scenario: E2E-099 Tasks must have an estimate, a deliverable and a criterion
    Given a task with no estimate, no deliverable or no acceptance criterion
    When I validate the plan
    Then the response status is 422
    Then the violation names the incomplete task

  @E2E-100 @suite-delivery
  Scenario: E2E-100 Materialization preview selects leaves in topological order
    Given a valid acyclic plan
    When I request a materialization preview
    Then the preview returns only leaf tasks
    Then the order is a valid topological order
    Then rerunning the preview performs no writes and returns the same list

  @E2E-101 @suite-delivery
  Scenario: E2E-101 Functional requirement coverage is enforced
    Given a plan with an unknown, missing or uncovered functional requirement
    When I validate coverage
    Then the response status is 422
    Then the uncovered requirement is named

  @E2E-102 @suite-delivery
  Scenario: E2E-102 Materialization creates cards with full traceability
    Given an approved plan and auto_ready disabled
    When I materialize it
    Then TODO cards are created using project keys
    Then each card carries story, estimate, role, dependencies, task_key, covers and sow_id
    Then parent links are preserved
    Then auto_ready is allowed only when every DoR check passes
