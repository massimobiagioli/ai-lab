//! Provider persistence for the edge API.
//!
//! The API stores the encrypted key material only; no endpoint returns the
//! plaintext key. Postgres is used when `AILAB_DATABASE_URL` is configured; the
//! in-memory store is used by tests and by the default `AppState`.

use std::collections::BTreeMap;
use std::fmt;
use std::sync::Mutex;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// Workspace seeded by the migrations.
pub const DEFAULT_WORKSPACE_ID: &str = "00000000-0000-7000-8000-000000000001";

/// A provider row returned by the API. Secret material is never included.
#[derive(Debug, Clone, Serialize)]
pub struct ProviderView {
    /// Provider id.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Provider kind (`deepseek`).
    pub kind: String,
    /// HTTPS base URL.
    pub base_url: String,
    /// Lifecycle status.
    pub status: String,
    /// Last four key characters.
    pub api_key_last4: String,
    /// Envelope key version.
    pub key_version: i32,
    /// Requests per minute.
    pub rate_limit_rpm: u32,
    /// Tokens per minute.
    pub rate_limit_tpm: u32,
    /// Request timeout in milliseconds.
    pub timeout_ms: u32,
    /// Last health probe.
    pub last_health_at: Option<DateTime<Utc>>,
    /// Last health detail.
    pub last_health_detail: Option<serde_json::Value>,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Update timestamp.
    pub updated_at: DateTime<Utc>,
}

/// Encrypted key material.
#[derive(Debug, Clone)]
pub struct EncryptedKeyMaterial {
    /// AES-256-GCM ciphertext.
    pub ciphertext: Vec<u8>,
    /// 96-bit nonce.
    pub nonce: [u8; 12],
    /// Envelope key version.
    pub key_version: i32,
    /// Last four key characters.
    pub last4: String,
}

/// New provider row.
#[derive(Debug, Clone)]
pub struct NewProvider {
    /// Provider id.
    pub id: String,
    /// Workspace id.
    pub workspace_id: String,
    /// Display name.
    pub name: String,
    /// Provider kind.
    pub kind: String,
    /// HTTPS base URL.
    pub base_url: String,
    /// Encrypted key material.
    pub key: EncryptedKeyMaterial,
    /// Requests per minute.
    pub rate_limit_rpm: u32,
    /// Tokens per minute.
    pub rate_limit_tpm: u32,
    /// Request timeout in milliseconds.
    pub timeout_ms: u32,
}

/// Provider store failure.
#[derive(Debug)]
pub enum ProviderStoreError {
    /// The provider is missing.
    NotFound,
    /// The provider name already exists.
    Conflict,
    /// The storage backend failed.
    Backend(String),
}

impl fmt::Display for ProviderStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound => formatter.write_str("provider not found"),
            Self::Conflict => formatter.write_str("provider already exists"),
            Self::Backend(message) => write!(formatter, "provider store failed: {message}"),
        }
    }
}

impl std::error::Error for ProviderStoreError {}

/// Provider persistence port.
#[async_trait]
pub trait ProviderStore: Send + Sync {
    /// All providers, newest first.
    async fn list(&self) -> Result<Vec<ProviderView>, ProviderStoreError>;
    /// One provider by id.
    async fn get(&self, id: &str) -> Result<Option<ProviderView>, ProviderStoreError>;
    /// One provider by name.
    async fn find_by_name(&self, name: &str) -> Result<Option<ProviderView>, ProviderStoreError>;
    /// Insert a provider.
    async fn create(&self, provider: NewProvider) -> Result<ProviderView, ProviderStoreError>;
    /// Rotate the encrypted key and increment the key version.
    async fn rotate_key(
        &self,
        id: &str,
        key: EncryptedKeyMaterial,
    ) -> Result<ProviderView, ProviderStoreError>;
}

#[derive(Debug, Clone)]
struct StoredProvider {
    view: ProviderView,
    key: EncryptedKeyMaterial,
}

