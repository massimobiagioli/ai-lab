//! Inventory context: providers, models, SCM credentials, agents, CVs and projects.

use chrono::{DateTime, Utc};
use serde_json::json;

use crate::aggregate::Aggregate;
use crate::error::DomainError;
use crate::event::DomainEvent;
use crate::ids::{
    AgentId, CardKey, LabId, ModelId, ProjectId, ProviderId, ScmCredentialId, WorkspaceId,
};
use crate::value::{Actor, AgentRole, AutonomyLevel, Slug, Version};

/// Supported LLM provider kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderKind {
    /// `DeepSeek` is the only v1 provider.
    DeepSeek,
}

/// Provider lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderStatus {
    /// Accepting requests.
    Active,
    /// Manually disabled.
    Disabled,
    /// Health probe failed.
    Degraded,
}

/// Encrypted secret material with only the last four characters readable.
#[derive(Clone, PartialEq, Eq)]
pub struct EncryptedKey {
    ciphertext: Vec<u8>,
    nonce: [u8; 12],
    key_version: i32,
    last4: String,
}

impl EncryptedKey {
    /// Build an encrypted key blob.
    pub fn new(
        ciphertext: Vec<u8>,
        nonce: [u8; 12],
        key_version: i32,
        last4: &str,
    ) -> Result<Self, DomainError> {
        if ciphertext.is_empty() {
            return Err(DomainError::validation("ciphertext", "must not be empty"));
        }
        if key_version < 1 {
            return Err(DomainError::validation("key_version", "must be at least 1"));
        }
        if last4.chars().count() > 4 {
            return Err(DomainError::validation("last4", "at most 4 characters"));
        }
        Ok(Self {
            ciphertext,
            nonce,
            key_version,
            last4: last4.to_owned(),
        })
    }

    /// Last four characters, safe for display.
    #[must_use]
    pub fn last4(&self) -> &str {
        &self.last4
    }

    /// Key version for envelope rotation.
    #[must_use]
    pub const fn key_version(&self) -> i32 {
        self.key_version
    }

    /// Ciphertext bytes (adapters only).
    #[must_use]
    pub fn ciphertext(&self) -> &[u8] {
        &self.ciphertext
    }

    /// Nonce bytes.
    #[must_use]
    pub const fn nonce(&self) -> &[u8; 12] {
        &self.nonce
    }
}

impl std::fmt::Debug for EncryptedKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EncryptedKey")
            .field("last4", &self.last4)
            .field("key_version", &self.key_version)
            .finish_non_exhaustive()
    }
}

/// LLM provider aggregate.
#[derive(Debug, Clone)]
pub struct Provider {
    id: ProviderId,
    workspace_id: WorkspaceId,
    name: String,
    kind: ProviderKind,
    base_url: String,
    key: EncryptedKey,
    status: ProviderStatus,
    rate_limit_rpm: u32,
    rate_limit_tpm: u32,
    timeout_ms: u32,
    version: Version,
}

/// Hosts a v1 provider may reach. Adding a provider kind requires an ADR and a
/// new entry here plus an adapter.
const ALLOWED_PROVIDER_HOSTS: &[&str] = &["api.deepseek.com"];

fn provider_host(url: &str) -> Option<&str> {
    let rest = url.strip_prefix("https://")?;
    let authority = rest.split(['/', '?', '#']).next()?;
    if authority.is_empty() || authority.contains('@') {
        return None;
    }
    let host = authority.split(':').next()?;
    if host.is_empty() {
        return None;
    }
    Some(host)
}

