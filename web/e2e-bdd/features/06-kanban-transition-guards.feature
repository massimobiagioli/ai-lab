@kanban @guards @p0 @fr-kan-020
Feature: Kanban transition guards

  @E2E-057
  Scenario: E2E-057 TODO to READY succeeds when the Definition of Ready is complete
    Given a TODO card with a canonical story, one criterion, one deliverable, estimate 3, priority must, an assignee, a future due date and a positive budget
    When I transition the card from TODO to READY
    Then the response status is 200
    Then the card column becomes READY
    Then a kanban.card.moved event is emitted

  @E2E-058
  Scenario: E2E-058 TODO to READY fails without an acceptance criterion
    Given a TODO card whose story has no acceptance criteria
    When I transition it to READY
    Then the response status is 422
    Then the violation rule is DOR_AC_MISSING and the field is story.acceptance_criteria

  @E2E-059
  Scenario: E2E-059 TODO to READY fails without a deliverable
    Given a TODO card whose story has no deliverables
    When I transition it to READY
    Then the response status is 422
    Then the violation rule is DOR_DELIVERABLE_MISSING

  @E2E-060
  Scenario: E2E-060 TODO to READY fails when the card still needs refinement
    Given a TODO card flagged needs_refinement true
    When I transition it to READY
    Then the response status is 422
    Then the violation rule is DOR_NEEDS_REFINEMENT

  @E2E-061
  Scenario: E2E-061 TODO to READY fails on an invalid estimate or priority
    Given a TODO card with estimate 13 or priority wont
    When I transition it to READY
    Then the response status is 422
    Then the violations include DOR_ESTIMATE_INVALID or DOR_PRIORITY_INVALID

  @E2E-062
  Scenario: E2E-062 TODO to READY fails without assignee, due date or budget
    Given a TODO card missing an assignee, a future due date or a positive budget
    When I transition it to READY
    Then the response status is 422
    Then the violations include DOR_ASSIGNEE_MISSING, DOR_DUE_DATE_INVALID or DOR_BUDGET_INVALID

  @E2E-063
  Scenario: E2E-063 TODO to ICEBOX requires a categorized reason
    Given a TODO card
    When I transition it to ICEBOX with category risk and a resolution note
    Then the response status is 200
    Then the blocked reason is stored and the card is in ICEBOX

  @E2E-064
  Scenario: E2E-064 ICEBOX to TODO requires a recorded resolution
    Given an ICEBOX card with a blocked reason
    When I transition it to TODO with a resolution text and actor
    Then the response status is 200
    Then the blocked reason is cleared
    Then needs_refinement is set to true

  @E2E-065
  Scenario: E2E-065 READY to DOING succeeds within the WIP limit
    Given a READY card assigned to an active agent below its DOING limit
    When the scheduler leases and claims the card
    Then the response status is 200
    Then the card column becomes DOING
    Then a run row exists for the attempt

  @E2E-066
  Scenario: E2E-066 READY to DOING fails when the WIP limit is exceeded
    Given the assignee already has the maximum number of DOING cards
    When I claim another READY card
    Then the response status is 422
    Then the error code is WIP_LIMIT_EXCEEDED

  @E2E-067
  Scenario: E2E-067 READY to TODO is a Tech Lead refinement action
    Given a READY card and a Tech Lead actor
    When the Tech Lead moves it back to TODO with a reason
    Then the response status is 200
    Then needs_refinement is set to true
    Then a non-Tech-Lead actor is rejected with 403 or 422

  @E2E-068
  Scenario: E2E-068 READY to ICEBOX blocks the card and notifies the assignee
    Given a READY card
    When I move it to ICEBOX with a categorized reason
    Then the response status is 200
    Then the assignee receives a notification
    Then the card remains visible in ICEBOX with its reason

  @E2E-069
  Scenario: E2E-069 DOING to REVIEW succeeds with a valid PR and green checks
    Given a DOING card with a run that succeeded, an open PR, a pushed branch, a non-empty log and a reviewer different from the assignee
    When I transition it to REVIEW
    Then the response status is 200
    Then the card column becomes REVIEW
    Then delivery.pr.opened and kanban.card.moved are emitted

  @E2E-070
  Scenario: E2E-070 Self-review is rejected by separation of duties
    Given a DOING card whose reviewer_id equals its assignee_id
    When I transition it to REVIEW
    Then the response status is 422
    Then the error code is SEPARATION_OF_DUTIES

  @E2E-071
  Scenario: E2E-071 DOING to READY is an abandon transition
    Given a DOING card with a running attempt
    When the Tech Lead or the assignee abandons it with a reason
    Then the response status is 200
    Then the run is cancelled
    Then the card returns to READY

  @E2E-072
  Scenario: E2E-072 DOING to ICEBOX requires a reason and a terminal run
    Given a DOING card whose run failed terminally
    When I move it to ICEBOX with a categorized reason
    Then the response status is 200
    Then the run is not left running

  @E2E-073
  Scenario: E2E-073 REVIEW to DONE succeeds only with the full Definition of Done
    Given a REVIEW card with an approved QA verdict, reviewer different from author, green CI, a merged PR, a merge SHA, complete DoD and no P0/P1
    When I transition it to DONE
    Then the response status is 200
    Then the card column becomes DONE
    Then delivery.merge.completed is emitted and CV evidence is appended

  @E2E-074
  Scenario: E2E-074 REVIEW to DONE reports every unmet DoD rule
    Given a REVIEW card missing the merge SHA, with failing CI or without QA approval
    When I transition it to DONE
    Then the response status is 422
    Then the violations include DOD_MERGE_MISSING, DOD_CI_NOT_GREEN or DOD_QA_NOT_APPROVED

  @E2E-075
  Scenario: E2E-075 REVIEW to DOING returns changes with findings attached
    Given a QA verdict CHANGES_REQUESTED with at least one finding
    When I transition the card from REVIEW to DOING
    Then the response status is 200
    Then the findings are attached to the card history
    Then the Dev can read the actionable feedback

  @E2E-076
  Scenario: E2E-076 REVIEW to ICEBOX requires a reason and findings
    Given a REVIEW card that cannot proceed
    When I move it to ICEBOX with a categorized reason and attached findings
    Then the response status is 200
    Then the card is not silently dropped
