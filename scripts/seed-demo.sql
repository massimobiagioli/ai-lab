-- Deterministic demo dataset for AI-Lab.
-- Idempotent: fixed UUIDs and ON CONFLICT DO NOTHING.
-- Run with: scripts/seed-demo.sh

BEGIN;

INSERT INTO labs (id, workspace_id, name, slug, description, autonomy_level, monthly_budget_microusd)
VALUES (
  'd0000000-0000-7000-8000-000000000001',
  '00000000-0000-7000-8000-000000000001',
  'Demo Lab', 'demo-lab',
  'Deterministic demo lab used by the golden path and screenshots.',
  2, 50000000
) ON CONFLICT (id) DO NOTHING;

INSERT INTO agents (id, lab_id, name, role, character_id, description, autonomy_level, max_concurrent_cards)
VALUES
  ('d1000000-0000-7000-8000-000000000001', 'd0000000-0000-7000-8000-000000000001',
   'Ada Lead', 'tech_lead', 'veteran_lead',
   'Owns delivery order, architecture, decomposition and merge decisions. Refines ideas into canonical cards, enforces DoR and DoD, and escalates budget or autonomy changes to the Human Operator.',
   2, 1),
  ('d1000000-0000-7000-8000-000000000002', 'd0000000-0000-7000-8000-000000000001',
   'Mira Analyst', 'analyst', 'systems_analyst',
   'Turns ideas into structured SOWs, plans and acceptance criteria. Maintains the documentation manifest, checks traceability, and keeps the glossary and diagrams current for every delivery.',
   2, 1),
  ('d1000000-0000-7000-8000-000000000003', 'd0000000-0000-7000-8000-000000000001',
   'Nico Dev', 'dev', 'scrappy_dev',
   'Implements cards with test-driven development, keeps the workspace green and opens focused pull requests. Owns the build, the tests and the operational runbooks for the code it ships.',
   2, 1),
  ('d1000000-0000-7000-8000-000000000004', 'd0000000-0000-7000-8000-000000000001',
   'Vera QA', 'qa', 'precise_consultant',
   'Verifies every card independently, runs the quality gates and records verdicts. Guards separation of duties, security checks and release readiness with precise, evidence-based review notes.',
   2, 1)
ON CONFLICT (id) DO NOTHING;

INSERT INTO projects (id, lab_id, name, slug, key_prefix, description, source_kind, source_ref, default_branch)
VALUES (
  'd2000000-0000-7000-8000-000000000001',
  'd0000000-0000-7000-8000-000000000001',
  'Demo Project', 'demo-project', 'AIL',
  'Deterministic project for the golden path.',
  'github', 'example/ai-lab-demo', 'main'
) ON CONFLICT (id) DO NOTHING;

INSERT INTO boards (id, project_id)
VALUES ('d3000000-0000-7000-8000-000000000001', 'd2000000-0000-7000-8000-000000000001')
ON CONFLICT (project_id) DO NOTHING;

INSERT INTO sows (id, project_id, lab_id, title, status, current_version, approved_at, approved_by, created_by_agent_id)
VALUES (
  'd5000000-0000-7000-8000-000000000001',
  'd2000000-0000-7000-8000-000000000001',
  'd0000000-0000-7000-8000-000000000001',
  'Demo SOW', 'approved', 1, now() - interval '2 days', 'demo', 'd1000000-0000-7000-8000-000000000002'
) ON CONFLICT (id) DO NOTHING;

INSERT INTO sow_versions (id, sow_id, version, document, markdown, content_hash, change_note)
VALUES (
  'd5100000-0000-7000-8000-000000000001',
  'd5000000-0000-7000-8000-000000000001', 1,
  '{"executive_summary":"Demo SOW","objectives":["Ship the demo"],"scope_in":["Kanban"],"scope_out":["Multi-tenant"]}'::jsonb,
  '# Demo SOW', 'demo-hash-1', 'Initial demo version'
) ON CONFLICT (id) DO NOTHING;

INSERT INTO plans (id, project_id, sow_id, title, status, current_version, created_by_agent_id)
VALUES (
  'd6000000-0000-7000-8000-000000000001',
  'd2000000-0000-7000-8000-000000000001',
  'd5000000-0000-7000-8000-000000000001',
  'Demo Plan', 'materialized', 1, 'd1000000-0000-7000-8000-000000000002'
) ON CONFLICT (id) DO NOTHING;