impl Provider {
    /// Create a provider with a validated name, HTTPS base URL and key blob.
    #[allow(clippy::too_many_arguments, clippy::similar_names)]
    pub fn create(
        id: ProviderId,
        workspace_id: WorkspaceId,
        name: impl Into<String>,
        kind: ProviderKind,
        base_url: impl Into<String>,
        key: EncryptedKey,
        rate_limit_rpm: u32,
        rate_limit_tpm: u32,
        timeout_ms: u32,
    ) -> Result<Self, DomainError> {
        let name = name.into();
        if !(2..=64).contains(&name.chars().count()) {
            return Err(DomainError::validation(
                "provider.name",
                "must be 2-64 characters",
            ));
        }
        let base_url = base_url.into();
        if !base_url.starts_with("https://") {
            return Err(DomainError::validation(
                "provider.base_url",
                "must use HTTPS and an allowlisted host",
            ));
        }
        let host = provider_host(&base_url).ok_or_else(|| {
            DomainError::validation("provider.base_url", "must be a valid HTTPS URL")
        })?;
        if !ALLOWED_PROVIDER_HOSTS.contains(&host) {
            return Err(DomainError::validation(
                "provider.base_url",
                "must be on the provider allowlist",
            ));
        }
        if rate_limit_rpm == 0 || rate_limit_tpm == 0 {
            return Err(DomainError::validation(
                "provider.limits",
                "rate limits must be positive",
            ));
        }
        if !(1000..=600_000).contains(&timeout_ms) {
            return Err(DomainError::validation(
                "provider.timeout_ms",
                "must be 1000-600000",
            ));
        }
        Ok(Self {
            id,
            workspace_id,
            name,
            kind,
            base_url,
            key,
            status: ProviderStatus::Active,
            rate_limit_rpm,
            rate_limit_tpm,
            timeout_ms,
            version: Version::ONE,
        })
    }

    /// Identity.
    #[must_use]
    pub const fn id(&self) -> ProviderId {
        self.id
    }

    /// Workspace scope.
    #[must_use]
    pub const fn workspace_id(&self) -> WorkspaceId {
        self.workspace_id
    }

    /// Display name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// HTTPS base URL.
    #[must_use]
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Requests-per-minute limit.
    #[must_use]
    pub const fn rate_limit_rpm(&self) -> u32 {
        self.rate_limit_rpm
    }

    /// Tokens-per-minute limit.
    #[must_use]
    pub const fn rate_limit_tpm(&self) -> u32 {
        self.rate_limit_tpm
    }

    /// Request timeout in milliseconds.
    #[must_use]
    pub const fn timeout_ms(&self) -> u32 {
        self.timeout_ms
    }

    /// Provider kind.
    #[must_use]
    pub const fn kind(&self) -> ProviderKind {
        self.kind
    }

    /// Only the last four characters are readable.
    #[must_use]
    pub fn last4(&self) -> &str {
        self.key.last4()
    }

    /// Current key version.
    #[must_use]
    pub const fn key_version(&self) -> i32 {
        self.key.key_version()
    }

    /// Current status.
    #[must_use]
    pub const fn status(&self) -> ProviderStatus {
        self.status
    }

    /// Current version.
    #[must_use]
    pub const fn version(&self) -> Version {
        self.version
    }

    /// Disable the provider.
    pub fn disable(&mut self) -> Result<(), DomainError> {
        if self.status == ProviderStatus::Disabled {
            return Err(DomainError::policy(
                "PROVIDER_ALREADY_DISABLED",
                "provider is already disabled",
            ));
        }
        self.status = ProviderStatus::Disabled;
        self.version = self.version.next()?;
        Ok(())
    }

    /// Re-enable the provider.
    pub fn enable(&mut self) -> Result<(), DomainError> {
        self.status = ProviderStatus::Active;
        self.version = self.version.next()?;
        Ok(())
    }

    /// Mark the provider degraded after a failed health probe.
    pub fn mark_degraded(&mut self) -> Result<(), DomainError> {
        self.status = ProviderStatus::Degraded;
        self.version = self.version.next()?;
        Ok(())
    }

    /// Rotate the encrypted key.
    pub fn rotate_key(&mut self, key: EncryptedKey) -> Result<(), DomainError> {
        self.key = key;
        self.version = self.version.next()?;
        Ok(())
    }
}

/// Model lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelStatus {
    /// Available.
    Active,
    /// Disabled.
    Disabled,
}

