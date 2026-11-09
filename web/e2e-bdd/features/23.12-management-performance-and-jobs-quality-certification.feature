@inventory @lab @performance @certification @p0 @ux-040
Feature: Management performance and Jobs-quality certification

  @E2E-475
  Scenario: E2E-475 Management interactions feel immediate
    Given the agents, projects, providers and settings routes
    When I search, filter, open a drawer or submit a form
    Then visual feedback starts within 100 ms
    Then the list load p95 is within the route budget
    Then no interaction blocks the main thread with a long task

  @E2E-476
  Scenario: E2E-476 Management routes meet the performance budget
    Given the management routes in a production build
    When Lighthouse and the bundle budgets run
    Then performance is at least 90 on desktop
    Then the initial JS and CSS budgets hold
    Then the route lazy-loads without a blank frame

  @E2E-477
  Scenario: E2E-477 Management visual baselines cover the responsive matrix
    Given the deterministic seed
    When I capture the agents, projects, providers and settings routes at desktop, tablet and phone widths
    Then every baseline exists
    Then the pixel diff is at most 0.1 percent with documented masks
    Then the baseline is generated from the seed, not hand-edited

  @E2E-478
  Scenario: E2E-478 Management pages do not shift or jank
    Given a slow network and a large list
    When the page loads and a drawer opens
    Then cumulative layout shift is at most 0.1
    Then no frame exceeds the long-task budget during interaction
    Then the content remains stable under the pointer

  @E2E-479
  Scenario: E2E-479 Every management scenario attaches evidence
    Given an executed management scenario
    When it completes
    Then it attaches a screenshot, trace, axe result, performance sample and the relevant logs
    Then the artifact paths are referenced by the E2E id
    Then secrets and personal data are redacted

  @E2E-480
  Scenario: E2E-480 The Jobs-quality management certification is release-blocking
    Given the complete management certification suite
    When the release candidate is evaluated
    Then clarity, hierarchy, consistency, responsiveness, usability, accessibility and performance all pass
    Then the visual baselines and evidence are current
    Then a single `@p0` failure blocks the release with the failing E2E ids
    Then the certified result is recorded as a reproducible artifact
