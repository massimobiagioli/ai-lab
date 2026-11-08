//! Docker-backed sandbox runner (WP-14).
//!
//! Containers are created with a read-only root, all capabilities dropped,
//! no host mounts except the run worktree, a PID/memory/CPU cap, a
//! `no-new-privileges` profile and no Docker socket. Exec is bounded by a
//! timeout and the caller applies the tool policy before calling.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use ailab_domain::AgentRole;
use async_trait::async_trait;
use bollard::API_DEFAULT_VERSION;
use bollard::Docker;
use bollard::container::{
    Config, CreateContainerOptions, LogsOptions, RemoveContainerOptions, StartContainerOptions,
};
use bollard::exec::{CreateExecOptions, StartExecOptions, StartExecResults};
use bollard::models::{ContainerStateStatusEnum, HostConfig};
use chrono::Utc;
use futures::StreamExt;

use crate::backend::{BackendError, CreatedSandbox, ExecOutcome, SandboxBackend, SandboxView};
use crate::sandbox::{SandboxSpec, SandboxState, ToolCall};

/// Docker-backed sandbox backend.
pub struct DockerBackend {
    docker: Docker,
    workspaces_dir: String,
    egress_network: String,
}

impl DockerBackend {
    /// Connect to a Docker Engine over HTTP (the socket proxy in Compose).
    pub fn connect(
        docker_host: &str,
        workspaces_dir: impl Into<String>,
        egress_network: impl Into<String>,
    ) -> Result<Self, BackendError> {
        let docker = Docker::connect_with_http(docker_host, 120, API_DEFAULT_VERSION)
            .map_err(|error| BackendError::Internal(format!("docker connect: {error}")))?;
        Ok(Self {
            docker,
            workspaces_dir: workspaces_dir.into(),
            egress_network: egress_network.into(),
        })
    }

    /// Build from the runner environment.
    pub fn from_env() -> Result<Self, BackendError> {
        let host = std::env::var("DOCKER_HOST")
            .unwrap_or_else(|_| "tcp://docker-socket-proxy:2375".to_owned());
        let workspaces = std::env::var("AILAB_WORKSPACES_DIR")
            .unwrap_or_else(|_| "/var/lib/ailab/workspaces".to_owned());
        let network = std::env::var("AILAB_SANDBOX_EGRESS_NETWORK")
            .unwrap_or_else(|_| "sandbox-egress".to_owned());
        Self::connect(&host, workspaces, network)
    }

    fn network_mode(&self, spec: &SandboxSpec) -> String {
        match &spec.egress {
            crate::sandbox::EgressPolicy::Disabled => "none".to_owned(),
            crate::sandbox::EgressPolicy::Allowlist(_) => self.egress_network.clone(),
        }
    }

    fn worktree_path(&self, spec: &SandboxSpec) -> String {
        format!(
            "{}/{}",
            self.workspaces_dir.trim_end_matches('/'),
            spec.worktree_ref
        )
    }

    async fn container(&self, sandbox_id: &str) -> Result<String, BackendError> {
        let info = self
            .docker
            .inspect_container(sandbox_id, None)
            .await
            .map_err(map_docker_error)?;
        info.id.ok_or(BackendError::NotFound)
    }
}

/// Maximum captured exec output per stream.
pub const OUTPUT_CAP_BYTES: usize = 20 * 1024;

fn truncate(mut value: String) -> String {
    if value.len() > OUTPUT_CAP_BYTES {
        value.truncate(OUTPUT_CAP_BYTES);
        value.push_str("\n[output truncated]");
    }
    value
}

/// Host configuration enforcing the WP-14 isolation controls.
#[must_use]
pub fn host_config_for(spec: &SandboxSpec, worktree_path: &str, network_mode: &str) -> HostConfig {
    HostConfig {
        binds: Some(vec![format!("{worktree_path}:/workspace:rw")]),
        cap_drop: Some(vec!["ALL".to_owned()]),
        memory: Some(i64::from(spec.memory_mb) * 1024 * 1024),
        nano_cpus: Some(i64::from(spec.cpu_millis) * 1_000_000),
        pids_limit: Some(512),
        readonly_rootfs: Some(true),
        network_mode: Some(network_mode.to_owned()),
        security_opt: Some(vec!["no-new-privileges".to_owned()]),
        tmpfs: Some(HashMap::from([(
            "/tmp".to_owned(),
            "rw,noexec,nosuid,size=512m".to_owned(),
        )])),
        auto_remove: Some(false),
        init: Some(true),
        ..Default::default()
    }
}

