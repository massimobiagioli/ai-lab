-- Fixed analytics fixture. Applied after APPENDIX-A-DATABASE.sql.
-- All timestamps are UTC. Expected values are asserted by .tools/test-analytics-views.sh.

SET TIME ZONE 'UTC';

INSERT INTO labs (id, workspace_id, name, slug, autonomy_level)
VALUES ('11111111-1111-7111-8111-111111111111',
        '00000000-0000-7000-8000-000000000001',
        'Analytics Lab', 'analytics-lab', 2);

INSERT INTO agents (id, lab_id, name, role, character_id, description, autonomy_level, max_concurrent_cards)
VALUES
  ('aaaaaaaa-aaaa-7aaa-8aaa-aaaaaaaaaaa1',
   '11111111-1111-7111-8111-111111111111',
   'Dev One', 'dev', 'scrappy_dev',
   'Fixture developer agent used only to verify analytics views; it never calls an LLM and exists solely to exercise cost, autonomy and flow aggregations with deterministic timestamps and values.',
   2, 1),
  ('aaaaaaaa-aaaa-7aaa-8aaa-aaaaaaaaaaa2',
   '11111111-1111-7111-8111-111111111111',
   'QA One', 'qa', 'precise_consultant',
   'Fixture QA agent used only to verify analytics views; it reviews fixture cards and exists solely to exercise autonomy and separation-of-duties aggregations with deterministic timestamps and values.',
   2, 1);

INSERT INTO projects (id, lab_id, name, slug, key_prefix, source_ref)
VALUES ('22222222-2222-7222-8222-222222222222',
        '11111111-1111-7111-8111-111111111111',
        'Analytics Project', 'analytics-project', 'AIL',
        'example/analytics-project');

INSERT INTO boards (id, project_id)
VALUES ('33333333-3333-7333-8333-333333333333',
        '22222222-2222-7222-8222-222222222222');

-- C1: rework card, human intervention, $1.
INSERT INTO cards (id, board_id, project_id, card_key, seq, title, story, column_name,
                   priority, estimate, assignee_id, reviewer_id, due_date, source,
                   merge_sha, pr_url, created_at, updated_at)
VALUES ('44444444-4444-7444-8444-444444444441',
        '33333333-3333-7333-8333-333333333333',
        '22222222-2222-7222-8222-222222222222',
        'AIL-1', 1, 'Fixture card one', '{}'::jsonb, 'DONE',
        'must', 3,
        'aaaaaaaa-aaaa-7aaa-8aaa-aaaaaaaaaaa1',
        'aaaaaaaa-aaaa-7aaa-8aaa-aaaaaaaaaaa2',
        '2026-02-01T00:00:00Z', 'human',
        'deadbeef01', 'https://example.test/pr/1',
        '2026-01-01T00:00:00Z', '2026-01-01T01:00:00Z');

-- C2: straight-through autonomous card, $2.
INSERT INTO cards (id, board_id, project_id, card_key, seq, title, story, column_name,
                   priority, estimate, assignee_id, reviewer_id, due_date, source,
                   merge_sha, pr_url, created_at, updated_at)
VALUES ('44444444-4444-7444-8444-444444444442',
        '33333333-3333-7333-8333-333333333333',
        '22222222-2222-7222-8222-222222222222',
        'AIL-2', 2, 'Fixture card two', '{}'::jsonb, 'DONE',
        'should', 5,
        'aaaaaaaa-aaaa-7aaa-8aaa-aaaaaaaaaaa1',
        'aaaaaaaa-aaaa-7aaa-8aaa-aaaaaaaaaaa2',
        '2026-02-01T00:00:00Z', 'human',
        'deadbeef02', 'https://example.test/pr/2',
        '2026-01-01T00:00:00Z', '2026-01-01T00:30:00Z');

-- C3: blocked then autonomous card, no cost.
INSERT INTO cards (id, board_id, project_id, card_key, seq, title, story, column_name,
                   priority, estimate, assignee_id, reviewer_id, due_date, source,
                   merge_sha, pr_url, created_at, updated_at)
