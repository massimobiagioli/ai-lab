//! Agile Kanban context: card aggregate, transition matrix and guards.

use chrono::{DateTime, Utc};
use serde_json::json;

use crate::error::DomainError;
use crate::event::DomainEvent;
use crate::ids::{AgentId, CardId, CardKey, LabId, ProjectId};
use crate::value::{Actor, Estimate, KanbanColumn, Money, Priority, Version};

/// Card source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardSource {
    /// Created by a human.
    Human,
    /// Created by an agent.
    Agent,
}

/// Blocked reason category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockedCategory {
    /// Waiting on a dependency.
    Dependency,
    /// Waiting on clarification.
    Clarification,
    /// External outage.
    External,
    /// Risk.
    Risk,
    /// Budget.
    Budget,
    /// Policy.
    Policy,
}

impl BlockedCategory {
    /// Stable wire name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Dependency => "dependency",
            Self::Clarification => "clarification",
            Self::External => "external",
            Self::Risk => "risk",
            Self::Budget => "budget",
            Self::Policy => "policy",
        }
    }
}

/// Blocked reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockedReason {
    /// Category.
    pub category: BlockedCategory,
    /// Explanation.
    pub text: String,
}

/// Canonical card story.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CardStory {
    /// Acceptance criteria.
    pub acceptance_criteria: Vec<String>,
    /// Concrete deliverables.
    pub deliverables: Vec<String>,
    /// Refinement flag.
    pub needs_refinement: bool,
}

/// WIP limits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WipLimits {
    /// `DOING` cards per agent.
    pub doing_per_agent: u8,
    /// `REVIEW` cards per QA.
    pub review_per_qa: u8,
}

impl Default for WipLimits {
    fn default() -> Self {
        Self {
            doing_per_agent: 1,
            review_per_qa: 3,
        }
    }
}

/// Context supplied to a transition.
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct TransitionContext {
    /// Fixed clock.
    pub now: DateTime<Utc>,
    /// Actor.
    pub actor: Actor,
    /// Actor is the Tech Lead.
    pub actor_is_tech_lead: bool,
    /// Actor is the card assignee.
    pub actor_is_assignee: bool,
    /// Current `DOING` count for the assignee.
    pub doing_count: u8,
    /// Current `REVIEW` count for the reviewer.
    pub review_count: u8,
    /// WIP limits.
    pub wip_limits: WipLimits,
    /// CI is green.
    pub ci_green: bool,
    /// QA approved.
    pub qa_approved: bool,
    /// `DoD` complete.
    pub dod_complete: bool,
    /// Run cancelled.
    pub run_cancelled: bool,
    /// Reason for a guarded move.
    pub reason: Option<String>,
    /// Resolution for ICEBOX to TODO.
    pub resolution: Option<String>,
}

impl TransitionContext {
    /// Baseline context with no approvals and default limits.
    #[must_use]
    pub fn new(now: DateTime<Utc>, actor: Actor) -> Self {
        Self {
            now,
            actor,
            actor_is_tech_lead: false,
            actor_is_assignee: false,
            doing_count: 0,
            review_count: 0,
            wip_limits: WipLimits::default(),
            ci_green: false,
            qa_approved: false,
            dod_complete: false,
            run_cancelled: false,
            reason: None,
            resolution: None,
        }
    }
}

/// Pure transition matrix from Appendix C.3. Same-column moves are denied.
#[must_use]
pub const fn is_allowed(from: KanbanColumn, to: KanbanColumn) -> bool {
    matches!(
        (from, to),
        (
            KanbanColumn::Todo,
            KanbanColumn::Ready | KanbanColumn::Icebox
        ) | (
            KanbanColumn::Ready,
            KanbanColumn::Todo | KanbanColumn::Doing | KanbanColumn::Icebox
        ) | (
            KanbanColumn::Doing,
            KanbanColumn::Ready | KanbanColumn::Review | KanbanColumn::Icebox
        ) | (
            KanbanColumn::Review,
            KanbanColumn::Doing | KanbanColumn::Done | KanbanColumn::Icebox
        ) | (KanbanColumn::Icebox, KanbanColumn::Todo)
    )
}

/// Card aggregate.
#[derive(Debug, Clone)]
#[allow(clippy::struct_field_names)]
pub struct Card {
    id: CardId,
    lab_id: LabId,
    project_id: ProjectId,
    card_key: CardKey,
    title: String,
    story: CardStory,
    column: KanbanColumn,
    priority: Priority,
    estimate: Option<Estimate>,
    assignee_id: Option<AgentId>,
    reviewer_id: Option<AgentId>,
    due_date: Option<DateTime<Utc>>,
    budget: Money,
    source: CardSource,
    pr_url: Option<String>,
    merge_sha: Option<String>,
    blocked_reason: Option<BlockedReason>,
    parent_id: Option<CardId>,
    version: Version,
    events: Vec<DomainEvent>,
}

