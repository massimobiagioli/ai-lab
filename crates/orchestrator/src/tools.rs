//! Typed tool registry and deny-by-default policy (WP-14 §3.5/§3.6).

use std::collections::BTreeSet;

use ailab_domain::AgentRole;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// Tool side effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SideEffect {
    /// Read-only.
    Read,
    /// Writes to the workspace.
    Write,
    /// External effect (SCM, network).
    External,
}

/// A typed tool contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolSpec {
    /// Name.
    pub name: &'static str,
    /// Description.
    pub description: &'static str,
    /// Input JSON Schema.
    pub input_schema: Value,
    /// Side effect.
    pub side_effect: SideEffect,
    /// Allowed roles.
    pub roles: &'static [AgentRole],
    /// Timeout seconds.
    pub timeout_s: u64,
    /// Output cap.
    pub max_output_bytes: usize,
}

/// Structured policy denial.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolDenial {
    /// Rule id.
    pub rule: String,
    /// Tool.
    pub tool: String,
    /// Argument that failed.
    pub argument: String,
    /// Reason.
    pub reason: String,
}

impl ToolDenial {
    fn new(rule: &str, tool: &str, argument: &str, reason: impl Into<String>) -> Self {
        Self {
            rule: rule.to_owned(),
            tool: tool.to_owned(),
            argument: argument.to_owned(),
            reason: reason.into(),
        }
    }
}

/// Tool registry.
#[derive(Debug, Clone)]
pub struct ToolRegistry {
    tools: Vec<ToolSpec>,
}

const ALL: &[AgentRole] = &[
    AgentRole::TechLead,
    AgentRole::Analyst,
    AgentRole::Dev,
    AgentRole::Qa,
];
const DEV_QA: &[AgentRole] = &[AgentRole::Dev, AgentRole::Qa];
const DEV: &[AgentRole] = &[AgentRole::Dev];
const QA: &[AgentRole] = &[AgentRole::Qa];
const LEAD: &[AgentRole] = &[AgentRole::TechLead];

fn spec(
    name: &'static str,
    description: &'static str,
    side_effect: SideEffect,
    roles: &'static [AgentRole],
    required: &[&str],
    timeout_s: u64,
    max_output_bytes: usize,
) -> ToolSpec {
    let required: Vec<&str> = required.to_vec();
    ToolSpec {
        name,
        description,
        input_schema: json!({
            "type": "object",
            "required": required,
            "additionalProperties": false
        }),
        side_effect,
        roles,
        timeout_s,
        max_output_bytes,
    }
}