INSERT INTO plan_versions (id, plan_id, version, graph, markdown)
VALUES (
  'd6100000-0000-7000-8000-000000000001',
  'd6000000-0000-7000-8000-000000000001', 1,
  '{"nodes":[{"id":"T-01"},{"id":"T-02"}],"edges":[{"from":"T-01","to":"T-02"}]}'::jsonb,
  '# Demo Plan'
) ON CONFLICT (id) DO NOTHING;

INSERT INTO plan_tasks (id, plan_id, version, task_key, title, role, estimate, depends_on)
VALUES
  ('d6200000-0000-7000-8000-000000000001', 'd6000000-0000-7000-8000-000000000001', 1, 'T-01', 'Build the board shell', 'dev', 3, '{}'),
  ('d6200000-0000-7000-8000-000000000002', 'd6000000-0000-7000-8000-000000000001', 1, 'T-02', 'Verify board transitions', 'qa', 2, '{T-01}')
ON CONFLICT (id) DO NOTHING;

INSERT INTO cards (id, board_id, project_id, card_key, seq, title, story, column_name, priority, estimate, assignee_id, reviewer_id, due_date, source, merge_sha, pr_url, budget_microusd, created_at, updated_at)
VALUES
  ('d4000000-0000-7000-8000-000000000001', 'd3000000-0000-7000-8000-000000000001', 'd2000000-0000-7000-8000-000000000001', 'AIL-1', 1, 'Draft the idea intake copy', '{}'::jsonb, 'TODO', 'should', NULL, NULL, NULL, NULL, 'human', NULL, NULL, 5000000, now() - interval '10 hours', now() - interval '10 hours'),
  ('d4000000-0000-7000-8000-000000000002', 'd3000000-0000-7000-8000-000000000001', 'd2000000-0000-7000-8000-000000000001', 'AIL-2', 2, 'Sketch the board empty state', '{}'::jsonb, 'TODO', 'could', NULL, NULL, NULL, NULL, 'human', NULL, NULL, 5000000, now() - interval '9 hours', now() - interval '9 hours'),
  ('d4000000-0000-7000-8000-000000000003', 'd3000000-0000-7000-8000-000000000001', 'd2000000-0000-7000-8000-000000000001', 'AIL-3', 3, 'Collect provider requirements', '{}'::jsonb, 'TODO', 'should', NULL, NULL, NULL, NULL, 'agent', NULL, NULL, 5000000, now() - interval '8 hours', now() - interval '8 hours'),
  ('d4000000-0000-7000-8000-000000000004', 'd3000000-0000-7000-8000-000000000001', 'd2000000-0000-7000-8000-000000000001', 'AIL-4', 4, 'Add the readiness checklist', '{}'::jsonb, 'READY', 'must', 3, 'd1000000-0000-7000-8000-000000000003', NULL, now() + interval '2 days', 'human', NULL, NULL, 5000000, now() - interval '8 hours', now() - interval '7 hours'),
  ('d4000000-0000-7000-8000-000000000005', 'd3000000-0000-7000-8000-000000000001', 'd2000000-0000-7000-8000-000000000001', 'AIL-5', 5, 'Define transition error copy', '{}'::jsonb, 'READY', 'should', 2, 'd1000000-0000-7000-8000-000000000002', NULL, now() + interval '3 days', 'agent', NULL, NULL, 5000000, now() - interval '7 hours', now() - interval '6 hours'),
  ('d4000000-0000-7000-8000-000000000006', 'd3000000-0000-7000-8000-000000000001', 'd2000000-0000-7000-8000-000000000001', 'AIL-6', 6, 'Render the six fixed columns', '{}'::jsonb, 'DOING', 'must', 5, 'd1000000-0000-7000-8000-000000000003', NULL, now() + interval '1 day', 'agent', NULL, NULL, 5000000, now() - interval '6 hours', now() - interval '5 hours'),
  ('d4000000-0000-7000-8000-000000000007', 'd3000000-0000-7000-8000-000000000001', 'd2000000-0000-7000-8000-000000000001', 'AIL-7', 7, 'Wire live card movement', '{}'::jsonb, 'DOING', 'must', 5, 'd1000000-0000-7000-8000-000000000003', NULL, now() + interval '2 days', 'agent', NULL, NULL, 5000000, now() - interval '5 hours', now() - interval '4 hours'),
  ('d4000000-0000-7000-8000-000000000008', 'd3000000-0000-7000-8000-000000000001', 'd2000000-0000-7000-8000-000000000001', 'AIL-8', 8, 'Add the card inspector', '{}'::jsonb, 'REVIEW', 'should', 3, 'd1000000-0000-7000-8000-000000000003', 'd1000000-0000-7000-8000-000000000004', now() + interval '1 day', 'agent', NULL, 'https://example.test/pr/8', 5000000, now() - interval '4 hours', now() - interval '3 hours'),
  ('d4000000-0000-7000-8000-000000000009', 'd3000000-0000-7000-8000-000000000001', 'd2000000-0000-7000-8000-000000000001', 'AIL-9', 9, 'Show overdue card badges', '{}'::jsonb, 'REVIEW', 'should', 2, 'd1000000-0000-7000-8000-000000000002', 'd1000000-0000-7000-8000-000000000004', now() + interval '1 day', 'agent', NULL, 'https://example.test/pr/9', 5000000, now() - interval '3 hours', now() - interval '2 hours'),
  ('d4000000-0000-7000-8000-000000000010', 'd3000000-0000-7000-8000-000000000001', 'd2000000-0000-7000-8000-000000000001', 'AIL-10', 10, 'Enforce DoR on move to ready', '{}'::jsonb, 'DONE', 'must', 5, 'd1000000-0000-7000-8000-000000000003', 'd1000000-0000-7000-8000-000000000004', now() - interval '1 day', 'agent', 'demo-sha-10', 'https://example.test/pr/10', 5000000, now() - interval '48 hours', now() - interval '24 hours'),
  ('d4000000-0000-7000-8000-000000000011', 'd3000000-0000-7000-8000-000000000001', 'd2000000-0000-7000-8000-000000000001', 'AIL-11', 11, 'Enforce DoD on move to done', '{}'::jsonb, 'DONE', 'must', 5, 'd1000000-0000-7000-8000-000000000003', 'd1000000-0000-7000-8000-000000000004', now() - interval '2 days', 'agent', 'demo-sha-11', 'https://example.test/pr/11', 5000000, now() - interval '72 hours', now() - interval '36 hours'),
  ('d4000000-0000-7000-8000-000000000012', 'd3000000-0000-7000-8000-000000000001', 'd2000000-0000-7000-8000-000000000001', 'AIL-12', 12, 'Record audit events for moves', '{}'::jsonb, 'DONE', 'must', 3, 'd1000000-0000-7000-8000-000000000002', 'd1000000-0000-7000-8000-000000000004', now() - interval '3 days', 'agent', 'demo-sha-12', 'https://example.test/pr/12', 5000000, now() - interval '96 hours', now() - interval '48 hours')
