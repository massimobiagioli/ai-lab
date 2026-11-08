//! Normative run pipeline, loop bounds, handoff, recovery and escalation.
//!
//! The pipeline is a pure state machine: I/O (LLM, sandbox, SCM) is performed
//! by adapters, while this module enforces stage order, budgets, loop bounds,
//! cancellation checkpoints, idempotency keys and crash recovery.

use chrono::{DateTime, Duration, Utc};
use thiserror::Error;

use ailab_domain::{AgentId, CardId, Money, RunId};

/// The 13 normative pipeline stages from WP-13 §3.3.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineStage {
    /// 1. Lease.
    Lease,
    /// 2. Context.
    Context,
    /// 3. Plan.
    Plan,
    /// 4. Implement.
    Implement,
    /// 5. Verify.
    Verify,
    /// 6. Commit.
    Commit,
    /// 7. Push.
    Push,
    /// 8. Pull request.
    Pr,
    /// 9. Card transition.
    Transition,
    /// 10. Review.
    Review,
    /// 11. Merge.
    Merge,
    /// 12. Done.
    Done,
    /// 13. Reflect.
    Reflect,
}

impl PipelineStage {
    /// All stages in order.
    pub const ALL: [Self; 13] = [
        Self::Lease,
        Self::Context,
        Self::Plan,
        Self::Implement,
        Self::Verify,
        Self::Commit,
        Self::Push,
        Self::Pr,
        Self::Transition,
        Self::Review,
        Self::Merge,
        Self::Done,
        Self::Reflect,
    ];

    /// The next linear stage.
    #[must_use]
    pub const fn next(self) -> Option<Self> {
        match self {
            Self::Lease => Some(Self::Context),
            Self::Context => Some(Self::Plan),
            Self::Plan => Some(Self::Implement),
            Self::Implement => Some(Self::Verify),
            Self::Verify => Some(Self::Commit),
            Self::Commit => Some(Self::Push),
            Self::Push => Some(Self::Pr),
            Self::Pr => Some(Self::Transition),
            Self::Transition => Some(Self::Review),
            Self::Review => Some(Self::Merge),
            Self::Merge => Some(Self::Done),
            Self::Done => Some(Self::Reflect),
            Self::Reflect => None,
        }
    }

    /// Catalog event emitted when the stage starts.
    #[must_use]
    pub const fn event(self) -> &'static str {
        match self {
            Self::Lease => "delivery.run.started",
            Self::Done => "delivery.run.completed",
            _ => "delivery.run.step",
        }
    }

    /// Stable name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Lease => "lease",
            Self::Context => "context",
            Self::Plan => "plan",
            Self::Implement => "implement",
            Self::Verify => "verify",
            Self::Commit => "commit",
            Self::Push => "push",
            Self::Pr => "pr",
            Self::Transition => "transition",
            Self::Review => "review",
            Self::Merge => "merge",
            Self::Done => "done",
            Self::Reflect => "reflect",
        }
    }
}

/// Per-stage time budgets in seconds (WP-13 §3.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::struct_field_names)]
pub struct StageBudget {
    /// Context.
    pub context_s: i64,
    /// Plan.
    pub plan_s: i64,
    /// Implement.
    pub implement_s: i64,
    /// Verify.
    pub verify_s: i64,
    /// Commit/push/PR.
    pub vcs_s: i64,
    /// Review.
    pub review_s: i64,
    /// Merge.
    pub merge_s: i64,
    /// Total wall clock.
    pub total_s: i64,
}

impl Default for StageBudget {
    fn default() -> Self {
        Self {
            context_s: 20,
            plan_s: 120,
            implement_s: 35 * 60,
            verify_s: 10 * 60,
            vcs_s: 5 * 60,
            review_s: 20 * 60,
            merge_s: 5 * 60,
            total_s: 45 * 60,
        }
    }
}