/// Model catalog aggregate.
#[derive(Debug, Clone)]
pub struct Model {
    id: ModelId,
    provider_id: ProviderId,
    name: String,
    context_window: u32,
    max_output_tokens: u32,
    input_cost_microusd_per_mtok: i64,
    output_cost_microusd_per_mtok: i64,
    is_default: bool,
    is_escalation: bool,
    status: ModelStatus,
}

impl Model {
    /// Create a model with non-negative costs and positive windows.
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        id: ModelId,
        provider_id: ProviderId,
        name: impl Into<String>,
        context_window: u32,
        max_output_tokens: u32,
        input_cost_microusd_per_mtok: i64,
        output_cost_microusd_per_mtok: i64,
        is_default: bool,
        is_escalation: bool,
    ) -> Result<Self, DomainError> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(DomainError::validation("model.name", "must not be empty"));
        }
        if is_default && is_escalation {
            return Err(DomainError::validation(
                "model.role",
                "a model cannot be both default and escalation",
            ));
        }
        if context_window == 0 || max_output_tokens == 0 {
            return Err(DomainError::validation(
                "model.window",
                "context and output windows must be positive",
            ));
        }
        if input_cost_microusd_per_mtok < 0 || output_cost_microusd_per_mtok < 0 {
            return Err(DomainError::validation(
                "model.cost",
                "costs must not be negative",
            ));
        }
        Ok(Self {
            id,
            provider_id,
            name,
            context_window,
            max_output_tokens,
            input_cost_microusd_per_mtok,
            output_cost_microusd_per_mtok,
            is_default,
            is_escalation,
            status: ModelStatus::Active,
        })
    }

    /// The two v1 models seeded with every provider.
    pub fn default_models(provider_id: ProviderId) -> Result<Vec<Self>, DomainError> {
        Ok(vec![
            Self::create(
                ModelId::generate(),
                provider_id,
                "deepseek-flash",
                131_072,
                8_192,
                0,
                0,
                true,
                false,
            )?,
            Self::create(
                ModelId::generate(),
                provider_id,
                "deepseek-v4-pro",
                131_072,
                16_384,
                0,
                0,
                false,
                true,
            )?,
        ])
    }

    /// Validate a provider's catalog: exactly one default and at most one
    /// escalation model.
    pub fn validate_catalog(models: &[Self]) -> Result<(), DomainError> {
        let defaults = models.iter().filter(|model| model.is_default).count();
        let escalations = models.iter().filter(|model| model.is_escalation).count();
        if defaults != 1 {
            return Err(DomainError::policy(
                "MODEL_DEFAULT_REQUIRED",
                "a provider must have exactly one default model",
            ));
        }
        if escalations > 1 {
            return Err(DomainError::policy(
                "MODEL_ESCALATION_UNIQUE",
                "a provider can have at most one escalation model",
            ));
        }
        Ok(())
    }

    /// Identity.
    #[must_use]
    pub const fn id(&self) -> ModelId {
        self.id
    }

    /// Model name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Context window in tokens.
    #[must_use]
    pub const fn context_window(&self) -> u32 {
        self.context_window
    }

    /// Maximum output tokens.
    #[must_use]
    pub const fn max_output_tokens(&self) -> u32 {
        self.max_output_tokens
    }

    /// Input cost per million tokens in micro-USD.
    #[must_use]
    pub const fn input_cost_microusd_per_mtok(&self) -> i64 {
        self.input_cost_microusd_per_mtok
    }

    /// Output cost per million tokens in micro-USD.
    #[must_use]
    pub const fn output_cost_microusd_per_mtok(&self) -> i64 {
        self.output_cost_microusd_per_mtok
    }

    /// Model status.
    #[must_use]
    pub const fn status(&self) -> ModelStatus {
        self.status
    }

    /// True for the default model.
    #[must_use]
    pub const fn is_default(&self) -> bool {
        self.is_default
    }

    /// True when the model is the escalation target.
    #[must_use]
    pub const fn is_escalation(&self) -> bool {
        self.is_escalation
    }

    /// Provider reference.
    #[must_use]
    pub const fn provider_id(&self) -> ProviderId {
        self.provider_id
    }
}