fn container_env(spec: &SandboxSpec) -> Option<Vec<String>> {
    let mut env = Vec::new();
    if let Some(proxy) = &spec.egress_proxy {
        env.push(format!("HTTP_PROXY={proxy}"));
        env.push(format!("HTTPS_PROXY={proxy}"));
        env.push("NO_PROXY=127.0.0.1,localhost".to_owned());
    }
    if let Some(token) = &spec.scm_token {
        env.push(format!("AILAB_SCM_TOKEN={}", token.expose()));
    }
    if env.is_empty() { None } else { Some(env) }
}

fn map_docker_error(error: bollard::errors::Error) -> BackendError {
    match error {
        bollard::errors::Error::DockerResponseServerError {
            status_code: 404, ..
        } => BackendError::NotFound,
        other => BackendError::Internal(other.to_string()),
    }
}

#[async_trait]
impl SandboxBackend for DockerBackend {
    async fn create(
        &self,
        spec: &SandboxSpec,
        _role: AgentRole,
    ) -> Result<CreatedSandbox, BackendError> {
        // Idempotency by label: reuse a running container for the same run.
        let filters = HashMap::from([(
            "label".to_owned(),
            vec![format!("ailab.run_id={}", spec.run_id)],
        )]);
        let existing = self
            .docker
            .list_containers(Some(bollard::container::ListContainersOptions {
                all: true,
                filters,
                ..Default::default()
            }))
            .await
            .map_err(map_docker_error)?;
        if let Some(container) = existing.first()
            && let Some(id) = container.id.as_deref()
        {
            return Ok(CreatedSandbox {
                sandbox_id: id.to_owned(),
                container_id: id.to_owned(),
                created_at: Utc::now(),
            });
        }

        let name = format!("ailab-sandbox-{}", uuid::Uuid::new_v4());
        let host_config =
            host_config_for(spec, &self.worktree_path(spec), &self.network_mode(spec));
        let config = Config {
            image: Some(spec.image.clone()),
            user: Some("10001:10001".to_owned()),
            working_dir: Some("/workspace".to_owned()),
            env: container_env(spec),
            labels: Some(HashMap::from([(
                "ailab.run_id".to_owned(),
                spec.run_id.clone(),
            )])),
            host_config: Some(host_config),
            tty: Some(false),
            ..Default::default()
        };
        let created = self
            .docker
            .create_container(
                Some(CreateContainerOptions {
                    name: name.as_str(),
                    platform: None,
                }),
                config,
            )
            .await
            .map_err(map_docker_error)?;
        self.docker
            .start_container(&created.id, None::<StartContainerOptions<String>>)
            .await
            .map_err(map_docker_error)?;
        Ok(CreatedSandbox {
            sandbox_id: created.id.clone(),
            container_id: created.id,
            created_at: Utc::now(),
        })
    }

    async fn status(&self, sandbox_id: &str) -> Result<SandboxView, BackendError> {
        let info = self
            .docker
            .inspect_container(sandbox_id, None)
            .await
            .map_err(map_docker_error)?;
        let state = info.state.unwrap_or_default();
        let status = match state.status {
            Some(ContainerStateStatusEnum::RUNNING | ContainerStateStatusEnum::PAUSED) => {
                SandboxState::Running
            }
            Some(ContainerStateStatusEnum::EXITED) => SandboxState::Stopped,
            Some(ContainerStateStatusEnum::DEAD) => SandboxState::Failed,
            Some(
                ContainerStateStatusEnum::CREATED
                | ContainerStateStatusEnum::RESTARTING
                | ContainerStateStatusEnum::REMOVING
                | ContainerStateStatusEnum::EMPTY,
            )
            | None => SandboxState::Creating,
        };
        Ok(SandboxView {
            sandbox_id: sandbox_id.to_owned(),
            status,
            exit_code: state.exit_code.and_then(|code| i32::try_from(code).ok()),
            started_at: Utc::now(),
            ended_at: None,
        })
    }

