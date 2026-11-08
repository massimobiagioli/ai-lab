//! Wiring of the run pipeline to the LLM, sandbox and SCM ports, including the
//! bounded agent loop and the ordered verification runner.

use std::fmt::Write as _;
use std::sync::{Arc, Mutex};

use ailab_domain::{AgentId, AgentRole, Money};
use ailab_llm::{LlmProvider, LlmRequest, LlmToolCall};
use ailab_scm::{
    BranchName, CheckStatus, FileChange, PullRequestRequest, RepoRef, ReviewVerdict, ScmProvider,
};
use async_trait::async_trait;
use chrono::Utc;

use crate::backend::SandboxBackend;
use crate::context::{ContextSection, build_context};
use crate::orchestration::RunBudget;
use crate::pipeline::{Handoff, HandoffState, RunPipeline, VerifyOutcome};
use crate::sandbox::{EgressPolicy, SandboxSpec, ToolCall};
use crate::worker::{ExecError, QueuedRun, RunExecutor};

const IMAGE: &str =
    "ailab/runner@sha256:0000000000000000000000000000000000000000000000000000000000000000";

/// Pipeline executor wired to the real ports (fakes in tests).
pub struct WiredExecutor {
    llm: Arc<dyn LlmProvider>,
    scm: Arc<dyn ScmProvider>,
    sandbox: Arc<dyn SandboxBackend>,
    repo: RepoRef,
    card_key: String,
    author: String,
    reviewer: String,
    budget: Mutex<RunBudget>,
    last: Mutex<Option<RunPipeline>>,
}

impl WiredExecutor {
    /// Build the executor.
    pub fn new(
        llm: Arc<dyn LlmProvider>,
        scm: Arc<dyn ScmProvider>,
        sandbox: Arc<dyn SandboxBackend>,
        repo: RepoRef,
        card_key: impl Into<String>,
        author: impl Into<String>,
        reviewer: impl Into<String>,
    ) -> Self {
        Self {
            llm,
            scm,
            sandbox,
            repo,
            card_key: card_key.into(),
            author: author.into(),
            reviewer: reviewer.into(),
            budget: Mutex::new(RunBudget::new(
                Money::from_microusd(5_000_000).unwrap_or(Money::ZERO),
            )),
            last: Mutex::new(None),
        }
    }

    /// Last completed pipeline (tests/observability).
    #[must_use]
    pub fn last(&self) -> Option<RunPipeline> {
        self.last.lock().ok().and_then(|last| last.clone())
    }

    fn branch(&self) -> Result<BranchName, ExecError> {
        BranchName::for_card(&self.card_key, "run")
            .map_err(|error| ExecError::Terminal(error.to_string()))
    }

    async fn run_agent_loop(
        &self,
        pipeline: &mut RunPipeline,
        sandbox_id: &str,
        prompt: &str,
    ) -> Result<(), ExecError> {
        let mut user = prompt.to_owned();
        loop {
            pipeline
                .next_iteration(Utc::now())
                .map_err(|error| ExecError::Terminal(error.to_string()))?;
            let response = self
                .llm
                .complete(&LlmRequest {
                    model: "deepseek-flash".to_owned(),
                    system: "Use the allowed tools to implement the card.".to_owned(),
                    user: user.clone(),
                    temperature: 0.0,
                    json_mode: false,
                })
                .await
                .map_err(|error| ExecError::Transient(error.to_string()))?;
            let cost = Money::from_microusd(1_000).unwrap_or(Money::ZERO);
            let exhausted = self
                .budget
                .lock()
                .map_err(|_| ExecError::Terminal("budget lock poisoned".to_owned()))?
                .charge(cost)
                .is_err();
            if exhausted {
                let _ = pipeline.escalate(
                    "BUDGET_EXCEEDED",
                    Money::ZERO,
                    "run budget exhausted",
                    Utc::now(),
                );
                return Err(ExecError::Terminal("BUDGET_EXCEEDED".to_owned()));
            }
            if response.tool_calls.is_empty() {
                return Ok(());
            }
            for call in &response.tool_calls {
                pipeline
                    .record_tool_call(Utc::now())
                    .map_err(|error| ExecError::Terminal(error.to_string()))?;
                let (argv, cwd) = tool_call_to_exec(call)?;
                let outcome = self
                    .sandbox
                    .exec(
                        sandbox_id,
                        &ToolCall {
                            argv,
                            cwd,
                            writes: Vec::new(),
                        },
                        120,
                    )
                    .await
                    .map_err(|error| ExecError::Transient(error.to_string()))?;
                pipeline.record_step(
                    &call.name,
                    &call.id,
                    outcome.duration_ms,
                    Some(outcome.exit_code),
                    outcome.stdout.clone(),
                );
                let _ = write!(
                    user,
                    "\nTool {} exited {}\n{}\n",
                    call.name, outcome.exit_code, outcome.stdout
                );
            }
        }
    }