impl ToolRegistry {
    /// Build the v1 registry (WP-14 §3.5).
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub fn v1() -> Self {
        let tools = vec![
            spec(
                "fs.list",
                "List a workspace directory",
                SideEffect::Read,
                ALL,
                &["path"],
                10,
                20_000,
            ),
            spec(
                "fs.read",
                "Read a workspace file",
                SideEffect::Read,
                ALL,
                &["path"],
                10,
                1_048_576,
            ),
            spec(
                "fs.search",
                "Search the workspace",
                SideEffect::Read,
                ALL,
                &["query"],
                30,
                20_000,
            ),
            spec(
                "fs.write",
                "Write a workspace file",
                SideEffect::Write,
                DEV_QA,
                &["path", "content"],
                10,
                1_048_576,
            ),
            spec(
                "fs.patch",
                "Apply a unified diff",
                SideEffect::Write,
                DEV_QA,
                &["path", "patch"],
                10,
                1_048_576,
            ),
            spec(
                "shell.run",
                "Run an allowlisted command",
                SideEffect::Write,
                DEV_QA,
                &["argv"],
                600,
                20_000,
            ),
            spec(
                "test.run",
                "Run the project test command",
                SideEffect::Write,
                DEV_QA,
                &[],
                900,
                20_000,
            ),
            spec(
                "git.status",
                "Read git status",
                SideEffect::Read,
                DEV_QA,
                &[],
                10,
                20_000,
            ),
            spec(
                "git.diff",
                "Read git diff",
                SideEffect::Read,
                DEV_QA,
                &[],
                10,
                20_000,
            ),
            spec(
                "git.commit",
                "Stage and commit with trailers",
                SideEffect::Write,
                DEV_QA,
                &["message"],
                30,
                20_000,
            ),
            spec(
                "git.push",
                "Push the current branch",
                SideEffect::External,
                DEV,
                &["branch"],
                30,
                20_000,
            ),
            spec(
                "scm.open_pr",
                "Create or update a PR",
                SideEffect::External,
                DEV,
                &["title"],
                30,
                20_000,
            ),
            spec(
                "scm.comment",
                "Comment on the card PR",
                SideEffect::External,
                ALL,
                &["body"],
                30,
                20_000,
            ),
            spec(
                "scm.review",
                "Submit a review verdict",
                SideEffect::External,
                QA,
                &["verdict"],
                30,
                20_000,
            ),
            spec(
                "scm.merge",
                "Merge after policy checks",
                SideEffect::External,
                LEAD,
                &["sha"],
                30,
                20_000,
            ),
            spec(
                "kanban.create_todo",
                "Create a TODO card",
                SideEffect::Write,
                ALL,
                &["title"],
                10,
                20_000,
            ),
            spec(
                "kanban.comment",
                "Comment on the own card",
                SideEffect::Write,
                ALL,
                &["body"],
                10,
                20_000,
            ),
            spec(
                "kanban.move_own",
                "Move the own card",
                SideEffect::Write,
                DEV_QA,
                &["to"],
                10,
                20_000,
            ),
            spec(
                "kanban.log",
                "Append a structured log",
                SideEffect::Write,
                ALL,
                &["message"],
                10,
                20_000,
            ),
            spec(
                "memory.search",
                "Scoped memory retrieval",
                SideEffect::Read,
                ALL,
                &["query"],
                30,
                20_000,
            ),
            spec(
                "memory.write_note",
                "Write a scoped note",
                SideEffect::Write,
                ALL,
                &["text"],
                10,
                8_192,
            ),
            spec(
                "llm.escalate",
                "Escalate to the reasoning model",
                SideEffect::External,
                ALL,
                &["reason"],
                10,
                20_000,
            ),
            spec(
                "office.speak",
                "Emit a speech bubble",
                SideEffect::Write,
                ALL,
                &["text"],
                5,
                240,
            ),
        ];
        Self { tools }
    }

    /// All tool specs.
    #[must_use]
    pub fn tools(&self) -> &[ToolSpec] {
        &self.tools
    }

    /// Look up a tool.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&ToolSpec> {
        self.tools.iter().find(|tool| tool.name == name)
    }

    /// Authorize a call by role, arguments, path and command policy.
    pub fn authorize(&self, role: AgentRole, tool: &str, args: &Value) -> Result<(), ToolDenial> {
        let Some(spec) = self.get(tool) else {
            return Err(ToolDenial::new(
                "UNKNOWN_TOOL",
                tool,
                "tool",
                "unknown tool",
            ));
        };
        if !spec.roles.contains(&role) {
            return Err(ToolDenial::new(
                "ROLE_NOT_ALLOWED",
                tool,
                "role",
                format!("tool is not allowed for {role:?}"),
            ));
        }
        if let Some(required) = spec.input_schema["required"].as_array() {
            for field in required {
                let Some(field) = field.as_str() else {
                    continue;
                };
                if args.get(field).is_none() {
                    return Err(ToolDenial::new(
                        "MISSING_ARGUMENT",
                        tool,
                        field,
                        "required argument is missing",
                    ));
                }
            }
        }
        for (field, expected) in [
            ("path", "string"),
            ("dir", "string"),
            ("content", "string"),
            ("query", "string"),
            ("patch", "string"),
            ("message", "string"),
            ("title", "string"),
            ("body", "string"),
            ("text", "string"),
            ("branch", "string"),
            ("sha", "string"),
            ("verdict", "string"),
            ("to", "string"),
            ("reason", "string"),
            ("argv", "array"),
        ] {
            if let Some(value) = args.get(field) {
                let ok = match expected {
                    "string" => value.is_string(),
                    "array" => value.is_array(),
                    _ => true,
                };
                if !ok {
                    return Err(ToolDenial::new(
                        "INVALID_ARGUMENT_TYPE",
                        tool,
                        field,
                        format!("{field} must be a {expected}"),
                    ));
                }
            }
        }
        if let Some(path) = args.get("path").and_then(Value::as_str) {
            check_path(tool, "path", path)?;
        }
        if let Some(dir) = args.get("dir").and_then(Value::as_str) {
            check_path(tool, "dir", dir)?;
        }
        if tool == "shell.run" {
            check_command(tool, args)?;
        }
        Ok(())
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::v1()
    }
}