/// In-memory provider store for tests and the default state.
#[derive(Default)]
pub struct InMemoryProviderStore {
    providers: Mutex<BTreeMap<String, StoredProvider>>,
}

impl InMemoryProviderStore {
    /// New empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
#[allow(clippy::significant_drop_tightening)]
impl ProviderStore for InMemoryProviderStore {
    async fn list(&self) -> Result<Vec<ProviderView>, ProviderStoreError> {
        let providers = self
            .providers
            .lock()
            .map_err(|_| ProviderStoreError::Backend("lock poisoned".to_owned()))?;
        let mut views = providers
            .values()
            .map(|stored| stored.view.clone())
            .collect::<Vec<_>>();
        views.sort_by_key(|left| std::cmp::Reverse(left.created_at));
        Ok(views)
    }

    async fn get(&self, id: &str) -> Result<Option<ProviderView>, ProviderStoreError> {
        let providers = self
            .providers
            .lock()
            .map_err(|_| ProviderStoreError::Backend("lock poisoned".to_owned()))?;
        Ok(providers.get(id).map(|stored| stored.view.clone()))
    }

    async fn find_by_name(&self, name: &str) -> Result<Option<ProviderView>, ProviderStoreError> {
        let providers = self
            .providers
            .lock()
            .map_err(|_| ProviderStoreError::Backend("lock poisoned".to_owned()))?;
        Ok(providers
            .values()
            .find(|stored| stored.view.name == name)
            .map(|stored| stored.view.clone()))
    }

    async fn create(&self, provider: NewProvider) -> Result<ProviderView, ProviderStoreError> {
        let mut providers = self
            .providers
            .lock()
            .map_err(|_| ProviderStoreError::Backend("lock poisoned".to_owned()))?;
        if providers
            .values()
            .any(|stored| stored.view.name == provider.name)
        {
            return Err(ProviderStoreError::Conflict);
        }
        let now = Utc::now();
        let view = ProviderView {
            id: provider.id.clone(),
            name: provider.name,
            kind: provider.kind,
            base_url: provider.base_url,
            status: "active".to_owned(),
            api_key_last4: provider.key.last4.clone(),
            key_version: provider.key.key_version,
            rate_limit_rpm: provider.rate_limit_rpm,
            rate_limit_tpm: provider.rate_limit_tpm,
            timeout_ms: provider.timeout_ms,
            last_health_at: None,
            last_health_detail: None,
            created_at: now,
            updated_at: now,
        };
        providers.insert(
            provider.id,
            StoredProvider {
                view: view.clone(),
                key: provider.key,
            },
        );
        Ok(view)
    }

