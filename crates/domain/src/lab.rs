//! Lab context: lab aggregate, settings validation and team invariants.

use chrono::{DateTime, Utc};

use crate::error::DomainError;
use crate::ids::{LabId, WorkspaceId};
use crate::inventory::Agent;
use crate::value::{AutonomyLevel, Money, Slug, Version};

/// Lab settings validated by the domain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LabSettings {
    /// WIP limit for `DOING` cards per agent.
    pub wip_doing_per_agent: u8,
    /// WIP limit for `REVIEW` cards per QA.
    pub wip_review_per_qa: u8,
    /// Default per-card budget in micro-USD.
    pub per_card_budget_microusd: i64,
    /// Default per-day budget in micro-USD.
    pub per_day_budget_microusd: i64,
    /// Hard stop when the budget is exhausted.
    pub hard_stop: bool,
}

impl Default for LabSettings {
    fn default() -> Self {
        Self {
            wip_doing_per_agent: 1,
            wip_review_per_qa: 3,
            per_card_budget_microusd: 5_000_000,
            per_day_budget_microusd: 50_000_000,
            hard_stop: true,
        }
    }
}

impl LabSettings {
    /// Validate ranges and budget ordering.
    pub fn validate(&self) -> Result<(), DomainError> {
        if self.wip_doing_per_agent == 0 {
            return Err(DomainError::validation(
                "settings.wip_doing_per_agent",
                "must be at least 1",
            ));
        }
        if self.wip_review_per_qa == 0 {
            return Err(DomainError::validation(
                "settings.wip_review_per_qa",
                "must be at least 1",
            ));
        }
        if self.per_card_budget_microusd < 0 || self.per_day_budget_microusd < 0 {
            return Err(DomainError::validation(
                "settings.budget",
                "budgets must not be negative",
            ));
        }
        if self.per_card_budget_microusd > self.per_day_budget_microusd {
            return Err(DomainError::validation(
                "settings.budget",
                "per-card budget cannot exceed per-day budget",
            ));
        }
        Ok(())
    }
}

/// Lab aggregate.
#[derive(Debug, Clone)]
pub struct Lab {
    id: LabId,
    workspace_id: WorkspaceId,
    name: String,
    slug: Slug,
    description: String,
    autonomy: AutonomyLevel,
    monthly_budget: Money,
    settings: LabSettings,
    archived_at: Option<DateTime<Utc>>,
    version: Version,
}