/// SCM provider kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScmKind {
    /// GitHub is the only v1 SCM.
    GitHub,
}

/// SCM credential aggregate with a write-only token.
#[derive(Debug, Clone)]
pub struct ScmCredential {
    id: ScmCredentialId,
    workspace_id: WorkspaceId,
    name: String,
    kind: ScmKind,
    token: EncryptedKey,
    verified: bool,
}

impl ScmCredential {
    /// Create a credential.
    pub fn create(
        id: ScmCredentialId,
        workspace_id: WorkspaceId,
        name: impl Into<String>,
        kind: ScmKind,
        token: EncryptedKey,
    ) -> Result<Self, DomainError> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(DomainError::validation("scm.name", "must not be empty"));
        }
        Ok(Self {
            id,
            workspace_id,
            name,
            kind,
            token,
            verified: false,
        })
    }

    /// Identity.
    #[must_use]
    pub const fn id(&self) -> ScmCredentialId {
        self.id
    }

    /// Display name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// SCM kind.
    #[must_use]
    pub const fn kind(&self) -> ScmKind {
        self.kind
    }

    /// Only the last four characters are readable.
    #[must_use]
    pub fn last4(&self) -> &str {
        self.token.last4()
    }

    /// Mark the credential verified after a successful probe.
    #[allow(clippy::missing_const_for_fn)]
    pub fn verify(&mut self) {
        self.verified = true;
    }

    /// True after a successful probe.
    #[must_use]
    pub const fn is_verified(&self) -> bool {
        self.verified
    }

    /// Workspace scope.
    #[must_use]
    pub const fn workspace_id(&self) -> WorkspaceId {
        self.workspace_id
    }
}

/// Agent lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentStatus {
    /// Available.
    Idle,
    /// Working.
    Working,
    /// Blocked.
    Blocked,
    /// Paused.
    Paused,
    /// Offline.
    Offline,
}

/// Persistent agent aggregate.
#[derive(Debug, Clone)]
pub struct Agent {
    id: AgentId,
    lab_id: LabId,
    name: String,
    role: AgentRole,
    character_id: String,
    description: String,
    autonomy: AutonomyLevel,
    status: AgentStatus,
    max_concurrent_cards: u8,
    cv_version: u32,
    retired_at: Option<DateTime<Utc>>,
    version: Version,
    events: Vec<DomainEvent>,
}