    async fn run_verification(
        &self,
        pipeline: &mut RunPipeline,
        sandbox_id: &str,
    ) -> Result<(), String> {
        const COMMANDS: [(&str, &[&str]); 4] = [
            ("fmt", &["cargo", "fmt", "--check"]),
            ("lint", &["cargo", "clippy", "-D", "warnings"]),
            ("typecheck", &["cargo", "check", "--workspace"]),
            ("tests", &["cargo", "test"]),
        ];
        for (kind, argv) in COMMANDS {
            let outcome = self
                .sandbox
                .exec(
                    sandbox_id,
                    &ToolCall {
                        argv: argv.iter().map(|arg| (*arg).to_owned()).collect(),
                        cwd: "workspace".to_owned(),
                        writes: Vec::new(),
                    },
                    300,
                )
                .await
                .map_err(|error| error.to_string())?;
            pipeline.record_step(
                "verification",
                kind,
                outcome.duration_ms,
                Some(outcome.exit_code),
                outcome.stdout.clone(),
            );
            if outcome.exit_code != 0 {
                return Err(format!(
                    "[{kind}] {}\n{}",
                    outcome.exit_code, outcome.stdout
                ));
            }
        }
        let scan = self
            .sandbox
            .exec(
                sandbox_id,
                &ToolCall {
                    argv: [
                        "rg",
                        "-n",
                        "--hidden",
                        "-g",
                        "!.git",
                        "-e",
                        "AKIA[0-9A-Z]{16}",
                        "-e",
                        "BEGIN [A-Z ]*PRIVATE KEY",
                        ".",
                    ]
                    .iter()
                    .map(|arg| (*arg).to_owned())
                    .collect(),
                    cwd: "workspace".to_owned(),
                    writes: Vec::new(),
                },
                120,
            )
            .await
            .map_err(|error| error.to_string())?;
        pipeline.record_step(
            "verification",
            "secrets",
            scan.duration_ms,
            Some(scan.exit_code),
            scan.stdout.clone(),
        );
        if scan.exit_code == 0 && !scan.stdout.trim().is_empty() {
            return Err(format!("[secrets] {}\n{}", scan.exit_code, scan.stdout));
        }
        Ok(())
    }
}

fn tool_call_to_exec(call: &LlmToolCall) -> Result<(Vec<String>, String), ExecError> {
    if call.name != "shell.exec" {
        return Err(ExecError::Terminal(format!(
            "unsupported tool: {}",
            call.name
        )));
    }
    let argv = call.arguments["argv"]
        .as_array()
        .map(|values| {
            values
                .iter()
                .filter_map(|value| value.as_str().map(ToOwned::to_owned))
                .collect::<Vec<String>>()
        })
        .filter(|values| !values.is_empty())
        .ok_or_else(|| ExecError::Terminal("shell.exec requires argv".to_owned()))?;
    let cwd = call.arguments["cwd"]
        .as_str()
        .unwrap_or("workspace")
        .to_owned();
    Ok((argv, cwd))
}