impl StageBudget {
    /// Budget for one stage.
    #[must_use]
    pub const fn for_stage(self, stage: PipelineStage) -> i64 {
        match stage {
            PipelineStage::Context => self.context_s,
            PipelineStage::Plan => self.plan_s,
            PipelineStage::Implement => self.implement_s,
            PipelineStage::Verify => self.verify_s,
            PipelineStage::Commit | PipelineStage::Push | PipelineStage::Pr => self.vcs_s,
            PipelineStage::Review => self.review_s,
            PipelineStage::Merge => self.merge_s,
            PipelineStage::Lease
            | PipelineStage::Transition
            | PipelineStage::Done
            | PipelineStage::Reflect => self.total_s,
        }
    }
}

/// Loop bounds (WP-13 §3.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::struct_field_names)]
pub struct LoopLimits {
    /// Maximum tool calls.
    pub max_tool_calls: u32,
    /// Maximum LLM iterations.
    pub max_iterations: u32,
    /// Maximum verification reworks.
    pub max_verify_loops: u8,
    /// Maximum review reworks.
    pub max_review_loops: u8,
}

impl Default for LoopLimits {
    fn default() -> Self {
        Self {
            max_tool_calls: 40,
            max_iterations: 25,
            max_verify_loops: 3,
            max_review_loops: 3,
        }
    }
}

/// Pipeline failure.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PipelineError {
    /// A stage was entered out of order.
    #[error("invalid stage transition: {from} -> {to}")]
    InvalidStage {
        /// Current stage.
        from: &'static str,
        /// Requested stage.
        to: &'static str,
    },
    /// The stage budget elapsed.
    #[error("stage budget exceeded: {0}")]
    StageTimeout(&'static str),
    /// The total wall clock elapsed.
    #[error("run wall clock exceeded")]
    RunTimeout,
    /// Too many tool calls.
    #[error("tool call budget exceeded")]
    ToolCallsExceeded,
    /// Too many iterations.
    #[error("iteration budget exceeded")]
    IterationsExceeded,
    /// Verification rework budget exhausted.
    #[error("verification loops exhausted")]
    VerificationExhausted,
    /// Review rework budget exhausted.
    #[error("review loops exhausted")]
    ReviewExhausted,
    /// Cancellation was requested.
    #[error("run cancelled")]
    Cancelled,
    /// The stage is not valid for this operation.
    #[error("wrong stage for operation: {0}")]
    WrongStage(&'static str),
}

/// One recorded stage transition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageRecord {
    /// Stage.
    pub stage: PipelineStage,
    /// Started at.
    pub started_at: DateTime<Utc>,
    /// Finished at, when complete.
    pub finished_at: Option<DateTime<Utc>>,
    /// Whether the stage succeeded.
    pub success: bool,
}

/// Verification rework outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerifyOutcome {
    /// Return to Implement with feedback.
    Rework,
    /// Verification failed terminally.
    Failed,
}

/// Review rework outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewOutcome {
    /// Return to Implement with findings.
    Rework,
    /// Escalate to ICEBOX after too many loops.
    Escalate,
}

/// Handoff state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandoffState {
    /// Waiting for QA.
    Pending,
    /// QA approved.
    Approved,
    /// QA requested changes.
    ChangesRequested,
}

/// Structured handoff object (WP-13 §3.8).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Handoff {
    /// From agent.
    pub from: AgentId,
    /// To QA agent.
    pub to: AgentId,
    /// Card.
    pub card_id: CardId,
    /// Run.
    pub run_id: RunId,
    /// Summary.
    pub summary: String,
    /// State.
    pub state: HandoffState,
    /// Open questions.
    pub open_questions: Vec<String>,
    /// Artifacts.
    pub artifacts: Vec<String>,
    /// Acceptance status.
    pub acceptance_status: String,
}

/// External PR state used for crash recovery.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExternalPrState {
    /// The PR is merged.
    Merged,
    /// The PR is open.
    Open,
    /// A branch exists but no PR.
    BranchOnly,
    /// No branch or PR.
    Absent,
}

/// Recovery action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryAction {
    /// Complete the run/card.
    Done,
    /// Resume at review/merge.
    ResumeReview,
    /// Resume at push/PR.
    ResumePush,
    /// Retry from implement.
    Retry,
}