fn check_path(tool: &str, argument: &str, path: &str) -> Result<(), ToolDenial> {
    let normalized = path.replace('\\', "/");
    let lowered = normalized.to_ascii_lowercase();
    let extension = std::path::Path::new(&lowered)
        .extension()
        .and_then(|extension| extension.to_str());
    let forbidden = normalized.starts_with('/')
        || normalized.contains("..")
        || normalized == ".git/hooks"
        || normalized.starts_with(".git/hooks/")
        || normalized.starts_with(".github/workflows/")
        || normalized == "CODEOWNERS"
        || lowered.starts_with(".env")
        || matches!(extension, Some("pem" | "key"))
        || normalized == "secrets"
        || normalized.starts_with("secrets/");
    if forbidden {
        return Err(ToolDenial::new(
            "PATH_FORBIDDEN",
            tool,
            argument,
            format!("path `{path}` is forbidden"),
        ));
    }
    Ok(())
}

const ALLOWED_COMMANDS: &[&str] = &[
    "cargo",
    "rustc",
    "cargo-clippy",
    "rustfmt",
    "npm",
    "npx",
    "node",
    "python",
    "python3",
    "pytest",
    "git",
    "rg",
    "ls",
    "cat",
    "sed",
    "jq",
    "make",
    "just",
    "patch",
    "diff",
];
const FORBIDDEN_BINARIES: &[&str] = &[
    "curl", "wget", "nc", "ssh", "scp", "docker", "sudo", "su", "sh", "bash",
];