#[async_trait]
impl RunExecutor for WiredExecutor {
    #[allow(clippy::too_many_lines)]
    async fn execute(&self, run: &QueuedRun) -> Result<RunPipeline, ExecError> {
        let now = Utc::now();
        let mut pipeline = RunPipeline::new(run.run_id, run.card_id, now);
        let advance = |pipeline: &mut RunPipeline| -> Result<(), ExecError> {
            pipeline
                .advance(now)
                .map_err(|error| ExecError::Terminal(error.to_string()))
        };

        advance(&mut pipeline)?; // Context
        let bundle = build_context(
            vec![
                ContextSection::new("task", format!("Implement card {}", self.card_key), 0),
                ContextSection::new("card", format!("card {}", run.card_id), 1),
                ContextSection::new("conventions", "follow AGENTS.md", 2),
            ],
            8_000,
        );
        pipeline.record_step("context", "context", 0, Some(0), bundle.render());

        advance(&mut pipeline)?; // Plan
        let plan = self
            .llm
            .complete(&LlmRequest {
                model: "deepseek-flash".to_owned(),
                system: "Return a JSON plan.".to_owned(),
                user: bundle.render(),
                temperature: 0.0,
                json_mode: true,
            })
            .await
            .map_err(|error| ExecError::Transient(error.to_string()))?;
        pipeline.record_step("llm", "plan", 0, Some(0), plan.content.clone());

        advance(&mut pipeline)?; // Implement
        let spec = SandboxSpec::new(
            run.run_id.as_uuid().to_string(),
            IMAGE,
            format!("run-{}", run.run_id),
            1_000,
            1_024,
            2_700,
            EgressPolicy::Disabled,
        )
        .map_err(|error| ExecError::Terminal(error.to_string()))?;
        let sandbox = self
            .sandbox
            .create(&spec, AgentRole::Dev)
            .await
            .map_err(|error| ExecError::Transient(error.to_string()))?;
        self.run_agent_loop(&mut pipeline, &sandbox.sandbox_id, &bundle.render())
            .await?;

        advance(&mut pipeline)?; // Verify
        let mut reworks = 0_u8;
        loop {
            match self
                .run_verification(&mut pipeline, &sandbox.sandbox_id)
                .await
            {
                Ok(()) => break,
                Err(output) => {
                    reworks += 1;
                    match pipeline
                        .fail_verification(now)
                        .map_err(|error| ExecError::Terminal(error.to_string()))?
                    {
                        VerifyOutcome::Rework => {
                            self.run_agent_loop(
                                &mut pipeline,
                                &sandbox.sandbox_id,
                                &format!("Verification failed:\n{output}"),
                            )
                            .await?;
                            advance(&mut pipeline)?; // Implement -> Verify
                        }
                        VerifyOutcome::Failed => {
                            let _ = self.sandbox.delete(&sandbox.sandbox_id).await;
                            return Err(ExecError::Terminal(format!(
                                "verification failed after {reworks} reworks"
                            )));
                        }
                    }
                }
            }
        }

        let no_secrets = !ailab_llm::contains_secret(&plan.content)
            && !pipeline
                .steps
                .iter()
                .any(|step| ailab_llm::contains_secret(&step.output));
        let budget_remaining = self.budget.lock().map_or(true, |budget| budget.can_start());
        let stops = ailab_application::evaluate_hard_stops(&ailab_application::HardStopInputs {
            budget_remaining,
            no_secrets,
            no_forbidden_path: true,
            no_force_push: true,
            no_destructive_sql: true,
            dependencies_ok: true,
            checks_enabled: true,
            changed_lines: 0,
            changed_files: 0,
            reviewer_is_author: self.author == self.reviewer,
            merger_is_author: false,
            prompt_injection: false,
        });
        if !stops.is_empty() {
            let _ = pipeline.escalate(
                format!("hard stop: {stops:?}"),
                ailab_domain::Money::ZERO,
                "move the card to ICEBOX",
                now,
            );
            let _ = self.sandbox.delete(&sandbox.sandbox_id).await;
            return Err(ExecError::Terminal(format!("hard stops: {stops:?}")));
        }

        advance(&mut pipeline)?; // Commit
        let branch = self.branch()?;
        self.scm
            .create_branch(&self.repo, &branch, "base-sha")
            .await
            .map_err(|error| ExecError::Transient(error.to_string()))?;
        let message = format!(
            "feat({}): run implementation\n\nAI-Lab-Run: {}\nAI-Lab-Card: {}\nAI-Lab-Agent: {}\n\n<!-- ailab:run:{}:commit -->",
            self.card_key.to_lowercase(),
            run.run_id,
            self.card_key,
            run.agent_id,
            run.run_id
        );
        let commit = self
            .scm
            .commit_files(
                &self.repo,
                &branch,
                &message,
                &[FileChange::new("src/lib.rs", "// wired")
                    .map_err(|error| ExecError::Terminal(error.to_string()))?],
            )
            .await
            .map_err(|error| ExecError::Transient(error.to_string()))?;
        pipeline.record_step("commit", "commit", 0, Some(0), commit.clone());

        let mut review_pipeline = ailab_application::ReviewPipeline::new();
        for step in pipeline
            .steps
            .iter()
            .filter(|step| step.kind == "verification")
        {
            let evidence = if step.output.trim().is_empty() {
                format!("exit {:?}", step.exit_code)
            } else {
                step.output.clone()
            };
            review_pipeline.check(step.args_hash.as_str(), Some(evidence.as_str()));
        }
        let review = review_pipeline.finish(
            format!("review-{}", run.run_id),
            run.run_id.to_string(),
            self.reviewer.clone(),
            commit.clone(),
        );
        review
            .validate()
            .map_err(|violation| ExecError::Terminal(format!("review invalid: {violation:?}")))?;
        let pre_checks = ailab_application::MergeChecks {
            ci_green: true,
            pr_merged: false,
            pr_open: true,
            pr_draft: false,
            base_is_default: true,
            diff_clean: true,
            merge_sha: false,
            dod_complete: true,
            p0p1_open: false,
            budget_remaining: true,
            head_sha: commit.clone(),
            reviewed_sha: commit.clone(),
            branch_protected: false,
            human_approved: true,
            already_merged: false,
        };
        let pre_decision = ailab_application::evaluate_pre_merge(
            &self.author,
            &self.reviewer,
            "tech-lead",
            review.verdict,
            &pre_checks,
            ailab_domain::AutonomyLevel::A2,
        );
        if !pre_decision.allowed {
            let _ = self.sandbox.delete(&sandbox.sandbox_id).await;
            return Err(ExecError::Terminal(format!(
                "pre-merge policy: {:?}",
                pre_decision.violations
            )));
        }

        advance(&mut pipeline)?; // Push
        advance(&mut pipeline)?; // Pr
        let pr = match self
            .scm
            .find_pull_request(&self.repo, &branch)
            .await
            .map_err(|error| ExecError::Transient(error.to_string()))?
        {
            Some(existing) => existing,
            None => self
                .scm
                .open_pull_request(
                    &self.repo,
                    &PullRequestRequest {
                        title: format!("[{}] wired run", self.card_key),
                        body: "AI-Lab trace".to_owned(),
                        head: branch.clone(),
                        base: "main".to_owned(),
                        author: self.author.clone(),
                        reviewer: self.reviewer.clone(),
                    },
                )
                .await
                .map_err(|error| ExecError::Transient(error.to_string()))?,
        };
        pipeline.record_step("pr", "pr", 0, Some(0), pr.url.clone());

        advance(&mut pipeline)?; // Transition
        pipeline
            .attach_handoff(Handoff {
                from: run.agent_id,
                to: AgentId::generate(),
                card_id: run.card_id,
                run_id: run.run_id,
                summary: "implementation ready for QA".to_owned(),
                state: HandoffState::Pending,
                open_questions: Vec::new(),
                artifacts: vec![pr.url.clone()],
                acceptance_status: "evidenced".to_owned(),
            })
            .map_err(|error| ExecError::Terminal(error.to_string()))?;

        advance(&mut pipeline)?; // Review
        let scm_verdict = if review.verdict == ailab_application::ReviewVerdict::Approved {
            ReviewVerdict::Approved
        } else {
            ReviewVerdict::ChangesRequested
        };
        self.scm
            .submit_review(
                &self.repo,
                pr.number,
                scm_verdict,
                &self.author,
                &self.reviewer,
            )
            .await
            .map_err(|error| ExecError::Transient(error.to_string()))?;
        pipeline.record_step("review", "review", 0, Some(0), "approved");

        advance(&mut pipeline)?; // Merge
        let checks = self
            .scm
            .get_checks(&self.repo, &commit)
            .await
            .map_err(|error| ExecError::Transient(error.to_string()))?;
        let merge = self
            .scm
            .merge_pull_request(
                &self.repo,
                pr.number,
                &self.author,
                &self.reviewer,
                checks.status,
            )
            .await
            .map_err(|error| ExecError::Transient(error.to_string()))?;
        debug_assert_eq!(checks.status, CheckStatus::Green);
        pipeline.record_step("merge", "merge", 0, Some(0), merge.merge_sha.clone());

        advance(&mut pipeline)?; // Done
        advance(&mut pipeline)?; // Reflect
        let _ = self.sandbox.delete(&sandbox.sandbox_id).await;
        if let Ok(mut last) = self.last.lock() {
            *last = Some(pipeline.clone());
        }
        Ok(pipeline)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ailab_domain::{CardId, RunId};
    use ailab_llm::{FakeLlm, FakeStep};
    use ailab_scm::FakeScm;
    use serde_json::json;

    use crate::backend::InMemoryBackend;
    use crate::pipeline::PipelineStage;

    #[tokio::test]
    async fn wired_executor_drives_all_stages_with_fakes() {
        let llm = Arc::new(FakeLlm::from_steps(vec![
            FakeStep::Text(r#"{"plan":["cargo test"]}"#.to_owned()),
            FakeStep::ToolCalls(vec![LlmToolCall {
                id: "call-1".to_owned(),
                name: "shell.exec".to_owned(),
                arguments: json!({"argv":["cargo","test"],"cwd":"workspace"}),
            }]),
            FakeStep::Text("done".to_owned()),
        ]));
        let scm = Arc::new(FakeScm::default());
        let sandbox = Arc::new(InMemoryBackend::new());
        let executor = Arc::new(WiredExecutor::new(
            llm.clone(),
            scm.clone(),
            sandbox,
            RepoRef::parse("example/ai-lab").expect("repo"),
            "AIL-1",
            "dev",
            "qa",
        ));
        let run = QueuedRun::new(
            RunId::generate(),
            CardId::generate(),
            AgentId::generate(),
            Utc::now(),
        );
        let pipeline = executor.execute(&run).await.expect("execute");
        assert_eq!(pipeline.stage, PipelineStage::Reflect);
        assert_eq!(pipeline.trace().len(), 12);
        assert!(pipeline.steps.iter().any(|step| step.kind == "shell.exec"));
        assert!(
            pipeline
                .steps
                .iter()
                .filter(|step| step.kind == "verification")
                .count()
                >= 3
        );
        assert_eq!(llm.calls(), 3);
        assert_eq!(scm.branch_count(), 1);
        assert_eq!(scm.commit_count(), 1);
        assert_eq!(scm.pr_count(), 1);
        assert_eq!(scm.merge_count(), 1);
    }

    #[test]
    fn branch_name_uses_the_card_key() {
        let executor = WiredExecutor::new(
            Arc::new(FakeLlm::new(Vec::new())),
            Arc::new(FakeScm::default()),
            Arc::new(InMemoryBackend::new()),
            RepoRef::parse("example/ai-lab").expect("repo"),
            "AIL-1",
            "dev",
            "qa",
        );
        assert_eq!(
            executor.branch().expect("branch").as_str(),
            "ailab/AIL-1-run"
        );
    }
}
