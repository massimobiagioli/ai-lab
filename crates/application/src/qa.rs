//! QA verdicts, findings, separation of duties, merge policy, hard stops and autonomy.

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
    /// Description.
    pub description: String,
}

/// One acceptance result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcceptanceResult {
    /// Acceptance criterion.
    pub criterion: String,
    /// Passed.
    pub passed: bool,
    /// Evidence.
    pub evidence: String,
}

/// A persisted review.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewRecord {
    /// Verdict.
    pub verdict: ReviewVerdict,
    /// Reviewed head SHA.
    pub reviewed_sha: String,
    /// Findings.
    pub findings: Vec<Finding>,
    /// Acceptance results.
    pub acceptance: Vec<AcceptanceResult>,
}

impl ReviewRecord {
    /// Validate the verdict: APPROVED requires every AC to pass and no
    /// P0/P1/P2 finding.
    pub fn validate(&self) -> Result<(), MergeViolation> {
        if self.verdict != ReviewVerdict::Approved {
            return Err(MergeViolation::new(
                "QA_NOT_APPROVED",
                "QA verdict is not APPROVED",
            ));
        }
        if self.acceptance.is_empty() || self.acceptance.iter().any(|result| !result.passed) {
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
}

/// Merge preconditions.
#[derive(Debug, Clone, Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct MergeChecks {
    /// CI is green.
    pub ci_green: bool,
    /// The PR is merged.
    pub pr_merged: bool,
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

/// Evaluate a merge against the ten policy conditions and autonomy.
#[must_use]
pub fn evaluate_merge(
    author: &str,
    reviewer: &str,
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
    if author == reviewer {
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

/// The 11 hard stops.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HardStop {
    /// Budget exhausted.
    BudgetExhausted,
    /// Provider breaker open.
    ProviderUnavailable,
    /// Policy denied.
    PolicyDenied,
    /// Secret detected.
    SecretDetected,
    /// Protected path write.
    ProtectedWrite,
    /// Stale review.
    StaleReview,
    /// CI failed.
    CiFailed,
    /// P0/P1 open.
    P0P1Open,
    /// `DoD` incomplete.
    DodIncomplete,
    /// Human gate required.
    HumanGateRequired,
    /// Separation of duties violated.
    SeparationOfDuties,
}

/// Inputs for the hard-stop evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)]
pub struct HardStopInputs {
    /// Budget remains.
    pub budget_remaining: bool,
    /// Provider healthy.
    pub provider_healthy: bool,
    /// Policy clean.
    pub policy_clean: bool,
    /// No secret in output.
    pub no_secrets: bool,
    /// No protected write.
    pub no_protected_write: bool,
    /// Review is fresh.
    pub review_fresh: bool,
    /// CI green.
    pub ci_green: bool,
    /// P0/P1 open.
    pub p0p1_open: bool,
    /// `DoD` complete.
    pub dod_complete: bool,
    /// Human approved.
    pub human_approved: bool,
    /// Separation of duties holds.
    pub separation_ok: bool,
}

/// Evaluate the hard stops; an empty vector means the run may proceed.
#[must_use]
pub fn evaluate_hard_stops(inputs: &HardStopInputs) -> Vec<HardStop> {
    let mut stops = Vec::new();
    if !inputs.budget_remaining {
        stops.push(HardStop::BudgetExhausted);
    }
    if !inputs.provider_healthy {
        stops.push(HardStop::ProviderUnavailable);
    }
    if !inputs.policy_clean {
        stops.push(HardStop::PolicyDenied);
    }
    if !inputs.no_secrets {
        stops.push(HardStop::SecretDetected);
    }
    if !inputs.no_protected_write {
        stops.push(HardStop::ProtectedWrite);
    }
    if !inputs.review_fresh {
        stops.push(HardStop::StaleReview);
    }
    if !inputs.ci_green {
        stops.push(HardStop::CiFailed);
    }
    if inputs.p0p1_open {
        stops.push(HardStop::P0P1Open);
    }
    if !inputs.dod_complete {
        stops.push(HardStop::DodIncomplete);
    }
    if !inputs.human_approved {
        stops.push(HardStop::HumanGateRequired);
    }
    if !inputs.separation_ok {
        stops.push(HardStop::SeparationOfDuties);
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
        ReviewRecord {
            verdict: ReviewVerdict::Approved,
            reviewed_sha: "sha-1".to_owned(),
            findings: vec![],
            acceptance: vec![AcceptanceResult {
                criterion: "works".to_owned(),
                passed: true,
                evidence: "test".to_owned(),
            }],
        }
    }

    #[test]
    fn a2_allows_a_clean_merge() {
        let decision = evaluate_merge(
            "alice",
            "bob",
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
        let decision = evaluate_merge(
            "alice",
            "alice",
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

    #[test]
    fn a1_requires_human_merge() {
        let mut checks = green();
        checks.human_approved = false;
        let decision = evaluate_merge(
            "alice",
            "bob",
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
    fn stale_review_and_protected_branch_block_the_merge() {
        let mut checks = green();
        checks.head_sha = "sha-2".to_owned();
        assert_eq!(
            evaluate_merge(
                "alice",
                "bob",
                ReviewVerdict::Approved,
                &checks,
                AutonomyLevel::A2
            )
            .violations[0]
                .code,
            "STALE_REVIEW"
        );
        let mut checks = green();
        checks.branch_protected = true;
        assert!(
            evaluate_merge(
                "alice",
                "bob",
                ReviewVerdict::Approved,
                &checks,
                AutonomyLevel::A2
            )
            .violations
            .iter()
            .any(|violation| violation.code == "PROTECTED_BRANCH")
        );
    }

    #[test]
    fn merge_is_idempotent_for_the_same_head() {
        let mut checks = green();
        checks.already_merged = true;
        let decision = evaluate_merge(
            "alice",
            "bob",
            ReviewVerdict::Approved,
            &checks,
            AutonomyLevel::A2,
        );
        assert!(decision.allowed);
        assert!(decision.idempotent);
    }

    #[test]
    fn approved_requires_every_ac_and_no_blocking_finding() {
        review().validate().expect("valid");
        let mut failed = review();
        failed.acceptance[0].passed = false;
        assert_eq!(failed.validate().expect_err("invalid").code, "AC_NOT_PASS");
        let mut finding = review();
        finding.findings.push(Finding {
            id: "F-1".to_owned(),
            severity: FindingSeverity::P1,
            description: "bug".to_owned(),
        });
        assert_eq!(
            finding.validate().expect_err("invalid").code,
            "BLOCKING_FINDING"
        );
        let mut changes = review();
        changes.verdict = ReviewVerdict::ChangesRequested;
        assert_eq!(
            changes.validate().expect_err("invalid").code,
            "QA_NOT_APPROVED"
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
    fn hard_stops_cover_the_eleven_conditions() {
        let inputs = HardStopInputs {
            budget_remaining: false,
            provider_healthy: false,
            policy_clean: false,
            no_secrets: false,
            no_protected_write: false,
            review_fresh: false,
            ci_green: false,
            p0p1_open: true,
            dod_complete: false,
            human_approved: false,
            separation_ok: false,
        };
        let stops = evaluate_hard_stops(&inputs);
        assert_eq!(stops.len(), 11);
        let clean = HardStopInputs {
            budget_remaining: true,
            provider_healthy: true,
            policy_clean: true,
            no_secrets: true,
            no_protected_write: true,
            review_fresh: true,
            ci_green: true,
            p0p1_open: false,
            dod_complete: true,
            human_approved: true,
            separation_ok: true,
        };
        assert_eq!(evaluate_hard_stops(&clean), Vec::<HardStop>::new());
    }
}
