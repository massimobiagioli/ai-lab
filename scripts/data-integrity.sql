-- Report-only data integrity checks. Every value must be zero.
SELECT jsonb_build_object(
  'card_projection_mismatch', (
    SELECT count(*)
    FROM cards c
    LEFT JOIN LATERAL (
      SELECT to_column FROM card_events WHERE card_id = c.id ORDER BY seq DESC LIMIT 1
    ) e ON true
    WHERE c.column_name IN ('TODO', 'READY', 'DOING', 'REVIEW', 'DONE')
      AND e.to_column IS DISTINCT FROM c.column_name
  ),
  'ready_missing_dor', (
    SELECT count(*) FROM cards
    WHERE column_name = 'READY'
      AND (assignee_id IS NULL OR due_date IS NULL OR estimate IS NULL OR priority = 'wont')
  ),
  'done_without_merge_sha', (
    SELECT count(*) FROM cards WHERE column_name = 'DONE' AND merge_sha IS NULL
  ),
  'review_without_reviewer', (
    SELECT count(*) FROM cards
    WHERE column_name = 'REVIEW'
      AND (reviewer_id IS NULL OR pr_url IS NULL OR reviewer_id = assignee_id)
  ),
  'orphan_cards', (
    SELECT count(*) FROM cards c
    LEFT JOIN projects p ON p.id = c.project_id
    LEFT JOIN boards b ON b.id = c.board_id
    WHERE p.id IS NULL OR b.id IS NULL
  ),
  'outbox_stuck', (
    SELECT count(*) FROM outbox
    WHERE published_at IS NULL AND created_at < now() - interval '10 minutes'
  ),
  'memory_chunk_nulls', (
    SELECT count(*) FROM memory_chunks WHERE embedding IS NULL
  )
) AS report;
