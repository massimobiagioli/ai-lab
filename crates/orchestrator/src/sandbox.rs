//! Sandbox specification, egress policy, tool policy and lifecycle.

use std::collections::BTreeSet;

use ailab_domain::AgentRole;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Egress policy for a sandbox.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EgressPolicy {
    /// No outbound network.
    Disabled,
    /// Allowlisted domains only.
    Allowlist(Vec<String>),
}

impl EgressPolicy {
    /// True when the host is allowed (exact or subdomain).
    #[must_use]
    pub fn allows(&self, host: &str) -> bool {
        match self {
            Self::Disabled => false,
            Self::Allowlist(domains) => domains.iter().any(|domain| {
                host.eq_ignore_ascii_case(domain)
                    || host
                        .to_ascii_lowercase()
                        .ends_with(&format!(".{}", domain.to_ascii_lowercase()))
            }),
        }
    }

    /// Validate domains.
    pub fn validate(&self) -> Result<(), SandboxError> {
        if let Self::Allowlist(domains) = self {
            if domains.is_empty() {
                return Err(SandboxError::Validation(
                    "egress allowlist must not be empty".to_owned(),
                ));
            }
            for domain in domains {
                let lower = domain.to_ascii_lowercase();
                if lower == "*"
                    || lower.contains('/')
                    || lower.contains(':')
                    || lower.starts_with('.')
                    || lower.is_empty()
                {
                    return Err(SandboxError::Validation(format!(
                        "invalid egress domain: {domain}"
                    )));
                }
            }
        }
        Ok(())
    }
}

/// Sandbox creation specification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SandboxSpec {
    /// Run id.
    pub run_id: String,
    /// Digest-pinned image.
    pub image: String,
    /// Git worktree reference.
    pub worktree_ref: String,
    /// CPU limit in millicores.
    pub cpu_millis: u32,
    /// Memory limit in MB.
    pub memory_mb: u32,
    /// Wall-clock timeout in seconds.
    pub timeout_s: u64,
    /// Egress policy.
    pub egress: EgressPolicy,
    /// Optional egress proxy URL (injected as HTTP(S)_PROXY).
    pub egress_proxy: Option<String>,
    /// Short-lived SCM token injected only into the sandbox env.
    #[serde(skip)]
    pub scm_token: Option<ailab_application::Secret<String>>,
}

/// Isolation flags asserted by the sandbox template.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)]
pub struct IsolationFlags {
    /// Root filesystem is read-only.
    pub read_only_root: bool,
    /// All capabilities dropped.
    pub cap_drop_all: bool,
    /// Non-root UID.
    pub uid: u32,
    /// PID limit.
    pub pids_limit: u32,
    /// No host mounts.
    pub no_host_mounts: bool,
    /// No Docker socket.
    pub no_docker_socket: bool,
    /// `no-new-privileges`.
    pub no_new_privileges: bool,
}

impl Default for IsolationFlags {
    fn default() -> Self {
        Self {
            read_only_root: true,
            cap_drop_all: true,
            uid: 10_001,
            pids_limit: 512,
            no_host_mounts: true,
            no_docker_socket: true,
            no_new_privileges: true,
        }
    }
}

impl IsolationFlags {
    /// All mandatory isolation controls are enabled.
    #[must_use]
    pub const fn is_secure(self) -> bool {
        self.read_only_root
            && self.cap_drop_all
            && self.uid != 0
            && self.pids_limit <= 512
            && self.no_host_mounts
            && self.no_docker_socket
            && self.no_new_privileges
    }
}

