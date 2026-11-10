//! Platform read projections.

use serde_json::Value;
use sqlx::PgPool;

use super::backend;

/// Key/value settings.
pub async fn get_settings(pool: &PgPool) -> Result<Value, String> {
    let text: String = sqlx::query_scalar(
        "SELECT COALESCE(json_object_agg(key, value), '{}'::json)::text FROM app_settings",
    )
    .fetch_one(pool)
    .await
    .map_err(backend)?;
    serde_json::from_str(&text).map_err(|error| error.to_string())
}
