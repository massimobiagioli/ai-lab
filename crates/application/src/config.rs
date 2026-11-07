//! Environment configuration loader with fail-fast validation.

use std::collections::BTreeSet;

/// Known `AILAB_*` variables. Unknown ones are reported as warnings, not errors.
pub const KNOWN_VARIABLES: &[&str] = &[
    "AILAB_ENV",
    "AILAB_BIND",
    "AILAB_PORT",
    "AILAB_DATABASE_URL",
    "AILAB_REDIS_URL",
    "AILAB_MASTER_KEY",
    "AILAB_API_TOKEN",
    "AILAB_LOG_LEVEL",
    "AILAB_LOG_FORMAT",
    "AILAB_OTEL_ENDPOINT",
    "AILAB_METRICS_ENABLED",
    "AILAB_ARTIFACTS_DIR",
    "AILAB_WORKSPACES_DIR",
    "AILAB_RUNNER_IMAGE",
    "AILAB_RUNNER_CPU",
    "AILAB_RUNNER_MEMORY_MB",
    "AILAB_RUNNER_TIMEOUT_S",
    "AILAB_RUNNER_TOKEN",
    "AILAB_RUNNER_URL",
    "AILAB_RUNNER_BIND",
    "AILAB_APP_DB_PASSWORD",
    "AILAB_MIGRATE_DB_PASSWORD",
    "AILAB_READONLY_DB_PASSWORD",
    "AILAB_WORKER_CONCURRENCY",
    "AILAB_SCHEDULER_TICK_MS",
    "AILAB_LEASE_TTL_S",
    "AILAB_LLM_DEFAULT_MODEL",
    "AILAB_LLM_ESCALATION_MODEL",
    "AILAB_GITHUB_API_URL",
    "AILAB_GITHUB_WEBHOOK_SECRET",
    "AILAB_ALLOWED_ORIGINS",
    "AILAB_CORS_ENABLED",
    "AILAB_RATE_LIMIT_READ_RPM",
    "AILAB_RATE_LIMIT_WRITE_RPM",
    "AILAB_MAX_REQUEST_BYTES",
    "AILAB_PG_POOL_MAX",
    "AILAB_REDIS_MAXMEMORY",
    "AILAB_EMBEDDING_MODEL",
    "AILAB_MEMORY_TOP_K",
    "AILAB_MEMORY_CACHE_TTL_S",
    "AILAB_PROMPT_RETENTION_DAYS",
    "AILAB_EPISODE_RETENTION_DAYS",
    "AILAB_BUDGET_DEFAULT_PER_CARD_USD",
    "AILAB_BUDGET_DEFAULT_PER_DAY_USD",
    "AILAB_SEED_DEMO",
];

/// Validated runtime configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    /// `production` or `development`.
    pub env: String,
    /// Bind address.
    pub bind: String,
    /// HTTP port.
    pub port: u16,
    /// Log level.
    pub log_level: String,
    /// Database URL, when configured.
    pub database_url: Option<String>,
    /// Redis URL, when configured.
    pub redis_url: Option<String>,
    /// Remote bearer token, when configured.
    pub api_token: Option<String>,
    /// Non-fatal findings.
    pub warnings: Vec<String>,
}

/// Configuration failure naming the offending variable.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("invalid configuration for {variable}: {message}")]
pub struct ConfigError {
    /// Variable name.
    pub variable: String,
    /// Reason.
    pub message: String,
}

impl ConfigError {
    fn new(variable: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            variable: variable.into(),
            message: message.into(),
        }
    }
}

fn is_loopback(bind: &str) -> bool {
    matches!(bind, "127.0.0.1" | "::1" | "localhost")
}

impl Config {
    /// Load from the process environment.
    pub fn from_env() -> Result<Self, ConfigError> {
        let warnings = unknown_variables(std::env::vars().map(|(key, _)| key));
        let mut config = Self::from_source(|key| std::env::var(key).ok())?;
        config.warnings = warnings;
        Ok(config)
    }

