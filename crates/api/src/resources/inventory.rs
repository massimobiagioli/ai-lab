//! Inventory read projections (agents, CV, characters, models, SCM, projects).

use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use super::{backend, get_json, list_json};

/// List agents, optionally filtered by lab.
pub async fn list_agents(pool: &PgPool, lab_id: Option<&str>) -> Result<Value, String> {
    let lab = lab_id
        .map(Uuid::parse_str)
        .transpose()
        .map_err(|_| "invalid lab id".to_owned())?;
    let text: String = sqlx::query_scalar(
        "SELECT COALESCE(json_agg(row_to_json(t)), '[]'::json)::text FROM (
            SELECT id, lab_id, name, role, character_id, description,
                   model_preference_id, autonomy_level, status, max_concurrent_cards,
                   cv_version, avatar_seed, retired_at, created_at, updated_at
            FROM agents
            WHERE ($1::uuid IS NULL OR lab_id = $1) AND retired_at IS NULL
            ORDER BY created_at DESC
        ) t",
    )
    .bind(lab)
    .fetch_one(pool)
    .await
    .map_err(backend)?;
    serde_json::from_str(&text).map_err(|error| error.to_string())
}

/// One agent.
pub async fn get_agent(pool: &PgPool, id: &str) -> Result<Option<Value>, String> {
    get_json(
        pool,
        "SELECT row_to_json(t)::text FROM (
            SELECT id, lab_id, name, role, character_id, description,
                   model_preference_id, autonomy_level, status, max_concurrent_cards,
                   cv_version, avatar_seed, retired_at, created_at, updated_at
            FROM agents WHERE id = $1
        ) t",
        id,
    )
    .await
}

/// Current agent CV plus the version history.
pub async fn get_agent_cv(pool: &PgPool, id: &str) -> Result<Option<Value>, String> {
    let uuid = Uuid::parse_str(id).map_err(|_| "invalid id".to_owned())?;
    let text: Option<String> = sqlx::query_scalar(
        "SELECT json_build_object(
            'current', (SELECT row_to_json(v) FROM (
                SELECT version, cv, summary, created_by, run_id, created_at
                FROM agent_cv_versions WHERE agent_id = $1 ORDER BY version DESC LIMIT 1
            ) v),
            'history', COALESCE((SELECT json_agg(row_to_json(h)) FROM (
                SELECT version, summary, created_at
                FROM agent_cv_versions WHERE agent_id = $1 ORDER BY version DESC
            ) h), '[]'::json)
         )::text",
    )
    .bind(uuid)
    .fetch_optional(pool)
    .await
    .map_err(backend)?;
    text.map(|text| serde_json::from_str(&text).map_err(|error| error.to_string()))
        .transpose()
}

/// Pause or resume an agent.
pub async fn set_agent_status(
    pool: &PgPool,
    id: &str,
    status: &str,
) -> Result<Option<Value>, String> {
    let uuid = Uuid::parse_str(id).map_err(|_| "invalid id".to_owned())?;
    let text: Option<String> = sqlx::query_scalar(
        "WITH updated AS (
            UPDATE agents SET status = $1 WHERE id = $2
            RETURNING id, lab_id, name, role, character_id, status, updated_at
         )
         SELECT row_to_json(t)::text FROM updated t",
    )
    .bind(status)
    .bind(uuid)
    .fetch_optional(pool)
    .await
    .map_err(backend)?;
    text.map(|text| serde_json::from_str(&text).map_err(|error| error.to_string()))
        .transpose()
}

/// Character roster.
pub async fn list_characters(pool: &PgPool) -> Result<Value, String> {
    list_json(
        pool,
        "SELECT COALESCE(json_agg(row_to_json(t)), '[]'::json)::text FROM (
            SELECT id, display_name, archetype, sprite_key, palette, speech_style
            FROM characters ORDER BY id
        ) t",
    )
    .await
}

/// Model catalog.
pub async fn list_models(pool: &PgPool) -> Result<Value, String> {
    list_json(
        pool,
        "SELECT COALESCE(json_agg(row_to_json(t)), '[]'::json)::text FROM (
            SELECT id, provider_id, name, display_name, context_window,
                   max_output_tokens, supports_tools, supports_reasoning,
                   supports_vision, input_cost_microusd_per_mtok,
                   output_cost_microusd_per_mtok, is_default, is_escalation,
                   status, created_at, updated_at
            FROM models ORDER BY provider_id, name
        ) t",
    )
    .await
}

/// SCM credentials without token material.
pub async fn list_scm_credentials(pool: &PgPool) -> Result<Value, String> {
    list_json(
        pool,
        "SELECT COALESCE(json_agg(row_to_json(t)), '[]'::json)::text FROM (
            SELECT id, workspace_id, name, kind, username, token_last4, key_version,
                   scopes, status, last_verified_at, created_at, updated_at
            FROM scm_credentials ORDER BY created_at DESC
        ) t",
    )
    .await
}

/// Projects.
pub async fn list_projects(pool: &PgPool) -> Result<Value, String> {
    list_json(
        pool,
        "SELECT COALESCE(json_agg(row_to_json(t)), '[]'::json)::text FROM (
            SELECT id, lab_id, name, slug, key_prefix, description, source_kind,
                   source_ref, default_branch, credential_id, status, card_seq,
                   settings, created_at, updated_at
            FROM projects ORDER BY created_at DESC
        ) t",
    )
    .await
}

/// One project.
pub async fn get_project(pool: &PgPool, id: &str) -> Result<Option<Value>, String> {
    get_json(
        pool,
        "SELECT row_to_json(t)::text FROM (
            SELECT id, lab_id, name, slug, key_prefix, description, source_kind,
                   source_ref, default_branch, credential_id, status, card_seq,
                   settings, created_at, updated_at
            FROM projects WHERE id = $1
        ) t",
        id,
    )
    .await
}