VALUES ('44444444-4444-7444-8444-444444444443',
        '33333333-3333-7333-8333-333333333333',
        '22222222-2222-7222-8222-222222222222',
        'AIL-3', 3, 'Fixture card three', '{}'::jsonb, 'DONE',
        'could', 2,
        'aaaaaaaa-aaaa-7aaa-8aaa-aaaaaaaaaaa2',
        'aaaaaaaa-aaaa-7aaa-8aaa-aaaaaaaaaaa1',
        '2026-02-01T00:00:00Z', 'human',
        'deadbeef03', 'https://example.test/pr/3',
        '2026-01-01T00:00:00Z', '2026-01-01T00:40:00Z');

-- C4: current DOING.
INSERT INTO cards (id, board_id, project_id, card_key, seq, title, story, column_name,
                   priority, estimate, assignee_id, due_date, source, created_at, updated_at)
VALUES ('44444444-4444-7444-8444-444444444444',
        '33333333-3333-7333-8333-333333333333',
        '22222222-2222-7222-8222-222222222222',
        'AIL-4', 4, 'Fixture card four', '{}'::jsonb, 'DOING',
        'should', 3,
        'aaaaaaaa-aaaa-7aaa-8aaa-aaaaaaaaaaa1',
        '2026-02-01T00:00:00Z', 'human',
        '2026-01-02T00:00:00Z', '2026-01-02T00:00:00Z');

-- C5: current REVIEW.
INSERT INTO cards (id, board_id, project_id, card_key, seq, title, story, column_name,
                   priority, estimate, assignee_id, reviewer_id, due_date, source,
                   pr_url, created_at, updated_at)
VALUES ('44444444-4444-7444-8444-444444444445',
        '33333333-3333-7333-8333-333333333333',
        '22222222-2222-7222-8222-222222222222',
        'AIL-5', 5, 'Fixture card five', '{}'::jsonb, 'REVIEW',
        'should', 3,
        'aaaaaaaa-aaaa-7aaa-8aaa-aaaaaaaaaaa1',
        'aaaaaaaa-aaaa-7aaa-8aaa-aaaaaaaaaaa2',
        '2026-02-01T00:00:00Z', 'human',
        'https://example.test/pr/5',
        '2026-01-02T00:00:00Z', '2026-01-02T00:00:00Z');

-- C6: current READY.
INSERT INTO cards (id, board_id, project_id, card_key, seq, title, story, column_name,
                   priority, estimate, assignee_id, due_date, source, created_at, updated_at)
VALUES ('44444444-4444-7444-8444-444444444446',
        '33333333-3333-7333-8333-333333333333',
        '22222222-2222-7222-8222-222222222222',
        'AIL-6', 6, 'Fixture card six', '{}'::jsonb, 'READY',
        'should', 3,
        'aaaaaaaa-aaaa-7aaa-8aaa-aaaaaaaaaaa1',
        '2026-02-01T00:00:00Z', 'human',
        '2026-01-02T00:00:00Z', '2026-01-02T00:00:00Z');

