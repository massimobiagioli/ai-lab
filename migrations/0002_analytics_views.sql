-- ============================================================================
-- AI-Lab — Read-only analytics views
-- Owner: WP-00 (definitions), WP-04 (schema lifecycle), WP-20 (dashboards)
-- Source of truth: .docs/sow/WP-00-GOVERNANCE.md §3.7, .docs/product/metrics.md
--
-- These views are read-only projections. They never write and never own data.
-- They are safe to drop and recreate because nothing persists a dependency on
-- them outside this file and the metrics endpoint.
--
-- Forward-only: when a view changes shape, add an ADR and a new migration file
-- (for example migrations/0003_analytics_views_v2.sql) instead of editing history.
-- ============================================================================


CREATE SCHEMA IF NOT EXISTS analytics;

-- ----------------------------------------------------------------------------
-- Base timeline: one row per card, with the first entry into each column.
-- Cards that never emitted an event keep NULL timestamps.
-- ----------------------------------------------------------------------------
CREATE OR REPLACE VIEW analytics.card_timeline AS
SELECT
  c.id            AS card_id,
  c.project_id,
  p.lab_id,
  c.created_at,
  min(e.created_at) FILTER (WHERE e.to_column = 'READY')  AS ready_at,
  min(e.created_at) FILTER (WHERE e.to_column = 'DOING')  AS first_doing_at,
  min(e.created_at) FILTER (WHERE e.to_column = 'REVIEW') AS first_review_at,
  min(e.created_at) FILTER (WHERE e.to_column = 'DONE')   AS done_at
FROM cards c
JOIN projects p ON p.id = c.project_id
LEFT JOIN card_events e ON e.card_id = c.id
GROUP BY c.id, c.project_id, p.lab_id, c.created_at;

COMMENT ON VIEW analytics.card_timeline IS
  'Card creation and first-entry timestamps per column, derived from immutable card_events.';

-- ----------------------------------------------------------------------------
-- Lead time: done_at - created_at for cards that reached DONE.
-- ----------------------------------------------------------------------------
CREATE OR REPLACE VIEW analytics.card_lead_time AS
SELECT
  lab_id,
  project_id,
  card_id,
  created_at,
  done_at,
  EXTRACT(EPOCH FROM (done_at - created_at)) AS lead_time_seconds
FROM analytics.card_timeline
WHERE done_at IS NOT NULL;

COMMENT ON VIEW analytics.card_lead_time IS
  'Lead time per completed card: DONE event minus card creation, in seconds.';

-- ----------------------------------------------------------------------------
-- Cycle time: done_at - ready_at for cards that reached READY and DONE.
-- ----------------------------------------------------------------------------
CREATE OR REPLACE VIEW analytics.card_cycle_time AS
SELECT
  lab_id,
  project_id,
  card_id,
  ready_at,
  done_at,
  EXTRACT(EPOCH FROM (done_at - ready_at)) AS cycle_time_seconds
FROM analytics.card_timeline
WHERE ready_at IS NOT NULL AND done_at IS NOT NULL;

COMMENT ON VIEW analytics.card_cycle_time IS
  'Cycle time per completed card: DONE event minus first READY event, in seconds.';

-- ----------------------------------------------------------------------------
-- Throughput: completed cards per ISO week.
-- ----------------------------------------------------------------------------
CREATE OR REPLACE VIEW analytics.throughput_weekly AS
SELECT
  lab_id,
  project_id,
  date_trunc('week', done_at) AS week_start,
  count(*)                    AS cards_done
FROM analytics.card_lead_time
GROUP BY lab_id, project_id, date_trunc('week', done_at);

COMMENT ON VIEW analytics.throughput_weekly IS
  'Cards that reached DONE per week and project. Week starts on Monday (ISO).';

-- ----------------------------------------------------------------------------
-- Current WIP: cards currently in DOING or REVIEW.
-- ----------------------------------------------------------------------------
CREATE OR REPLACE VIEW analytics.wip_current AS
SELECT
  p.lab_id,
  c.project_id,
  c.column_name,
  c.assignee_id,
  count(*) AS card_count
FROM cards c
JOIN projects p ON p.id = c.project_id
WHERE c.column_name IN ('DOING', 'REVIEW')
GROUP BY p.lab_id, c.project_id, c.column_name, c.assignee_id;

COMMENT ON VIEW analytics.wip_current IS
  'Live WIP gauge: cards in DOING or REVIEW, grouped by project, column and assignee.';