impl Lab {
    /// Create a lab; the provided team must contain exactly one active Tech Lead.
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        id: LabId,
        workspace_id: WorkspaceId,
        name: impl Into<String>,
        slug: impl AsRef<str>,
        description: impl Into<String>,
        autonomy: AutonomyLevel,
        monthly_budget: Money,
        settings: LabSettings,
        team: &[Agent],
    ) -> Result<Self, DomainError> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(DomainError::validation("lab.name", "must not be empty"));
        }
        let slug = Slug::new(slug.as_ref())?;
        settings.validate()?;
        Agent::validate_team(team)?;
        for agent in team {
            if agent.lab_id() != id {
                return Err(DomainError::validation(
                    "lab.team",
                    "every team member must belong to this lab",
                ));
            }
        }
        Ok(Self {
            id,
            workspace_id,
            name,
            slug,
            description: description.into(),
            autonomy,
            monthly_budget,
            settings,
            archived_at: None,
            version: Version::ONE,
        })
    }

    /// Identity.
    #[must_use]
    pub const fn id(&self) -> LabId {
        self.id
    }

    /// Workspace scope.
    #[must_use]
    pub const fn workspace_id(&self) -> WorkspaceId {
        self.workspace_id
    }

    /// Display name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Description.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Slug.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn slug(&self) -> &Slug {
        &self.slug
    }

    /// Autonomy level.
    #[must_use]
    pub const fn autonomy(&self) -> AutonomyLevel {
        self.autonomy
    }

    /// Monthly budget.
    #[must_use]
    pub const fn monthly_budget(&self) -> Money {
        self.monthly_budget
    }

    /// Current settings.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn settings(&self) -> &LabSettings {
        &self.settings
    }

    /// True when archived.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn is_archived(&self) -> bool {
        self.archived_at.is_some()
    }

    /// Change the autonomy level after validation.
    #[allow(clippy::missing_const_for_fn)]
    pub fn set_autonomy(&mut self, autonomy: AutonomyLevel) {
        self.autonomy = autonomy;
    }

    /// Replace settings after validation.
    pub fn set_settings(&mut self, settings: LabSettings) -> Result<(), DomainError> {
        settings.validate()?;
        self.settings = settings;
        Ok(())
    }

    /// Archive the lab.
    pub fn archive(&mut self, at: DateTime<Utc>) -> Result<(), DomainError> {
        if self.archived_at.is_some() {
            return Err(DomainError::policy(
                "LAB_ALREADY_ARCHIVED",
                "lab is already archived",
            ));
        }
        self.archived_at = Some(at);
        self.version = self.version.next()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::AgentId;
    use crate::inventory::Agent;
    use crate::value::AgentRole;
    use chrono::Utc;

    fn description() -> String {
        "A sufficiently long fixture description used to exercise the one hundred twenty to six hundred character invariant in the lab tests.".to_owned()
    }

    fn agent(lab: LabId, role: AgentRole) -> Agent {
        Agent::create(
            AgentId::generate(),
            lab,
            "Fixture",
            role,
            "veteran_lead",
            description(),
            AutonomyLevel::A2,
            1,
            Utc::now(),
        )
        .expect("valid agent")
    }

    fn settings() -> LabSettings {
        LabSettings::default()
    }

    #[test]
    fn lab_requires_one_tech_lead() {
        let lab = LabId::generate();
        let workspace = WorkspaceId::generate();
        let no_lead = vec![agent(lab, AgentRole::Dev)];
        assert!(
            Lab::create(
                lab,
                workspace,
                "Demo",
                "demo-lab",
                "",
                AutonomyLevel::A2,
                Money::ZERO,
                settings(),
                &no_lead
            )
            .is_err()
        );
        let lead = vec![agent(lab, AgentRole::TechLead)];
        let created = Lab::create(
            lab,
            workspace,
            "Demo",
            "demo-lab",
            "",
            AutonomyLevel::A2,
            Money::ZERO,
            settings(),
            &lead,
        )
        .expect("lab with a lead");
        assert_eq!(created.autonomy(), AutonomyLevel::A2);
    }

    #[test]
    fn settings_validate_ranges() {
        let mut bad = settings();
        bad.wip_doing_per_agent = 0;
        assert!(bad.validate().is_err());
        let mut bad_budget = settings();
        bad_budget.per_card_budget_microusd = 60_000_000;
        assert!(bad_budget.validate().is_err());
        assert!(settings().validate().is_ok());
    }

    #[test]
    fn lab_archive_is_idempotent_guarded() {
        let lab = LabId::generate();
        let lead = vec![agent(lab, AgentRole::TechLead)];
        let mut created = Lab::create(
            lab,
            WorkspaceId::generate(),
            "Demo",
            "demo-lab",
            "",
            AutonomyLevel::A2,
            Money::ZERO,
            settings(),
            &lead,
        )
        .expect("lab");
        created.archive(Utc::now()).expect("archive");
        assert!(created.is_archived());
        assert!(created.archive(Utc::now()).is_err());
    }

    #[test]
    fn foreign_team_member_is_rejected() {
        let lab = LabId::generate();
        let foreign = vec![agent(LabId::generate(), AgentRole::TechLead)];
        assert!(
            Lab::create(
                lab,
                WorkspaceId::generate(),
                "Demo",
                "demo-lab",
                "",
                AutonomyLevel::A2,
                Money::ZERO,
                settings(),
                &foreign
            )
            .is_err()
        );
    }
}