-- Card events.
INSERT INTO card_events (id, card_id, seq, event_type, from_column, to_column, actor_type, actor_id, created_at)
VALUES
  ('70000000-0000-7000-8000-000000000101', '44444444-4444-7444-8444-444444444441', 1, 'created', NULL, 'TODO', 'system', NULL, '2026-01-01T00:00:00Z'),
  ('70000000-0000-7000-8000-000000000102', '44444444-4444-7444-8444-444444444441', 2, 'moved', 'TODO', 'READY', 'agent', 'aaaaaaaa-aaaa-7aaa-8aaa-aaaaaaaaaaa1', '2026-01-01T00:10:00Z'),
  ('70000000-0000-7000-8000-000000000103', '44444444-4444-7444-8444-444444444441', 3, 'moved', 'READY', 'DOING', 'agent', 'aaaaaaaa-aaaa-7aaa-8aaa-aaaaaaaaaaa1', '2026-01-01T00:20:00Z'),
  ('70000000-0000-7000-8000-000000000104', '44444444-4444-7444-8444-444444444441', 4, 'moved', 'DOING', 'REVIEW', 'agent', 'aaaaaaaa-aaaa-7aaa-8aaa-aaaaaaaaaaa1', '2026-01-01T00:30:00Z'),
  ('70000000-0000-7000-8000-000000000105', '44444444-4444-7444-8444-444444444441', 5, 'moved', 'REVIEW', 'DOING', 'agent', 'aaaaaaaa-aaaa-7aaa-8aaa-aaaaaaaaaaa2', '2026-01-01T00:40:00Z'),
  ('70000000-0000-7000-8000-000000000106', '44444444-4444-7444-8444-444444444441', 6, 'commented', NULL, NULL, 'human', NULL, '2026-01-01T00:45:00Z'),
  ('70000000-0000-7000-8000-000000000107', '44444444-4444-7444-8444-444444444441', 7, 'moved', 'DOING', 'REVIEW', 'agent', 'aaaaaaaa-aaaa-7aaa-8aaa-aaaaaaaaaaa1', '2026-01-01T00:50:00Z'),
  ('70000000-0000-7000-8000-000000000108', '44444444-4444-7444-8444-444444444441', 8, 'moved', 'REVIEW', 'DONE', 'agent', 'aaaaaaaa-aaaa-7aaa-8aaa-aaaaaaaaaaa2', '2026-01-01T01:00:00Z'),

  ('70000000-0000-7000-8000-000000000201', '44444444-4444-7444-8444-444444444442', 1, 'created', NULL, 'TODO', 'system', NULL, '2026-01-01T00:00:00Z'),
  ('70000000-0000-7000-8000-000000000202', '44444444-4444-7444-8444-444444444442', 2, 'moved', 'TODO', 'READY', 'agent', 'aaaaaaaa-aaaa-7aaa-8aaa-aaaaaaaaaaa1', '2026-01-01T00:05:00Z'),
  ('70000000-0000-7000-8000-000000000203', '44444444-4444-7444-8444-444444444442', 3, 'moved', 'READY', 'DOING', 'agent', 'aaaaaaaa-aaaa-7aaa-8aaa-aaaaaaaaaaa1', '2026-01-01T00:10:00Z'),
  ('70000000-0000-7000-8000-000000000204', '44444444-4444-7444-8444-444444444442', 4, 'moved', 'DOING', 'REVIEW', 'agent', 'aaaaaaaa-aaaa-7aaa-8aaa-aaaaaaaaaaa1', '2026-01-01T00:20:00Z'),
  ('70000000-0000-7000-8000-000000000205', '44444444-4444-7444-8444-444444444442', 5, 'moved', 'REVIEW', 'DONE', 'agent', 'aaaaaaaa-aaaa-7aaa-8aaa-aaaaaaaaaaa2', '2026-01-01T00:30:00Z'),

  ('70000000-0000-7000-8000-000000000301', '44444444-4444-7444-8444-444444444443', 1, 'created', NULL, 'TODO', 'system', NULL, '2026-01-01T00:00:00Z'),
  ('70000000-0000-7000-8000-000000000302', '44444444-4444-7444-8444-444444444443', 2, 'moved', 'TODO', 'READY', 'agent', 'aaaaaaaa-aaaa-7aaa-8aaa-aaaaaaaaaaa2', '2026-01-01T00:05:00Z'),
  ('70000000-0000-7000-8000-000000000303', '44444444-4444-7444-8444-444444444443', 3, 'blocked', NULL, NULL, 'agent', 'aaaaaaaa-aaaa-7aaa-8aaa-aaaaaaaaaaa2', '2026-01-01T00:08:00Z'),
  ('70000000-0000-7000-8000-000000000304', '44444444-4444-7444-8444-444444444443', 4, 'unblocked', NULL, NULL, 'agent', 'aaaaaaaa-aaaa-7aaa-8aaa-aaaaaaaaaaa2', '2026-01-01T00:18:00Z'),
  ('70000000-0000-7000-8000-000000000305', '44444444-4444-7444-8444-444444444443', 5, 'moved', 'READY', 'DOING', 'agent', 'aaaaaaaa-aaaa-7aaa-8aaa-aaaaaaaaaaa2', '2026-01-01T00:19:00Z'),
  ('70000000-0000-7000-8000-000000000306', '44444444-4444-7444-8444-444444444443', 6, 'moved', 'DOING', 'REVIEW', 'agent', 'aaaaaaaa-aaaa-7aaa-8aaa-aaaaaaaaaaa2', '2026-01-01T00:25:00Z'),
  ('70000000-0000-7000-8000-000000000307', '44444444-4444-7444-8444-444444444443', 7, 'moved', 'REVIEW', 'DONE', 'agent', 'aaaaaaaa-aaaa-7aaa-8aaa-aaaaaaaaaaa1', '2026-01-01T00:40:00Z');