-- ----------------------------------------------------------------------------
-- Blocked time: pair each blocked event with the next unblocked event.
-- Only fully closed intervals contribute to blocked_seconds_closed.
-- ----------------------------------------------------------------------------
CREATE OR REPLACE VIEW analytics.blocked_intervals AS
SELECT
  b.id          AS blocked_event_id,
  b.card_id,
  b.created_at  AS blocked_at,
  u.unblocked_at,
  CASE
    WHEN u.unblocked_at IS NOT NULL
      THEN EXTRACT(EPOCH FROM (u.unblocked_at - b.created_at))
  END AS blocked_seconds
FROM card_events b
LEFT JOIN LATERAL (
  SELECT min(u.created_at) AS unblocked_at
  FROM card_events u
  WHERE u.card_id = b.card_id
    AND u.event_type = 'unblocked'
    AND u.created_at > b.created_at
) u ON true
WHERE b.event_type = 'blocked';

COMMENT ON VIEW analytics.blocked_intervals IS
  'Each blocked event paired with the next unblocked event for the same card.';

CREATE OR REPLACE VIEW analytics.blocked_time AS
SELECT
  p.lab_id,
  i.card_id,
  sum(COALESCE(i.blocked_seconds, 0))                    AS blocked_seconds_closed,
  count(*) FILTER (WHERE i.unblocked_at IS NULL)         AS open_blocked_intervals
FROM analytics.blocked_intervals i
JOIN cards c ON c.id = i.card_id
JOIN projects p ON p.id = c.project_id
GROUP BY p.lab_id, i.card_id;

COMMENT ON VIEW analytics.blocked_time IS
  'Total closed blocked seconds and open blocked intervals per card.';

-- ----------------------------------------------------------------------------
-- Active time and flow efficiency.
-- Active time is the sum of recorded run step durations for the card.
-- Flow efficiency is clamped to [0, 1].
-- ----------------------------------------------------------------------------
CREATE OR REPLACE VIEW analytics.card_active_time AS
SELECT
  r.card_id,
  c.project_id,
  p.lab_id,
  COALESCE(sum(rs.duration_ms), 0) AS active_ms
FROM runs r
JOIN cards c ON c.id = r.card_id
JOIN projects p ON p.id = c.project_id
LEFT JOIN run_steps rs ON rs.run_id = r.id
GROUP BY r.card_id, c.project_id, p.lab_id;

COMMENT ON VIEW analytics.card_active_time IS
  'Sum of run step durations per card, across all attempts.';

CREATE OR REPLACE VIEW analytics.flow_efficiency AS
SELECT
  t.lab_id,
  t.project_id,
  t.card_id,
  t.lead_time_seconds,
  COALESCE(a.active_ms, 0) / 1000.0 AS active_seconds,
  CASE
    WHEN t.lead_time_seconds > 0
      THEN LEAST(1.0, (COALESCE(a.active_ms, 0) / 1000.0) / t.lead_time_seconds)
    ELSE 0
  END AS flow_efficiency
FROM analytics.card_lead_time t
LEFT JOIN analytics.card_active_time a ON a.card_id = t.card_id;

COMMENT ON VIEW analytics.flow_efficiency IS
  'Active run time divided by lead time per completed card, clamped to [0, 1].';

-- ----------------------------------------------------------------------------
-- Rework: cards with at least one REVIEW -> DOING transition.
-- Denominator: cards that reached DONE.
-- ----------------------------------------------------------------------------
CREATE OR REPLACE VIEW analytics.rework_rate AS
WITH done_cards AS (
  SELECT lab_id, project_id, card_id FROM analytics.card_lead_time
),
rework_cards AS (
  SELECT DISTINCT card_id
  FROM card_events
  WHERE from_column = 'REVIEW' AND to_column = 'DOING'
)
SELECT
  d.lab_id,
  d.project_id,
  count(*) FILTER (WHERE r.card_id IS NOT NULL) AS rework_cards,
  count(*)                                      AS done_cards,
  CASE
    WHEN count(*) > 0
      THEN count(*) FILTER (WHERE r.card_id IS NOT NULL)::numeric / count(*)
    ELSE 0
  END AS rework_rate
FROM done_cards d
LEFT JOIN rework_cards r ON r.card_id = d.card_id
GROUP BY d.lab_id, d.project_id;

COMMENT ON VIEW analytics.rework_rate IS
  'Share of completed cards that returned from REVIEW to DOING at least once.';

-- ----------------------------------------------------------------------------
-- Cost per card: canonical source is cost_ledger.
-- ----------------------------------------------------------------------------
CREATE OR REPLACE VIEW analytics.card_cost AS
SELECT
  c.id AS card_id,
  c.project_id,
  p.lab_id,
  COALESCE(sum(cl.amount_microusd), 0) AS cost_microusd
