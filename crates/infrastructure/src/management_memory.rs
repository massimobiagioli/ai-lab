//! In-memory management service for tests and the no-database default.

use std::collections::BTreeMap;
use std::sync::Mutex;

use ailab_application::{
    CreateAgentInput, CreateLabRequest, CreateProjectRequest, CreateProviderRequest,
    ManagementError, ManagementService, RotateProviderKeyRequest, UpdateAgentRequest,
    UpdateProjectRequest,
};
use async_trait::async_trait;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::crypto::MasterKey;

/// In-memory management service.
#[derive(Default)]
pub struct InMemoryManagementService {
    providers: Mutex<BTreeMap<String, Value>>,
    master_key: Option<MasterKey>,
}

impl InMemoryManagementService {
    /// New empty service.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Attach a master key for provider create/rotate tests.
    #[must_use]
    pub fn with_master_key(mut self, master_key: MasterKey) -> Self {
        self.master_key = Some(master_key);
        self
    }

    fn validate_key(&self, key: &str) -> Result<(), ManagementError> {
        if self.master_key.is_none() {
            return Err(ManagementError::validation(
                "MASTER_KEY_MISSING",
                "AILAB_MASTER_KEY is not configured",
            ));
        }
        if !key.starts_with("sk-")
            || !(20..=256).contains(&key.chars().count())
            || key.chars().any(char::is_whitespace)
        {
            return Err(ManagementError::validation(
                "VALIDATION_FAILED",
                "api_key must start with sk-, be 20-256 characters and contain no whitespace",
            ));
        }
        Ok(())
    }
}

fn last4(key: &str) -> String {
    key.chars()
        .rev()
        .take(4)
        .collect::<String>()
        .chars()
        .rev()
        .collect()
}

#[async_trait]
impl ManagementService for InMemoryManagementService {
    async fn list_providers(&self) -> Result<Value, ManagementError> {
        let providers = self
            .providers
            .lock()
            .map_err(|_| ManagementError::Backend("lock poisoned".to_owned()))?;
        Ok(json!({"data": providers.values().cloned().collect::<Vec<_>>()}))
    }

    async fn get_provider(&self, id: &str) -> Result<Option<Value>, ManagementError> {
        Ok(self
            .providers
            .lock()
            .map_err(|_| ManagementError::Backend("lock poisoned".to_owned()))?
            .get(id)
            .cloned())
    }

    async fn create_provider(
        &self,
        request: CreateProviderRequest,
    ) -> Result<Value, ManagementError> {
        self.validate_key(&request.api_key)?;
        let id = Uuid::now_v7().to_string();
        let provider = json!({
            "id": id,
            "name": request.name,
            "kind": request.kind,
            "base_url": request.base_url,
            "status": "active",
            "api_key_last4": last4(&request.api_key),
            "key_version": 1,
            "rate_limit_rpm": request.rate_limit_rpm.unwrap_or(600),
            "rate_limit_tpm": request.rate_limit_tpm.unwrap_or(120_000),
            "timeout_ms": request.timeout_ms.unwrap_or(120_000),
        });
        self.providers
            .lock()
            .map_err(|_| ManagementError::Backend("lock poisoned".to_owned()))?
            .insert(id, provider.clone());
        Ok(provider)
    }

    async fn rotate_provider_key(
        &self,
        id: &str,
        request: RotateProviderKeyRequest,
    ) -> Result<Value, ManagementError> {
        self.validate_key(&request.api_key)?;
        let mut providers = self
            .providers
            .lock()
            .map_err(|_| ManagementError::Backend("lock poisoned".to_owned()))?;
        let provider = providers.get_mut(id).ok_or(ManagementError::NotFound)?;
        provider["api_key_last4"] = Value::String(last4(&request.api_key));
        let version = provider["key_version"].as_i64().unwrap_or(1) + 1;
        provider["key_version"] = Value::from(version);
        let result = provider.clone();
        drop(providers);
        Ok(result)
    }

    async fn verify_provider(&self, id: &str) -> Result<Value, ManagementError> {
        let mut providers = self
            .providers
            .lock()
            .map_err(|_| ManagementError::Backend("lock poisoned".to_owned()))?;
        let provider = providers.get_mut(id).ok_or(ManagementError::NotFound)?;
        provider["status"] = Value::String("active".to_owned());
        let result = provider.clone();
        drop(providers);
        Ok(result)
    }

    async fn list_labs(&self) -> Result<Value, ManagementError> {
        Ok(json!({"data": []}))
    }

    async fn get_lab(&self, _id: &str) -> Result<Option<Value>, ManagementError> {
        Ok(None)
    }

    async fn list_agents(&self, _lab_id: Option<&str>) -> Result<Value, ManagementError> {
        Ok(json!({"data": []}))
    }

    async fn get_agent(&self, _id: &str) -> Result<Option<Value>, ManagementError> {
        Ok(None)
    }

    async fn get_agent_cv(&self, _id: &str) -> Result<Option<Value>, ManagementError> {
        Ok(None)
    }

    async fn set_agent_status(
        &self,
        _id: &str,
        _status: &str,
    ) -> Result<Option<Value>, ManagementError> {
        Err(ManagementError::NotFound)
    }

    async fn list_characters(&self) -> Result<Value, ManagementError> {
        Ok(json!({"data": []}))
    }

    async fn list_models(&self) -> Result<Value, ManagementError> {
        Ok(json!({"data": []}))
    }

    async fn list_scm_credentials(&self) -> Result<Value, ManagementError> {
        Ok(json!({"data": []}))
    }

    async fn list_projects(&self) -> Result<Value, ManagementError> {
        Ok(json!({"data": []}))
    }

    async fn get_project(&self, _id: &str) -> Result<Option<Value>, ManagementError> {
        Ok(None)
    }

    async fn get_settings(&self) -> Result<Value, ManagementError> {
        Ok(json!({}))
    }

    async fn create_lab(&self, _request: CreateLabRequest) -> Result<Value, ManagementError> {
        Err(ManagementError::validation(
            "DATABASE_UNAVAILABLE",
            "the management service requires a database",
        ))
    }

    async fn create_agent(
        &self,
        _lab_id: &str,
        _input: CreateAgentInput,
    ) -> Result<Value, ManagementError> {
        Err(ManagementError::validation(
            "DATABASE_UNAVAILABLE",
            "the management service requires a database",
        ))
    }

    async fn update_agent(
        &self,
        _id: &str,
        _request: UpdateAgentRequest,
    ) -> Result<Value, ManagementError> {
        Err(ManagementError::NotFound)
    }

    async fn retire_agent(&self, _id: &str) -> Result<Value, ManagementError> {
        Err(ManagementError::NotFound)
    }

    async fn create_project(
        &self,
        _request: CreateProjectRequest,
    ) -> Result<Value, ManagementError> {
        Err(ManagementError::validation(
            "DATABASE_UNAVAILABLE",
            "the management service requires a database",
        ))
    }

    async fn update_project(
        &self,
        _id: &str,
        _request: UpdateProjectRequest,
    ) -> Result<Value, ManagementError> {
        Err(ManagementError::NotFound)
    }

    async fn sync_project(&self, _id: &str) -> Result<Value, ManagementError> {
        Err(ManagementError::NotFound)
    }
}