-- Runs and steps.
INSERT INTO runs (id, card_id, project_id, lab_id, agent_id, attempt, status, started_at, ended_at)
VALUES
  ('55555555-5555-7555-8555-555555555551', '44444444-4444-7444-8444-444444444441',
   '22222222-2222-7222-8222-222222222222', '11111111-1111-7111-8111-111111111111',
   'aaaaaaaa-aaaa-7aaa-8aaa-aaaaaaaaaaa1', 1, 'succeeded', '2026-01-01T00:21:00Z', '2026-01-01T00:59:00Z'),
  ('55555555-5555-7555-8555-555555555552', '44444444-4444-7444-8444-444444444442',
   '22222222-2222-7222-8222-222222222222', '11111111-1111-7111-8111-111111111111',
   'aaaaaaaa-aaaa-7aaa-8aaa-aaaaaaaaaaa1', 1, 'succeeded', '2026-01-01T00:11:00Z', '2026-01-01T00:29:00Z'),
  ('55555555-5555-7555-8555-555555555553', '44444444-4444-7444-8444-444444444443',
   '22222222-2222-7222-8222-222222222222', '11111111-1111-7111-8111-111111111111',
   'aaaaaaaa-aaaa-7aaa-8aaa-aaaaaaaaaaa2', 1, 'succeeded', '2026-01-01T00:20:00Z', '2026-01-01T00:39:00Z');

INSERT INTO run_steps (id, run_id, seq, step_type, summary, duration_ms, created_at)
VALUES
  ('66666666-6666-7666-8666-666666666661', '55555555-5555-7555-8555-555555555551', 1, 'command', 'fixture step', 60000, '2026-01-01T00:59:00Z'),
  ('66666666-6666-7666-8666-666666666662', '55555555-5555-7555-8555-555555555552', 1, 'command', 'fixture step', 30000, '2026-01-01T00:29:00Z'),
  ('66666666-6666-7666-8666-666666666663', '55555555-5555-7555-8555-555555555553', 1, 'command', 'fixture step', 40000, '2026-01-01T00:39:00Z');

INSERT INTO cost_ledger (id, lab_id, project_id, card_id, agent_id, run_id, source, amount_microusd, occurred_at)
VALUES
  ('88888888-8888-7888-8888-888888888881', '11111111-1111-7111-8111-111111111111',
   '22222222-2222-7222-8222-222222222222', '44444444-4444-7444-8444-444444444441',
   'aaaaaaaa-aaaa-7aaa-8aaa-aaaaaaaaaaa1', '55555555-5555-7555-8555-555555555551',
   'llm', 1000000, '2026-01-01T00:55:00Z'),
  ('88888888-8888-7888-8888-888888888882', '11111111-1111-7111-8111-111111111111',
   '22222222-2222-7222-8222-222222222222', '44444444-4444-7444-8444-444444444442',
   'aaaaaaaa-aaaa-7aaa-8aaa-aaaaaaaaaaa1', '55555555-5555-7555-8555-555555555552',
   'llm', 2000000, '2026-01-01T00:25:00Z');