impl ExternalPrState {
    /// Map external state to the recovery action (WP-13 §3.9).
    #[must_use]
    pub const fn recovery(self) -> RecoveryAction {
        match self {
            Self::Merged => RecoveryAction::Done,
            Self::Open => RecoveryAction::ResumeReview,
            Self::BranchOnly => RecoveryAction::ResumePush,
            Self::Absent => RecoveryAction::Retry,
        }
    }
}

/// Escalation record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Escalation {
    /// Card.
    pub card_id: CardId,
    /// Run.
    pub run_id: RunId,
    /// Reason.
    pub reason: String,
    /// Last 20 stage steps.
    pub last_steps: Vec<StageRecord>,
    /// Cost so far.
    pub cost: Money,
    /// Proposed next action.
    pub proposed_action: String,
    /// When it was raised.
    pub at: DateTime<Utc>,
}

/// A human intervention decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HumanDecision {
    /// Actor label.
    pub actor: String,
    /// Decision.
    pub decision: String,
    /// When.
    pub at: DateTime<Utc>,
}

/// One recorded tool/LLM step (WP-13 §3.5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunStep {
    /// Sequence within the run.
    pub seq: u32,
    /// Stage.
    pub stage: PipelineStage,
    /// Step kind (tool, llm, verification).
    pub kind: String,
    /// Hash of the arguments.
    pub args_hash: String,
    /// Duration in milliseconds.
    pub duration_ms: u64,
    /// Exit status when applicable.
    pub exit_code: Option<i32>,
    /// Truncated output (≤ 20 KB).
    pub output: String,
    /// Trace id.
    pub trace_id: String,
}

/// Run pipeline state machine.
#[derive(Debug, Clone)]
pub struct RunPipeline {
    /// Run.
    pub run_id: RunId,
    /// Card.
    pub card_id: CardId,
    /// Trace id shared by LLM/SCM/sandbox spans.
    pub trace_id: String,
    /// Current stage.
    pub stage: PipelineStage,
    /// Run start.
    pub started_at: DateTime<Utc>,
    /// Current stage start.
    pub stage_started_at: DateTime<Utc>,
    /// Stage records.
    pub records: Vec<StageRecord>,
    /// Tool/LLM steps.
    pub steps: Vec<RunStep>,
    /// Tool calls used.
    pub tool_calls: u32,
    /// LLM iterations used.
    pub iterations: u32,
    /// Verification reworks used.
    pub verify_loops: u8,
    /// Review reworks used.
    pub review_loops: u8,
    /// Cancellation requested.
    pub cancelled: bool,
    /// Handoff, once the card reaches review.
    pub handoff: Option<Handoff>,
    /// Escalation, once raised.
    pub escalation: Option<Escalation>,
    /// Human decisions.
    pub human_decisions: Vec<HumanDecision>,
    /// Limits.
    pub limits: LoopLimits,
    /// Budget.
    pub budget: StageBudget,
}

impl RunPipeline {
    /// Start a run at the lease stage.
    #[must_use]
    pub fn new(run_id: RunId, card_id: CardId, now: DateTime<Utc>) -> Self {
        Self {
            run_id,
            card_id,
            trace_id: format!("trace:{}", run_id.as_uuid()),
            stage: PipelineStage::Lease,
            started_at: now,
            stage_started_at: now,
            records: Vec::new(),
            steps: Vec::new(),
            tool_calls: 0,
            iterations: 0,
            verify_loops: 0,
            review_loops: 0,
            cancelled: false,
            handoff: None,
            escalation: None,
            human_decisions: Vec::new(),
            limits: LoopLimits::default(),
            budget: StageBudget::default(),
        }
    }

    /// Enter the next linear stage.
    pub fn advance(&mut self, now: DateTime<Utc>) -> Result<(), PipelineError> {
        self.checkpoint(now)?;
        let Some(next) = self.stage.next() else {
            return Err(PipelineError::InvalidStage {
                from: self.stage.as_str(),
                to: "end",
            });
        };
        self.finish_current(now, true);
        self.stage = next;
        self.stage_started_at = now;
        self.records.push(StageRecord {
            stage: next,
            started_at: now,
            finished_at: None,
            success: false,
        });
        Ok(())
    }

