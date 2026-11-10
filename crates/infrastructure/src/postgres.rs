//! PostgreSQL composition helper for the API and worker.

use std::sync::Arc;

use ailab_application::{ManagementService, RunStore};
use sqlx::postgres::PgPoolOptions;

use crate::crypto::MasterKey;
use crate::management_postgres::PostgresManagementService;
use crate::run_store::PostgresRunStore;

/// PostgreSQL-backed application services.
pub struct PostgresServices {
    /// Run store.
    pub runs: Arc<dyn RunStore>,
    /// Management service.
    pub management: Arc<dyn ManagementService>,
}

/// Build lazily connected PostgreSQL services.
pub fn connect_lazy(url: &str) -> Result<PostgresServices, sqlx::Error> {
    let pool = PgPoolOptions::new().max_connections(10).connect_lazy(url)?;
    let master_key = std::env::var("AILAB_MASTER_KEY")
        .ok()
        .and_then(|encoded| MasterKey::from_base64(&encoded, 1).ok());
    Ok(PostgresServices {
        runs: Arc::new(PostgresRunStore::new(pool.clone())),
        management: Arc::new(PostgresManagementService::new(pool, master_key)),
    })
}