impl Agent {
    /// Create an agent, validating the description length and card capacity.
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        id: AgentId,
        lab_id: LabId,
        name: impl Into<String>,
        role: AgentRole,
        character_id: impl Into<String>,
        description: impl Into<String>,
        autonomy: AutonomyLevel,
        max_concurrent_cards: u8,
        occurred_at: DateTime<Utc>,
    ) -> Result<Self, DomainError> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(DomainError::validation("agent.name", "must not be empty"));
        }
        let description = description.into();
        let chars = description.chars().count();
        if !(120..=600).contains(&chars) {
            return Err(DomainError::validation(
                "agent.description",
                "must be 120-600 characters",
            ));
        }
        let character_id = character_id.into();
        if character_id.trim().is_empty() {
            return Err(DomainError::validation(
                "agent.character_id",
                "must not be empty",
            ));
        }
        if !(1..=4).contains(&max_concurrent_cards) {
            return Err(DomainError::validation(
                "agent.max_concurrent_cards",
                "must be 1-4",
            ));
        }
        let event = DomainEvent::new(
            "inventory.agent.created",
            lab_id,
            Actor::system(),
            json!({"agent_id": id.to_string(), "role": role.as_str(), "character_id": character_id}),
            occurred_at,
        )?;
        Ok(Self {
            id,
            lab_id,
            name,
            role,
            character_id,
            description,
            autonomy,
            status: AgentStatus::Idle,
            max_concurrent_cards,
            cv_version: 1,
            retired_at: None,
            version: Version::ONE,
            events: vec![event],
        })
    }

    /// Deterministic, always-valid description used when the LLM is unavailable.
    ///
    /// The result is within the 120–600 character invariant so agent creation
    /// never blocks on the description generator (WP-07 T07, AC4).
    #[must_use]
    pub fn template_description(role: AgentRole, character_id: &str) -> String {
        format!(
            "The {role} works as a {character} for this lab and follows the role charter. It keeps work inside its assigned cards, verifies with tests before claiming completion, records evidence for every acceptance criterion, respects the tool policy and separation of duties, and escalates ambiguity to the Tech Lead instead of guessing.",
            role = role.as_str(),
            character = character_id,
        )
    }

    /// Identity.
    #[must_use]
    pub const fn id(&self) -> AgentId {
        self.id
    }

    /// Lab scope.
    #[must_use]
    pub const fn lab_id(&self) -> LabId {
        self.lab_id
    }

    /// Display name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Character slug.
    #[must_use]
    pub fn character_id(&self) -> &str {
        &self.character_id
    }

    /// Capability description.
    #[must_use]
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Autonomy level.
    #[must_use]
    pub const fn autonomy(&self) -> AutonomyLevel {
        self.autonomy
    }

    /// Maximum concurrent cards.
    #[must_use]
    pub const fn max_concurrent_cards(&self) -> u8 {
        self.max_concurrent_cards
    }

    /// Role.
    #[must_use]
    pub const fn role(&self) -> AgentRole {
        self.role
    }

    /// Current lifecycle status.
    #[must_use]
    pub const fn status(&self) -> AgentStatus {
        self.status
    }

    /// Current CV version.
    #[must_use]
    pub const fn cv_version(&self) -> u32 {
        self.cv_version
    }

    /// True when the agent can receive new cards.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn can_receive_cards(&self) -> bool {
        self.retired_at.is_none() && self.status != AgentStatus::Paused
    }

    /// Pause the agent.
    #[allow(clippy::missing_const_for_fn)]
    pub fn pause(&mut self) {
        self.status = AgentStatus::Paused;
    }

    /// Resume the agent.
    #[allow(clippy::missing_const_for_fn)]
    pub fn resume(&mut self) {
        self.status = AgentStatus::Idle;
    }

    /// Retire the agent; the CV and history are preserved.
    pub fn retire(&mut self, at: DateTime<Utc>) -> Result<(), DomainError> {
        if self.retired_at.is_some() {
            return Err(DomainError::policy(
                "AGENT_ALREADY_RETIRED",
                "agent is already retired",
            ));
        }
        self.retired_at = Some(at);
        self.status = AgentStatus::Offline;
        self.version = self.version.next()?;
        Ok(())
    }

    /// Append an immutable CV version.
    pub fn append_cv(
        &mut self,
        summary: impl Into<String>,
        at: DateTime<Utc>,
    ) -> Result<(), DomainError> {
        let summary = summary.into();
        if summary.trim().is_empty() {
            return Err(DomainError::validation("cv.summary", "must not be empty"));
        }
        self.cv_version += 1;
        let event = DomainEvent::new(
            "inventory.agent.cv_updated",
            self.lab_id,
            Actor::agent(self.id),
            json!({"agent_id": self.id.to_string(), "version": self.cv_version, "summary": summary}),
            at,
        )?;
        self.events.push(event);
        Ok(())
    }

    /// Validate the team: exactly one active Tech Lead per lab.
    pub fn validate_team(agents: &[Self]) -> Result<(), DomainError> {
        let leads = agents
            .iter()
            .filter(|agent| agent.role == AgentRole::TechLead && agent.retired_at.is_none())
            .count();
        match leads {
            1 => Ok(()),
            0 => Err(DomainError::policy(
                "TECH_LEAD_REQUIRED",
                "a lab requires exactly one active Tech Lead",
            )),
            _ => Err(DomainError::policy(
                "TECH_LEAD_EXISTS",
                "a lab can have only one active Tech Lead",
            )),
        }
    }
}