    fn finish_current(&mut self, now: DateTime<Utc>, success: bool) {
        if let Some(record) = self
            .records
            .iter_mut()
            .rev()
            .find(|record| record.stage == self.stage && record.finished_at.is_none())
        {
            record.finished_at = Some(now);
            record.success = success;
        }
    }

    /// Enforce the wall clock and stage budget.
    pub fn checkpoint(&self, now: DateTime<Utc>) -> Result<(), PipelineError> {
        if self.cancelled {
            return Err(PipelineError::Cancelled);
        }
        if (now - self.started_at).num_seconds() > self.budget.total_s {
            return Err(PipelineError::RunTimeout);
        }
        if (now - self.stage_started_at).num_seconds() > self.budget.for_stage(self.stage) {
            return Err(PipelineError::StageTimeout(self.stage.as_str()));
        }
        Ok(())
    }

    /// Record one tool call.
    pub fn record_tool_call(&mut self, now: DateTime<Utc>) -> Result<(), PipelineError> {
        self.checkpoint(now)?;
        self.tool_calls += 1;
        if self.tool_calls > self.limits.max_tool_calls {
            return Err(PipelineError::ToolCallsExceeded);
        }
        Ok(())
    }

    /// Record one LLM iteration.
    pub fn next_iteration(&mut self, now: DateTime<Utc>) -> Result<(), PipelineError> {
        self.checkpoint(now)?;
        self.iterations += 1;
        if self.iterations > self.limits.max_iterations {
            return Err(PipelineError::IterationsExceeded);
        }
        Ok(())
    }

    /// Record a verification failure and decide rework/failure.
    pub fn fail_verification(
        &mut self,
        now: DateTime<Utc>,
    ) -> Result<VerifyOutcome, PipelineError> {
        if self.stage != PipelineStage::Verify {
            return Err(PipelineError::WrongStage("verification"));
        }
        self.verify_loops += 1;
        if self.verify_loops > self.limits.max_verify_loops {
            self.finish_current(now, false);
            return Ok(VerifyOutcome::Failed);
        }
        self.finish_current(now, false);
        self.stage = PipelineStage::Implement;
        self.stage_started_at = now;
        self.records.push(StageRecord {
            stage: PipelineStage::Implement,
            started_at: now,
            finished_at: None,
            success: false,
        });
        Ok(VerifyOutcome::Rework)
    }

    /// Record a changes-requested review and decide rework/escalation.
    pub fn request_changes(&mut self, now: DateTime<Utc>) -> Result<ReviewOutcome, PipelineError> {
        if self.stage != PipelineStage::Review {
            return Err(PipelineError::WrongStage("review"));
        }
        self.review_loops += 1;
        if self.review_loops > self.limits.max_review_loops {
            return Ok(ReviewOutcome::Escalate);
        }
        self.finish_current(now, false);
        self.stage = PipelineStage::Implement;
        self.stage_started_at = now;
        self.records.push(StageRecord {
            stage: PipelineStage::Implement,
            started_at: now,
            finished_at: None,
            success: false,
        });
        Ok(ReviewOutcome::Rework)
    }

    /// Attach the QA handoff at the transition stage.
    pub fn attach_handoff(&mut self, handoff: Handoff) -> Result<(), PipelineError> {
        if self.stage != PipelineStage::Transition {
            return Err(PipelineError::WrongStage("handoff"));
        }
        self.handoff = Some(handoff);
        Ok(())
    }

    /// Record one tool/LLM step, truncating output to 20 KB.
    pub fn record_step(
        &mut self,
        kind: impl Into<String>,
        args_hash: impl Into<String>,
        duration_ms: u64,
        exit_code: Option<i32>,
        output: impl Into<String>,
    ) -> RunStep {
        let mut output = output.into();
        if output.chars().count() > 20_000 {
            output = output.chars().take(20_000).collect();
        }
        let step = RunStep {
            seq: u32::try_from(self.steps.len() + 1).unwrap_or(u32::MAX),
            stage: self.stage,
            kind: kind.into(),
            args_hash: args_hash.into(),
            duration_ms,
            exit_code,
            output,
            trace_id: self.trace_id.clone(),
        };
        self.steps.push(step.clone());
        step
    }

