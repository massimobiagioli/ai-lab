//! QA review pipeline, verdicts, merge policy, hard stops and autonomy.

use ailab_domain::AutonomyLevel;
use serde::{Deserialize, Serialize};

/// Review verdict.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewVerdict {
    /// Approved.
    Approved,
    /// Changes requested.
    ChangesRequested,
    /// Blocked by a finding or hard stop.
    Blocked,
}

/// Finding severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingSeverity {
    /// Release blocker.
    P0,
    /// Major.
    P1,
    /// Minor.
    P2,
    /// Cosmetic.
    P3,
}

/// One review finding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    /// Stable id.
    pub id: String,
    /// Severity.
    pub severity: FindingSeverity,
    /// File.
    pub file: String,
    /// Line.
    pub line: Option<u32>,
    /// Issue.
    pub issue: String,
    /// Suggested fix.
    pub fix: String,
}

/// Acceptance status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AcceptanceStatus {
    /// Passed.
    Pass,
    /// Failed.
    Fail,
    /// Not tested.
    NotTested,
}

/// One acceptance result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcceptanceResult {
    /// Acceptance criterion.
    pub criterion: String,
    /// Status.
    pub status: AcceptanceStatus,
    /// Evidence.
    pub evidence: String,
}

/// A persisted review.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewRecord {
    /// Review id.
    pub review_id: String,
    /// Run id.
    pub run_id: String,
    /// Reviewer id.
    pub reviewer_id: String,
    /// Reviewed head SHA.
    pub head_sha: String,
    /// Verdict.
    pub verdict: ReviewVerdict,
    /// Findings.
    pub findings: Vec<Finding>,
    /// Acceptance results.
    pub acceptance: Vec<AcceptanceResult>,
}

impl ReviewRecord {
    /// Validate the verdict: APPROVED requires every AC to pass and no
    /// P0/P1/P2 finding.
    pub fn validate(&self) -> Result<(), MergeViolation> {
        if self.verdict == ReviewVerdict::Blocked {
            return Err(MergeViolation::new("QA_BLOCKED", "the review is blocked"));
        }
        if self.verdict != ReviewVerdict::Approved {
            return Err(MergeViolation::new(
                "QA_NOT_APPROVED",
                "QA verdict is not APPROVED",
            ));
        }
        if self.acceptance.is_empty()
            || self
                .acceptance
                .iter()
                .any(|result| result.status != AcceptanceStatus::Pass)
        {
            return Err(MergeViolation::new(
                "AC_NOT_PASS",
                "every acceptance criterion must pass",
            ));
        }
        if self
            .findings
            .iter()
            .any(|finding| finding.severity <= FindingSeverity::P2)
        {
            return Err(MergeViolation::new(
                "BLOCKING_FINDING",
                "a P0/P1/P2 finding is open",
            ));
        }
        Ok(())
    }

    /// Compute the verdict from the acceptance matrix and findings.
    #[must_use]
    pub fn compute_verdict(&self) -> ReviewVerdict {
        if self
            .findings
            .iter()
            .any(|finding| finding.severity == FindingSeverity::P0)
        {
            ReviewVerdict::Blocked
        } else if self
            .findings
            .iter()
            .any(|finding| matches!(finding.severity, FindingSeverity::P1 | FindingSeverity::P2))
            || self
                .acceptance
                .iter()
                .any(|result| result.status != AcceptanceStatus::Pass)
        {
            ReviewVerdict::ChangesRequested
        } else {
            ReviewVerdict::Approved
        }
    }
}

/// Incremental review pipeline.
#[derive(Debug, Clone, Default)]
pub struct ReviewPipeline {
    acceptance: Vec<AcceptanceResult>,
    findings: Vec<Finding>,
    next_finding: u32,
}