    async fn rotate_key(
        &self,
        id: &str,
        key: EncryptedKeyMaterial,
    ) -> Result<ProviderView, ProviderStoreError> {
        let mut providers = self
            .providers
            .lock()
            .map_err(|_| ProviderStoreError::Backend("lock poisoned".to_owned()))?;
        let stored = providers.get_mut(id).ok_or(ProviderStoreError::NotFound)?;
        stored.view.api_key_last4.clone_from(&key.last4);
        stored.view.key_version = key.key_version;
        stored.view.updated_at = Utc::now();
        stored.key = key;
        Ok(stored.view.clone())
    }
}

/// Postgres-backed provider store.
#[derive(Debug, Clone)]
pub struct PostgresProviderStore {
    pool: PgPool,
}

impl PostgresProviderStore {
    /// Build from a pool.
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn view_from_row(row: &sqlx::postgres::PgRow) -> Result<ProviderView, ProviderStoreError> {
        let id: Uuid = row
            .try_get("id")
            .map_err(|error| ProviderStoreError::Backend(error.to_string()))?;
        let last_health_detail: Option<String> = row
            .try_get("last_health_detail_text")
            .map_err(|error| ProviderStoreError::Backend(error.to_string()))?;
        Ok(ProviderView {
            id: id.to_string(),
            name: row
                .try_get("name")
                .map_err(|error| ProviderStoreError::Backend(error.to_string()))?,
            kind: row
                .try_get("kind")
                .map_err(|error| ProviderStoreError::Backend(error.to_string()))?,
            base_url: row
                .try_get("base_url")
                .map_err(|error| ProviderStoreError::Backend(error.to_string()))?,
            status: row
                .try_get("status")
                .map_err(|error| ProviderStoreError::Backend(error.to_string()))?,
            api_key_last4: row
                .try_get("api_key_last4")
                .map_err(|error| ProviderStoreError::Backend(error.to_string()))?,
            key_version: row
                .try_get("key_version")
                .map_err(|error| ProviderStoreError::Backend(error.to_string()))?,
            rate_limit_rpm: u32::try_from(
                row.try_get::<i32, _>("rate_limit_rpm")
                    .map_err(|error| ProviderStoreError::Backend(error.to_string()))?,
            )
            .map_err(|_| ProviderStoreError::Backend("invalid rate_limit_rpm".to_owned()))?,
            rate_limit_tpm: u32::try_from(
                row.try_get::<i32, _>("rate_limit_tpm")
                    .map_err(|error| ProviderStoreError::Backend(error.to_string()))?,
            )
            .map_err(|_| ProviderStoreError::Backend("invalid rate_limit_tpm".to_owned()))?,
            timeout_ms: u32::try_from(
                row.try_get::<i32, _>("timeout_ms")
                    .map_err(|error| ProviderStoreError::Backend(error.to_string()))?,
            )
            .map_err(|_| ProviderStoreError::Backend("invalid timeout_ms".to_owned()))?,
            last_health_at: row
                .try_get("last_health_at")
                .map_err(|error| ProviderStoreError::Backend(error.to_string()))?,
            last_health_detail: last_health_detail
                .and_then(|text| serde_json::from_str(&text).ok()),
            created_at: row
                .try_get("created_at")
                .map_err(|error| ProviderStoreError::Backend(error.to_string()))?,
            updated_at: row
                .try_get("updated_at")
                .map_err(|error| ProviderStoreError::Backend(error.to_string()))?,
        })
    }

    async fn seed_models(&self, provider_id: Uuid) -> Result<(), ProviderStoreError> {
        let models = [
            ("deepseek-flash", "DeepSeek Flash", 8192_i32, true, false),
            ("deepseek-v4-pro", "DeepSeek V4 Pro", 16384_i32, false, true),
        ];
        for (name, display_name, max_output, is_default, is_escalation) in models {
            sqlx::query(
                "INSERT INTO models (
                    id, provider_id, name, display_name, context_window,
                    max_output_tokens, supports_tools, supports_reasoning,
                    supports_vision, input_cost_microusd_per_mtok,
                    output_cost_microusd_per_mtok, is_default, is_escalation
                 ) VALUES ($1, $2, $3, $4, 131072, $5, true, true, false, 0, 0, $6, $7)
                 ON CONFLICT (provider_id, name) DO NOTHING",
            )
            .bind(Uuid::now_v7())
            .bind(provider_id)
            .bind(name)
            .bind(display_name)
            .bind(max_output)
            .bind(is_default)
            .bind(is_escalation)
            .execute(&self.pool)
            .await
            .map_err(|error| ProviderStoreError::Backend(error.to_string()))?;
        }
        Ok(())
    }
}

const PROVIDER_COLUMNS: &str = "id, name, kind, base_url, status, api_key_last4,
    key_version, rate_limit_rpm, rate_limit_tpm, timeout_ms, last_health_at,
    last_health_detail::text AS last_health_detail_text, created_at, updated_at";

#[async_trait]
impl ProviderStore for PostgresProviderStore {
    async fn list(&self) -> Result<Vec<ProviderView>, ProviderStoreError> {
        let query = format!("SELECT {PROVIDER_COLUMNS} FROM providers ORDER BY created_at DESC");
        let rows = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(|error| ProviderStoreError::Backend(error.to_string()))?;
        rows.iter().map(Self::view_from_row).collect()
    }