    /// Request cancellation at the next checkpoint.
    pub const fn request_cancel(&mut self) {
        self.cancelled = true;
    }

    /// Raise an escalation with the last 20 steps.
    #[must_use]
    pub fn escalate(
        &mut self,
        reason: impl Into<String>,
        cost: Money,
        proposed_action: impl Into<String>,
        now: DateTime<Utc>,
    ) -> Escalation {
        let start = self.records.len().saturating_sub(20);
        let escalation = Escalation {
            card_id: self.card_id,
            run_id: self.run_id,
            reason: reason.into(),
            last_steps: self.records[start..].to_vec(),
            cost,
            proposed_action: proposed_action.into(),
            at: now,
        };
        self.escalation = Some(escalation.clone());
        escalation
    }

    /// Record a human intervention.
    pub fn record_human_decision(&mut self, decision: HumanDecision) {
        self.human_decisions.push(decision);
    }

    /// The ordered step trace.
    #[must_use]
    pub fn trace(&self) -> &[StageRecord] {
        &self.records
    }

    /// Idempotency key for an external operation.
    #[must_use]
    pub fn idempotency_key(&self, operation: &str, target: &str) -> String {
        format!("{}:{}:{}", self.run_id, operation, target)
    }
}

/// Idempotency key independent of a pipeline instance.
#[must_use]
pub fn idempotency_key(run_id: RunId, operation: &str, target: &str) -> String {
    format!("{run_id}:{operation}:{target}")
}

/// Static recovery mapping from external state.
#[must_use]
pub const fn recover(state: ExternalPrState) -> RecoveryAction {
    state.recovery()
}

