//! Read-only resource queries for the management API.
//!
//! Queries are grouped by owning bounded context so the architecture fitness
//! check can verify table ownership. `row_to_json` keeps the shape close to the
//! database contract.

pub mod inventory;
pub mod lab;
pub mod platform;

pub use inventory::*;
pub use lab::*;
pub use platform::*;

use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn backend(error: sqlx::Error) -> String {
    error.to_string()
}

pub(crate) async fn list_json(pool: &PgPool, sql: &str) -> Result<Value, String> {
    let text: String = sqlx::query_scalar(sql)
        .fetch_one(pool)
        .await
        .map_err(backend)?;
    serde_json::from_str(&text).map_err(|error| error.to_string())
}

pub(crate) async fn get_json(pool: &PgPool, sql: &str, id: &str) -> Result<Option<Value>, String> {
    let uuid = Uuid::parse_str(id).map_err(|_| "invalid id".to_owned())?;
    let text: Option<String> = sqlx::query_scalar(sql)
        .bind(uuid)
        .fetch_optional(pool)
        .await
        .map_err(backend)?;
    text.map(|text| serde_json::from_str(&text).map_err(|error| error.to_string()))
        .transpose()
}
