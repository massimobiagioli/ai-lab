-- ============================================================================
-- AI-Lab — Appendix A — Database Schema (PostgreSQL 18 + pgvector 0.8.7)
-- Document: AILAB-SPEC-001 / APPENDIX-A
-- Status: NORMATIVE
-- Notes:
--   * Forward-only migrations; this file is the consolidated v1.0 schema.
--   * All timestamps are TIMESTAMPTZ (UTC). All PKs are UUIDv7 app-generated,
--     except seeded catalog rows.
--   * Money is BIGINT micro-USD (1 USD = 1_000_000).
--   * Redis is never authoritative; everything durable lives here.
-- ============================================================================


CREATE EXTENSION IF NOT EXISTS vector;
CREATE EXTENSION IF NOT EXISTS pgcrypto;
CREATE EXTENSION IF NOT EXISTS pg_trgm;
CREATE EXTENSION IF NOT EXISTS citext;

-- ----------------------------------------------------------------------------
-- 0. Utility: updated_at trigger
-- ----------------------------------------------------------------------------
CREATE OR REPLACE FUNCTION set_updated_at() RETURNS trigger AS $$
BEGIN
  NEW.updated_at := now();
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- ----------------------------------------------------------------------------
-- 1. Tenancy root (single-tenant in v1, forward-compatible)
-- ----------------------------------------------------------------------------
CREATE TABLE workspaces (
  id            UUID PRIMARY KEY,
  name          TEXT NOT NULL,
  slug          CITEXT NOT NULL UNIQUE,
  settings      JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TRIGGER trg_workspaces_updated BEFORE UPDATE ON workspaces
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();

INSERT INTO workspaces (id, name, slug)
VALUES ('00000000-0000-7000-8000-000000000001', 'Default Workspace', 'default');

-- ----------------------------------------------------------------------------
-- 2. Labs
-- ----------------------------------------------------------------------------
CREATE TABLE labs (
  id                    UUID PRIMARY KEY,
  workspace_id          UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
  name                  TEXT NOT NULL,
  slug                  CITEXT NOT NULL,
  description           TEXT NOT NULL DEFAULT '',
  autonomy_level        SMALLINT NOT NULL DEFAULT 2 CHECK (autonomy_level BETWEEN 0 AND 3),
  monthly_budget_microusd BIGINT NOT NULL DEFAULT 50000000 CHECK (monthly_budget_microusd >= 0),
  settings              JSONB NOT NULL DEFAULT '{}'::jsonb,
  archived_at           TIMESTAMPTZ,
  created_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (workspace_id, slug)
);
CREATE TRIGGER trg_labs_updated BEFORE UPDATE ON labs
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();
CREATE INDEX idx_labs_workspace ON labs(workspace_id) WHERE archived_at IS NULL;

-- ----------------------------------------------------------------------------
-- 3. Providers, models, encrypted credentials
-- ----------------------------------------------------------------------------
CREATE TABLE providers (
  id                  UUID PRIMARY KEY,
  workspace_id        UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
  name                TEXT NOT NULL,
  kind                TEXT NOT NULL CHECK (kind IN ('deepseek')),
  base_url            TEXT NOT NULL,
  api_key_ciphertext  BYTEA NOT NULL,
  api_key_nonce       BYTEA NOT NULL,
  api_key_last4       TEXT NOT NULL CHECK (char_length(api_key_last4) <= 4),
  key_version         INTEGER NOT NULL DEFAULT 1,
  status              TEXT NOT NULL DEFAULT 'active'
                        CHECK (status IN ('active','disabled','degraded')),
  rate_limit_rpm      INTEGER NOT NULL DEFAULT 600 CHECK (rate_limit_rpm > 0),
  rate_limit_tpm      INTEGER NOT NULL DEFAULT 120000 CHECK (rate_limit_tpm > 0),
  timeout_ms          INTEGER NOT NULL DEFAULT 120000 CHECK (timeout_ms BETWEEN 1000 AND 600000),
  last_health_at      TIMESTAMPTZ,
  last_health_detail  JSONB,
  created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (workspace_id, name)
);
CREATE TRIGGER trg_providers_updated BEFORE UPDATE ON providers
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE models (
  id                        UUID PRIMARY KEY,
  provider_id               UUID NOT NULL REFERENCES providers(id) ON DELETE CASCADE,
  name                      TEXT NOT NULL,                -- API identifier
  display_name              TEXT NOT NULL,
  context_window            INTEGER NOT NULL,
  max_output_tokens         INTEGER NOT NULL,
  supports_tools            BOOLEAN NOT NULL DEFAULT true,
  supports_reasoning        BOOLEAN NOT NULL DEFAULT true,
  supports_vision           BOOLEAN NOT NULL DEFAULT false,
  input_cost_microusd_per_mtok  BIGINT NOT NULL DEFAULT 0,
  output_cost_microusd_per_mtok BIGINT NOT NULL DEFAULT 0,
  default_temperature       NUMERIC(3,2) NOT NULL DEFAULT 0.20,
  default_reasoning_effort  TEXT NOT NULL DEFAULT 'medium'
                              CHECK (default_reasoning_effort IN ('low','medium','high')),
  is_default                BOOLEAN NOT NULL DEFAULT false,
  is_escalation             BOOLEAN NOT NULL DEFAULT false,
  status                    TEXT NOT NULL DEFAULT 'active'
                              CHECK (status IN ('active','disabled')),
  created_at                TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at                TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (provider_id, name)
);
CREATE TRIGGER trg_models_updated BEFORE UPDATE ON models
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();
CREATE UNIQUE INDEX idx_models_one_default
  ON models(provider_id) WHERE is_default AND status = 'active';

CREATE TABLE scm_credentials (
  id                  UUID PRIMARY KEY,
  workspace_id        UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
  name                TEXT NOT NULL,
  kind                TEXT NOT NULL CHECK (kind IN ('github_pat','github_app')),
  username            TEXT,
  token_ciphertext    BYTEA NOT NULL,
  token_nonce         BYTEA NOT NULL,
  token_last4         TEXT NOT NULL,
  key_version         INTEGER NOT NULL DEFAULT 1,
  scopes              TEXT[] NOT NULL DEFAULT '{}',
  status              TEXT NOT NULL DEFAULT 'active'
                        CHECK (status IN ('active','disabled','expired')),
  last_verified_at    TIMESTAMPTZ,
  created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (workspace_id, name)
);
CREATE TRIGGER trg_scm_credentials_updated BEFORE UPDATE ON scm_credentials
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- ----------------------------------------------------------------------------
-- 4. Character roster (seeded catalog; see Appendix C.1)
-- ----------------------------------------------------------------------------
CREATE TABLE characters (
  id              TEXT PRIMARY KEY,          -- stable slug, e.g. 'veteran_lead'
  display_name    TEXT NOT NULL,
  archetype       TEXT NOT NULL,
  sprite_key      TEXT NOT NULL,
  palette         JSONB NOT NULL DEFAULT '{}'::jsonb,
  speech_style    TEXT NOT NULL,
  is_active       BOOLEAN NOT NULL DEFAULT true
);

-- ----------------------------------------------------------------------------
-- 5. Agents and CV
-- ----------------------------------------------------------------------------
CREATE TABLE agents (
  id                    UUID PRIMARY KEY,
  lab_id                UUID NOT NULL REFERENCES labs(id) ON DELETE CASCADE,
  name                  TEXT NOT NULL,
  role                  TEXT NOT NULL
                          CHECK (role IN ('tech_lead','analyst','dev','qa')),
  character_id          TEXT NOT NULL REFERENCES characters(id),
  description           TEXT NOT NULL CHECK (char_length(description) BETWEEN 120 AND 600),
  model_preference_id   UUID REFERENCES models(id),
  autonomy_level        SMALLINT CHECK (autonomy_level BETWEEN 0 AND 3),
  status                TEXT NOT NULL DEFAULT 'idle'
                          CHECK (status IN ('idle','working','blocked','paused','offline')),
  max_concurrent_cards  SMALLINT NOT NULL DEFAULT 1 CHECK (max_concurrent_cards BETWEEN 1 AND 4),
  cv_version            INTEGER NOT NULL DEFAULT 1,
  avatar_seed           BIGINT NOT NULL DEFAULT 0,
  retired_at            TIMESTAMPTZ,
  created_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (lab_id, name)
);
CREATE TRIGGER trg_agents_updated BEFORE UPDATE ON agents
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();
CREATE INDEX idx_agents_lab ON agents(lab_id) WHERE retired_at IS NULL;
CREATE UNIQUE INDEX idx_agents_one_tech_lead
  ON agents(lab_id) WHERE role = 'tech_lead' AND retired_at IS NULL;

CREATE TABLE agent_cv_versions (
  id              UUID PRIMARY KEY,
  agent_id        UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
  version         INTEGER NOT NULL,
  cv              JSONB NOT NULL,
  summary         TEXT NOT NULL,
  created_by      TEXT NOT NULL CHECK (created_by IN ('reflection','human','seed')),
  run_id          UUID,
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (agent_id, version)
);
CREATE INDEX idx_cv_agent_version ON agent_cv_versions(agent_id, version DESC);

-- CV JSON Schema (validated in application; documented here for clarity):
-- {
--   "skills": [{"name":"rust","category":"language","proficiency":1..5,
--               "evidence":[{"card_id":"uuid","run_id":"uuid","summary":"...","at":"iso"}],
--               "last_used_at":"iso"}],
--   "delivered_cards": 0,
--   "review_stats": {"approved":0,"changes_requested":0,"blocked":0},
--   "failure_modes": ["..."],
--   "preferred_tools": ["..."],
--   "languages": ["..."],
--   "frameworks": ["..."],
--   "notes": "..."
-- }

-- ----------------------------------------------------------------------------
-- 6. Projects
-- ----------------------------------------------------------------------------
CREATE TABLE projects (
  id                    UUID PRIMARY KEY,
  lab_id                UUID NOT NULL REFERENCES labs(id) ON DELETE CASCADE,
  name                  TEXT NOT NULL,
  slug                  CITEXT NOT NULL,
  key_prefix            TEXT NOT NULL CHECK (key_prefix ~ '^[A-Z][A-Z0-9]{1,9}$'),
  description           TEXT NOT NULL DEFAULT '',
  source_kind           TEXT NOT NULL DEFAULT 'github' CHECK (source_kind IN ('github')),
  source_ref            TEXT NOT NULL,                 -- 'owner/repo'
  default_branch        TEXT NOT NULL DEFAULT 'main',
  credential_id         UUID REFERENCES scm_credentials(id) ON DELETE SET NULL,
  status                TEXT NOT NULL DEFAULT 'active'
                          CHECK (status IN ('active','archived')),
  card_seq              BIGINT NOT NULL DEFAULT 0,
  settings              JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (lab_id, slug),
  UNIQUE (lab_id, key_prefix),
  UNIQUE (lab_id, source_ref)
);
CREATE TRIGGER trg_projects_updated BEFORE UPDATE ON projects
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();
CREATE INDEX idx_projects_lab ON projects(lab_id) WHERE status = 'active';

-- ----------------------------------------------------------------------------
-- 7. Kanban
-- ----------------------------------------------------------------------------
CREATE TABLE boards (
  id          UUID PRIMARY KEY,
  project_id  UUID NOT NULL UNIQUE REFERENCES projects(id) ON DELETE CASCADE,
  created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TRIGGER trg_boards_updated BEFORE UPDATE ON boards
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE cards (
  id                    UUID PRIMARY KEY,
  board_id              UUID NOT NULL REFERENCES boards(id) ON DELETE CASCADE,
  project_id            UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  card_key              TEXT NOT NULL,
  seq                   BIGINT NOT NULL,
  title                 TEXT NOT NULL CHECK (char_length(title) BETWEEN 1 AND 80),
  story                 JSONB NOT NULL DEFAULT '{}'::jsonb,
  -- story shape:
  -- {"persona":"...","capability":"...","outcome":"...","context":"...",
  --  "acceptance_criteria":[{"id":"AC-1","given":"...","when":"...","then":["..."]}],
  --  "deliverables":[{"path":"...","definition_of_done":"..."}],
  --  "out_of_scope":["..."],"risks":[{"risk":"...","mitigation":"..."}],
  --  "needs_refinement":true}
  column_name           TEXT NOT NULL DEFAULT 'TODO'
                          CHECK (column_name IN ('TODO','READY','DOING','REVIEW','DONE','ICEBOX')),
  priority              TEXT NOT NULL DEFAULT 'should'
                          CHECK (priority IN ('must','should','could','wont')),
  estimate              SMALLINT CHECK (estimate IN (1,2,3,5,8,13)),
  labels                TEXT[] NOT NULL DEFAULT '{}',
  assignee_id           UUID REFERENCES agents(id) ON DELETE SET NULL,
  reviewer_id           UUID REFERENCES agents(id) ON DELETE SET NULL,
  due_date              TIMESTAMPTZ,
  blocked_reason        JSONB,       -- {"category":"dependency|clarification|external|risk|budget|policy","text":"..."}
  source                TEXT NOT NULL DEFAULT 'human' CHECK (source IN ('human','agent')),
  created_by_agent_id   UUID REFERENCES agents(id) ON DELETE SET NULL,
  parent_id             UUID REFERENCES cards(id) ON DELETE SET NULL,
  sow_id                UUID,
  plan_task_id          UUID,
  budget_microusd       BIGINT NOT NULL DEFAULT 5000000 CHECK (budget_microusd >= 0),
  spent_microusd        BIGINT NOT NULL DEFAULT 0 CHECK (spent_microusd >= 0),
  merge_sha             TEXT,
  pr_url                TEXT,
  branch_name           TEXT,
  needs_refinement      BOOLEAN NOT NULL DEFAULT false,
  overdue_notified_at   TIMESTAMPTZ,
  version               INTEGER NOT NULL DEFAULT 1,
  created_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (project_id, seq),
  UNIQUE (project_id, card_key),
  CHECK (column_name <> 'ICEBOX' OR blocked_reason IS NOT NULL),
  CHECK (column_name <> 'READY' OR (assignee_id IS NOT NULL AND due_date IS NOT NULL
         AND estimate IS NOT NULL AND priority <> 'wont')),
  CHECK (column_name <> 'REVIEW' OR (reviewer_id IS NOT NULL AND pr_url IS NOT NULL)),
  CHECK (column_name <> 'DONE' OR merge_sha IS NOT NULL),
  CHECK (reviewer_id IS NULL OR assignee_id IS NULL OR reviewer_id <> assignee_id)
);
CREATE TRIGGER trg_cards_updated BEFORE UPDATE ON cards
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();
CREATE INDEX idx_cards_board_column ON cards(board_id, column_name);
CREATE INDEX idx_cards_assignee ON cards(assignee_id) WHERE column_name IN ('READY','DOING');
CREATE INDEX idx_cards_due ON cards(due_date) WHERE column_name NOT IN ('DONE','ICEBOX');
CREATE INDEX idx_cards_parent ON cards(parent_id);
CREATE INDEX idx_cards_sow ON cards(sow_id);
CREATE INDEX idx_cards_labels ON cards USING GIN(labels);
CREATE INDEX idx_cards_title_trgm ON cards USING GIN(title gin_trgm_ops);

CREATE TABLE card_events (
  id            UUID PRIMARY KEY,
  card_id       UUID NOT NULL REFERENCES cards(id) ON DELETE CASCADE,
  seq           BIGINT NOT NULL,
  event_type    TEXT NOT NULL,   -- 'created','moved','updated','assigned','commented','blocked','unblocked'
  from_column   TEXT,
  to_column     TEXT,
  actor_type    TEXT NOT NULL CHECK (actor_type IN ('human','agent','system')),
  actor_id      UUID,
  reason        TEXT,
  metadata      JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (card_id, seq)
);
CREATE INDEX idx_card_events_card ON card_events(card_id, seq);
CREATE INDEX idx_card_events_created ON card_events(created_at);

CREATE TABLE card_comments (
  id            UUID PRIMARY KEY,
  card_id       UUID NOT NULL REFERENCES cards(id) ON DELETE CASCADE,
  author_type   TEXT NOT NULL CHECK (author_type IN ('human','agent','system')),
  author_id     UUID,
  body          TEXT NOT NULL CHECK (char_length(body) BETWEEN 1 AND 10000),
  created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_card_comments_card ON card_comments(card_id, created_at);

-- ----------------------------------------------------------------------------
-- 8. SOW and Plan
-- ----------------------------------------------------------------------------
CREATE TABLE sows (
  id                UUID PRIMARY KEY,
  project_id        UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  lab_id            UUID NOT NULL REFERENCES labs(id) ON DELETE CASCADE,
  title             TEXT NOT NULL,
  status            TEXT NOT NULL DEFAULT 'draft'
                      CHECK (status IN ('draft','in_review','approved','superseded','rejected')),
  current_version   INTEGER NOT NULL DEFAULT 1,
  approved_at       TIMESTAMPTZ,
  approved_by       TEXT,
  created_by_agent_id UUID REFERENCES agents(id) ON DELETE SET NULL,
  created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at        TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TRIGGER trg_sows_updated BEFORE UPDATE ON sows
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();
CREATE INDEX idx_sows_project ON sows(project_id, created_at DESC);

CREATE TABLE sow_versions (
  id            UUID PRIMARY KEY,
  sow_id        UUID NOT NULL REFERENCES sows(id) ON DELETE CASCADE,
  version       INTEGER NOT NULL,
  document      JSONB NOT NULL,      -- structured SOW, validated by JSON Schema
  markdown      TEXT NOT NULL,
  content_hash  TEXT NOT NULL,
  change_note   TEXT,
  created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (sow_id, version)
);

CREATE TABLE plans (
  id                UUID PRIMARY KEY,
  project_id        UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  sow_id            UUID REFERENCES sows(id) ON DELETE SET NULL,
  title             TEXT NOT NULL,
  status            TEXT NOT NULL DEFAULT 'draft'
                      CHECK (status IN ('draft','approved','materialized','superseded')),
  current_version   INTEGER NOT NULL DEFAULT 1,
  created_by_agent_id UUID REFERENCES agents(id) ON DELETE SET NULL,
  created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at        TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TRIGGER trg_plans_updated BEFORE UPDATE ON plans
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE plan_versions (
  id            UUID PRIMARY KEY,
  plan_id       UUID NOT NULL REFERENCES plans(id) ON DELETE CASCADE,
  version       INTEGER NOT NULL,
  graph         JSONB NOT NULL,      -- {"nodes":[...],"edges":[{"from":"t1","to":"t2"}]}
  markdown      TEXT NOT NULL,
  created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (plan_id, version)
);

CREATE TABLE plan_tasks (
  id              UUID PRIMARY KEY,
  plan_id         UUID NOT NULL REFERENCES plans(id) ON DELETE CASCADE,
  version         INTEGER NOT NULL,
  task_key        TEXT NOT NULL,     -- stable within plan version, e.g. 'T-01'
  title           TEXT NOT NULL,
  description     TEXT NOT NULL DEFAULT '',
  role            TEXT NOT NULL CHECK (role IN ('tech_lead','analyst','dev','qa')),
  estimate        SMALLINT NOT NULL CHECK (estimate IN (1,2,3,5,8,13)),
  milestone       TEXT,
  depends_on      TEXT[] NOT NULL DEFAULT '{}',
  deliverables    JSONB NOT NULL DEFAULT '[]'::jsonb,
  acceptance      JSONB NOT NULL DEFAULT '[]'::jsonb,
  UNIQUE (plan_id, version, task_key)
);
CREATE INDEX idx_plan_tasks_plan ON plan_tasks(plan_id, version);

-- ----------------------------------------------------------------------------
-- 9. Runs, steps, LLM ledger, decisions
-- ----------------------------------------------------------------------------
CREATE TABLE runs (
  id                    UUID PRIMARY KEY,
  card_id               UUID NOT NULL REFERENCES cards(id) ON DELETE CASCADE,
  project_id            UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  lab_id                UUID NOT NULL REFERENCES labs(id) ON DELETE CASCADE,
  agent_id              UUID NOT NULL REFERENCES agents(id),
  attempt               SMALLINT NOT NULL DEFAULT 1,
  status                TEXT NOT NULL DEFAULT 'queued'
                          CHECK (status IN ('queued','running','succeeded','failed','cancelled','timed_out')),
  lease_id              UUID,
  sandbox_id            TEXT,
  plannotator           TEXT,
  started_at            TIMESTAMPTZ,
  ended_at              TIMESTAMPTZ,
  heartbeat_at          TIMESTAMPTZ,
  tokens_in             BIGINT NOT NULL DEFAULT 0,
  tokens_out            BIGINT NOT NULL DEFAULT 0,
  cached_tokens         BIGINT NOT NULL DEFAULT 0,
  cost_microusd         BIGINT NOT NULL DEFAULT 0,
  error_code            TEXT,
  error_detail          TEXT,
  result_summary        TEXT,
  pr_url                TEXT,
  merge_sha             TEXT,
  trace_id              TEXT,
  created_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (card_id, attempt)
);
CREATE TRIGGER trg_runs_updated BEFORE UPDATE ON runs
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();
CREATE INDEX idx_runs_card ON runs(card_id, attempt DESC);
CREATE INDEX idx_runs_agent_status ON runs(agent_id, status);
CREATE INDEX idx_runs_status_heartbeat ON runs(status, heartbeat_at) WHERE status = 'running';
CREATE INDEX idx_runs_lab_created ON runs(lab_id, created_at DESC);

CREATE TABLE run_steps (
  id            UUID PRIMARY KEY,
  run_id        UUID NOT NULL REFERENCES runs(id) ON DELETE CASCADE,
  seq           INTEGER NOT NULL,
  step_type     TEXT NOT NULL
                  CHECK (step_type IN ('plan','tool_call','llm_request','command','test',
                                       'commit','push','pr','review','decision','roadblock',
                                       'handoff','verification','reflection')),
  summary       TEXT NOT NULL,
  detail        JSONB NOT NULL DEFAULT '{}'::jsonb,
  tokens_in     BIGINT NOT NULL DEFAULT 0,
  tokens_out    BIGINT NOT NULL DEFAULT 0,
  cost_microusd BIGINT NOT NULL DEFAULT 0,
  duration_ms   INTEGER,
  created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (run_id, seq)
);
CREATE INDEX idx_run_steps_run ON run_steps(run_id, seq);

CREATE TABLE llm_requests (
  id                UUID PRIMARY KEY,
  lab_id            UUID NOT NULL REFERENCES labs(id) ON DELETE CASCADE,
  project_id        UUID REFERENCES projects(id) ON DELETE SET NULL,
  card_id           UUID REFERENCES cards(id) ON DELETE SET NULL,
  agent_id          UUID REFERENCES agents(id) ON DELETE SET NULL,
  run_id            UUID REFERENCES runs(id) ON DELETE SET NULL,
  provider_id       UUID NOT NULL REFERENCES providers(id),
  model_id          UUID NOT NULL REFERENCES models(id),
  model_name        TEXT NOT NULL,
  purpose           TEXT NOT NULL,   -- 'plan','code','review','reflection','sow','classify'
  prompt_hash       TEXT NOT NULL,
  prompt_tokens     BIGINT NOT NULL,
  completion_tokens BIGINT NOT NULL,
  cached_tokens     BIGINT NOT NULL DEFAULT 0,
  reasoning_tokens  BIGINT NOT NULL DEFAULT 0,
  cost_microusd     BIGINT NOT NULL DEFAULT 0,
  latency_ms        INTEGER NOT NULL,
  status            TEXT NOT NULL CHECK (status IN ('ok','error','timeout','rate_limited','invalid_output')),
  error_code        TEXT,
  trace_id          TEXT,
  created_at        TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_llm_lab_created ON llm_requests(lab_id, created_at DESC);
CREATE INDEX idx_llm_card ON llm_requests(card_id);
CREATE INDEX idx_llm_run ON llm_requests(run_id);

CREATE TABLE decisions (
  id            UUID PRIMARY KEY,
  lab_id        UUID NOT NULL REFERENCES labs(id) ON DELETE CASCADE,
  project_id    UUID REFERENCES projects(id) ON DELETE CASCADE,
  card_id       UUID REFERENCES cards(id) ON DELETE SET NULL,
  run_id        UUID REFERENCES runs(id) ON DELETE SET NULL,
  title         TEXT NOT NULL,
  context       TEXT NOT NULL,
  decision      TEXT NOT NULL,
  alternatives  JSONB NOT NULL DEFAULT '[]'::jsonb,
  consequences  TEXT,
  reversible    BOOLEAN NOT NULL DEFAULT true,
  actor_type    TEXT NOT NULL CHECK (actor_type IN ('human','agent','system')),
  actor_id      UUID,
  created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_decisions_project ON decisions(project_id, created_at DESC);

-- ----------------------------------------------------------------------------
-- 10. Memory (three scopes) and embeddings
-- ----------------------------------------------------------------------------
CREATE TABLE memory_documents (
  id            UUID PRIMARY KEY,
  lab_id        UUID NOT NULL REFERENCES labs(id) ON DELETE CASCADE,
  project_id    UUID REFERENCES projects(id) ON DELETE CASCADE,
  agent_id      UUID REFERENCES agents(id) ON DELETE CASCADE,
  scope         TEXT NOT NULL CHECK (scope IN ('lab','project','agent')),
  kind          TEXT NOT NULL
                  CHECK (kind IN ('sow','plan','adr','note','transcript','artifact',
                                  'policy','skill','episode_summary','review')),
  title         TEXT NOT NULL,
  content       TEXT NOT NULL,
  content_hash  TEXT NOT NULL,
  source_ref    TEXT,
  created_by    TEXT NOT NULL CHECK (created_by IN ('human','agent','system','compaction')),
  created_by_agent_id UUID REFERENCES agents(id) ON DELETE SET NULL,
  pinned        BOOLEAN NOT NULL DEFAULT false,
  version       INTEGER NOT NULL DEFAULT 1,
  created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
  CHECK (
    (scope = 'lab'     AND project_id IS NULL AND agent_id IS NULL) OR
    (scope = 'project' AND project_id IS NOT NULL AND agent_id IS NULL) OR
    (scope = 'agent'   AND project_id IS NULL AND agent_id IS NOT NULL)
  )
);
CREATE TRIGGER trg_memory_documents_updated BEFORE UPDATE ON memory_documents
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();
CREATE INDEX idx_mem_docs_scope ON memory_documents(lab_id, scope, project_id, agent_id);
CREATE INDEX idx_mem_docs_hash ON memory_documents(content_hash);
CREATE INDEX idx_mem_docs_fts ON memory_documents
  USING GIN(to_tsvector('english', title || ' ' || content));

CREATE TABLE memory_chunks (
  id            UUID PRIMARY KEY,
  document_id   UUID NOT NULL REFERENCES memory_documents(id) ON DELETE CASCADE,
  lab_id        UUID NOT NULL REFERENCES labs(id) ON DELETE CASCADE,
  project_id    UUID REFERENCES projects(id) ON DELETE CASCADE,
  agent_id      UUID REFERENCES agents(id) ON DELETE CASCADE,
  scope         TEXT NOT NULL CHECK (scope IN ('lab','project','agent')),
  ordinal       INTEGER NOT NULL,
  text          TEXT NOT NULL,
  token_count   INTEGER NOT NULL,
  embedding     vector(384) NOT NULL,
  metadata      JSONB NOT NULL DEFAULT '{}'::jsonb,
  tsv           tsvector GENERATED ALWAYS AS (to_tsvector('english', text)) STORED,
  created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (document_id, ordinal)
);
CREATE INDEX idx_mem_chunks_scope ON memory_chunks(lab_id, scope, project_id, agent_id);
CREATE INDEX idx_mem_chunks_hnsw ON memory_chunks
  USING hnsw (embedding vector_cosine_ops) WITH (m = 16, ef_construction = 64);
CREATE INDEX idx_mem_chunks_tsv ON memory_chunks USING GIN(tsv);

-- Episodic memory: raw run events, compacted into documents later.
CREATE TABLE memory_episodes (
  id            UUID PRIMARY KEY,
  lab_id        UUID NOT NULL REFERENCES labs(id) ON DELETE CASCADE,
  project_id    UUID REFERENCES projects(id) ON DELETE CASCADE,
  agent_id      UUID REFERENCES agents(id) ON DELETE CASCADE,
  card_id       UUID REFERENCES cards(id) ON DELETE SET NULL,
  run_id        UUID REFERENCES runs(id) ON DELETE SET NULL,
  kind          TEXT NOT NULL CHECK (kind IN ('event','transcript','tool_call','lesson')),
  content       TEXT NOT NULL,
  metadata      JSONB NOT NULL DEFAULT '{}'::jsonb,
  compacted_at  TIMESTAMPTZ,
  created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_episodes_lab_created ON memory_episodes(lab_id, created_at DESC);
CREATE INDEX idx_episodes_uncompacted ON memory_episodes(created_at)
  WHERE compacted_at IS NULL;

-- ----------------------------------------------------------------------------
-- 11. Events, outbox, audit
-- ----------------------------------------------------------------------------
CREATE TABLE domain_events (
  id            UUID PRIMARY KEY,
  lab_id        UUID NOT NULL REFERENCES labs(id) ON DELETE CASCADE,
  seq           BIGINT NOT NULL,
  event_type    TEXT NOT NULL,
  aggregate_type TEXT NOT NULL,
  aggregate_id  UUID NOT NULL,
  actor_type    TEXT NOT NULL CHECK (actor_type IN ('human','agent','system')),
  actor_id      UUID,
  payload       JSONB NOT NULL,
  schema_version INTEGER NOT NULL DEFAULT 1,
  occurred_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (lab_id, seq)
);
CREATE INDEX idx_events_lab_seq ON domain_events(lab_id, seq);
CREATE INDEX idx_events_type_time ON domain_events(event_type, occurred_at DESC);
CREATE INDEX idx_events_aggregate ON domain_events(aggregate_type, aggregate_id);

CREATE TABLE outbox (
  id              UUID PRIMARY KEY,
  aggregate_type  TEXT NOT NULL,
  aggregate_id    UUID NOT NULL,
  event_type      TEXT NOT NULL,
  payload         JSONB NOT NULL,
  headers         JSONB NOT NULL DEFAULT '{}'::jsonb,
  attempts        SMALLINT NOT NULL DEFAULT 0,
  last_error      TEXT,
  available_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
  published_at    TIMESTAMPTZ,
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_outbox_unpublished ON outbox(available_at)
  WHERE published_at IS NULL;

CREATE TABLE audit_log (
  id            UUID PRIMARY KEY,
  lab_id        UUID REFERENCES labs(id) ON DELETE SET NULL,
  actor_type    TEXT NOT NULL CHECK (actor_type IN ('human','agent','system')),
  actor_id      UUID,
  action        TEXT NOT NULL,
  target_type   TEXT NOT NULL,
  target_id     TEXT,
  before        JSONB,
  after         JSONB,
  ip            INET,
  user_agent    TEXT,
  trace_id      TEXT,
  created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_audit_lab_time ON audit_log(lab_id, created_at DESC);
CREATE INDEX idx_audit_action_time ON audit_log(action, created_at DESC);

CREATE TABLE github_webhook_deliveries (
  delivery_id   TEXT PRIMARY KEY,
  event         TEXT NOT NULL,
  action        TEXT,
  payload_hash  TEXT NOT NULL,
  received_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
  processed_at  TIMESTAMPTZ,
  status        TEXT NOT NULL DEFAULT 'received'
                  CHECK (status IN ('received','processed','ignored','failed')),
  error         TEXT
);
CREATE INDEX idx_webhooks_unprocessed ON github_webhook_deliveries(received_at)
  WHERE processed_at IS NULL;

-- ----------------------------------------------------------------------------
-- 12. Budgets, maintenance, settings
-- ----------------------------------------------------------------------------
CREATE TABLE cost_ledger (
  id            UUID PRIMARY KEY,
  lab_id        UUID NOT NULL REFERENCES labs(id) ON DELETE CASCADE,
  project_id    UUID REFERENCES projects(id) ON DELETE SET NULL,
  card_id       UUID REFERENCES cards(id) ON DELETE SET NULL,
  agent_id      UUID REFERENCES agents(id) ON DELETE SET NULL,
  run_id        UUID REFERENCES runs(id) ON DELETE SET NULL,
  source        TEXT NOT NULL CHECK (source IN ('llm','sandbox','other')),
  amount_microusd BIGINT NOT NULL,
  occurred_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_cost_lab_time ON cost_ledger(lab_id, occurred_at DESC);
CREATE INDEX idx_cost_card ON cost_ledger(card_id);

CREATE TABLE maintenance_jobs (
  id            UUID PRIMARY KEY,
  name          TEXT NOT NULL UNIQUE,
  cron          TEXT NOT NULL,
  enabled       BOOLEAN NOT NULL DEFAULT true,
  last_run_at   TIMESTAMPTZ,
  next_run_at   TIMESTAMPTZ,
  last_status   TEXT,
  last_error    TEXT,
  config        JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TRIGGER trg_maintenance_updated BEFORE UPDATE ON maintenance_jobs
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE app_settings (
  key           TEXT PRIMARY KEY,
  value         JSONB NOT NULL,
  updated_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ----------------------------------------------------------------------------
-- 13. Seed: character roster (10)
-- ----------------------------------------------------------------------------
INSERT INTO characters (id, display_name, archetype, sprite_key, palette, speech_style) VALUES
  ('veteran_lead',  'The Veteran Lead',  'Silver-haired lead, glasses, cardigan, calm authority',
   'char_veteran_lead',  '{"hair":"#c9c9c9","skin":"#e8b98f","top":"#6b4f3a","accent":"#f2b84b"}',
   'Measured, decisive, asks one sharp question at a time.'),
  ('systems_analyst','The Systems Analyst','Curly-haired analyst, hoodie, thick notebook',
   'char_systems_analyst','{"hair":"#2b1d16","skin":"#8d5524","top":"#2e3a6b","accent":"#57c785"}',
   'Precise, curious, structures everything into lists.'),
  ('scrappy_dev',   'The Scrappy Dev',   'Buzz-cut young dev, oversized tee, headphones around neck',
   'char_scrappy_dev',   '{"hair":"#1b1b1b","skin":"#f1c27d","top":"#c0392b","accent":"#f2e9d8"}',
   'Fast, informal, explains with examples and small jokes.'),
  ('staff_architect','The Staff Architect','Tall slim architect, curly hair, black turtleneck',
   'char_staff_architect','{"hair":"#141414","skin":"#5c3317","top":"#141b3c","accent":"#9fb3e6"}',
   'Big-picture, diagrams first, names trade-offs explicitly.'),
  ('precise_consultant','The Precise Consultant','Tailored suit, tie, notebook, immaculate desk',
   'char_precise_consultant','{"hair":"#1a1a1a","skin":"#c68642","top":"#2e3a6b","accent":"#f2e9d8"}',
   'Formal, exact, quotes requirements by ID.'),
  ('confident_muse','The Confident Muse','Glamorous red-dress product strategist',
   'char_confident_muse','{"hair":"#3b1f14","skin":"#f1c27d","top":"#c0392b","accent":"#c45aa0"}',
   'Bold, persuasive, focuses on the user outcome.'),
  ('steady_operator','The Steady Operator','Heads-up display, headset, blazer, unhurried',
   'char_steady_operator','{"hair":"#7a5230","skin":"#e8b98f","top":"#2a9d8f","accent":"#f2b84b"}',
   'Warm, pragmatic, keeps everyone unblocked.'),
  ('eager_intern',  'The Eager Intern',  'Teen dev, cap, backpack, always taking notes',
   'char_eager_intern',  '{"hair":"#3b2a1a","skin":"#f1c27d","top":"#57c785","accent":"#f2b84b"}',
   'Enthusiastic, asks "why", volunteers for small tasks.'),
  ('pixel_artisan', 'The Pixel Artisan', 'Beanie, tattoo sleeves, utility apron, craftsperson',
   'char_pixel_artisan','{"hair":"#2b1d16","skin":"#c68642","top":"#e07a3f","accent":"#2a9d8f"}',
   'Detail-obsessed, visual, talks in frames and pixels.'),
  ('quiet_professor','The Quiet Professor','Bearded senior, tweed jacket, chalk dust',
   'char_quiet_professor','{"hair":"#b0b0b0","skin":"#e8b98f","top":"#6b4f3a","accent":"#9fb3e6"}',
   'Reflective, cites principles, spots edge cases.');

-- ----------------------------------------------------------------------------
-- 14. Seed: default models (provider credential added at runtime)
-- ----------------------------------------------------------------------------
-- Models are created per provider at runtime because provider_id is dynamic.
-- The API seeds these two rows when a DeepSeek provider is created:
--   deepseek-flash   (default)   context 131072, max output 8192
--   deepseek-v4-pro  (escalation) context 131072, max output 16384

-- ----------------------------------------------------------------------------
-- 15. Seed: maintenance jobs
-- ----------------------------------------------------------------------------
INSERT INTO maintenance_jobs (id, name, cron, config) VALUES
  (gen_random_uuid(), 'memory_compaction',      '0 3 * * *',   '{"episode_age_days":30,"retention_days":180}'),
  (gen_random_uuid(), 'cv_reflection_backfill', '0 4 * * *',   '{"max_runs":500}'),
  (gen_random_uuid(), 'dependency_check',       '0 6 * * 1',   '{"create_todo":true}'),
  (gen_random_uuid(), 'stale_card_scan',        '0 7 * * *',   '{"stale_days":7}'),
  (gen_random_uuid(), 'overdue_notify',         '*/30 * * * *','{}'),
  (gen_random_uuid(), 'data_integrity',         '0 2 * * *',   '{"report_only":true}'),
  (gen_random_uuid(), 'provider_health',        '* * * * *',   '{"timeout_ms":5000}');

-- ----------------------------------------------------------------------------
-- 16. Deferred foreign keys (tables created later in the file)
-- ----------------------------------------------------------------------------
ALTER TABLE cards
  ADD CONSTRAINT fk_cards_sow
  FOREIGN KEY (sow_id) REFERENCES sows(id) ON DELETE SET NULL;
ALTER TABLE cards
  ADD CONSTRAINT fk_cards_plan_task
  FOREIGN KEY (plan_task_id) REFERENCES plan_tasks(id) ON DELETE SET NULL;
ALTER TABLE agent_cv_versions
  ADD CONSTRAINT fk_cv_run
  FOREIGN KEY (run_id) REFERENCES runs(id) ON DELETE SET NULL;

