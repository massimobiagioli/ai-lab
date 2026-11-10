//! Management application service port and request DTOs.

use std::fmt;

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

/// Management operation failure.
#[derive(Debug)]
pub enum ManagementError {
    /// A field or invariant failed.
    Validation {
        /// Stable code.
        code: &'static str,
        /// Human-readable detail.
        message: String,
    },
    /// A unique or conflicting state.
    Conflict {
        /// Stable code.
        code: &'static str,
        /// Human-readable detail.
        message: String,
    },
    /// The resource does not exist.
    NotFound,
    /// The adapter failed.
    Backend(String),
}

impl ManagementError {
    /// Build a validation error.
    pub fn validation(code: &'static str, message: impl Into<String>) -> Self {
        Self::Validation {
            code,
            message: message.into(),
        }
    }

    /// Build a conflict error.
    pub fn conflict(code: &'static str, message: impl Into<String>) -> Self {
        Self::Conflict {
            code,
            message: message.into(),
        }
    }
}

impl fmt::Display for ManagementError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Validation { code, message } | Self::Conflict { code, message } => {
                write!(formatter, "{code}: {message}")
            }
            Self::NotFound => formatter.write_str("not found"),
            Self::Backend(message) => write!(formatter, "backend failed: {message}"),
        }
    }
}

impl std::error::Error for ManagementError {}

/// Provider create request.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateProviderRequest {
    /// Display name.
    pub name: String,
    /// Provider kind (`deepseek`).
    pub kind: String,
    /// HTTPS base URL.
    pub base_url: String,
    /// Plaintext API key.
    pub api_key: String,
    /// Requests per minute.
    #[serde(default)]
    pub rate_limit_rpm: Option<u32>,
    /// Tokens per minute.
    #[serde(default)]
    pub rate_limit_tpm: Option<u32>,
    /// Request timeout.
    #[serde(default)]
    pub timeout_ms: Option<u32>,
}

/// Provider key rotation request.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RotateProviderKeyRequest {
    /// Plaintext API key.
    pub api_key: String,
}

/// Agent fields accepted on create.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateAgentInput {
    /// Display name.
    pub name: String,
    /// Role, required when creating an agent directly.
    #[serde(default)]
    pub role: Option<String>,
    /// Character id.
    pub character_id: String,
    /// 120-600 character description.
    pub description: String,
    /// Preferred model.
    #[serde(default)]
    pub model_preference_id: Option<Uuid>,
    /// Autonomy override.
    #[serde(default)]
    pub autonomy_level: Option<i16>,
    /// Concurrent card capacity.
    #[serde(default)]
    pub max_concurrent_cards: Option<i16>,
    /// Avatar seed.
    #[serde(default)]
    pub avatar_seed: Option<i64>,
}

/// Create-lab request.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateLabRequest {
    /// Display name.
    pub name: String,
    /// URL-safe slug.
    pub slug: String,
    /// Optional description.
    #[serde(default)]
    pub description: Option<String>,
    /// Autonomy level 0-3.
    #[serde(default)]
    pub autonomy_level: Option<i16>,
    /// Monthly budget in micro-USD.
    #[serde(default)]
    pub monthly_budget_microusd: Option<i64>,
    /// Lab settings.
    #[serde(default)]
    pub settings: Option<Value>,
    /// Mandatory Tech Lead.
    #[serde(default)]
    pub tech_lead: Option<CreateAgentInput>,
}

/// Create-project request.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateProjectRequest {
    /// Owning lab.
    pub lab_id: Uuid,
    /// Display name.
    pub name: String,
    /// URL-safe slug.
    pub slug: String,
    /// Card key prefix.
    pub key_prefix: String,
    /// Optional description.
    #[serde(default)]
    pub description: Option<String>,
    /// Source kind, `github`.
    #[serde(default)]
    pub source_kind: Option<String>,
    /// `owner/repo`.
    pub source_ref: String,
    /// Default branch.
    #[serde(default)]
    pub default_branch: Option<String>,
    /// SCM credential.
    #[serde(default)]
    pub credential_id: Option<Uuid>,
    /// Project settings.
    #[serde(default)]
    pub settings: Option<Value>,
}