FROM cards c
JOIN projects p ON p.id = c.project_id
LEFT JOIN cost_ledger cl ON cl.card_id = c.id
GROUP BY c.id, c.project_id, p.lab_id;

COMMENT ON VIEW analytics.card_cost IS
  'Total recorded cost in micro-USD per card, from cost_ledger.';

CREATE OR REPLACE VIEW analytics.cost_per_card AS
SELECT
  t.lab_id,
  t.project_id,
  count(*)                                              AS done_cards,
  sum(cc.cost_microusd)                                 AS cost_microusd,
  sum(cc.cost_microusd)::numeric / count(*)             AS avg_cost_microusd,
  percentile_cont(0.5) WITHIN GROUP (ORDER BY cc.cost_microusd) AS median_cost_microusd
FROM analytics.card_lead_time t
JOIN analytics.card_cost cc ON cc.card_id = t.card_id
GROUP BY t.lab_id, t.project_id;

COMMENT ON VIEW analytics.cost_per_card IS
  'Completed cards, total, average and median cost in micro-USD per project.';

-- ----------------------------------------------------------------------------
-- Autonomous success: completed cards executed by an A2 agent with no human
-- event between READY and DONE.
-- Limitation: agents.autonomy_level is the current value, not a run snapshot.
-- ----------------------------------------------------------------------------
CREATE OR REPLACE VIEW analytics.autonomous_success AS
WITH done_cards AS (
  SELECT
    t.lab_id,
    t.project_id,
    t.card_id,
    t.ready_at,
    t.done_at,
    c.assignee_id
  FROM analytics.card_timeline t
  JOIN cards c ON c.id = t.card_id
  WHERE t.done_at IS NOT NULL
),
human_intervention AS (
  SELECT DISTINCT d.card_id
  FROM done_cards d
  JOIN card_events e ON e.card_id = d.card_id
  WHERE e.actor_type = 'human'
    AND d.ready_at IS NOT NULL
    AND e.created_at > d.ready_at
    AND e.created_at <= d.done_at
    AND e.event_type IN ('moved', 'updated', 'assigned', 'commented', 'blocked', 'unblocked')
)
SELECT
  d.lab_id,
  d.project_id,
  count(*) AS done_cards,
  count(*) FILTER (
    WHERE COALESCE(a.autonomy_level, -1) = 2 AND i.card_id IS NULL
  ) AS autonomous_done_cards,
  CASE
    WHEN count(*) > 0
      THEN count(*) FILTER (
        WHERE COALESCE(a.autonomy_level, -1) = 2 AND i.card_id IS NULL
      )::numeric / count(*)
    ELSE 0
  END AS autonomous_success_rate
FROM done_cards d
LEFT JOIN agents a ON a.id = d.assignee_id
LEFT JOIN human_intervention i ON i.card_id = d.card_id
GROUP BY d.lab_id, d.project_id;

COMMENT ON VIEW analytics.autonomous_success IS
  'Share of completed cards run at autonomy A2 with no human event between READY and DONE.';

-- ----------------------------------------------------------------------------
-- Flow summary: lead and cycle time percentiles per project.
-- ----------------------------------------------------------------------------
CREATE OR REPLACE VIEW analytics.flow_summary AS
WITH lead AS (
  SELECT
    lab_id,
    project_id,
    count(*) AS done_cards,
    percentile_cont(0.5)  WITHIN GROUP (ORDER BY lead_time_seconds) AS p50_lead_time_seconds,
    percentile_cont(0.95) WITHIN GROUP (ORDER BY lead_time_seconds) AS p95_lead_time_seconds
  FROM analytics.card_lead_time
  GROUP BY lab_id, project_id
),
cycle AS (
  SELECT
    lab_id,
    project_id,
    percentile_cont(0.5)  WITHIN GROUP (ORDER BY cycle_time_seconds) AS p50_cycle_time_seconds,
    percentile_cont(0.95) WITHIN GROUP (ORDER BY cycle_time_seconds) AS p95_cycle_time_seconds
  FROM analytics.card_cycle_time
  GROUP BY lab_id, project_id
)
SELECT
  l.lab_id,
  l.project_id,
  l.done_cards,
  l.p50_lead_time_seconds,
  l.p95_lead_time_seconds,
  c.p50_cycle_time_seconds,
  c.p95_cycle_time_seconds
FROM lead l
LEFT JOIN cycle c ON c.lab_id = l.lab_id AND c.project_id = l.project_id;

COMMENT ON VIEW analytics.flow_summary IS
  'Lead and cycle time percentiles per project.';