impl ReviewPipeline {
    /// New empty pipeline.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record an acceptance result.
    pub fn check(&mut self, criterion: impl Into<String>, evidence: Option<&str>) {
        let evidence = evidence.unwrap_or_default();
        let status = if evidence.trim().is_empty() {
            AcceptanceStatus::NotTested
        } else {
            AcceptanceStatus::Pass
        };
        self.acceptance.push(AcceptanceResult {
            criterion: criterion.into(),
            status,
            evidence: evidence.to_owned(),
        });
    }

    /// Record a failed acceptance result.
    pub fn fail(&mut self, criterion: impl Into<String>, evidence: impl Into<String>) {
        self.acceptance.push(AcceptanceResult {
            criterion: criterion.into(),
            status: AcceptanceStatus::Fail,
            evidence: evidence.into(),
        });
    }

    /// Add a finding.
    pub fn finding(
        &mut self,
        severity: FindingSeverity,
        file: impl Into<String>,
        line: Option<u32>,
        issue: impl Into<String>,
        fix: impl Into<String>,
    ) {
        self.next_finding += 1;
        self.findings.push(Finding {
            id: format!("F-{}", self.next_finding),
            severity,
            file: file.into(),
            line,
            issue: issue.into(),
            fix: fix.into(),
        });
    }

    /// Finish the review and compute the verdict.
    #[must_use]
    pub fn finish(
        self,
        review_id: impl Into<String>,
        run_id: impl Into<String>,
        reviewer_id: impl Into<String>,
        head_sha: impl Into<String>,
    ) -> ReviewRecord {
        let mut record = ReviewRecord {
            review_id: review_id.into(),
            run_id: run_id.into(),
            reviewer_id: reviewer_id.into(),
            head_sha: head_sha.into(),
            verdict: ReviewVerdict::Approved,
            findings: self.findings,
            acceptance: self.acceptance,
        };
        record.verdict = record.compute_verdict();
        record
    }
}

/// Merge preconditions.
#[derive(Debug, Clone, Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct MergeChecks {
    /// CI is green.
    pub ci_green: bool,
    /// The PR is merged.
    pub pr_merged: bool,
    /// The PR is open.
    pub pr_open: bool,
    /// The PR is a draft.
    pub pr_draft: bool,
    /// The base is the default branch.
    pub base_is_default: bool,
    /// The diff has no forbidden path or secret.
    pub diff_clean: bool,
    /// The merge SHA is recorded.
    pub merge_sha: bool,
    /// `DoD` is complete.
    pub dod_complete: bool,
    /// A P0/P1 is open.
    pub p0p1_open: bool,
    /// Budget remains.
    pub budget_remaining: bool,
    /// Current head SHA.
    pub head_sha: String,
    /// SHA the review was performed on.
    pub reviewed_sha: String,
    /// The branch is protected.
    pub branch_protected: bool,
    /// A human approved the merge.
    pub human_approved: bool,
    /// The PR is already merged.
    pub already_merged: bool,
}

/// One merge violation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergeViolation {
    /// Stable code.
    pub code: String,
    /// Explanation.
    pub message: String,
}

impl MergeViolation {
    fn new(code: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.to_owned(),
            message: message.into(),
        }
    }
}

/// Merge decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergeDecision {
    /// Allowed.
    pub allowed: bool,
    /// Idempotent replay of an already-merged PR.
    pub idempotent: bool,
    /// Violations.
    pub violations: Vec<MergeViolation>,
}