    async fn logs(&self, sandbox_id: &str) -> Result<Vec<String>, BackendError> {
        let mut stream = self.docker.logs(
            sandbox_id,
            Some(LogsOptions::<String> {
                stdout: true,
                stderr: true,
                follow: false,
                ..Default::default()
            }),
        );
        let mut lines = Vec::new();
        while let Some(item) = stream.next().await {
            let output = item.map_err(map_docker_error)?;
            lines.push(output.to_string());
        }
        Ok(lines)
    }

    async fn exec(
        &self,
        sandbox_id: &str,
        call: &ToolCall,
        timeout_s: u64,
    ) -> Result<ExecOutcome, BackendError> {
        let start = Instant::now();
        let container = self.container(sandbox_id).await?;
        let timeout = Duration::from_secs(timeout_s.max(1));
        let result = tokio::time::timeout(timeout, async {
            let exec = self
                .docker
                .create_exec(
                    &container,
                    CreateExecOptions {
                        cmd: Some(call.argv.clone()),
                        working_dir: Some(call.cwd.clone()),
                        attach_stdout: Some(true),
                        attach_stderr: Some(true),
                        ..Default::default()
                    },
                )
                .await
                .map_err(map_docker_error)?;
            let started = self
                .docker
                .start_exec(&exec.id, None::<StartExecOptions>)
                .await
                .map_err(map_docker_error)?;
            let mut stdout = String::new();
            let mut stderr = String::new();
            if let StartExecResults::Attached { mut output, .. } = started {
                while let Some(item) = output.next().await {
                    match item.map_err(map_docker_error)? {
                        bollard::container::LogOutput::StdOut { message }
                        | bollard::container::LogOutput::Console { message } => {
                            stdout.push_str(&String::from_utf8_lossy(&message));
                        }
                        bollard::container::LogOutput::StdErr { message } => {
                            stderr.push_str(&String::from_utf8_lossy(&message));
                        }
                        bollard::container::LogOutput::StdIn { .. } => {}
                    }
                }
            }
            let inspect = self
                .docker
                .inspect_exec(&exec.id)
                .await
                .map_err(map_docker_error)?;
            Ok::<_, BackendError>((inspect.exit_code.unwrap_or(-1), stdout, stderr))
        })
        .await;
        match result {
            Ok(Ok((exit_code, stdout, stderr))) => Ok(ExecOutcome {
                exit_code: i32::try_from(exit_code).unwrap_or(-1),
                stdout: truncate(stdout),
                stderr: truncate(stderr),
                duration_ms: u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
            }),
            Ok(Err(error)) => Err(error),
            Err(_) => Err(BackendError::Timeout),
        }
    }