impl SandboxSpec {
    /// Validate the run id, digest pin, worktree and limits.
    pub fn new(
        run_id: impl Into<String>,
        image: impl Into<String>,
        worktree_ref: impl Into<String>,
        cpu_millis: u32,
        memory_mb: u32,
        timeout_s: u64,
        egress: EgressPolicy,
    ) -> Result<Self, SandboxError> {
        let run_id = run_id.into();
        if run_id.is_empty() || run_id.contains('/') || run_id.contains("..") {
            return Err(SandboxError::Validation("invalid run id".to_owned()));
        }
        let image = image.into();
        let Some((_, digest)) = image.split_once("@sha256:") else {
            return Err(SandboxError::Validation(
                "image must be digest-pinned with @sha256:".to_owned(),
            ));
        };
        if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(SandboxError::Validation("invalid image digest".to_owned()));
        }
        let worktree_ref = worktree_ref.into();
        if worktree_ref.is_empty() || worktree_ref.starts_with('/') || worktree_ref.contains("..") {
            return Err(SandboxError::Validation("invalid worktree ref".to_owned()));
        }
        if !(100..=4_000).contains(&cpu_millis) {
            return Err(SandboxError::Validation(
                "cpu_millis must be 100-4000".to_owned(),
            ));
        }
        if !(128..=8_192).contains(&memory_mb) {
            return Err(SandboxError::Validation(
                "memory_mb must be 128-8192".to_owned(),
            ));
        }
        if !(1..=7_200).contains(&timeout_s) {
            return Err(SandboxError::Validation(
                "timeout_s must be 1-7200".to_owned(),
            ));
        }
        egress.validate()?;
        Ok(Self {
            run_id,
            image,
            worktree_ref,
            cpu_millis,
            memory_mb,
            timeout_s,
            egress,
            egress_proxy: None,
            scm_token: None,
        })
    }

    /// Inject the egress proxy URL.
    #[must_use]
    pub fn with_egress_proxy(mut self, proxy: impl Into<String>) -> Self {
        self.egress_proxy = Some(proxy.into());
        self
    }

    /// Inject the short-lived SCM token (never serialized or logged).
    #[must_use]
    pub fn with_scm_token(mut self, token: ailab_application::Secret<String>) -> Self {
        self.scm_token = Some(token);
        self
    }

    /// Mandatory isolation flags.
    #[must_use]
    #[allow(clippy::unused_self)]
    pub const fn isolation(&self) -> IsolationFlags {
        IsolationFlags {
            read_only_root: true,
            cap_drop_all: true,
            uid: 10_001,
            pids_limit: 512,
            no_host_mounts: true,
            no_docker_socket: true,
            no_new_privileges: true,
        }
    }
}

/// A tool call proposed by an agent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolCall {
    /// Command and arguments.
    pub argv: Vec<String>,
    /// Working directory.
    pub cwd: String,
    /// Paths written by the call.
    pub writes: Vec<String>,
}

/// Structured policy denial.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyDenial {
    /// Rule id.
    pub rule: String,
    /// Explanation.
    pub message: String,
}

impl PolicyDenial {
    fn new(rule: &str, message: impl Into<String>) -> Self {
        Self {
            rule: rule.to_owned(),
            message: message.into(),
        }
    }
}

/// Tool policy by role.
#[derive(Debug, Clone)]
pub struct ToolPolicy {
    allowed_commands: BTreeSet<String>,
}

/// Commands that are never allowed.
pub const FORBIDDEN_COMMANDS: &[&str] = &[
    "sudo", "mount", "docker", "nsenter", "chroot", "setcap", "curl", "wget", "nc", "ssh", "scp",
    "bash", "sh",
];

/// Paths that an agent may never write.
pub const PROTECTED_PATHS: &[&str] = &[
    ".github/workflows",
    "CODEOWNERS",
    ".env",
    "secrets",
    ".git/config",
];

impl ToolPolicy {
    /// Build the allowlist for a role.
    #[must_use]
    pub fn for_role(role: AgentRole) -> Self {
        let commands: &[&str] = match role {
            AgentRole::TechLead | AgentRole::Analyst => &["git", "rg", "jq"],
            AgentRole::Dev => &["cargo", "rustc", "npm", "npx", "git", "rg", "jq", "make"],
            AgentRole::Qa => &["cargo", "npm", "npx", "git", "rg", "jq"],
        };
        Self {
            allowed_commands: commands
                .iter()
                .map(|command| (*command).to_owned())
                .collect(),
        }
    }