/// Update-agent request.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateAgentRequest {
    /// Display name.
    #[serde(default)]
    pub name: Option<String>,
    /// Description.
    #[serde(default)]
    pub description: Option<String>,
    /// Preferred model.
    #[serde(default)]
    pub model_preference_id: Option<Uuid>,
    /// Autonomy override.
    #[serde(default)]
    pub autonomy_level: Option<i16>,
    /// Concurrent card capacity.
    #[serde(default)]
    pub max_concurrent_cards: Option<i16>,
    /// Status.
    #[serde(default)]
    pub status: Option<String>,
}

/// Update-project request.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateProjectRequest {
    /// Display name.
    #[serde(default)]
    pub name: Option<String>,
    /// Description.
    #[serde(default)]
    pub description: Option<String>,
    /// Default branch.
    #[serde(default)]
    pub default_branch: Option<String>,
    /// SCM credential.
    #[serde(default)]
    pub credential_id: Option<Uuid>,
    /// Status.
    #[serde(default)]
    pub status: Option<String>,
    /// Settings.
    #[serde(default)]
    pub settings: Option<Value>,
}

/// Management application service.
#[async_trait]
pub trait ManagementService: Send + Sync {
    /// List providers.
    async fn list_providers(&self) -> Result<Value, ManagementError>;
    /// One provider.
    async fn get_provider(&self, id: &str) -> Result<Option<Value>, ManagementError>;
    /// Create a provider.
    async fn create_provider(
        &self,
        request: CreateProviderRequest,
    ) -> Result<Value, ManagementError>;
    /// Rotate a provider key.
    async fn rotate_provider_key(
        &self,
        id: &str,
        request: RotateProviderKeyRequest,
    ) -> Result<Value, ManagementError>;
    /// Verify a provider.
    async fn verify_provider(&self, id: &str) -> Result<Value, ManagementError>;

    /// List labs.
    async fn list_labs(&self) -> Result<Value, ManagementError>;
    /// One lab.
    async fn get_lab(&self, id: &str) -> Result<Option<Value>, ManagementError>;
    /// List agents.
    async fn list_agents(&self, lab_id: Option<&str>) -> Result<Value, ManagementError>;
    /// One agent.
    async fn get_agent(&self, id: &str) -> Result<Option<Value>, ManagementError>;
    /// Agent CV.
    async fn get_agent_cv(&self, id: &str) -> Result<Option<Value>, ManagementError>;
    /// Pause or resume an agent.
    async fn set_agent_status(
        &self,
        id: &str,
        status: &str,
    ) -> Result<Option<Value>, ManagementError>;
    /// Character roster.
    async fn list_characters(&self) -> Result<Value, ManagementError>;
    /// Model catalog.
    async fn list_models(&self) -> Result<Value, ManagementError>;
    /// SCM credentials.
    async fn list_scm_credentials(&self) -> Result<Value, ManagementError>;
    /// Projects.
    async fn list_projects(&self) -> Result<Value, ManagementError>;
    /// One project.
    async fn get_project(&self, id: &str) -> Result<Option<Value>, ManagementError>;
    /// Settings.
    async fn get_settings(&self) -> Result<Value, ManagementError>;

    /// Create a lab and its Tech Lead.
    async fn create_lab(&self, request: CreateLabRequest) -> Result<Value, ManagementError>;
    /// Create an agent.
    async fn create_agent(
        &self,
        lab_id: &str,
        input: CreateAgentInput,
    ) -> Result<Value, ManagementError>;
    /// Update an agent.
    async fn update_agent(
        &self,
        id: &str,
        request: UpdateAgentRequest,
    ) -> Result<Value, ManagementError>;
    /// Retire an agent.
    async fn retire_agent(&self, id: &str) -> Result<Value, ManagementError>;
    /// Create a project and its board.
    async fn create_project(&self, request: CreateProjectRequest)
    -> Result<Value, ManagementError>;
    /// Update a project.
    async fn update_project(
        &self,
        id: &str,
        request: UpdateProjectRequest,
    ) -> Result<Value, ManagementError>;
    /// Sync a project.
    async fn sync_project(&self, id: &str) -> Result<Value, ManagementError>;
}