    async fn get(&self, id: &str) -> Result<Option<ProviderView>, ProviderStoreError> {
        let uuid = Uuid::parse_str(id).map_err(|_| ProviderStoreError::NotFound)?;
        let query = format!("SELECT {PROVIDER_COLUMNS} FROM providers WHERE id = $1");
        let row = sqlx::query(&query)
            .bind(uuid)
            .fetch_optional(&self.pool)
            .await
            .map_err(|error| ProviderStoreError::Backend(error.to_string()))?;
        row.as_ref().map(Self::view_from_row).transpose()
    }

    async fn find_by_name(&self, name: &str) -> Result<Option<ProviderView>, ProviderStoreError> {
        let query = format!("SELECT {PROVIDER_COLUMNS} FROM providers WHERE name = $1");
        let row = sqlx::query(&query)
            .bind(name)
            .fetch_optional(&self.pool)
            .await
            .map_err(|error| ProviderStoreError::Backend(error.to_string()))?;
        row.as_ref().map(Self::view_from_row).transpose()
    }

    async fn create(&self, provider: NewProvider) -> Result<ProviderView, ProviderStoreError> {
        let id = Uuid::parse_str(&provider.id)
            .map_err(|_| ProviderStoreError::Backend("invalid provider id".to_owned()))?;
        let workspace_id = Uuid::parse_str(&provider.workspace_id)
            .map_err(|_| ProviderStoreError::Backend("invalid workspace id".to_owned()))?;
        let result = sqlx::query(
            "INSERT INTO providers (
                id, workspace_id, name, kind, base_url, api_key_ciphertext,
                api_key_nonce, api_key_last4, key_version, status,
                rate_limit_rpm, rate_limit_tpm, timeout_ms
             ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'active', $10, $11, $12)",
        )
        .bind(id)
        .bind(workspace_id)
        .bind(&provider.name)
        .bind(&provider.kind)
        .bind(&provider.base_url)
        .bind(&provider.key.ciphertext)
        .bind(provider.key.nonce.to_vec())
        .bind(&provider.key.last4)
        .bind(provider.key.key_version)
        .bind(i32::try_from(provider.rate_limit_rpm).unwrap_or(i32::MAX))
        .bind(i32::try_from(provider.rate_limit_tpm).unwrap_or(i32::MAX))
        .bind(i32::try_from(provider.timeout_ms).unwrap_or(i32::MAX))
        .execute(&self.pool)
        .await;
        if let Err(error) = result {
            if error
                .as_database_error()
                .is_some_and(sqlx::error::DatabaseError::is_unique_violation)
            {
                return Err(ProviderStoreError::Conflict);
            }
            return Err(ProviderStoreError::Backend(error.to_string()));
        }
        self.seed_models(id).await?;
        self.get(&id.to_string()).await?.ok_or_else(|| {
            ProviderStoreError::Backend("provider disappeared after create".to_owned())
        })
    }

    async fn rotate_key(
        &self,
        id: &str,
        key: EncryptedKeyMaterial,
    ) -> Result<ProviderView, ProviderStoreError> {
        let uuid = Uuid::parse_str(id).map_err(|_| ProviderStoreError::NotFound)?;
        let result = sqlx::query(
            "UPDATE providers SET
                api_key_ciphertext = $1, api_key_nonce = $2, api_key_last4 = $3,
                key_version = $4, updated_at = now()
             WHERE id = $5",
        )
        .bind(&key.ciphertext)
        .bind(key.nonce.to_vec())
        .bind(&key.last4)
        .bind(key.key_version)
        .bind(uuid)
        .execute(&self.pool)
        .await
        .map_err(|error| ProviderStoreError::Backend(error.to_string()))?;
        if result.rows_affected() == 0 {
            return Err(ProviderStoreError::NotFound);
        }
        self.get(id).await?.ok_or(ProviderStoreError::NotFound)
    }
}