    /// Load from a key-value source (used by tests).
    pub fn from_source<F>(get: F) -> Result<Self, ConfigError>
    where
        F: Fn(&str) -> Option<String>,
    {
        let env = get("AILAB_ENV").unwrap_or_else(|| "production".to_owned());
        let bind = get("AILAB_BIND").unwrap_or_else(|| "127.0.0.1".to_owned());
        let port = match get("AILAB_PORT") {
            Some(value) => value
                .parse::<u16>()
                .map_err(|_| ConfigError::new("AILAB_PORT", "must be a TCP port"))?,
            None => 8080,
        };
        let log_level = get("AILAB_LOG_LEVEL").unwrap_or_else(|| "info".to_owned());
        if !matches!(
            log_level.as_str(),
            "trace" | "debug" | "info" | "warn" | "error"
        ) {
            return Err(ConfigError::new(
                "AILAB_LOG_LEVEL",
                "must be trace|debug|info|warn|error",
            ));
        }
        let api_token = get("AILAB_API_TOKEN").filter(|value| !value.is_empty());
        if !is_loopback(&bind) {
            match api_token.as_deref() {
                Some(token) if token.len() >= 32 => {}
                Some(_) => {
                    return Err(ConfigError::new(
                        "AILAB_API_TOKEN",
                        "must be at least 32 bytes for a non-loopback bind",
                    ));
                }
                None => {
                    return Err(ConfigError::new(
                        "AILAB_API_TOKEN",
                        "required for a non-loopback bind",
                    ));
                }
            }
        }
        Ok(Self {
            env,
            bind,
            port,
            log_level,
            database_url: get("AILAB_DATABASE_URL"),
            redis_url: get("AILAB_REDIS_URL"),
            api_token,
            warnings: Vec::new(),
        })
    }
}

/// Report unknown `AILAB_*` variable names.
#[must_use]
pub fn unknown_variables<I>(keys: I) -> Vec<String>
where
    I: IntoIterator<Item = String>,
{
    let known: BTreeSet<&str> = KNOWN_VARIABLES.iter().copied().collect();
    keys.into_iter()
        .filter(|key| key.starts_with("AILAB_") && !known.contains(key.as_str()))
        .map(|key| format!("unknown AILAB variable: {key}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source<'a>(pairs: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
        move |key| {
            pairs
                .iter()
                .find(|(name, _)| *name == key)
                .map(|(_, value)| (*value).to_owned())
        }
    }

    #[test]
    fn defaults_are_loopback_and_8080() {
        let config = Config::from_source(source(&[])).expect("valid");
        assert_eq!(config.bind, "127.0.0.1");
        assert_eq!(config.port, 8080);
        assert_eq!(config.log_level, "info");
    }

    #[test]
    fn invalid_port_names_the_variable() {
        let error = Config::from_source(source(&[("AILAB_PORT", "nope")])).expect_err("invalid");
        assert_eq!(error.variable, "AILAB_PORT");
    }

    #[test]
    fn invalid_log_level_names_the_variable() {
        let error =
            Config::from_source(source(&[("AILAB_LOG_LEVEL", "verbose")])).expect_err("invalid");
        assert_eq!(error.variable, "AILAB_LOG_LEVEL");
    }

    #[test]
    fn non_loopback_requires_a_token() {
        let error = Config::from_source(source(&[("AILAB_BIND", "0.0.0.0")])).expect_err("invalid");
        assert_eq!(error.variable, "AILAB_API_TOKEN");
        let token = "x".repeat(32);
        let config = Config::from_source(source(&[
            ("AILAB_BIND", "0.0.0.0"),
            ("AILAB_API_TOKEN", &token),
        ]))
        .expect("valid");
        assert_eq!(config.api_token.as_deref(), Some(token.as_str()));
    }

    #[test]
    fn unknown_variables_are_reported() {
        let warnings = unknown_variables(vec![
            "AILAB_PORT".to_owned(),
            "PATH".to_owned(),
            "AILAB_MYSTERY".to_owned(),
        ]);
        assert_eq!(warnings, vec!["unknown AILAB variable: AILAB_MYSTERY"]);
    }
}
