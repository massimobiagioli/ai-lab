//! Lab read projections.

use serde_json::Value;
use sqlx::PgPool;

use super::{get_json, list_json};

/// List active labs.
pub async fn list_labs(pool: &PgPool) -> Result<Value, String> {
    list_json(
        pool,
        "SELECT COALESCE(json_agg(row_to_json(t)), '[]'::json)::text FROM (
            SELECT id, workspace_id, name, slug, description, autonomy_level,
                   monthly_budget_microusd, settings, archived_at, created_at, updated_at
            FROM labs WHERE archived_at IS NULL ORDER BY created_at DESC
        ) t",
    )
    .await
}

/// One lab.
pub async fn get_lab(pool: &PgPool, id: &str) -> Result<Option<Value>, String> {
    get_json(
        pool,
        "SELECT row_to_json(t)::text FROM (
            SELECT id, workspace_id, name, slug, description, autonomy_level,
                   monthly_budget_microusd, settings, archived_at, created_at, updated_at
            FROM labs WHERE id = $1
        ) t",
        id,
    )
    .await
}