impl Aggregate for Agent {
    type Id = AgentId;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn version(&self) -> Version {
        self.version
    }

    fn uncommitted_events(&self) -> &[DomainEvent] {
        &self.events
    }

    fn mark_events_committed(&mut self) {
        self.events.clear();
    }
}

/// Project lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectStatus {
    /// Active.
    Active,
    /// Archived.
    Archived,
}

/// Project aggregate.
#[derive(Debug, Clone)]
pub struct Project {
    id: ProjectId,
    lab_id: LabId,
    name: String,
    slug: Slug,
    key_prefix: String,
    source_ref: String,
    default_branch: String,
    status: ProjectStatus,
    card_seq: i64,
    version: Version,
    events: Vec<DomainEvent>,
}

impl Project {
    /// Create a GitHub-connected project.
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        id: ProjectId,
        lab_id: LabId,
        name: impl Into<String>,
        slug: impl AsRef<str>,
        key_prefix: impl AsRef<str>,
        source_ref: impl Into<String>,
        default_branch: impl Into<String>,
        occurred_at: DateTime<Utc>,
    ) -> Result<Self, DomainError> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(DomainError::validation("project.name", "must not be empty"));
        }
        let slug = Slug::new(slug.as_ref())?;
        let key_prefix = key_prefix.as_ref().to_owned();
        CardKey::new(&key_prefix, 1)?;
        let source_ref = source_ref.into();
        if !source_ref.contains('/') {
            return Err(DomainError::validation(
                "project.source_ref",
                "expected owner/repo",
            ));
        }
        let default_branch = default_branch.into();
        if default_branch.trim().is_empty() {
            return Err(DomainError::validation(
                "project.default_branch",
                "must not be empty",
            ));
        }
        let event = DomainEvent::new(
            "inventory.project.connected",
            lab_id,
            Actor::system(),
            json!({"project_id": id.to_string(), "source_ref": source_ref, "default_branch": default_branch}),
            occurred_at,
        )?;
        Ok(Self {
            id,
            lab_id,
            name,
            slug,
            key_prefix,
            source_ref,
            default_branch,
            status: ProjectStatus::Active,
            card_seq: 0,
            version: Version::ONE,
            events: vec![event],
        })
    }

    /// Allocate the next card key.
    pub fn next_card_seq(&mut self) -> Result<CardKey, DomainError> {
        if self.status != ProjectStatus::Active {
            return Err(DomainError::policy(
                "PROJECT_ARCHIVED",
                "archived projects cannot allocate card keys",
            ));
        }
        self.card_seq = self
            .card_seq
            .checked_add(1)
            .ok_or_else(|| DomainError::validation("project.card_seq", "overflow"))?;
        self.version = self.version.next()?;
        CardKey::new(&self.key_prefix, self.card_seq)
    }

    /// Archive the project.
    pub fn archive(&mut self) -> Result<(), DomainError> {
        self.status = ProjectStatus::Archived;
        self.version = self.version.next()?;
        Ok(())
    }

    /// Identity.
    #[must_use]
    pub const fn id(&self) -> ProjectId {
        self.id
    }

    /// Lab scope.
    #[must_use]
    pub const fn lab_id(&self) -> LabId {
        self.lab_id
    }

    /// Display name.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Default branch.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn default_branch(&self) -> &str {
        &self.default_branch
    }

    /// Slug.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn slug(&self) -> &Slug {
        &self.slug
    }

    /// GitHub `owner/repo`.
    #[must_use]
    pub fn source_ref(&self) -> &str {
        &self.source_ref
    }
}

impl Aggregate for Project {
    type Id = ProjectId;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn version(&self) -> Version {
        self.version
    }

    fn uncommitted_events(&self) -> &[DomainEvent] {
        &self.events
    }