/// Evaluate a merge against the policy conditions and autonomy.
#[must_use]
pub fn evaluate_merge(
    author: &str,
    reviewer: &str,
    merger: &str,
    verdict: ReviewVerdict,
    checks: &MergeChecks,
    autonomy: AutonomyLevel,
) -> MergeDecision {
    if checks.already_merged
        && checks.merge_sha
        && checks.head_sha == checks.reviewed_sha
        && !checks.head_sha.is_empty()
    {
        return MergeDecision {
            allowed: true,
            idempotent: true,
            violations: Vec::new(),
        };
    }
    let mut violations = Vec::new();
    if author == reviewer || author == merger {
        violations.push(MergeViolation::new(
            "SEPARATION_OF_DUTIES",
            "reviewer or merger equals the author",
        ));
    }
    if verdict != ReviewVerdict::Approved {
        violations.push(MergeViolation::new(
            "QA_NOT_APPROVED",
            "QA verdict is not APPROVED",
        ));
    }
    if !checks.ci_green {
        violations.push(MergeViolation::new("CI_NOT_GREEN", "CI is not green"));
    }
    if !checks.pr_open || checks.pr_draft || !checks.base_is_default {
        violations.push(MergeViolation::new(
            "PR_NOT_READY",
            "the PR must be open, non-draft and target the default branch",
        ));
    }
    if !checks.diff_clean {
        violations.push(MergeViolation::new(
            "FORBIDDEN_DIFF",
            "the diff touches a forbidden path or contains a secret",
        ));
    }
    if !checks.pr_merged {
        violations.push(MergeViolation::new("PR_NOT_MERGED", "the PR is not merged"));
    }
    if !checks.merge_sha {
        violations.push(MergeViolation::new(
            "MERGE_SHA_MISSING",
            "the merge SHA is missing",
        ));
    }
    if !checks.dod_complete {
        violations.push(MergeViolation::new(
            "DOD_INCOMPLETE",
            "the Definition of Done is incomplete",
        ));
    }
    if checks.p0p1_open {
        violations.push(MergeViolation::new("P0_P1_OPEN", "a P0/P1 defect is open"));
    }
    if !checks.budget_remaining {
        violations.push(MergeViolation::new(
            "BUDGET_EXHAUSTED",
            "the run budget is exhausted",
        ));
    }
    if checks.head_sha != checks.reviewed_sha || checks.head_sha.is_empty() {
        violations.push(MergeViolation::new(
            "STALE_REVIEW",
            "the head SHA differs from the reviewed SHA",
        ));
    }
    if checks.branch_protected {
        violations.push(MergeViolation::new(
            "PROTECTED_BRANCH",
            "the branch is protected",
        ));
    }
    if matches!(autonomy, AutonomyLevel::A0 | AutonomyLevel::A1) && !checks.human_approved {
        violations.push(MergeViolation::new(
            "HUMAN_MERGE_REQUIRED",
            "the autonomy level requires a human merger",
        ));
    }
    MergeDecision {
        allowed: violations.is_empty(),
        idempotent: false,
        violations,
    }
}

/// Rework loop outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReworkOutcome {
    /// Return to DOING for another attempt.
    Rework,
    /// Move to ICEBOX after the third loop.
    Icebox,
}

/// Bounded review rework loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReworkLoop {
    /// Changes-requested rounds so far.
    pub rounds: u8,
    /// Maximum rounds.
    pub max_rounds: u8,
}

impl Default for ReworkLoop {
    fn default() -> Self {
        Self {
            rounds: 0,
            max_rounds: 3,
        }
    }
}

impl ReworkLoop {
    /// Record a changes-requested round.
    pub const fn changes_requested(&mut self) -> ReworkOutcome {
        self.rounds = self.rounds.saturating_add(1);
        if self.rounds >= self.max_rounds {
            ReworkOutcome::Icebox
        } else {
            ReworkOutcome::Rework
        }
    }
}

/// The 11 hard stops from WP-15 §3.7.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HardStop {
    /// Budget exceeded.
    BudgetExceeded,
    /// Secret in the diff or output.
    SecretDetected,
    /// Forbidden path touched.
    ForbiddenPath,
    /// Force-push/history rewrite/protected-branch write.
    ForcePush,
    /// Destructive SQL or data deletion.
    DestructiveSql,
    /// Critical CVE or disallowed license.
    DependencyRisk,
    /// Request to disable/skip checks.
    DisableChecks,
    /// PR > 1,500 lines or > 40 files.
    LargePullRequest,
    /// Self-review.
    SelfReview,
    /// Self-merge.
    SelfMerge,
    /// Prompt-injection attempt against policy.
    PromptInjection,
}

