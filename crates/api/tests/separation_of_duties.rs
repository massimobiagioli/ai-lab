//! Integration test: a reviewer/merger conflict reaches the client as an
//! RFC 9457 Problem Details response with HTTP 422 and code
//! `SEPARATION_OF_DUTIES` (WP-00 AC7).

use ailab_application::{MergeChecks, ProblemDetails, ReviewVerdict, evaluate_merge};
use ailab_domain::kanban::{Card, CardSource, CardStory, TransitionContext};
use ailab_domain::{
    Actor, AgentId, AutonomyLevel, CardId, CardKey, DomainError, Estimate, KanbanColumn, LabId,
    Money, Priority, ProjectId,
};
use chrono::{Duration, Utc};

fn story() -> CardStory {
    CardStory {
        acceptance_criteria: vec!["Given a card When reviewed by its author Then 422".to_owned()],
        deliverables: vec![".docs/product/kanban.md".to_owned()],
        needs_refinement: false,
    }
}

fn ready_card() -> (Card, AgentId) {
    let now = Utc::now();
    let auth = AgentId::generate();
    let mut card = Card::create(
        CardId::generate(),
        LabId::generate(),
        ProjectId::generate(),
        CardKey::new("AIL", 1).expect("key"),
        "Reject self review",
        story(),
        Priority::Must,
        Money::from_microusd(5_000_000).expect("budget"),
        CardSource::Human,
        now,
    )
    .expect("card");
    card.refine(story(), Estimate::new(3).expect("estimate"), Priority::Must)
        .expect("refine");
    card.assign(auth, now + Duration::days(1)).expect("assign");
    card.move_to(
        KanbanColumn::Ready,
        &TransitionContext::new(now, Actor::human()),
    )
    .expect("ready");
    (card, auth)
}

#[test]
fn author_cannot_review_own_card_and_maps_to_422() {
    let (mut card, author) = ready_card();

    let error = card
        .attach_pr(author, "https://example.test/pr/1")
        .expect_err("self review must be rejected");
    assert_eq!(error, DomainError::SeparationOfDuties);

    let problem = ProblemDetails::from(error);
    assert_eq!(problem.status, 422);
    assert_eq!(problem.code, "SEPARATION_OF_DUTIES");

    let payload = serde_json::to_value(&problem).expect("serialize");
    assert_eq!(payload["status"], 422);
    assert_eq!(payload["code"], "SEPARATION_OF_DUTIES");
}

#[test]
fn merge_policy_rejects_author_as_reviewer() {
    let checks = MergeChecks {
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
    };
    let decision = evaluate_merge(
        "agent-dev",
        "agent-dev",
        ReviewVerdict::Approved,
        &checks,
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