impl Card {
    /// Create a card in `TODO`.
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        id: CardId,
        lab_id: LabId,
        project_id: ProjectId,
        card_key: CardKey,
        title: impl Into<String>,
        story: CardStory,
        priority: Priority,
        budget: Money,
        source: CardSource,
        occurred_at: DateTime<Utc>,
    ) -> Result<Self, DomainError> {
        let title = title.into();
        let chars = title.chars().count();
        if !(1..=80).contains(&chars) {
            return Err(DomainError::validation(
                "card.title",
                "must be 1-80 characters",
            ));
        }
        let event = DomainEvent::new(
            "kanban.card.created",
            lab_id,
            Actor::system(),
            json!({"card_id": id.to_string(), "key": card_key.to_string(), "column": "TODO", "source": match source { CardSource::Human => "human", CardSource::Agent => "agent" }}),
            occurred_at,
        )?;
        Ok(Self {
            id,
            lab_id,
            project_id,
            card_key,
            title,
            story,
            column: KanbanColumn::Todo,
            priority,
            estimate: None,
            assignee_id: None,
            reviewer_id: None,
            due_date: None,
            budget,
            source,
            pr_url: None,
            merge_sha: None,
            blocked_reason: None,
            parent_id: None,
            version: Version::ONE,
            events: vec![event],
        })
    }

    /// Identity.
    #[must_use]
    pub const fn id(&self) -> CardId {
        self.id
    }

    /// Lab scope.
    #[must_use]
    pub const fn lab_id(&self) -> LabId {
        self.lab_id
    }

    /// Project scope.
    #[must_use]
    pub const fn project_id(&self) -> ProjectId {
        self.project_id
    }

    /// Human key.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn card_key(&self) -> &CardKey {
        &self.card_key
    }

    /// Current column.
    #[must_use]
    pub const fn column(&self) -> KanbanColumn {
        self.column
    }

    /// Version.
    #[must_use]
    pub const fn version(&self) -> Version {
        self.version
    }

    /// Priority.
    #[must_use]
    pub const fn priority(&self) -> Priority {
        self.priority
    }

    /// Source.
    #[must_use]
    pub const fn source(&self) -> CardSource {
        self.source
    }

    /// Estimate.
    #[must_use]
    pub const fn estimate(&self) -> Option<Estimate> {
        self.estimate
    }

    /// Assignee.
    #[must_use]
    pub const fn assignee_id(&self) -> Option<AgentId> {
        self.assignee_id
    }

    /// Reviewer.
    #[must_use]
    pub const fn reviewer_id(&self) -> Option<AgentId> {
        self.reviewer_id
    }

    /// Merge SHA.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn merge_sha(&self) -> Option<&str> {
        self.merge_sha.as_deref()
    }

    /// Story.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn story(&self) -> &CardStory {
        &self.story
    }

    /// Blocked reason.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn blocked_reason(&self) -> Option<&BlockedReason> {
        self.blocked_reason.as_ref()
    }

    /// Parent card for a reopen.
    #[must_use]
    pub const fn parent_id(&self) -> Option<CardId> {
        self.parent_id
    }

    /// Uncommitted events.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn uncommitted_events(&self) -> &[DomainEvent] {
        &self.events
    }

    /// Refine the story, estimate and priority.
    pub fn refine(
        &mut self,
        story: CardStory,
        estimate: Estimate,
        priority: Priority,
    ) -> Result<(), DomainError> {
        self.story = story;
        self.estimate = Some(estimate);
        self.priority = priority;
        self.version = self.version.next()?;
        Ok(())
    }

    /// Assign an owner and a due date.
    pub fn assign(
        &mut self,
        assignee: AgentId,
        due_date: DateTime<Utc>,
    ) -> Result<(), DomainError> {
        self.assignee_id = Some(assignee);
        self.due_date = Some(due_date);
        self.version = self.version.next()?;
        Ok(())
    }

    /// Attach a PR and a reviewer; reviewer must differ from the assignee.
    pub fn attach_pr(
        &mut self,
        reviewer: AgentId,
        pr_url: impl Into<String>,
    ) -> Result<(), DomainError> {
        if self.assignee_id == Some(reviewer) {
            return Err(DomainError::SeparationOfDuties);
        }
        self.reviewer_id = Some(reviewer);
        self.pr_url = Some(pr_url.into());
        self.version = self.version.next()?;
        Ok(())
    }

    /// Record the merge SHA.
    pub fn merge(&mut self, sha: impl Into<String>) -> Result<(), DomainError> {
        let sha = sha.into();
        if sha.trim().is_empty() {
            return Err(DomainError::validation(
                "card.merge_sha",
                "must not be empty",
            ));
        }
        self.merge_sha = Some(sha);
        self.version = self.version.next()?;
        Ok(())
    }

    /// Set a blocked reason before moving to `ICEBOX`.
    pub fn block(&mut self, reason: BlockedReason) -> Result<(), DomainError> {
        if reason.text.trim().is_empty() {
            return Err(DomainError::validation(
                "card.blocked_reason",
                "must not be empty",
            ));
        }
        self.blocked_reason = Some(reason);
        self.version = self.version.next()?;
        Ok(())
    }

    /// Move the card according to the matrix and guards.
    pub fn move_to(
        &mut self,
        to: KanbanColumn,
        ctx: &TransitionContext,
    ) -> Result<(), DomainError> {
        if self.column == to || !is_allowed(self.column, to) {
            return Err(DomainError::InvalidTransition);
        }
        match (self.column, to) {
            (KanbanColumn::Todo, KanbanColumn::Ready) => self.guard_dor(ctx)?,
            (KanbanColumn::Ready, KanbanColumn::Doing) => self.guard_doing(ctx)?,
            (KanbanColumn::Ready, KanbanColumn::Todo) => {
                if !ctx.actor_is_tech_lead || ctx.reason.is_none() {
                    return Err(DomainError::InvalidTransition);
                }
                self.story.needs_refinement = true;
            }
            (KanbanColumn::Doing, KanbanColumn::Review) => self.guard_review(ctx)?,
            (KanbanColumn::Doing, KanbanColumn::Ready) => {
                if !(ctx.actor_is_tech_lead || ctx.actor_is_assignee)
                    || !ctx.run_cancelled
                    || ctx.reason.is_none()
                {
                    return Err(DomainError::InvalidTransition);
                }
            }
            (KanbanColumn::Review, KanbanColumn::Done) => self.guard_done(ctx)?,
            (KanbanColumn::Review, KanbanColumn::Doing) => {}
            (_, KanbanColumn::Icebox) => {
                if self.blocked_reason.is_none() {
                    return Err(DomainError::validation(
                        "card.blocked_reason",
                        "required before ICEBOX",
                    ));
                }
            }
            (KanbanColumn::Icebox, KanbanColumn::Todo) => {
                if ctx.resolution.as_deref().is_none_or(str::is_empty) {
                    return Err(DomainError::validation(
                        "card.resolution",
                        "required to leave ICEBOX",
                    ));
                }
                self.blocked_reason = None;
                self.story.needs_refinement = true;
            }
            _ => return Err(DomainError::InvalidTransition),
        }
        let from = self.column;
        self.column = to;
        self.version = self.version.next()?;
        let event = DomainEvent::new(
            "kanban.card.moved",
            self.lab_id,
            ctx.actor.clone(),
            json!({"card_id": self.id.to_string(), "key": self.card_key.to_string(), "from": from.as_str(), "to": to.as_str(), "reason": ctx.reason}),
            ctx.now,
        )?;
        self.events.push(event);
        Ok(())
    }

    fn guard_dor(&self, ctx: &TransitionContext) -> Result<(), DomainError> {
        if self.story.needs_refinement
            || self.story.acceptance_criteria.is_empty()
            || self.story.deliverables.is_empty()
            || self.estimate.is_none()
            || self.priority == Priority::Wont
            || self.assignee_id.is_none()
            || self.due_date.is_none_or(|due| due <= ctx.now)
            || self.budget == Money::ZERO
        {
            return Err(DomainError::validation(
                "card.dor",
                "definition of ready is incomplete",
            ));
        }
        Ok(())
    }

    fn guard_doing(&self, ctx: &TransitionContext) -> Result<(), DomainError> {
        if self.assignee_id.is_none() {
            return Err(DomainError::validation("card.assignee", "required"));
        }
        if ctx.doing_count >= ctx.wip_limits.doing_per_agent {
            return Err(DomainError::policy(
                "WIP_LIMIT_EXCEEDED",
                "doing WIP limit reached for the assignee",
            ));
        }
        Ok(())
    }

    fn guard_review(&self, ctx: &TransitionContext) -> Result<(), DomainError> {
        let Some(reviewer) = self.reviewer_id else {
            return Err(DomainError::validation("card.reviewer", "required"));
        };
        if self.assignee_id == Some(reviewer) {
            return Err(DomainError::SeparationOfDuties);
        }
        if self.pr_url.is_none() || !ctx.ci_green {
            return Err(DomainError::validation(
                "card.review",
                "a PR and green CI are required",
            ));
        }
        if ctx.review_count >= ctx.wip_limits.review_per_qa {
            return Err(DomainError::policy(
                "WIP_LIMIT_EXCEEDED",
                "review WIP limit reached for the reviewer",
            ));
        }
        Ok(())
    }

    fn guard_done(&self, ctx: &TransitionContext) -> Result<(), DomainError> {
        let Some(reviewer) = self.reviewer_id else {
            return Err(DomainError::validation("card.reviewer", "required"));
        };
        if self.assignee_id == Some(reviewer) {
            return Err(DomainError::SeparationOfDuties);
        }
        if self.pr_url.is_none()
            || self.merge_sha.is_none()
            || !ctx.qa_approved
            || !ctx.ci_green
            || !ctx.dod_complete
        {
            return Err(DomainError::validation(
                "card.dod",
                "QA approval, merge SHA, green CI and DoD are required",
            ));
        }
        Ok(())
    }

    /// Reopen a done card as a new card in `TODO` with a parent link.
    pub fn reopen(
        &self,
        new_id: CardId,
        new_key: CardKey,
        occurred_at: DateTime<Utc>,
    ) -> Result<Self, DomainError> {
        if self.column != KanbanColumn::Done {
            return Err(DomainError::policy(
                "CARD_NOT_DONE",
                "only a done card can be reopened",
            ));
        }
        let mut card = Self::create(
            new_id,
            self.lab_id,
            self.project_id,
            new_key,
            self.title.clone(),
            self.story.clone(),
            self.priority,
            self.budget,
            CardSource::Human,
            occurred_at,
        )?;
        card.parent_id = Some(self.id);
        Ok(card)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, TimeZone};

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0)
            .single()
            .expect("timestamp")
    }

    fn story() -> CardStory {
        CardStory {
            acceptance_criteria: vec!["Given a card When moved Then it works".to_owned()],
            deliverables: vec!["docs/kanban.md".to_owned()],
            needs_refinement: false,
        }
    }

    fn card() -> Card {
        Card::create(
            CardId::generate(),
            LabId::generate(),
            ProjectId::generate(),
            CardKey::new("AIL", 1).expect("key"),
            "Add the board",
            story(),
            Priority::Must,
            Money::from_microusd(5_000_000).expect("budget"),
            CardSource::Human,
            now(),
        )
        .expect("card")
    }

    fn ready_card() -> (Card, AgentId, AgentId) {
        let mut card = card();
        let assignee = AgentId::generate();
        let reviewer = AgentId::generate();
        card.refine(story(), Estimate::new(3).expect("estimate"), Priority::Must)
            .expect("refine");
        card.assign(assignee, now() + Duration::days(1))
            .expect("assign");
        card.move_to(
            KanbanColumn::Ready,
            &TransitionContext::new(now(), Actor::human()),
        )
        .expect("ready");
        (card, assignee, reviewer)
    }

    fn done_card() -> Card {
        let (mut card, _assignee, reviewer) = ready_card();
        let mut ctx = TransitionContext::new(now(), Actor::human());
        ctx.doing_count = 0;
        card.move_to(KanbanColumn::Doing, &ctx).expect("doing");
        card.attach_pr(reviewer, "https://example.test/pr/1")
            .expect("pr");
        ctx.ci_green = true;
        card.move_to(KanbanColumn::Review, &ctx).expect("review");
        card.merge("sha-1").expect("merge");
        ctx.qa_approved = true;
        ctx.dod_complete = true;
        card.move_to(KanbanColumn::Done, &ctx).expect("done");
        card
    }

    #[test]
    fn transition_matrix_has_exactly_twelve_allowed_moves() {
        let columns = [
            KanbanColumn::Todo,
            KanbanColumn::Ready,
            KanbanColumn::Doing,
            KanbanColumn::Review,
            KanbanColumn::Done,
            KanbanColumn::Icebox,
        ];
        let allowed = [
            (KanbanColumn::Todo, KanbanColumn::Ready),
            (KanbanColumn::Todo, KanbanColumn::Icebox),
            (KanbanColumn::Ready, KanbanColumn::Todo),
            (KanbanColumn::Ready, KanbanColumn::Doing),
            (KanbanColumn::Ready, KanbanColumn::Icebox),
            (KanbanColumn::Doing, KanbanColumn::Ready),
            (KanbanColumn::Doing, KanbanColumn::Review),
            (KanbanColumn::Doing, KanbanColumn::Icebox),
            (KanbanColumn::Review, KanbanColumn::Doing),
            (KanbanColumn::Review, KanbanColumn::Done),
            (KanbanColumn::Review, KanbanColumn::Icebox),
            (KanbanColumn::Icebox, KanbanColumn::Todo),
        ];
        let mut count = 0;
        for from in columns {
            for to in columns {
                let expected = allowed.contains(&(from, to));
                assert_eq!(is_allowed(from, to), expected, "{from:?} -> {to:?}");
                if expected {
                    count += 1;
                }
            }
        }
        assert_eq!(count, 12);
        assert!(!is_allowed(KanbanColumn::Done, KanbanColumn::Todo));
        assert!(!is_allowed(KanbanColumn::Todo, KanbanColumn::Todo));
    }

    #[test]
    fn dor_guards_todo_to_ready() {
        let mut card = card();
        let ctx = TransitionContext::new(now(), Actor::human());
        assert!(card.move_to(KanbanColumn::Ready, &ctx).is_err());
        card.refine(story(), Estimate::new(3).expect("estimate"), Priority::Must)
            .expect("refine");
        card.assign(AgentId::generate(), now() + Duration::days(1))
            .expect("assign");
        card.move_to(KanbanColumn::Ready, &ctx).expect("ready");
        assert_eq!(card.column(), KanbanColumn::Ready);
    }

    #[test]
    fn review_and_done_guards() {
        let (mut card, _assignee, reviewer) = ready_card();
        let mut ctx = TransitionContext::new(now(), Actor::human());
        card.move_to(KanbanColumn::Doing, &ctx).expect("doing");
        assert!(card.move_to(KanbanColumn::Review, &ctx).is_err());
        card.attach_pr(reviewer, "https://example.test/pr/1")
            .expect("pr");
        ctx.ci_green = true;
        card.move_to(KanbanColumn::Review, &ctx).expect("review");
        assert!(card.move_to(KanbanColumn::Done, &ctx).is_err());
        card.merge("sha-1").expect("merge");
        ctx.qa_approved = true;
        ctx.dod_complete = true;
        card.move_to(KanbanColumn::Done, &ctx).expect("done");
        assert_eq!(card.column(), KanbanColumn::Done);
    }

    #[test]
    fn separation_of_duties_is_enforced() {
        let (mut card, assignee, _) = ready_card();
        assert_eq!(
            card.attach_pr(assignee, "https://example.test/pr/1"),
            Err(DomainError::SeparationOfDuties)
        );
    }

    #[test]
    fn wip_limits_block_transitions() {
        let (mut card, _, _) = ready_card();
        let mut ctx = TransitionContext::new(now(), Actor::human());
        ctx.doing_count = 1;
        ctx.wip_limits.doing_per_agent = 1;
        let error = card.move_to(KanbanColumn::Doing, &ctx).expect_err("wip");
        assert_eq!(error.code(), "WIP_LIMIT_EXCEEDED");
    }

    #[test]
    fn icebox_requires_reason_and_resolution() {
        let mut card = card();
        let ctx = TransitionContext::new(now(), Actor::human());
        assert!(card.move_to(KanbanColumn::Icebox, &ctx).is_err());
        card.block(BlockedReason {
            category: BlockedCategory::Dependency,
            text: "waiting on WP-12".to_owned(),
        })
        .expect("block");
        card.move_to(KanbanColumn::Icebox, &ctx).expect("icebox");
        let mut ctx = TransitionContext::new(now(), Actor::human());
        ctx.resolution = Some("dependency landed".to_owned());
        card.move_to(KanbanColumn::Todo, &ctx).expect("todo");
        assert!(card.blocked_reason().is_none());
        assert!(card.story().needs_refinement);
    }

    #[test]
    fn reopen_creates_a_new_card_with_parent() {
        let card = done_card();
        let reopened = card
            .reopen(
                CardId::generate(),
                CardKey::new("AIL", 2).expect("key"),
                now(),
            )
            .expect("reopen");
        assert_eq!(reopened.column(), KanbanColumn::Todo);
        assert_eq!(reopened.parent_id(), Some(card.id()));
        assert_eq!(card.column(), KanbanColumn::Done);
    }

    #[test]
    fn transitions_emit_events() {
        let card = card();
        assert_eq!(card.uncommitted_events().len(), 1);
        assert_eq!(
            card.uncommitted_events()[0].event_type,
            "kanban.card.created"
        );
        let (card, _, _) = ready_card();
        let last = card.uncommitted_events().last().expect("event");
        assert_eq!(last.event_type, "kanban.card.moved");
    }
}