/// Inputs for the hard-stop evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)]
pub struct HardStopInputs {
    /// Budget remains.
    pub budget_remaining: bool,
    /// No secret in diff/output.
    pub no_secrets: bool,
    /// No forbidden path.
    pub no_forbidden_path: bool,
    /// No force-push/protected write.
    pub no_force_push: bool,
    /// No destructive SQL.
    pub no_destructive_sql: bool,
    /// Dependencies clean.
    pub dependencies_ok: bool,
    /// Checks enabled.
    pub checks_enabled: bool,
    /// Changed lines.
    pub changed_lines: u32,
    /// Changed files.
    pub changed_files: u32,
    /// Reviewer equals author.
    pub reviewer_is_author: bool,
    /// Merger equals author.
    pub merger_is_author: bool,
    /// Prompt injection detected.
    pub prompt_injection: bool,
}

/// Evaluate the hard stops; an empty vector means the run may proceed.
#[must_use]
pub fn evaluate_hard_stops(inputs: &HardStopInputs) -> Vec<HardStop> {
    let mut stops = Vec::new();
    if !inputs.budget_remaining {
        stops.push(HardStop::BudgetExceeded);
    }
    if !inputs.no_secrets {
        stops.push(HardStop::SecretDetected);
    }
    if !inputs.no_forbidden_path {
        stops.push(HardStop::ForbiddenPath);
    }
    if !inputs.no_force_push {
        stops.push(HardStop::ForcePush);
    }
    if !inputs.no_destructive_sql {
        stops.push(HardStop::DestructiveSql);
    }
    if !inputs.dependencies_ok {
        stops.push(HardStop::DependencyRisk);
    }
    if !inputs.checks_enabled {
        stops.push(HardStop::DisableChecks);
    }
    if inputs.changed_lines > 1_500 || inputs.changed_files > 40 {
        stops.push(HardStop::LargePullRequest);
    }
    if inputs.reviewer_is_author {
        stops.push(HardStop::SelfReview);
    }
    if inputs.merger_is_author {
        stops.push(HardStop::SelfMerge);
    }
    if inputs.prompt_injection {
        stops.push(HardStop::PromptInjection);
    }
    stops
}

#[cfg(test)]
mod tests {
    use super::*;

    fn green() -> MergeChecks {
        MergeChecks {
            ci_green: true,
            pr_merged: true,
            pr_open: true,
            pr_draft: false,
            base_is_default: true,
            diff_clean: true,
            merge_sha: true,
            dod_complete: true,
            p0p1_open: false,
            budget_remaining: true,
            head_sha: "sha-1".to_owned(),
            reviewed_sha: "sha-1".to_owned(),
            branch_protected: false,
            human_approved: true,
            already_merged: false,
        }
    }

    fn review() -> ReviewRecord {
        let mut pipeline = ReviewPipeline::new();
        pipeline.check("works", Some("test"));
        pipeline.finish("review-1", "run-1", "qa", "sha-1")
    }

    #[test]
    fn a2_allows_a_clean_merge() {
        let decision = evaluate_merge(
            "alice",
            "bob",
            "carol",
            ReviewVerdict::Approved,
            &green(),
            AutonomyLevel::A2,
        );
        assert!(decision.allowed);
        assert!(!decision.idempotent);
        assert_eq!(decision.violations, Vec::<MergeViolation>::new());
    }

    #[test]
    fn separation_of_duties_is_enforced() {
        for (reviewer, merger) in [("alice", "carol"), ("bob", "alice")] {
            let decision = evaluate_merge(
                "alice",
                reviewer,
                merger,
                ReviewVerdict::Approved,
                &green(),
                AutonomyLevel::A2,
            );
            assert!(!decision.allowed);
            assert!(
                decision
                    .violations
                    .iter()
                    .any(|violation| violation.code == "SEPARATION_OF_DUTIES")
            );
        }
    }

