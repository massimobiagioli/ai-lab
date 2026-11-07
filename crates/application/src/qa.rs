//! QA verdicts, separation of duties and autonomy-aware merge policy.

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

/// Merge preconditions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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
    /// Violations.
    pub violations: Vec<MergeViolation>,
}

/// Evaluate a merge against separation of duties, checks, `DoD` and autonomy.
#[must_use]
pub fn evaluate_merge(
    author: &str,
    reviewer: &str,
    verdict: ReviewVerdict,
    checks: MergeChecks,
    autonomy: AutonomyLevel,
) -> MergeDecision {
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
    if matches!(autonomy, AutonomyLevel::A0 | AutonomyLevel::A1) {
        violations.push(MergeViolation::new(
            "HUMAN_MERGE_REQUIRED",
            "the autonomy level requires a human merger",
        ));
    }
    MergeDecision {
        allowed: violations.is_empty(),
        violations,
    }
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
        }
    }

    #[test]
    fn a2_allows_a_clean_merge() {
        let decision = evaluate_merge(
            "alice",
            "bob",
            ReviewVerdict::Approved,
            green(),
            AutonomyLevel::A2,
        );
        assert!(decision.allowed);
        assert_eq!(decision.violations, Vec::<MergeViolation>::new());
    }

    #[test]
    fn separation_of_duties_is_enforced() {
        let decision = evaluate_merge(
            "alice",
            "alice",
            ReviewVerdict::Approved,
            green(),
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
        let decision = evaluate_merge(
            "alice",
            "bob",
            ReviewVerdict::Approved,
            green(),
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
        };
        let decision = evaluate_merge(
            "alice",
            "bob",
            ReviewVerdict::ChangesRequested,
            checks,
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
        ] {
            assert!(codes.contains(&code), "missing {code}");
        }
    }
}