    /// Evaluate a call and return a structured denial.
    pub fn evaluate(&self, call: &ToolCall) -> Result<(), PolicyDenial> {
        let Some(command) = call.argv.first() else {
            return Err(PolicyDenial::new("EMPTY_COMMAND", "command is empty"));
        };
        if FORBIDDEN_COMMANDS.contains(&command.as_str()) {
            return Err(PolicyDenial::new(
                "FORBIDDEN_COMMAND",
                format!("command `{command}` is forbidden in a sandbox"),
            ));
        }
        if !self.allowed_commands.contains(command) {
            return Err(PolicyDenial::new(
                "COMMAND_NOT_ALLOWED",
                format!("command `{command}` is not allowed for this role"),
            ));
        }
        if command == "git"
            && call.argv.iter().any(|arg| arg == "push")
            && call.argv.iter().any(|arg| arg == "--force" || arg == "-f")
        {
            return Err(PolicyDenial::new("FORCE_PUSH", "force-push is forbidden"));
        }
        if (command == "cargo" || command == "npm") && call.argv.iter().any(|arg| arg == "publish")
        {
            return Err(PolicyDenial::new(
                "PUBLISH_FORBIDDEN",
                "publishing packages is forbidden",
            ));
        }
        for path in std::iter::once(&call.cwd).chain(call.writes.iter()) {
            if path.starts_with('/') || path.contains("..") {
                return Err(PolicyDenial::new(
                    "PATH_TRAVERSAL",
                    format!("path `{path}` escapes the workspace"),
                ));
            }
            if PROTECTED_PATHS
                .iter()
                .any(|protected| path == protected || path.starts_with(&format!("{protected}/")))
            {
                return Err(PolicyDenial::new(
                    "PROTECTED_PATH",
                    format!("path `{path}` is protected"),
                ));
            }
        }
        Ok(())
    }
}

/// Sandbox lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SandboxState {
    /// Creating.
    Creating,
    /// Running.
    Running,
    /// Stopped.
    Stopped,
    /// Failed.
    Failed,
}

/// Sandbox handle.
#[derive(Debug, Clone)]
pub struct Sandbox {
    spec: SandboxSpec,
    state: SandboxState,
    exit_code: Option<i32>,
}

impl Sandbox {
    /// Create a sandbox in `Creating`.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn new(spec: SandboxSpec) -> Self {
        Self {
            spec,
            state: SandboxState::Creating,
            exit_code: None,
        }
    }

    /// Spec.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn spec(&self) -> &SandboxSpec {
        &self.spec
    }

    /// State.
    #[must_use]
    pub const fn state(&self) -> SandboxState {
        self.state
    }

    /// Mark running.
    #[allow(clippy::missing_const_for_fn)]
    pub fn mark_running(&mut self) {
        self.state = SandboxState::Running;
    }

    /// Execute an allowlisted call.
    pub fn exec(&self, call: &ToolCall, policy: &ToolPolicy) -> Result<(), SandboxError> {
        if self.state != SandboxState::Running {
            return Err(SandboxError::InvalidState(
                "sandbox is not running".to_owned(),
            ));
        }
        policy.evaluate(call).map_err(SandboxError::Policy)
    }

    /// Stop the sandbox.
    #[allow(clippy::missing_const_for_fn)]
    pub fn stop(&mut self, exit_code: i32) {
        self.state = SandboxState::Stopped;
        self.exit_code = Some(exit_code);
    }

    /// Exit code after stop.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub const fn exit_code(&self) -> Option<i32> {
        self.exit_code
    }
}