    #[test]
    fn a1_requires_human_merge() {
        let mut checks = green();
        checks.human_approved = false;
        let decision = evaluate_merge(
            "alice",
            "bob",
            "carol",
            ReviewVerdict::Approved,
            &checks,
            AutonomyLevel::A1,
        );
        assert!(!decision.allowed);
        assert!(
            decision
                .violations
                .iter()
                .any(|violation| violation.code == "HUMAN_MERGE_REQUIRED")
        );
    }

    #[test]
    fn failed_checks_and_dod_are_collected() {
        let checks = MergeChecks {
            ci_green: false,
            pr_merged: false,
            pr_open: false,
            pr_draft: true,
            base_is_default: false,
            diff_clean: false,
            merge_sha: false,
            dod_complete: false,
            p0p1_open: true,
            budget_remaining: false,
            head_sha: "sha-2".to_owned(),
            reviewed_sha: "sha-1".to_owned(),
            branch_protected: true,
            human_approved: false,
            already_merged: false,
        };
        let decision = evaluate_merge(
            "alice",
            "bob",
            "carol",
            ReviewVerdict::ChangesRequested,
            &checks,
            AutonomyLevel::A2,
        );
        let codes: Vec<&str> = decision
            .violations
            .iter()
            .map(|violation| violation.code.as_str())
            .collect();
        for code in [
            "QA_NOT_APPROVED",
            "CI_NOT_GREEN",
            "PR_NOT_READY",
            "FORBIDDEN_DIFF",
            "PR_NOT_MERGED",
            "MERGE_SHA_MISSING",
            "DOD_INCOMPLETE",
            "P0_P1_OPEN",
            "BUDGET_EXHAUSTED",
            "STALE_REVIEW",
            "PROTECTED_BRANCH",
        ] {
            assert!(codes.contains(&code), "missing {code}");
        }
    }

    #[test]
    fn merge_is_idempotent_for_the_same_head() {
        let mut checks = green();
        checks.already_merged = true;
        let decision = evaluate_merge(
            "alice",
            "bob",
            "carol",
            ReviewVerdict::Approved,
            &checks,
            AutonomyLevel::A2,
        );
        assert!(decision.allowed);
        assert!(decision.idempotent);
    }

    #[test]
    fn review_pipeline_computes_verdicts() {
        let record = review();
        assert_eq!(record.verdict, ReviewVerdict::Approved);
        record.validate().expect("valid");

        let mut pipeline = ReviewPipeline::new();
        pipeline.check("works", None);
        let record = pipeline.finish("r", "run", "qa", "sha");
        assert_eq!(record.verdict, ReviewVerdict::ChangesRequested);
        assert_eq!(
            record.validate().expect_err("invalid").code,
            "QA_NOT_APPROVED"
        );

        let mut pipeline = ReviewPipeline::new();
        pipeline.check("works", Some("test"));
        pipeline.finding(FindingSeverity::P1, "src/lib.rs", Some(3), "bug", "fix");
        let record = pipeline.finish("r", "run", "qa", "sha");
        assert_eq!(record.verdict, ReviewVerdict::ChangesRequested);
        assert_eq!(
            record.validate().expect_err("invalid").code,
            "QA_NOT_APPROVED"
        );

        let mut pipeline = ReviewPipeline::new();
        pipeline.check("works", Some("test"));
        pipeline.finding(FindingSeverity::P0, "src/lib.rs", None, "breach", "fix");
        let record = pipeline.finish("r", "run", "qa", "sha");
        assert_eq!(record.verdict, ReviewVerdict::Blocked);
        assert_eq!(record.validate().expect_err("invalid").code, "QA_BLOCKED");

        let mut invalid_ac = review();
        invalid_ac.acceptance[0].status = AcceptanceStatus::Fail;
        assert_eq!(
            invalid_ac.validate().expect_err("invalid").code,
            "AC_NOT_PASS"
        );
        let mut blocking = review();
        blocking.findings.push(Finding {
            id: "F-9".to_owned(),
            severity: FindingSeverity::P1,
            file: "src/lib.rs".to_owned(),
            line: None,
            issue: "bug".to_owned(),
            fix: "fix".to_owned(),
        });
        assert_eq!(
            blocking.validate().expect_err("invalid").code,
            "BLOCKING_FINDING"
        );
    }