/// Convenience for tests and callers: `now + seconds`.
#[must_use]
pub fn at(now: DateTime<Utc>, seconds: i64) -> DateTime<Utc> {
    now + Duration::seconds(seconds)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pipeline() -> RunPipeline {
        RunPipeline::new(RunId::generate(), CardId::generate(), Utc::now())
    }

    #[test]
    fn stages_advance_in_order_and_record_the_trace() {
        let mut run = pipeline();
        let now = Utc::now();
        for stage in PipelineStage::ALL.iter().skip(1) {
            run.advance(now).expect("advance");
            assert_eq!(run.stage, *stage);
        }
        assert_eq!(run.records.len(), 12);
        assert_eq!(run.stage, PipelineStage::Reflect);
        assert!(run.advance(now).is_err(), "reflect is terminal");
        assert!(run.trace_id.contains(&run.run_id.as_uuid().to_string()));
    }

    #[test]
    fn tool_and_iteration_bounds_stop_the_loop() {
        let mut run = pipeline();
        let now = Utc::now();
        for _ in 0..40 {
            run.record_tool_call(now).expect("tool");
        }
        assert_eq!(
            run.record_tool_call(now),
            Err(PipelineError::ToolCallsExceeded)
        );
        for _ in 0..25 {
            run.next_iteration(now).expect("iteration");
        }
        assert_eq!(
            run.next_iteration(now),
            Err(PipelineError::IterationsExceeded)
        );
    }

    #[test]
    fn verification_rework_is_bounded() {
        let mut run = pipeline();
        let now = Utc::now();
        run.advance(now).expect("context");
        run.advance(now).expect("plan");
        run.advance(now).expect("implement");
        run.advance(now).expect("verify");
        for _ in 0..3 {
            assert_eq!(
                run.fail_verification(now).expect("rework"),
                VerifyOutcome::Rework
            );
            run.advance(now).expect("verify");
        }
        assert_eq!(
            run.fail_verification(now).expect("failed"),
            VerifyOutcome::Failed
        );
    }

    #[test]
    fn review_rework_is_bounded() {
        let mut run = pipeline();
        let now = Utc::now();
        for _ in 0..9 {
            run.advance(now).expect("advance");
        }
        assert_eq!(run.stage, PipelineStage::Review);
        for _ in 0..3 {
            assert_eq!(
                run.request_changes(now).expect("rework"),
                ReviewOutcome::Rework
            );
            for _ in 0..6 {
                run.advance(now).expect("advance");
            }
        }
        assert_eq!(
            run.request_changes(now).expect("escalate"),
            ReviewOutcome::Escalate
        );
    }

    #[test]
    fn cancellation_stops_at_the_checkpoint() {
        let mut run = pipeline();
        let now = Utc::now();
        run.request_cancel();
        assert_eq!(run.checkpoint(now), Err(PipelineError::Cancelled));
        assert_eq!(run.record_tool_call(now), Err(PipelineError::Cancelled));
    }

    #[test]
    fn wall_clock_and_stage_budget_are_enforced() {
        let mut run = pipeline();
        let now = Utc::now();
        run.advance(now).expect("context");
        let later = at(now, 21);
        assert_eq!(
            run.checkpoint(later),
            Err(PipelineError::StageTimeout("context"))
        );
        let run = pipeline();
        let start = run.started_at;
        assert_eq!(
            run.checkpoint(at(start, 2_701)),
            Err(PipelineError::RunTimeout)
        );
    }

    #[test]
    fn handoff_requires_the_transition_stage() {
        let mut run = pipeline();
        let now = Utc::now();
        let handoff = Handoff {
            from: AgentId::generate(),
            to: AgentId::generate(),
            card_id: run.card_id,
            run_id: run.run_id,
            summary: "ready for QA".to_owned(),
            state: HandoffState::Pending,
            open_questions: vec![],
            artifacts: vec!["pr-url".to_owned()],
            acceptance_status: "all criteria evidenced".to_owned(),
        };
        assert!(run.attach_handoff(handoff.clone()).is_err());
        for _ in 0..8 {
            run.advance(now).expect("advance");
        }
        assert_eq!(run.stage, PipelineStage::Transition);
        run.attach_handoff(handoff).expect("handoff");
        assert_eq!(
            run.handoff.as_ref().map(|handoff| handoff.state),
            Some(HandoffState::Pending)
        );
    }

    #[test]
    fn recovery_maps_external_state() {
        assert_eq!(recover(ExternalPrState::Merged), RecoveryAction::Done);
        assert_eq!(recover(ExternalPrState::Open), RecoveryAction::ResumeReview);
        assert_eq!(
            recover(ExternalPrState::BranchOnly),
            RecoveryAction::ResumePush
        );
        assert_eq!(recover(ExternalPrState::Absent), RecoveryAction::Retry);
    }

    #[test]
    fn escalation_and_human_decisions_are_recorded() {
        let mut run = pipeline();
        let now = Utc::now();
        run.record_tool_call(now).expect("tool");
        let escalation = run.escalate(
            "review loops exhausted",
            Money::from_microusd(1_500).expect("money"),
            "move the card to ICEBOX",
            now,
        );
        assert_eq!(escalation.last_steps.len(), 0);
        assert!(run.escalation.is_some());
        run.record_human_decision(HumanDecision {
            actor: "operator".to_owned(),
            decision: "retry once".to_owned(),
            at: now,
        });
        assert_eq!(run.human_decisions.len(), 1);
    }

    #[test]
    fn steps_are_recorded_and_truncated() {
        let mut run = pipeline();
        let first = run.record_step("tool", "hash-1", 12, Some(0), "ok");
        assert_eq!(first.seq, 1);
        assert_eq!(first.trace_id, run.trace_id);
        let large = run.record_step("tool", "hash-2", 3, Some(1), "x".repeat(30_000));
        assert_eq!(large.output.chars().count(), 20_000);
        assert_eq!(run.steps.len(), 2);
    }

    #[test]
    fn stages_map_to_catalog_events() {
        assert_eq!(PipelineStage::Lease.event(), "delivery.run.started");
        assert_eq!(PipelineStage::Done.event(), "delivery.run.completed");
        assert_eq!(PipelineStage::Implement.event(), "delivery.run.step");
    }

    #[test]
    fn idempotency_keys_are_stable() {
        let run = pipeline();
        assert_eq!(
            run.idempotency_key("pr", "head-sha"),
            idempotency_key(run.run_id, "pr", "head-sha")
        );
        assert!(run.idempotency_key("merge", "sha").contains("merge"));
    }
}
