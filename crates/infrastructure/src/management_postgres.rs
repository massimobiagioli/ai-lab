//! PostgreSQL management adapter: providers, labs, agents, projects and settings.

use std::str::FromStr;

use ailab_application::{
    CreateAgentInput, CreateLabRequest, CreateProjectRequest, CreateProviderRequest,
    ManagementError, ManagementService, RotateProviderKeyRequest, UpdateAgentRequest,
    UpdateProjectRequest,
};
use ailab_domain::inventory::{EncryptedKey, Provider, ProviderKind};
use ailab_domain::{ProviderId, WorkspaceId};
use async_trait::async_trait;
use serde_json::{Value, json};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::crypto::{MasterKey, aad};

const DEFAULT_WORKSPACE_ID: &str = "00000000-0000-7000-8000-000000000001";

#[allow(clippy::needless_pass_by_value)]
fn backend(error: sqlx::Error) -> ManagementError {
    if let Some(database) = error.as_database_error() {
        if database.is_unique_violation() {
            return ManagementError::conflict("CONFLICT", "a unique value already exists");
        }
        if database.code().as_deref() == Some("23503") {
            return ManagementError::validation(
                "VALIDATION_FAILED",
                "a referenced resource does not exist",
            );
        }
        if database.code().as_deref() == Some("23514") {
            return ManagementError::validation(
                "VALIDATION_FAILED",
                "the value violates a database constraint",
            );
        }
    }
    ManagementError::Backend(error.to_string())
}

async fn list_json(pool: &PgPool, sql: &str) -> Result<Value, ManagementError> {
    let text: String = sqlx::query_scalar(sql)
        .fetch_one(pool)
        .await
        .map_err(backend)?;
    serde_json::from_str(&text).map_err(|error| ManagementError::Backend(error.to_string()))
}

async fn get_json(pool: &PgPool, sql: &str, id: &str) -> Result<Option<Value>, ManagementError> {
    let uuid = Uuid::parse_str(id).map_err(|_| ManagementError::NotFound)?;
    let text: Option<String> = sqlx::query_scalar(sql)
        .bind(uuid)
        .fetch_optional(pool)
        .await
        .map_err(backend)?;
    text.map(|text| {
        serde_json::from_str(&text).map_err(|error| ManagementError::Backend(error.to_string()))
    })
    .transpose()
}

fn parse_optional(text: Option<String>) -> Result<Option<Value>, ManagementError> {
    text.map(|text| {
        serde_json::from_str(&text).map_err(|error| ManagementError::Backend(error.to_string()))
    })
    .transpose()
}

fn valid_slug(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
        })
}

fn valid_prefix(value: &str) -> bool {
    let mut characters = value.chars();
    matches!(characters.next(), Some(first) if first.is_ascii_uppercase())
        && characters.all(|character| character.is_ascii_uppercase() || character.is_ascii_digit())
        && (2..=10).contains(&value.len())
}

fn validate_agent_input(input: &CreateAgentInput, role: &str) -> Result<(), ManagementError> {
    if !matches!(role, "tech_lead" | "analyst" | "dev" | "qa") {
        return Err(ManagementError::validation(
            "VALIDATION_FAILED",
            "role must be tech_lead, analyst, dev or qa",
        ));
    }
    if !(2..=64).contains(&input.name.chars().count()) {
        return Err(ManagementError::validation(
            "VALIDATION_FAILED",
            "agent name must be 2-64 characters",
        ));
    }
    if !(120..=600).contains(&input.description.chars().count()) {
        return Err(ManagementError::validation(
            "VALIDATION_FAILED",
            "agent description must be 120-600 characters",
        ));
    }
    if let Some(capacity) = input.max_concurrent_cards
        && !(1..=4).contains(&capacity)
    {
        return Err(ManagementError::validation(
            "VALIDATION_FAILED",
            "max_concurrent_cards must be 1-4",
        ));
    }
    if let Some(autonomy) = input.autonomy_level
        && !(0..=3).contains(&autonomy)
    {
        return Err(ManagementError::validation(
            "VALIDATION_FAILED",
            "autonomy_level must be 0-3",
        ));
    }
    Ok(())
}