    async fn delete(&self, sandbox_id: &str) -> Result<(), BackendError> {
        self.docker
            .remove_container(
                sandbox_id,
                Some(RemoveContainerOptions {
                    force: true,
                    ..Default::default()
                }),
            )
            .await
            .map_err(map_docker_error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sandbox::EgressPolicy;

    fn spec() -> SandboxSpec {
        SandboxSpec::new(
            format!("run-{}", uuid::Uuid::new_v4()),
            "debian:trixie-slim@sha256:a72b4433de1cf7d6bae0c4cee74d0547f58601a643b3375f070c51600d63b6b2",
            "escape-drill",
            1_000,
            1_024,
            60,
            EgressPolicy::Disabled,
        )
        .expect("spec")
    }

    fn backend() -> DockerBackend {
        let dir = std::env::temp_dir().join(format!("ailab-escape-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(dir.join("escape-drill")).expect("worktree");
        DockerBackend::connect(
            "tcp://localhost:2375",
            dir.to_string_lossy().to_string(),
            "none",
        )
        .expect("connect")
    }

    #[test]
    fn egress_and_token_env_are_injected_without_leaking() {
        let spec = spec()
            .with_egress_proxy("http://egress-proxy:3128")
            .with_scm_token(ailab_application::Secret::new("short-token".to_owned()));
        let env = container_env(&spec).expect("env");
        assert!(
            env.iter()
                .any(|value| value == "HTTP_PROXY=http://egress-proxy:3128")
        );
        assert!(
            env.iter()
                .any(|value| value == "AILAB_SCM_TOKEN=short-token")
        );
        let debug = format!("{spec:?}");
        assert!(!debug.contains("short-token"));
    }

    #[test]
    fn host_config_enforces_isolation() {
        let spec = spec();
        let host = host_config_for(&spec, "/tmp/workspaces/escape-drill", "none");
        assert_eq!(host.readonly_rootfs, Some(true));
        assert_eq!(host.cap_drop, Some(vec!["ALL".to_owned()]));
        assert!(host.pids_limit.is_some_and(|limit| limit <= 512));
        assert_eq!(host.network_mode.as_deref(), Some("none"));
        assert!(
            host.security_opt
                .as_ref()
                .is_some_and(|options| options.iter().any(|option| option == "no-new-privileges"))
        );
        let binds = host.binds.unwrap_or_default();
        assert_eq!(
            binds,
            vec!["/tmp/workspaces/escape-drill:/workspace:rw".to_owned()]
        );
        assert!(!binds.iter().any(|bind| bind.contains("docker.sock")));
        assert!(!binds.iter().any(|bind| bind == "/:/workspace:rw"));
    }

    /// Live escape suite; run with `cargo test -p ailab-orchestrator -- --ignored`.
    #[tokio::test]
    #[ignore = "requires a reachable Docker Engine and the debian:trixie-slim image"]
    async fn docker_escape_suite() {
        let backend = backend();
        let created = backend
            .create(&spec(), AgentRole::Dev)
            .await
            .expect("create");
        let check = |argv: &[&str]| ToolCall {
            argv: argv.iter().map(|arg| (*arg).to_owned()).collect(),
            cwd: "workspace".to_owned(),
            writes: vec![],
        };
        let socket = backend
            .exec(
                &created.sandbox_id,
                &check(&["ls", "/var/run/docker.sock"]),
                10,
            )
            .await
            .expect("exec");
        assert_ne!(socket.exit_code, 0, "docker socket must be unreachable");
        let shadow = backend
            .exec(&created.sandbox_id, &check(&["cat", "/etc/shadow"]), 10)
            .await
            .expect("exec");
        assert_ne!(shadow.exit_code, 0, "/etc/shadow must be unreadable");
        let root = backend
            .exec(&created.sandbox_id, &check(&["ls", "/"]), 10)
            .await
            .expect("exec");
        assert_eq!(root.exit_code, 0, "the read-only root must run");
        backend.delete(&created.sandbox_id).await.expect("delete");
    }

    /// Live Docker test; run with `cargo test -p ailab-orchestrator -- --ignored`.
    #[tokio::test]
    #[ignore = "requires a reachable Docker Engine and the debian:trixie-slim image"]
    async fn docker_create_exec_logs_and_delete() {
        let backend = backend();
        let spec = spec();
        let created = backend.create(&spec, AgentRole::Dev).await.expect("create");
        let call = ToolCall {
            argv: vec!["echo".to_owned(), "hello".to_owned()],
            cwd: "/workspace".to_owned(),
            writes: vec![],
        };
        let outcome = backend
            .exec(&created.sandbox_id, &call, 30)
            .await
            .expect("exec");
        assert_eq!(outcome.exit_code, 0);
        assert!(outcome.stdout.contains("hello"));
        let logs = backend.logs(&created.sandbox_id).await.expect("logs");
        assert_ne!(logs.len(), 0);
        backend.delete(&created.sandbox_id).await.expect("delete");
        assert_eq!(
            backend.status(&created.sandbox_id).await,
            Err(BackendError::NotFound)
        );
    }
}