/// Sandbox error.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SandboxError {
    /// Invalid specification.
    #[error("validation failed: {0}")]
    Validation(String),
    /// Policy denial.
    #[error("policy denied: {}", .0.message)]
    Policy(PolicyDenial),
    /// Invalid lifecycle state.
    #[error("invalid state: {0}")]
    InvalidState(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec() -> SandboxSpec {
        SandboxSpec::new(
            "run-1",
            "ailab/runner@sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            "worktrees/run-1",
            1_000,
            512,
            900,
            EgressPolicy::Allowlist(vec!["crates.io".to_owned()]),
        )
        .expect("spec")
    }

    fn call(argv: &[&str], cwd: &str, writes: &[&str]) -> ToolCall {
        ToolCall {
            argv: argv.iter().map(|arg| (*arg).to_owned()).collect(),
            cwd: cwd.to_owned(),
            writes: writes.iter().map(|path| (*path).to_owned()).collect(),
        }
    }

    #[test]
    fn spec_validation_and_isolation() {
        assert!(spec().isolation().is_secure());
        assert!(
            SandboxSpec::new(
                "run-1",
                "ailab/runner:latest",
                "worktrees/run-1",
                1_000,
                512,
                900,
                EgressPolicy::Disabled
            )
            .is_err()
        );
        assert!(
            SandboxSpec::new(
                "run-1",
                "ailab/runner@sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
                "../escape",
                1_000,
                512,
                900,
                EgressPolicy::Disabled
            )
            .is_err()
        );
        assert!(
            SandboxSpec::new(
                "run-1",
                "ailab/runner@sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
                "worktrees/run-1",
                1_000,
                512,
                900,
                EgressPolicy::Allowlist(vec!["*".to_owned()])
            )
            .is_err()
        );
    }

    #[test]
    fn egress_is_allowlisted() {
        let policy = EgressPolicy::Allowlist(vec!["crates.io".to_owned()]);
        assert!(policy.allows("crates.io"));
        assert!(policy.allows("static.crates.io"));
        assert!(!policy.allows("example.com"));
        assert!(!EgressPolicy::Disabled.allows("crates.io"));
    }

    #[test]
    fn tool_policy_allows_and_denies() {
        let policy = ToolPolicy::for_role(AgentRole::Dev);
        assert!(
            policy
                .evaluate(&call(&["cargo", "test"], "workspace", &[]))
                .is_ok()
        );
        for command in ["sudo", "curl", "wget", "ssh", "docker", "bash"] {
            let denial = policy
                .evaluate(&call(&[command], "workspace", &[]))
                .expect_err("denied");
            assert_eq!(denial.rule, "FORBIDDEN_COMMAND");
        }
        assert_eq!(
            policy
                .evaluate(&call(&["git", "push", "--force"], "workspace", &[]))
                .expect_err("force")
                .rule,
            "FORCE_PUSH"
        );
        assert_eq!(
            policy
                .evaluate(&call(&["cargo", "publish"], "workspace", &[]))
                .expect_err("publish")
                .rule,
            "PUBLISH_FORBIDDEN"
        );
        assert_eq!(
            policy
                .evaluate(&call(&["cargo", "test"], "workspace", &[".env"]))
                .expect_err("protected")
                .rule,
            "PROTECTED_PATH"
        );
        assert_eq!(
            policy
                .evaluate(&call(&["cargo", "test"], "../outside", &[]))
                .expect_err("traversal")
                .rule,
            "PATH_TRAVERSAL"
        );
    }

    #[test]
    fn analyst_cannot_run_cargo() {
        let policy = ToolPolicy::for_role(AgentRole::Analyst);
        assert_eq!(
            policy
                .evaluate(&call(&["cargo", "test"], "workspace", &[]))
                .expect_err("denied")
                .rule,
            "COMMAND_NOT_ALLOWED"
        );
    }

    #[test]
    fn sandbox_lifecycle() {
        let mut sandbox = Sandbox::new(spec());
        let policy = ToolPolicy::for_role(AgentRole::Dev);
        assert!(
            sandbox
                .exec(&call(&["cargo", "test"], "workspace", &[]), &policy)
                .is_err()
        );
        sandbox.mark_running();
        assert!(
            sandbox
                .exec(&call(&["cargo", "test"], "workspace", &[]), &policy)
                .is_ok()
        );
        sandbox.stop(0);
        assert_eq!(sandbox.state(), SandboxState::Stopped);
        assert_eq!(sandbox.exit_code(), Some(0));
    }
}
