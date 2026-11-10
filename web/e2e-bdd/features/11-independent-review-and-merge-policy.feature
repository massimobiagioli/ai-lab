@delivery @qa @p0 @fr-agt-001
Feature: Independent review and merge policy

  @E2E-119 @suite-delivery
  Scenario: E2E-119 An approved verdict requires every criterion to pass
    Given a review with all acceptance criteria marked pass and no P0, P1 or P2 finding
    When QA submits the verdict
    Then the verdict is approved
    Then the review carries review_id, run_id, reviewer_id, head_sha and the matrix

  @E2E-120 @suite-delivery
  Scenario: E2E-120 Changes requested requires actionable findings
    Given a review with at least one failing criterion
    When QA submits changes_requested without findings
    Then the verdict is rejected
    When findings with severity, file and fix are attached
    Then the verdict is accepted and the card returns to DOING

  @E2E-121 @suite-delivery
  Scenario: E2E-121 Self-review is rejected
    Given a reviewer who is also the author
    When the review is submitted
    Then the response status is 422
    Then the error code is SEPARATION_OF_DUTIES

  @E2E-122 @suite-delivery
  Scenario: E2E-122 Merge requires all policy conditions
    Given a PR that is not merged, has failing CI or lacks QA approval
    When a merge is attempted
    Then the merge is refused with PR_NOT_MERGED, CI_NOT_GREEN or QA_NOT_APPROVED
    When all conditions hold and the merge SHA is recorded
    Then the merge proceeds

  @E2E-123 @suite-delivery
  Scenario: E2E-123 A0 and A1 require a human merge for agent-initiated merges
    Given a lab at autonomy A0 or A1
    When an agent attempts to merge
    Then the merge is refused with HUMAN_MERGE_REQUIRED
    Then the operator is prompted to merge

  @E2E-124 @suite-delivery
  Scenario: E2E-124 A2 and A3 allow autonomous merge after the checks
    Given a lab at autonomy A2 or A3 with QA approval, green CI, no P0/P1 and budget remaining
    When the agent merges with a squash merge
    Then the merge succeeds
    Then the card can reach DONE

  @E2E-125 @suite-delivery
  Scenario: E2E-125 Repeated merges of the same head are idempotent
    Given a merge already completed for head SHA abc123
    When the merge is invoked again for the same head
    Then the response is success
    Then no second merge is performed

  @E2E-126 @suite-delivery
  Scenario: E2E-126 Forbidden changes and protected branches are hard stops
    Given a diff touching .github/workflows, a secret or more than 1500 lines
    When the merge policy evaluates it
    Then the merge is refused with FORBIDDEN_DIFF or the matching hard stop
    Then a merge targeting a protected branch is refused with PROTECTED_BRANCH