fn check_command(tool: &str, args: &Value) -> Result<(), ToolDenial> {
    let command_args: Vec<&str> = args["argv"]
        .as_array()
        .map(|values| values.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    let Some(binary) = command_args.first() else {
        return Err(ToolDenial::new(
            "EMPTY_COMMAND",
            tool,
            "argv",
            "argv is empty",
        ));
    };
    if FORBIDDEN_BINARIES.contains(binary) {
        return Err(ToolDenial::new(
            "FORBIDDEN_COMMAND",
            tool,
            "argv",
            format!("command `{binary}` is forbidden"),
        ));
    }
    if !ALLOWED_COMMANDS.contains(binary) {
        return Err(ToolDenial::new(
            "COMMAND_NOT_ALLOWED",
            tool,
            "argv",
            format!("command `{binary}` is not allowlisted"),
        ));
    }
    for operand in command_args.iter().filter(|arg| !arg.starts_with('-')) {
        let lowered = operand.to_ascii_lowercase();
        let extension = std::path::Path::new(&lowered)
            .extension()
            .and_then(|extension| extension.to_str());
        let protected = lowered.starts_with(".env")
            || lowered.contains("/.env")
            || lowered.contains(".github/workflows")
            || lowered == "codeowners"
            || lowered.contains("/codeowners")
            || lowered.contains("secrets")
            || matches!(extension, Some("pem" | "key"))
            || lowered.contains(".git/hooks");
        if protected {
            return Err(ToolDenial::new(
                "PATH_FORBIDDEN",
                tool,
                "argv",
                format!("argument `{operand}` touches a protected path"),
            ));
        }
    }
    let joined: BTreeSet<&str> = command_args.iter().copied().collect();
    for denied in ["--force", "-f", "--hard", "--no-verify", "publish"] {
        if joined.contains(denied) {
            return Err(ToolDenial::new(
                "FORBIDDEN_COMMAND",
                tool,
                "argv",
                format!("argument `{denied}` is forbidden"),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_has_every_spec_tool() {
        let registry = ToolRegistry::v1();
        assert!(registry.tools().len() >= 22);
        for name in [
            "fs.read",
            "fs.write",
            "shell.run",
            "scm.open_pr",
            "scm.merge",
            "memory.search",
            "office.speak",
        ] {
            assert!(registry.get(name).is_some(), "missing {name}");
        }
    }

    #[test]
    fn role_allowlist_is_enforced() {
        let registry = ToolRegistry::v1();
        assert!(
            registry
                .authorize(
                    AgentRole::Dev,
                    "fs.write",
                    &json!({"path": "src/lib.rs", "content": "x"})
                )
                .is_ok()
        );
        assert_eq!(
            registry
                .authorize(
                    AgentRole::Analyst,
                    "fs.write",
                    &json!({"path": "a", "content": "x"})
                )
                .expect_err("denied")
                .rule,
            "ROLE_NOT_ALLOWED"
        );
        assert_eq!(
            registry
                .authorize(AgentRole::Dev, "scm.merge", &json!({"sha": "x"}))
                .expect_err("denied")
                .rule,
            "ROLE_NOT_ALLOWED"
        );
    }

    #[test]
    fn unknown_tools_and_missing_arguments_are_rejected() {
        let registry = ToolRegistry::v1();
        assert_eq!(
            registry
                .authorize(AgentRole::Dev, "nope", &json!({}))
                .expect_err("denied")
                .rule,
            "UNKNOWN_TOOL"
        );
        assert_eq!(
            registry
                .authorize(AgentRole::Dev, "fs.read", &json!({}))
                .expect_err("denied")
                .rule,
            "MISSING_ARGUMENT"
        );
    }

    #[test]
    fn path_policy_denies_secrets_and_protected_paths() {
        let registry = ToolRegistry::v1();
        for path in [
            ".env",
            ".env.local",
            "certs/server.pem",
            "key.key",
            ".github/workflows/ci.yml",
            "CODEOWNERS",
            ".git/hooks/pre-commit",
            "../escape",
            "/etc/passwd",
        ] {
            let denial = registry
                .authorize(
                    AgentRole::Dev,
                    "fs.write",
                    &json!({"path": path, "content": "x"}),
                )
                .expect_err(path);
            assert_eq!(denial.rule, "PATH_FORBIDDEN", "{path}");
        }
        assert!(
            registry
                .authorize(AgentRole::Dev, "fs.read", &json!({"path": "src/lib.rs"}))
                .is_ok()
        );
    }

    #[test]
    fn shell_run_denies_protected_argv_paths() {
        let registry = ToolRegistry::v1();
        for argv in [
            json!(["cat", ".env"]),
            json!(["sed", "-i", ".github/workflows/ci.yml"]),
            json!(["patch", "secrets/credentials.patch"]),
            json!(["cat", "certs/server.pem"]),
        ] {
            let denial = registry
                .authorize(AgentRole::Dev, "shell.run", &json!({"argv": argv}))
                .expect_err("denied");
            assert_eq!(denial.rule, "PATH_FORBIDDEN", "{argv}");
        }
    }

    #[test]
    fn malformed_inputs_are_rejected_without_panicking() {
        let registry = ToolRegistry::v1();
        let inputs = [
            json!({}),
            json!({"path": null}),
            json!({"path": ""}),
            json!({"path": "\u{0}"}),
            json!({"argv": []}),
            json!({"argv": [null]}),
            json!({"argv": "--force"}),
            json!({"text": 1}),
        ];
        for input in inputs {
            let _ = registry.authorize(AgentRole::Dev, "shell.run", &input);
            let _ = registry.authorize(AgentRole::Dev, "fs.read", &input);
            let _ = registry.authorize(AgentRole::Qa, "fs.write", &input);
        }
    }

    #[test]
    fn command_policy_allows_tests_and_denies_hosts_and_force_push() {
        let registry = ToolRegistry::v1();
        assert!(
            registry
                .authorize(
                    AgentRole::Dev,
                    "shell.run",
                    &json!({"argv": ["cargo", "test"]})
                )
                .is_ok()
        );
        for argv in [
            json!(["sudo", "rm", "-rf", "/"]),
            json!(["curl", "https://example.test"]),
            json!(["git", "push", "--force"]),
            json!(["cargo", "publish"]),
            json!(["git", "commit", "--no-verify"]),
        ] {
            assert_eq!(
                registry
                    .authorize(AgentRole::Dev, "shell.run", &json!({"argv": argv}))
                    .expect_err("denied")
                    .rule,
                "FORBIDDEN_COMMAND"
            );
        }
    }
}