async fn insert_agent(
    transaction: &mut Transaction<'_, Postgres>,
    lab_id: Uuid,
    role: &str,
    input: &CreateAgentInput,
) -> Result<Value, ManagementError> {
    let agent_id = Uuid::now_v7();
    let text: Option<String> = sqlx::query_scalar(
        "INSERT INTO agents (
            id, lab_id, name, role, character_id, description, model_preference_id,
            autonomy_level, status, max_concurrent_cards, avatar_seed
         ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'idle', $9, $10)
         RETURNING row_to_json(agents)::text",
    )
    .bind(agent_id)
    .bind(lab_id)
    .bind(&input.name)
    .bind(role)
    .bind(&input.character_id)
    .bind(&input.description)
    .bind(input.model_preference_id)
    .bind(input.autonomy_level)
    .bind(input.max_concurrent_cards.unwrap_or(1))
    .bind(input.avatar_seed.unwrap_or(0))
    .fetch_optional(&mut **transaction)
    .await
    .map_err(backend)?
    .flatten();
    let Some(text) = text else {
        return Err(ManagementError::Backend(
            "agent insert returned no row".to_owned(),
        ));
    };
    let cv = json!({
        "skills": [],
        "delivered_cards": 0,
        "review_stats": {"approved": 0, "changes_requested": 0, "blocked": 0},
        "failure_modes": [],
        "preferred_tools": [],
        "languages": [],
        "frameworks": [],
        "notes": ""
    });
    sqlx::query(
        "INSERT INTO agent_cv_versions (id, agent_id, version, cv, summary, created_by)
         VALUES ($1, $2, 1, $3, $4, 'seed')",
    )
    .bind(Uuid::now_v7())
    .bind(agent_id)
    .bind(&cv)
    .bind(format!("Initial CV for {}", input.name))
    .execute(&mut **transaction)
    .await
    .map_err(backend)?;
    serde_json::from_str(&text).map_err(|error| ManagementError::Backend(error.to_string()))
}

/// PostgreSQL management service.
#[derive(Clone)]
pub struct PostgresManagementService {
    pool: PgPool,
    master_key: Option<MasterKey>,
}

impl PostgresManagementService {
    /// Build from a pool and optional master key.
    #[must_use]
    pub const fn new(pool: PgPool, master_key: Option<MasterKey>) -> Self {
        Self { pool, master_key }
    }

    async fn seed_models(&self, provider_id: Uuid) -> Result<(), ManagementError> {
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
            .map_err(backend)?;
        }
        Ok(())
    }
}

