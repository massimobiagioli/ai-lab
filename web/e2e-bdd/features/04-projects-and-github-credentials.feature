@inventory @scm @p0 @fr-inv-030
Feature: Projects and GitHub credentials

  @E2E-027
  Scenario: E2E-027 Create a GitHub project referencing an SCM credential
    Given a lab and an SCM credential exist
    When I submit a project with name, unique slug, owner/repo and credential_id
    Then the response status is 201
    Then the project stores a credential reference, not a raw token
    Then the default branch defaults to main

  @E2E-028
  Scenario: E2E-028 Duplicate project slug or key prefix is rejected
    Given a lab already has a project with slug "csv-lab" and prefix "CSV"
    When I create another project in the same lab with the same slug or prefix
    Then the response status is 422
    Then the error code is VALIDATION_FAILED

  @E2E-029
  Scenario: E2E-029 Project key prefix pattern is enforced
    Given the project create form
    When I submit a prefix such as "1abc", "CSV-TOOL" or "abcdefghijk"
    Then the response status is 422
    When I submit "C" + up to nine uppercase alphanumerics
    Then the response status is 201

  @E2E-030
  Scenario: E2E-030 The SCM token is write-only
    Given I create a GitHub credential with a fine-grained PAT
    Then the response exposes only the last four characters
    Then no subsequent GET returns the plaintext token
    Then the stored value is encrypted ciphertext with a key version

  @E2E-031
  Scenario: E2E-031 Project sync is queued and reconciles repository state
    Given an active GitHub-connected project
    When I call POST /api/v1/projects/{project_id}/sync
    Then the response status is 202
    Then a sync job records the repository metadata and default branch
    Then the result is observable in the project detail

  @E2E-032
  Scenario: E2E-032 Unsafe repositories are refused
    Given a repository that is archived, disabled or empty
    When I connect it or trigger a sync
    Then the operation is refused with a validation error
    Then no branch, PR or check operation is attempted

  @E2E-033
  Scenario: E2E-033 Branch naming and path safety are enforced
    Given card key "AIL-42" and title "CSV inference"
    When the adapter creates the branch
    Then the branch name is ailab/AIL-42-csv-inference
    Then a path containing "..", a leading slash or ".git/" is rejected
    Then a branch name with unsafe characters is rejected

  @E2E-034
  Scenario: E2E-034 Pull request conventions and webhook verification
    Given a branch and a card
    When the adapter opens the pull request
    Then the title is "[AIL-42] CSV inference"
    Then the body contains card, summary, changes, acceptance criteria, verification, risks and the AI-Lab trace
    When GitHub delivers a webhook with a valid X-Hub-Signature-256
    Then the webhook is accepted
    When the signature is invalid or the delivery is a replay older than five minutes
    Then it is rejected and no state is applied