    fn mark_events_committed(&mut self) {
        self.events.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn key() -> EncryptedKey {
        EncryptedKey::new(vec![1, 2, 3], [0; 12], 1, "abcd").expect("valid key")
    }

    fn description() -> String {
        "A sufficiently long fixture description used to exercise the 120 to 600 character invariant in the inventory domain tests.".to_owned()
    }

    #[test]
    fn encrypted_key_debug_redacts_ciphertext() {
        let text = format!("{:?}", key());
        assert!(!text.contains("[1, 2, 3]"));
        assert!(text.contains("abcd"));
    }

    #[test]
    fn provider_lifecycle_and_validation() {
        let mut provider = Provider::create(
            ProviderId::generate(),
            WorkspaceId::generate(),
            "DeepSeek",
            ProviderKind::DeepSeek,
            "https://api.deepseek.com",
            key(),
            60,
            120_000,
            5_000,
        )
        .expect("valid provider");
        assert_eq!(provider.status(), ProviderStatus::Active);
        assert_eq!(provider.last4(), "abcd");
        provider.disable().expect("disable");
        assert_eq!(provider.status(), ProviderStatus::Disabled);
        assert!(provider.disable().is_err());
        provider.enable().expect("enable");
        provider
            .rotate_key(EncryptedKey::new(vec![9], [0; 12], 2, "wxyz").expect("key"))
            .expect("rotate");
        assert_eq!(provider.key_version(), 2);
        assert!(
            Provider::create(
                ProviderId::generate(),
                WorkspaceId::generate(),
                "Bad",
                ProviderKind::DeepSeek,
                "http://insecure.example",
                key(),
                60,
                120_000,
                5_000
            )
            .is_err()
        );
    }

    #[test]
    fn provider_rejects_non_allowlisted_and_invalid_fields() {
        let workspace = WorkspaceId::generate();
        let cases = [
            ("https://169.254.169.254/latest", 60, 120_000, 5_000),
            ("https://api.deepseek.com.evil.test", 60, 120_000, 5_000),
            ("https://user@api.deepseek.com", 60, 120_000, 5_000),
            ("https://api.deepseek.com", 0, 120_000, 5_000),
            ("https://api.deepseek.com", 60, 0, 5_000),
            ("https://api.deepseek.com", 60, 120_000, 500),
        ];
        for (url, rpm, tpm, timeout) in cases {
            assert!(
                Provider::create(
                    ProviderId::generate(),
                    workspace,
                    "DeepSeek",
                    ProviderKind::DeepSeek,
                    url,
                    key(),
                    rpm,
                    tpm,
                    timeout,
                )
                .is_err(),
                "expected rejection for {url} {rpm} {tpm} {timeout}"
            );
        }
        assert!(
            Provider::create(
                ProviderId::generate(),
                workspace,
                "x",
                ProviderKind::DeepSeek,
                "https://api.deepseek.com",
                key(),
                60,
                120_000,
                5_000,
            )
            .is_err()
        );
    }

    #[test]
    fn default_models_are_seeded() {
        let provider = ProviderId::generate();
        let models = Model::default_models(provider).expect("models");
        assert_eq!(models.len(), 2);
        assert!(models[0].is_default());
        assert!(!models[0].is_escalation());
        assert_eq!(models[0].name(), "deepseek-flash");
        assert!(!models[1].is_default());
        assert!(models[1].is_escalation());
        assert_eq!(models[1].name(), "deepseek-v4-pro");
        assert_eq!(models[0].provider_id(), provider);
        Model::validate_catalog(&models).expect("one default, one escalation");
    }

    #[test]
    fn model_catalog_rejects_duplicate_roles() {
        let provider = ProviderId::generate();
        let mut models = Model::default_models(provider).expect("models");
        models[1] = Model::create(
            ModelId::generate(),
            provider,
            "deepseek-v4-pro",
            131_072,
            16_384,
            0,
            0,
            true,
            false,
        )
        .expect("second default");
        assert!(Model::validate_catalog(&models).is_err());
        assert!(
            Model::create(
                ModelId::generate(),
                provider,
                "both",
                131_072,
                16_384,
                0,
                0,
                true,
                true,
            )
            .is_err()
        );
    }

    #[test]
    fn template_description_is_always_valid() {
        let description = Agent::template_description(AgentRole::Dev, "scrappy_dev");
        let length = description.chars().count();
        assert!((120..=600).contains(&length), "length {length}");
        let agent = Agent::create(
            AgentId::generate(),
            LabId::generate(),
            "Nico",
            AgentRole::Dev,
            "scrappy_dev",
            Agent::template_description(AgentRole::Dev, "scrappy_dev"),
            AutonomyLevel::A2,
            1,
            chrono::Utc::now(),
        )
        .expect("template description satisfies the invariant");
        assert_eq!(agent.cv_version(), 1);
    }

    #[test]
    fn agent_lifecycle_cv_and_events() {
        let mut agent = Agent::create(
            AgentId::generate(),
            LabId::generate(),
            "Nico",
            AgentRole::Dev,
            "scrappy_dev",
            description(),
            AutonomyLevel::A2,
            1,
            Utc::now(),
        )
        .expect("valid agent");
        assert_eq!(agent.cv_version(), 1);
        assert_eq!(agent.uncommitted_events().len(), 1);
        agent
            .append_cv("First card delivered", Utc::now())
            .expect("cv");
        assert_eq!(agent.cv_version(), 2);
        assert_eq!(agent.uncommitted_events().len(), 2);
        agent.pause();
        assert!(!agent.can_receive_cards());
        agent.resume();
        assert!(agent.can_receive_cards());
        agent.retire(Utc::now()).expect("retire");
        assert!(!agent.can_receive_cards());
        assert!(agent.retire(Utc::now()).is_err());
        agent.mark_events_committed();
        assert!(agent.uncommitted_events().is_empty());
    }

    #[test]
    fn agent_rejects_short_description() {
        let result = Agent::create(
            AgentId::generate(),
            LabId::generate(),
            "Short",
            AgentRole::Dev,
            "scrappy_dev",
            "too short",
            AutonomyLevel::A2,
            1,
            Utc::now(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn team_requires_exactly_one_tech_lead() {
        let lab = LabId::generate();
        let lead = Agent::create(
            AgentId::generate(),
            lab,
            "Ada",
            AgentRole::TechLead,
            "veteran_lead",
            description(),
            AutonomyLevel::A2,
            1,
            Utc::now(),
        )
        .expect("lead");
        assert!(Agent::validate_team(std::slice::from_ref(&lead)).is_ok());
        assert!(Agent::validate_team(&[]).is_err());
        let mut second = lead.clone();
        second.id = AgentId::generate();
        assert!(Agent::validate_team(&[lead, second]).is_err());
    }

    #[test]
    fn project_allocates_card_sequence_and_events() {
        let mut project = Project::create(
            ProjectId::generate(),
            LabId::generate(),
            "Demo",
            "demo-project",
            "AIL",
            "example/ai-lab-demo",
            "main",
            Utc::now(),
        )
        .expect("project");
        assert_eq!(project.next_card_seq().expect("key").to_string(), "AIL-1");
        assert_eq!(project.next_card_seq().expect("key").to_string(), "AIL-2");
        assert_eq!(project.uncommitted_events().len(), 1);
        project.archive().expect("archive");
        assert!(project.next_card_seq().is_err());
        assert!(
            Project::create(
                ProjectId::generate(),
                LabId::generate(),
                "Bad",
                "bad",
                "ail",
                "example/repo",
                "main",
                Utc::now()
            )
            .is_err()
        );
    }

    #[test]
    fn scm_credential_is_write_only() {
        let mut credential = ScmCredential::create(
            ScmCredentialId::generate(),
            WorkspaceId::generate(),
            "GitHub",
            ScmKind::GitHub,
            key(),
        )
        .expect("credential");
        assert_eq!(credential.last4(), "abcd");
        assert!(!credential.is_verified());
        credential.verify();
        assert!(credential.is_verified());
    }
}