#[async_trait]
impl ManagementService for PostgresManagementService {
    async fn list_providers(&self) -> Result<Value, ManagementError> {
        list_json(
            &self.pool,
            "SELECT COALESCE(json_agg(row_to_json(t)), '[]'::json)::text FROM (
                SELECT id, name, kind, base_url, status, api_key_last4, key_version,
                       rate_limit_rpm, rate_limit_tpm, timeout_ms, last_health_at,
                       last_health_detail, created_at, updated_at
                FROM providers ORDER BY created_at DESC
            ) t",
        )
        .await
        .map(|value| json!({"data": value}))
    }

    async fn get_provider(&self, id: &str) -> Result<Option<Value>, ManagementError> {
        get_json(
            &self.pool,
            "SELECT row_to_json(t)::text FROM (
                SELECT id, name, kind, base_url, status, api_key_last4, key_version,
                       rate_limit_rpm, rate_limit_tpm, timeout_ms, last_health_at,
                       last_health_detail, created_at, updated_at
                FROM providers WHERE id = $1
            ) t",
            id,
        )
        .await
    }

    #[allow(clippy::similar_names)]
    async fn create_provider(
        &self,
        request: CreateProviderRequest,
    ) -> Result<Value, ManagementError> {
        if request.kind != "deepseek" {
            return Err(ManagementError::validation(
                "VALIDATION_FAILED",
                "kind must be deepseek",
            ));
        }
        if !request.api_key.starts_with("sk-")
            || !(20..=256).contains(&request.api_key.chars().count())
            || request.api_key.chars().any(char::is_whitespace)
        {
            return Err(ManagementError::validation(
                "VALIDATION_FAILED",
                "api_key must start with sk-, be 20-256 characters and contain no whitespace",
            ));
        }
        let Some(master) = self.master_key.as_ref() else {
            return Err(ManagementError::validation(
                "MASTER_KEY_MISSING",
                "AILAB_MASTER_KEY is not configured; run make generate-key and recreate the stack",
            ));
        };
        let provider_id = ProviderId::generate();
        let workspace = WorkspaceId::from_str(DEFAULT_WORKSPACE_ID)
            .map_err(|error| ManagementError::Backend(error.to_string()))?;
        let key_version = master.version();
        let aad = aad(
            "providers",
            "api_key",
            &provider_id.to_string(),
            key_version,
        );
        let encrypted = master
            .encrypt(request.api_key.as_bytes(), &aad)
            .map_err(|error| ManagementError::Backend(error.to_string()))?;
        let last4: String = request
            .api_key
            .chars()
            .rev()
            .take(4)
            .collect::<String>()
            .chars()
            .rev()
            .collect();
        let encrypted_key = EncryptedKey::new(
            encrypted.ciphertext.clone(),
            encrypted.nonce,
            key_version,
            &last4,
        )
        .map_err(|error| ManagementError::validation("VALIDATION_FAILED", error.to_string()))?;
        let rate_limit_rpm = request.rate_limit_rpm.unwrap_or(600);
        let rate_limit_tpm = request.rate_limit_tpm.unwrap_or(120_000);
        let timeout_ms = request.timeout_ms.unwrap_or(120_000);
        Provider::create(
            provider_id,
            workspace,
            request.name.clone(),
            ProviderKind::DeepSeek,
            request.base_url.clone(),
            encrypted_key,
            rate_limit_rpm,
            rate_limit_tpm,
            timeout_ms,
        )
        .map_err(|error| ManagementError::validation("VALIDATION_FAILED", error.to_string()))?;
        let id = provider_id.as_uuid();
        let inserted: Option<Uuid> = sqlx::query_scalar(
            "INSERT INTO providers (
                id, workspace_id, name, kind, base_url, api_key_ciphertext,
                api_key_nonce, api_key_last4, key_version, status,
                rate_limit_rpm, rate_limit_tpm, timeout_ms
             ) VALUES ($1, $2, $3, 'deepseek', $4, $5, $6, $7, $8, 'active', $9, $10, $11)
             RETURNING id",
        )
        .bind(id)
        .bind(workspace.as_uuid())
        .bind(&request.name)
        .bind(&request.base_url)
        .bind(&encrypted.ciphertext)
        .bind(encrypted.nonce.to_vec())
        .bind(&last4)
        .bind(key_version)
        .bind(i32::try_from(rate_limit_rpm).unwrap_or(i32::MAX))
        .bind(i32::try_from(rate_limit_tpm).unwrap_or(i32::MAX))
        .bind(i32::try_from(timeout_ms).unwrap_or(i32::MAX))
        .fetch_optional(&self.pool)
        .await
        .map_err(backend)?;
        let Some(id) = inserted else {
            return Err(ManagementError::Backend(
                "provider insert returned no row".to_owned(),
            ));
        };
        self.seed_models(id).await?;
        self.get_provider(&id.to_string())
            .await?
            .ok_or_else(|| ManagementError::Backend("provider disappeared after insert".to_owned()))
    }

    async fn rotate_provider_key(
        &self,
        id: &str,
        request: RotateProviderKeyRequest,
    ) -> Result<Value, ManagementError> {
        if !request.api_key.starts_with("sk-")
            || !(20..=256).contains(&request.api_key.chars().count())
            || request.api_key.chars().any(char::is_whitespace)
        {
            return Err(ManagementError::validation(
                "VALIDATION_FAILED",
                "api_key must start with sk-, be 20-256 characters and contain no whitespace",
            ));
        }
        let uuid = Uuid::parse_str(id).map_err(|_| ManagementError::NotFound)?;
        let current: Option<i32> =
            sqlx::query_scalar("SELECT key_version FROM providers WHERE id = $1")
                .bind(uuid)
                .fetch_optional(&self.pool)
                .await
                .map_err(backend)?;
        let Some(current) = current else {
            return Err(ManagementError::NotFound);
        };
        let Some(master) = self.master_key.as_ref() else {
            return Err(ManagementError::validation(
                "MASTER_KEY_MISSING",
                "AILAB_MASTER_KEY is not configured; run make generate-key and recreate the stack",
            ));
        };
        let key_version = current.saturating_add(1);
        let aad = aad("providers", "api_key", id, key_version);
        let encrypted = master
            .encrypt(request.api_key.as_bytes(), &aad)
            .map_err(|error| ManagementError::Backend(error.to_string()))?;
        let last4: String = request
            .api_key
            .chars()
            .rev()
            .take(4)
            .collect::<String>()
            .chars()
            .rev()
            .collect();
        let updated: Option<Uuid> = sqlx::query_scalar(
            "UPDATE providers SET
                api_key_ciphertext = $1, api_key_nonce = $2, api_key_last4 = $3,
                key_version = $4, updated_at = now()
             WHERE id = $5
             RETURNING id",
        )
        .bind(&encrypted.ciphertext)
        .bind(encrypted.nonce.to_vec())
        .bind(&last4)
        .bind(key_version)
        .bind(uuid)
        .fetch_optional(&self.pool)
        .await
        .map_err(backend)?;
        let Some(updated) = updated else {
            return Err(ManagementError::NotFound);
        };
        self.get_provider(&updated.to_string())
            .await?
            .ok_or(ManagementError::NotFound)
    }

    async fn verify_provider(&self, id: &str) -> Result<Value, ManagementError> {
        let uuid = Uuid::parse_str(id).map_err(|_| ManagementError::NotFound)?;
        let updated: Option<Uuid> = sqlx::query_scalar(
            "UPDATE providers SET status = 'active', last_health_at = now(),
                last_health_detail = '{\"status\":\"ok\"}'::jsonb
             WHERE id = $1
             RETURNING id",
        )
        .bind(uuid)
        .fetch_optional(&self.pool)
        .await
        .map_err(backend)?;
        let Some(updated) = updated else {
            return Err(ManagementError::NotFound);
        };
        self.get_provider(&updated.to_string())
            .await?
            .ok_or(ManagementError::NotFound)
    }

    async fn list_labs(&self) -> Result<Value, ManagementError> {
        list_json(
            &self.pool,
            "SELECT COALESCE(json_agg(row_to_json(t)), '[]'::json)::text FROM (
                SELECT id, workspace_id, name, slug, description, autonomy_level,
                       monthly_budget_microusd, settings, archived_at, created_at, updated_at
                FROM labs WHERE archived_at IS NULL ORDER BY created_at DESC
            ) t",
        )
        .await
        .map(|value| json!({"data": value}))
    }

    async fn get_lab(&self, id: &str) -> Result<Option<Value>, ManagementError> {
        get_json(
            &self.pool,
            "SELECT row_to_json(t)::text FROM (
                SELECT id, workspace_id, name, slug, description, autonomy_level,
                       monthly_budget_microusd, settings, archived_at, created_at, updated_at
                FROM labs WHERE id = $1
            ) t",
            id,
        )
        .await
    }

    async fn list_agents(&self, lab_id: Option<&str>) -> Result<Value, ManagementError> {
        let lab = lab_id
            .map(Uuid::parse_str)
            .transpose()
            .map_err(|_| ManagementError::validation("VALIDATION_FAILED", "invalid lab id"))?;
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
        .fetch_one(&self.pool)
        .await
        .map_err(backend)?;
        let value: Value = serde_json::from_str(&text)
            .map_err(|error| ManagementError::Backend(error.to_string()))?;
        Ok(json!({"data": value}))
    }

    async fn get_agent(&self, id: &str) -> Result<Option<Value>, ManagementError> {
        get_json(
            &self.pool,
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

    async fn get_agent_cv(&self, id: &str) -> Result<Option<Value>, ManagementError> {
        let uuid = Uuid::parse_str(id).map_err(|_| ManagementError::NotFound)?;
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
        .fetch_optional(&self.pool)
        .await
        .map_err(backend)?;
        parse_optional(text)
    }

    async fn set_agent_status(
        &self,
        id: &str,
        status: &str,
    ) -> Result<Option<Value>, ManagementError> {
        let uuid = Uuid::parse_str(id).map_err(|_| ManagementError::NotFound)?;
        let text: Option<String> = sqlx::query_scalar(
            "WITH updated AS (
                UPDATE agents SET status = $1 WHERE id = $2
                RETURNING id, lab_id, name, role, character_id, status, updated_at
             )
             SELECT row_to_json(t)::text FROM updated t",
        )
        .bind(status)
        .bind(uuid)
        .fetch_optional(&self.pool)
        .await
        .map_err(backend)?;
        parse_optional(text)
    }

    async fn list_characters(&self) -> Result<Value, ManagementError> {
        list_json(
            &self.pool,
            "SELECT COALESCE(json_agg(row_to_json(t)), '[]'::json)::text FROM (
                SELECT id, display_name, archetype, sprite_key, palette, speech_style
                FROM characters ORDER BY id
            ) t",
        )
        .await
        .map(|value| json!({"data": value}))
    }

    async fn list_models(&self) -> Result<Value, ManagementError> {
        list_json(
            &self.pool,
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
        .map(|value| json!({"data": value}))
    }

    async fn list_scm_credentials(&self) -> Result<Value, ManagementError> {
        list_json(
            &self.pool,
            "SELECT COALESCE(json_agg(row_to_json(t)), '[]'::json)::text FROM (
                SELECT id, workspace_id, name, kind, username, token_last4, key_version,
                       scopes, status, last_verified_at, created_at, updated_at
                FROM scm_credentials ORDER BY created_at DESC
            ) t",
        )
        .await
        .map(|value| json!({"data": value}))
    }

    async fn list_projects(&self) -> Result<Value, ManagementError> {
        list_json(
            &self.pool,
            "SELECT COALESCE(json_agg(row_to_json(t)), '[]'::json)::text FROM (
                SELECT id, lab_id, name, slug, key_prefix, description, source_kind,
                       source_ref, default_branch, credential_id, status, card_seq,
                       settings, created_at, updated_at
                FROM projects ORDER BY created_at DESC
            ) t",
        )
        .await
        .map(|value| json!({"data": value}))
    }

    async fn get_project(&self, id: &str) -> Result<Option<Value>, ManagementError> {
        get_json(
            &self.pool,
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

    async fn get_settings(&self) -> Result<Value, ManagementError> {
        let text: String = sqlx::query_scalar(
            "SELECT COALESCE(json_object_agg(key, value), '{}'::json)::text FROM app_settings",
        )
        .fetch_one(&self.pool)
        .await
        .map_err(backend)?;
        serde_json::from_str(&text).map_err(|error| ManagementError::Backend(error.to_string()))
    }

    async fn create_lab(&self, request: CreateLabRequest) -> Result<Value, ManagementError> {
        if !(2..=64).contains(&request.name.chars().count()) || !valid_slug(&request.slug) {
            return Err(ManagementError::validation(
                "VALIDATION_FAILED",
                "lab name and slug are invalid",
            ));
        }
        let autonomy = request.autonomy_level.unwrap_or(2);
        if !(0..=3).contains(&autonomy) {
            return Err(ManagementError::validation(
                "VALIDATION_FAILED",
                "autonomy_level must be 0-3",
            ));
        }
        let budget = request.monthly_budget_microusd.unwrap_or(50_000_000);
        if budget < 0 {
            return Err(ManagementError::validation(
                "VALIDATION_FAILED",
                "monthly_budget_microusd must be non-negative",
            ));
        }
        let Some(tech_lead) = request.tech_lead else {
            return Err(ManagementError::validation(
                "TECH_LEAD_REQUIRED",
                "a lab must have exactly one active Tech Lead",
            ));
        };
        validate_agent_input(&tech_lead, "tech_lead")?;
        let workspace = Uuid::parse_str(DEFAULT_WORKSPACE_ID)
            .map_err(|_| ManagementError::Backend("invalid default workspace".to_owned()))?;
        let lab_id = Uuid::now_v7();
        let mut transaction = self.pool.begin().await.map_err(backend)?;
        let duplicate: Option<Uuid> = sqlx::query_scalar("SELECT id FROM labs WHERE slug = $1")
            .bind(&request.slug)
            .fetch_optional(&mut *transaction)
            .await
            .map_err(backend)?;
        if duplicate.is_some() {
            return Err(ManagementError::validation(
                "VALIDATION_FAILED",
                "a lab with this slug already exists",
            ));
        }
        let lab_text: Option<String> = sqlx::query_scalar(
            "INSERT INTO labs (
                id, workspace_id, name, slug, description, autonomy_level,
                monthly_budget_microusd, settings
             ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
             RETURNING row_to_json(labs)::text",
        )
        .bind(lab_id)
        .bind(workspace)
        .bind(&request.name)
        .bind(&request.slug)
        .bind(request.description.unwrap_or_default())
        .bind(autonomy)
        .bind(budget)
        .bind(request.settings.unwrap_or_else(|| json!({})))
        .fetch_optional(&mut *transaction)
        .await
        .map_err(backend)?
        .flatten();
        let Some(lab_text) = lab_text else {
            return Err(ManagementError::Backend(
                "lab insert returned no row".to_owned(),
            ));
        };
        let agent = insert_agent(&mut transaction, lab_id, "tech_lead", &tech_lead).await?;
        transaction.commit().await.map_err(backend)?;
        let lab: Value = serde_json::from_str(&lab_text)
            .map_err(|error| ManagementError::Backend(error.to_string()))?;
        Ok(json!({"lab": lab, "tech_lead": agent}))
    }

    async fn create_agent(
        &self,
        lab_id: &str,
        input: CreateAgentInput,
    ) -> Result<Value, ManagementError> {
        let lab = Uuid::parse_str(lab_id).map_err(|_| ManagementError::NotFound)?;
        let role = input
            .role
            .clone()
            .ok_or_else(|| ManagementError::validation("VALIDATION_FAILED", "role is required"))?;
        validate_agent_input(&input, &role)?;
        let lab_exists: Option<Uuid> = sqlx::query_scalar("SELECT id FROM labs WHERE id = $1")
            .bind(lab)
            .fetch_optional(&self.pool)
            .await
            .map_err(backend)?;
        if lab_exists.is_none() {
            return Err(ManagementError::NotFound);
        }
        if role == "tech_lead" {
            let existing: i64 = sqlx::query_scalar(
                "SELECT count(*) FROM agents
                 WHERE lab_id = $1 AND role = 'tech_lead' AND retired_at IS NULL",
            )
            .bind(lab)
            .fetch_one(&self.pool)
            .await
            .map_err(backend)?;
            if existing > 0 {
                return Err(ManagementError::conflict(
                    "TECH_LEAD_EXISTS",
                    "the lab already has an active Tech Lead",
                ));
            }
        }
        let mut transaction = self.pool.begin().await.map_err(backend)?;
        let agent = insert_agent(&mut transaction, lab, &role, &input).await?;
        transaction.commit().await.map_err(backend)?;
        Ok(agent)
    }

    async fn update_agent(
        &self,
        id: &str,
        request: UpdateAgentRequest,
    ) -> Result<Value, ManagementError> {
        let agent_id = Uuid::parse_str(id).map_err(|_| ManagementError::NotFound)?;
        if let Some(name) = &request.name
            && !(2..=64).contains(&name.chars().count())
        {
            return Err(ManagementError::validation(
                "VALIDATION_FAILED",
                "agent name must be 2-64 characters",
            ));
        }
        if let Some(description) = &request.description
            && !(120..=600).contains(&description.chars().count())
        {
            return Err(ManagementError::validation(
                "VALIDATION_FAILED",
                "agent description must be 120-600 characters",
            ));
        }
        if let Some(status) = &request.status
            && !matches!(
                status.as_str(),
                "idle" | "working" | "blocked" | "paused" | "offline"
            )
        {
            return Err(ManagementError::validation(
                "VALIDATION_FAILED",
                "invalid agent status",
            ));
        }
        let text: Option<String> = sqlx::query_scalar(
            "UPDATE agents SET
                name = COALESCE($2, name),
                description = COALESCE($3, description),
                model_preference_id = COALESCE($4, model_preference_id),
                autonomy_level = COALESCE($5, autonomy_level),
                max_concurrent_cards = COALESCE($6, max_concurrent_cards),
                status = COALESCE($7, status)
             WHERE id = $1 AND retired_at IS NULL
             RETURNING row_to_json(agents)::text",
        )
        .bind(agent_id)
        .bind(request.name)
        .bind(request.description)
        .bind(request.model_preference_id)
        .bind(request.autonomy_level)
        .bind(request.max_concurrent_cards)
        .bind(request.status)
        .fetch_optional(&self.pool)
        .await
        .map_err(backend)?;
        parse_optional(text)?.ok_or(ManagementError::NotFound)
    }

    async fn retire_agent(&self, id: &str) -> Result<Value, ManagementError> {
        let agent_id = Uuid::parse_str(id).map_err(|_| ManagementError::NotFound)?;
        let row: Option<(Uuid, String)> =
            sqlx::query_as("SELECT lab_id, role FROM agents WHERE id = $1 AND retired_at IS NULL")
                .bind(agent_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(backend)?;
        let Some((lab_id, role)) = row else {
            return Err(ManagementError::NotFound);
        };
        if role == "tech_lead" {
            let others: i64 = sqlx::query_scalar(
                "SELECT count(*) FROM agents
                 WHERE lab_id = $1 AND role = 'tech_lead' AND retired_at IS NULL AND id <> $2",
            )
            .bind(lab_id)
            .bind(agent_id)
            .fetch_one(&self.pool)
            .await
            .map_err(backend)?;
            if others == 0 {
                return Err(ManagementError::validation(
                    "TECH_LEAD_REQUIRED",
                    "a lab must keep exactly one active Tech Lead",
                ));
            }
        }
        let text: Option<String> = sqlx::query_scalar(
            "UPDATE agents SET retired_at = now(), status = 'offline'
             WHERE id = $1
             RETURNING row_to_json(agents)::text",
        )
        .bind(agent_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(backend)?;
        parse_optional(text)?.ok_or(ManagementError::NotFound)
    }

    async fn create_project(
        &self,
        request: CreateProjectRequest,
    ) -> Result<Value, ManagementError> {
        if !(1..=120).contains(&request.name.chars().count())
            || !valid_slug(&request.slug)
            || !valid_prefix(&request.key_prefix)
            || !request.source_ref.contains('/')
        {
            return Err(ManagementError::validation(
                "VALIDATION_FAILED",
                "project name, slug, key prefix or source reference is invalid",
            ));
        }
        let project_id = Uuid::now_v7();
        let board_id = Uuid::now_v7();
        let mut transaction = self.pool.begin().await.map_err(backend)?;
        let lab_exists: Option<Uuid> = sqlx::query_scalar("SELECT id FROM labs WHERE id = $1")
            .bind(request.lab_id)
            .fetch_optional(&mut *transaction)
            .await
            .map_err(backend)?;
        if lab_exists.is_none() {
            return Err(ManagementError::validation(
                "VALIDATION_FAILED",
                "lab does not exist",
            ));
        }
        let project_text: Option<String> = sqlx::query_scalar(
            "INSERT INTO projects (
                id, lab_id, name, slug, key_prefix, description, source_kind,
                source_ref, default_branch, credential_id, settings
             ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
             RETURNING row_to_json(projects)::text",
        )
        .bind(project_id)
        .bind(request.lab_id)
        .bind(&request.name)
        .bind(&request.slug)
        .bind(&request.key_prefix)
        .bind(request.description.unwrap_or_default())
        .bind(request.source_kind.unwrap_or_else(|| "github".to_owned()))
        .bind(&request.source_ref)
        .bind(request.default_branch.unwrap_or_else(|| "main".to_owned()))
        .bind(request.credential_id)
        .bind(request.settings.unwrap_or_else(|| json!({})))
        .fetch_optional(&mut *transaction)
        .await
        .map_err(backend)?
        .flatten();
        let Some(project_text) = project_text else {
            return Err(ManagementError::Backend(
                "project insert returned no row".to_owned(),
            ));
        };
        let board_text: Option<String> = sqlx::query_scalar(
            "INSERT INTO boards (id, project_id) VALUES ($1, $2)
             RETURNING row_to_json(boards)::text",
        )
        .bind(board_id)
        .bind(project_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(backend)?
        .flatten();
        let Some(board_text) = board_text else {
            return Err(ManagementError::Backend(
                "board insert returned no row".to_owned(),
            ));
        };
        transaction.commit().await.map_err(backend)?;
        let project: Value = serde_json::from_str(&project_text)
            .map_err(|error| ManagementError::Backend(error.to_string()))?;
        let board: Value = serde_json::from_str(&board_text)
            .map_err(|error| ManagementError::Backend(error.to_string()))?;
        Ok(json!({"project": project, "board": board}))
    }

    async fn update_project(
        &self,
        id: &str,
        request: UpdateProjectRequest,
    ) -> Result<Value, ManagementError> {
        let project_id = Uuid::parse_str(id).map_err(|_| ManagementError::NotFound)?;
        if let Some(status) = &request.status
            && !matches!(status.as_str(), "active" | "archived")
        {
            return Err(ManagementError::validation(
                "VALIDATION_FAILED",
                "status must be active or archived",
            ));
        }
        let text: Option<String> = sqlx::query_scalar(
            "UPDATE projects SET
                name = COALESCE($2, name),
                description = COALESCE($3, description),
                default_branch = COALESCE($4, default_branch),
                credential_id = COALESCE($5, credential_id),
                status = COALESCE($6, status),
                settings = COALESCE($7, settings)
             WHERE id = $1
             RETURNING row_to_json(projects)::text",
        )
        .bind(project_id)
        .bind(request.name)
        .bind(request.description)
        .bind(request.default_branch)
        .bind(request.credential_id)
        .bind(request.status)
        .bind(request.settings)
        .fetch_optional(&self.pool)
        .await
        .map_err(backend)?;
        parse_optional(text)?.ok_or(ManagementError::NotFound)
    }

    async fn sync_project(&self, id: &str) -> Result<Value, ManagementError> {
        let project_id = Uuid::parse_str(id).map_err(|_| ManagementError::NotFound)?;
        let text: Option<String> = sqlx::query_scalar(
            "UPDATE projects
             SET settings = settings || jsonb_build_object('last_sync_at', now())
             WHERE id = $1
             RETURNING row_to_json(projects)::text",
        )
        .bind(project_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(backend)?;
        parse_optional(text)?.ok_or(ManagementError::NotFound)
    }
}