    #[test]
    fn rework_moves_to_icebox_after_three_rounds() {
        let mut rework = ReworkLoop::default();
        assert_eq!(rework.changes_requested(), ReworkOutcome::Rework);
        assert_eq!(rework.changes_requested(), ReworkOutcome::Rework);
        assert_eq!(rework.changes_requested(), ReworkOutcome::Icebox);
    }

    #[test]
    fn each_hard_stop_fires_individually() {
        let base = HardStopInputs {
            budget_remaining: true,
            no_secrets: true,
            no_forbidden_path: true,
            no_force_push: true,
            no_destructive_sql: true,
            dependencies_ok: true,
            checks_enabled: true,
            changed_lines: 10,
            changed_files: 2,
            reviewer_is_author: false,
            merger_is_author: false,
            prompt_injection: false,
        };
        let cases = [
            (
                HardStop::BudgetExceeded,
                HardStopInputs {
                    budget_remaining: false,
                    ..base
                },
            ),
            (
                HardStop::SecretDetected,
                HardStopInputs {
                    no_secrets: false,
                    ..base
                },
            ),
            (
                HardStop::ForbiddenPath,
                HardStopInputs {
                    no_forbidden_path: false,
                    ..base
                },
            ),
            (
                HardStop::ForcePush,
                HardStopInputs {
                    no_force_push: false,
                    ..base
                },
            ),
            (
                HardStop::DestructiveSql,
                HardStopInputs {
                    no_destructive_sql: false,
                    ..base
                },
            ),
            (
                HardStop::DependencyRisk,
                HardStopInputs {
                    dependencies_ok: false,
                    ..base
                },
            ),
            (
                HardStop::DisableChecks,
                HardStopInputs {
                    checks_enabled: false,
                    ..base
                },
            ),
            (
                HardStop::LargePullRequest,
                HardStopInputs {
                    changed_lines: 2_000,
                    ..base
                },
            ),
            (
                HardStop::SelfReview,
                HardStopInputs {
                    reviewer_is_author: true,
                    ..base
                },
            ),
            (
                HardStop::SelfMerge,
                HardStopInputs {
                    merger_is_author: true,
                    ..base
                },
            ),
            (
                HardStop::PromptInjection,
                HardStopInputs {
                    prompt_injection: true,
                    ..base
                },
            ),
        ];
        for (expected, inputs) in cases {
            assert_eq!(evaluate_hard_stops(&inputs), vec![expected], "{expected:?}");
        }
    }

    #[test]
    fn hard_stops_cover_the_eleven_conditions() {
        let inputs = HardStopInputs {
            budget_remaining: false,
            no_secrets: false,
            no_forbidden_path: false,
            no_force_push: false,
            no_destructive_sql: false,
            dependencies_ok: false,
            checks_enabled: false,
            changed_lines: 2_000,
            changed_files: 50,
            reviewer_is_author: true,
            merger_is_author: true,
            prompt_injection: true,
        };
        let stops = evaluate_hard_stops(&inputs);
        assert_eq!(stops.len(), 11);
        let clean = HardStopInputs {
            budget_remaining: true,
            no_secrets: true,
            no_forbidden_path: true,
            no_force_push: true,
            no_destructive_sql: true,
            dependencies_ok: true,
            checks_enabled: true,
            changed_lines: 10,
            changed_files: 2,
            reviewer_is_author: false,
            merger_is_author: false,
            prompt_injection: false,
        };
        assert_eq!(evaluate_hard_stops(&clean), Vec::<HardStop>::new());
    }
}
