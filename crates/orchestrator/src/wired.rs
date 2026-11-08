//! Wiring of the run pipeline to the LLM, sandbox and SCM ports.

use std::sync::{Arc, Mutex};

use ailab_domain::AgentId;
use ailab_llm::{LlmProvider, LlmRequest};
use ailab_scm::{
    BranchName, CheckStatus, FileChange, PullRequestRequest, RepoRef, ReviewVerdict, ScmProvider,
};
use async_trait::async_trait;
use chrono::Utc;

use crate::backend::SandboxBackend;
use crate::context::{ContextSection, build_context};
use crate::pipeline::{Handoff, HandoffState, RunPipeline};
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
            ],
            8_000,
        );
        pipeline.record_step("context", "context", 0, Some(0), bundle.render());

        advance(&mut pipeline)?; // Plan
        let request = LlmRequest {
            model: "deepseek-flash".to_owned(),
            system: "Return a JSON plan.".to_owned(),
            user: bundle.render(),
            temperature: 0.0,
            json_mode: true,
        };
        let plan = self
            .llm
            .complete(&request)
            .await
            .map_err(|error| ExecError::Transient(error.to_string()))?;
        pipeline.record_step("llm", "plan", 0, Some(0), plan.content.clone());

        advance(&mut pipeline)?; // Implement
        let spec = SandboxSpec::new(
            run.run_id.as_uuid().to_string(),
            IMAGE,
            format!("run-{}", run.run_id),
            1_000,
            512,
            900,
            EgressPolicy::Disabled,
        )
        .map_err(|error| ExecError::Terminal(error.to_string()))?;
        let sandbox = self
            .sandbox
            .create(&spec, ailab_domain::AgentRole::Dev)
            .await
            .map_err(|error| ExecError::Transient(error.to_string()))?;
        let exec = ToolCall {
            argv: vec!["cargo".to_owned(), "test".to_owned()],
            cwd: "workspace".to_owned(),
            writes: vec![],
        };
        let outcome = self
            .sandbox
            .exec(&sandbox.sandbox_id, &exec, 120)
            .await
            .map_err(|error| ExecError::Transient(error.to_string()))?;
        pipeline.record_step(
            "tool",
            "test",
            outcome.duration_ms,
            Some(outcome.exit_code),
            outcome.stdout.clone(),
        );

        advance(&mut pipeline)?; // Verify
        let verify = self
            .sandbox
            .exec(&sandbox.sandbox_id, &exec, 120)
            .await
            .map_err(|error| ExecError::Transient(error.to_string()))?;
        if verify.exit_code != 0 {
            let _ = self.sandbox.delete(&sandbox.sandbox_id).await;
            return Err(ExecError::Terminal("verification failed".to_owned()));
        }
        pipeline.record_step(
            "verification",
            "verify",
            verify.duration_ms,
            Some(verify.exit_code),
            verify.stdout,
        );

        advance(&mut pipeline)?; // Commit
        let branch = self.branch()?;
        self.scm
            .create_branch(&self.repo, &branch, "base-sha")
            .await
            .map_err(|error| ExecError::Transient(error.to_string()))?;
        let message = format!(
            "feat({}): run implementation\n\nAI-Lab-Run: {}\nAI-Lab-Card: {}\nAI-Lab-Agent: {}",
            self.card_key.to_lowercase(),
            run.run_id,
            self.card_key,
            run.agent_id
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

        advance(&mut pipeline)?; // Push
        advance(&mut pipeline)?; // Pr
        let pr = self
            .scm
            .open_pull_request(
                &self.repo,
                &PullRequestRequest {
                    title: format!("[{}] wired run", self.card_key),
                    body: "AI-Lab trace".to_owned(),
                    head: branch,
                    base: "main".to_owned(),
                    author: self.author.clone(),
                    reviewer: self.reviewer.clone(),
                },
            )
            .await
            .map_err(|error| ExecError::Transient(error.to_string()))?;
        pipeline.record_step("pr", "pr", 0, Some(0), pr.url.clone());

        advance(&mut pipeline)?; // Transition
        let handoff = Handoff {
            from: run.agent_id,
            to: AgentId::generate(),
            card_id: run.card_id,
            run_id: run.run_id,
            summary: "implementation ready for QA".to_owned(),
            state: HandoffState::Pending,
            open_questions: Vec::new(),
            artifacts: vec![pr.url.clone()],
            acceptance_status: "evidenced".to_owned(),
        };
        pipeline
            .attach_handoff(handoff)
            .map_err(|error| ExecError::Terminal(error.to_string()))?;

        advance(&mut pipeline)?; // Review
        self.scm
            .submit_review(
                &self.repo,
                pr.number,
                ReviewVerdict::Approved,
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
        pipeline.record_step("merge", "merge", 0, Some(0), merge.merge_sha);

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
    use std::sync::Arc;

    use ailab_domain::{CardId, RunId};
    use ailab_llm::FakeLlm;
    use ailab_scm::FakeScm;

    use crate::backend::InMemoryBackend;
    use crate::orchestration::LeaseManager;
    use crate::pipeline::PipelineStage;
    use crate::worker::Worker;

    #[tokio::test]
    async fn wired_executor_drives_all_stages_with_fakes() {
        let llm = Arc::new(FakeLlm::new(vec![r#"{"plan":["cargo test"]}"#.to_owned()]));
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
        let worker = Worker::new(Arc::new(LeaseManager::new(900)), executor.clone(), 1);
        worker
            .enqueue(QueuedRun::new(
                RunId::generate(),
                CardId::generate(),
                AgentId::generate(),
                Utc::now(),
            ))
            .expect("enqueue");
        let report = worker.tick(Utc::now()).await.expect("tick");
        assert_eq!(report.completed, 1);
        let pipeline = executor.last().expect("pipeline");
        assert_eq!(pipeline.stage, PipelineStage::Reflect);
        assert_eq!(pipeline.trace().len(), 12);
        assert!(pipeline.steps.len() >= 6);
        assert_eq!(llm.calls(), 1);
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