ON CONFLICT (id) DO NOTHING;

UPDATE projects SET card_seq = GREATEST(card_seq, 12)
WHERE id = 'd2000000-0000-7000-8000-000000000001';

-- Card events: created plus each transition. Idempotent by (card_id, seq).
WITH events(card_id, seq, event_type, from_column, to_column, actor_type, actor_id, minutes_ago) AS (
  VALUES
    ('d4000000-0000-7000-8000-000000000001', 1, 'created', NULL, 'TODO', 'system', NULL, 600),
    ('d4000000-0000-7000-8000-000000000002', 1, 'created', NULL, 'TODO', 'system', NULL, 540),
    ('d4000000-0000-7000-8000-000000000003', 1, 'created', NULL, 'TODO', 'system', NULL, 480),
    ('d4000000-0000-7000-8000-000000000004', 1, 'created', NULL, 'TODO', 'system', NULL, 480),
    ('d4000000-0000-7000-8000-000000000004', 2, 'moved', 'TODO', 'READY', 'agent', 'd1000000-0000-7000-8000-000000000001', 420),
    ('d4000000-0000-7000-8000-000000000005', 1, 'created', NULL, 'TODO', 'system', NULL, 420),
    ('d4000000-0000-7000-8000-000000000005', 2, 'moved', 'TODO', 'READY', 'agent', 'd1000000-0000-7000-8000-000000000001', 360),
    ('d4000000-0000-7000-8000-000000000006', 1, 'created', NULL, 'TODO', 'system', NULL, 360),
    ('d4000000-0000-7000-8000-000000000006', 2, 'moved', 'TODO', 'READY', 'agent', 'd1000000-0000-7000-8000-000000000001', 300),
    ('d4000000-0000-7000-8000-000000000006', 3, 'moved', 'READY', 'DOING', 'agent', 'd1000000-0000-7000-8000-000000000003', 240),
    ('d4000000-0000-7000-8000-000000000007', 1, 'created', NULL, 'TODO', 'system', NULL, 300),
    ('d4000000-0000-7000-8000-000000000007', 2, 'moved', 'TODO', 'READY', 'agent', 'd1000000-0000-7000-8000-000000000001', 240),
    ('d4000000-0000-7000-8000-000000000007', 3, 'moved', 'READY', 'DOING', 'agent', 'd1000000-0000-7000-8000-000000000003', 180),
    ('d4000000-0000-7000-8000-000000000008', 1, 'created', NULL, 'TODO', 'system', NULL, 240),
    ('d4000000-0000-7000-8000-000000000008', 2, 'moved', 'TODO', 'READY', 'agent', 'd1000000-0000-7000-8000-000000000001', 180),
    ('d4000000-0000-7000-8000-000000000008', 3, 'moved', 'READY', 'DOING', 'agent', 'd1000000-0000-7000-8000-000000000003', 120),
    ('d4000000-0000-7000-8000-000000000008', 4, 'moved', 'DOING', 'REVIEW', 'agent', 'd1000000-0000-7000-8000-000000000003', 60),
    ('d4000000-0000-7000-8000-000000000009', 1, 'created', NULL, 'TODO', 'system', NULL, 180),
    ('d4000000-0000-7000-8000-000000000009', 2, 'moved', 'TODO', 'READY', 'agent', 'd1000000-0000-7000-8000-000000000001', 150),
    ('d4000000-0000-7000-8000-000000000009', 3, 'moved', 'READY', 'DOING', 'agent', 'd1000000-0000-7000-8000-000000000002', 120),
    ('d4000000-0000-7000-8000-000000000009', 4, 'moved', 'DOING', 'REVIEW', 'agent', 'd1000000-0000-7000-8000-000000000002', 60),
    ('d4000000-0000-7000-8000-000000000010', 1, 'created', NULL, 'TODO', 'system', NULL, 2880),
    ('d4000000-0000-7000-8000-000000000010', 2, 'moved', 'TODO', 'READY', 'agent', 'd1000000-0000-7000-8000-000000000001', 2700),
    ('d4000000-0000-7000-8000-000000000010', 3, 'moved', 'READY', 'DOING', 'agent', 'd1000000-0000-7000-8000-000000000003', 2400),
    ('d4000000-0000-7000-8000-000000000010', 4, 'moved', 'DOING', 'REVIEW', 'agent', 'd1000000-0000-7000-8000-000000000003', 2000),
    ('d4000000-0000-7000-8000-000000000010', 5, 'moved', 'REVIEW', 'DONE', 'agent', 'd1000000-0000-7000-8000-000000000004', 1440),
    ('d4000000-0000-7000-8000-000000000011', 1, 'created', NULL, 'TODO', 'system', NULL, 4320),
    ('d4000000-0000-7000-8000-000000000011', 2, 'moved', 'TODO', 'READY', 'agent', 'd1000000-0000-7000-8000-000000000001', 4200),
    ('d4000000-0000-7000-8000-000000000011', 3, 'moved', 'READY', 'DOING', 'agent', 'd1000000-0000-7000-8000-000000000003', 3600),
    ('d4000000-0000-7000-8000-000000000011', 4, 'moved', 'DOING', 'REVIEW', 'agent', 'd1000000-0000-7000-8000-000000000003', 3000),
    ('d4000000-0000-7000-8000-000000000011', 5, 'moved', 'REVIEW', 'DONE', 'agent', 'd1000000-0000-7000-8000-000000000004', 2160),
    ('d4000000-0000-7000-8000-000000000012', 1, 'created', NULL, 'TODO', 'system', NULL, 5760),
    ('d4000000-0000-7000-8000-000000000012', 2, 'moved', 'TODO', 'READY', 'agent', 'd1000000-0000-7000-8000-000000000001', 5400),
    ('d4000000-0000-7000-8000-000000000012', 3, 'moved', 'READY', 'DOING', 'agent', 'd1000000-0000-7000-8000-000000000002', 4800),
    ('d4000000-0000-7000-8000-000000000012', 4, 'moved', 'DOING', 'REVIEW', 'agent', 'd1000000-0000-7000-8000-000000000002', 4000),
    ('d4000000-0000-7000-8000-000000000012', 5, 'moved', 'REVIEW', 'DONE', 'agent', 'd1000000-0000-7000-8000-000000000004', 2880)
)
INSERT INTO card_events (id, card_id, seq, event_type, from_column, to_column, actor_type, actor_id, created_at)
SELECT gen_random_uuid(), card_id::uuid, seq, event_type, from_column, to_column, actor_type, actor_id::uuid,
       now() - (minutes_ago || ' minutes')::interval
FROM events
ON CONFLICT (card_id, seq) DO NOTHING;

COMMIT;
