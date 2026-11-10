@memory @p0 @fr-mem-001
Feature: Scoped memory and hybrid retrieval

  @E2E-127 @suite-platform
  Scenario: E2E-127 Exactly three memory scopes exist
    Given the memory API
    When I create a document with scope lab, project or agent and a matching scope id
    Then the document is accepted
    When I use scope global or an unknown scope
    Then the response status is 422

  @E2E-128 @suite-platform
  Scenario: E2E-128 Scope id must match the scope
    Given a project-scoped document
    When I provide a lab id as scope_id
    Then the response status is 422
    Then no document or embedding is written

  @E2E-129 @suite-platform
  Scenario: E2E-129 Chunking is deterministic
    Given a document of N words
    When I ingest it twice
    Then the chunk boundaries and token counts are identical
    Then chunks target 800 words with 120-word overlap

  @E2E-130 @suite-platform
  Scenario: E2E-130 Embeddings have 384 dimensions
    Given an ingested chunk
    When I inspect its embedding
    Then the vector has exactly 384 dimensions

  @E2E-131 @suite-platform
  Scenario: E2E-131 Secret-shaped documents are quarantined
    Given a document containing a known secret shape such as sk-, ghp_ or a PEM header
    When I ingest it
    Then the response status is 422
    Then the error code is MEMORY_QUARANTINED
    Then the document is not embedded

  @E2E-132 @suite-platform
  Scenario: E2E-132 Hybrid retrieval ranks by the specified formula
    Given a fixed retrieval fixture
    When I search
    Then the weighted score is 0.7 * max(cosine, 0) + 0.3 * lexical_rank
    Then the final score applies exp(-age_days / 30) and the scope boost
    Then results are ordered by score to the fixture tolerance

  @E2E-133 @suite-platform
  Scenario: E2E-133 Retrieval never crosses scopes
    Given a lab, a project and an agent each with distinct documents
    When I search the project scope for text that exists only in the lab scope
    Then no lab-scoped chunk is returned
    Then the SQL filter is applied before scoring

  @E2E-134 @suite-platform
  Scenario: E2E-134 Near-duplicate chunks are deduplicated
    Given two chunks with cosine similarity at least 0.92 in the same scope
    When I search
    Then only one of them is returned

  @E2E-135 @suite-platform
  Scenario: E2E-135 Compaction and retention respect pins
    Given episodes older than 30 days and raw episodes older than 180 days
    When compaction runs
    Then old episodes are summarized into semantic documents
    Then raw episodes older than 180 days are deleted
    Then pinned episodes are preserved

  @E2E-136 @suite-platform
  Scenario: E2E-136 Every retrieval is logged for audit and evaluation
    Given a search request
    When it completes
    Then a log row records the query, scopes, chunk ids and scores

  @E2E-137 @suite-platform
  Scenario: E2E-137 Memory notes can be created, pinned and deleted
    Given the memory explorer
    When I add a note, pin it and later delete it
    Then each action succeeds and is audited
    Then the pinned note survives compaction

  @E2E-138 @suite-platform
  Scenario: E2E-138 Memory import and export respect size and privacy rules
    Given a memory import larger than 10 MB
    When I submit it
    Then the request is rejected
    Then given a valid export request the data is returned without plaintext secrets
