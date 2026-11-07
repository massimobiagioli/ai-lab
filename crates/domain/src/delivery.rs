//! Delivery context: versioned SOW and acyclic plan materialization.

use std::collections::{BTreeMap, BTreeSet};

use crate::error::DomainError;
use crate::ids::{LabId, PlanId, ProjectId, SowId};
use crate::kanban::CardStory;
use crate::value::{AgentRole, ContentHash, Estimate};

/// Mandatory SOW sections.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SowSection {
    /// Executive summary.
    ExecutiveSummary,
    /// Objectives.
    Objectives,
    /// In scope.
    ScopeIn,
    /// Out of scope.
    ScopeOut,
    /// Stakeholders.
    Stakeholders,
    /// Functional requirements.
    FunctionalRequirements,
    /// Non-functional requirements.
    NonFunctionalRequirements,
    /// Architecture.
    Architecture,
    /// Milestones.
    Milestones,
    /// Deliverables.
    Deliverables,
    /// Acceptance criteria.
    AcceptanceCriteria,
    /// Assumptions.
    Assumptions,
    /// Risks.
    Risks,
    /// Timeline.
    Timeline,
    /// Budget.
    Budget,
    /// Governance.
    Governance,
    /// Change control.
    ChangeControl,
}

impl SowSection {
    /// Human-readable label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::ExecutiveSummary => "Executive summary",
            Self::Objectives => "Objectives",
            Self::ScopeIn => "Scope in",
            Self::ScopeOut => "Scope out",
            Self::Stakeholders => "Stakeholders",
            Self::FunctionalRequirements => "Functional requirements",
            Self::NonFunctionalRequirements => "Non-functional requirements",
            Self::Architecture => "Architecture",
            Self::Milestones => "Milestones",
            Self::Deliverables => "Deliverables",
            Self::AcceptanceCriteria => "Acceptance criteria",
            Self::Assumptions => "Assumptions",
            Self::Risks => "Risks",
            Self::Timeline => "Timeline",
            Self::Budget => "Budget",
            Self::Governance => "Governance",
            Self::ChangeControl => "Change control",
        }
    }
}

/// The 17 mandatory sections.
pub const REQUIRED_SECTIONS: [SowSection; 17] = [
    SowSection::ExecutiveSummary,
    SowSection::Objectives,
    SowSection::ScopeIn,
    SowSection::ScopeOut,
    SowSection::Stakeholders,
    SowSection::FunctionalRequirements,
    SowSection::NonFunctionalRequirements,
    SowSection::Architecture,
    SowSection::Milestones,
    SowSection::Deliverables,
    SowSection::AcceptanceCriteria,
    SowSection::Assumptions,
    SowSection::Risks,
    SowSection::Timeline,
    SowSection::Budget,
    SowSection::Governance,
    SowSection::ChangeControl,
];

/// Validated SOW document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SowDocument {
    sections: BTreeMap<SowSection, String>,
}

impl SowDocument {
    /// Validate required sections and semantics.
    pub fn new(sections: BTreeMap<SowSection, String>) -> Result<Self, DomainError> {
        for section in REQUIRED_SECTIONS {
            match sections.get(&section) {
                Some(value) if !value.trim().is_empty() => {}
                _ => {
                    return Err(DomainError::validation(
                        "sow.sections",
                        format!("missing or empty section: {}", section.label()),
                    ));
                }
            }
        }
        let functional = sections
            .get(&SowSection::FunctionalRequirements)
            .map_or("", String::as_str);
        let mut ids = BTreeSet::new();
        for token in functional.split_whitespace() {
            if token.starts_with("FR-") && !ids.insert(token.to_owned()) {
                return Err(DomainError::validation(
                    "sow.functional_requirements",
                    format!("duplicate requirement id: {token}"),
                ));
            }
        }
        Ok(Self { sections })
    }

    /// Section content.
    #[must_use]
    pub fn section(&self, section: SowSection) -> &str {
        self.sections.get(&section).map_or("", String::as_str)
    }

    /// Deterministic Markdown rendering.
    #[must_use]
    pub fn markdown(&self) -> String {
        let mut output = String::new();
        for section in REQUIRED_SECTIONS {
            output.push_str("## ");
            output.push_str(section.label());
            output.push_str("\n\n");
            output.push_str(self.section(section));
            output.push_str("\n\n");
        }
        output
    }

    /// Content hash.
    #[must_use]
    pub fn content_hash(&self) -> ContentHash {
        ContentHash::from_bytes(self.markdown().as_bytes())
    }
}

/// SOW lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SowStatus {
    /// Draft.
    Draft,
    /// In review.
    InReview,
    /// Approved and frozen.
    Approved,
    /// Rejected.
    Rejected,
    /// Superseded by a newer approved version.
    Superseded,
}

/// Immutable SOW version.
#[derive(Debug, Clone)]
pub struct SowVersion {
    /// Version number.
    pub version: u32,
    /// Document.
    pub document: SowDocument,
    /// Content hash.
    pub content_hash: ContentHash,
    /// Change note.
    pub change_note: String,
}

/// SOW aggregate.
#[derive(Debug, Clone)]
pub struct Sow {
    id: SowId,
    project_id: ProjectId,
    lab_id: LabId,
    title: String,
    status: SowStatus,
    versions: Vec<SowVersion>,
    approved_by: Option<String>,
}

impl Sow {
    /// Create a draft SOW at version 1.
    pub fn create(
        id: SowId,
        project_id: ProjectId,
        lab_id: LabId,
        title: impl Into<String>,
        document: SowDocument,
    ) -> Result<Self, DomainError> {
        let title = title.into();
        if title.trim().is_empty() {
            return Err(DomainError::validation("sow.title", "must not be empty"));
        }
        let content_hash = document.content_hash();
        Ok(Self {
            id,
            project_id,
            lab_id,
            title,
            status: SowStatus::Draft,
            versions: vec![SowVersion {
                version: 1,
                document,
                content_hash,
                change_note: "Initial version".to_owned(),
            }],
            approved_by: None,
        })
    }

    /// Identity.
    #[must_use]
    pub const fn id(&self) -> SowId {
        self.id
    }

    /// Project scope.
    #[must_use]
    pub const fn project_id(&self) -> ProjectId {
        self.project_id
    }

    /// Lab scope.
    #[must_use]
    pub const fn lab_id(&self) -> LabId {
        self.lab_id
    }

    /// Status.
    #[must_use]
    pub const fn status(&self) -> SowStatus {
        self.status
    }

    /// Title.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Current version.
    #[must_use]
    pub fn current(&self) -> Option<&SowVersion> {
        self.versions.last()
    }

    /// Immutable versions.
    #[must_use]
    pub fn versions(&self) -> &[SowVersion] {
        &self.versions
    }

    /// Submit for review.
    pub fn submit(&mut self) -> Result<(), DomainError> {
        if self.status != SowStatus::Draft {
            return Err(DomainError::policy(
                "SOW_NOT_DRAFT",
                "only a draft SOW can be submitted",
            ));
        }
        self.status = SowStatus::InReview;
        Ok(())
    }

    /// Approve the current version.
    pub fn approve(&mut self, actor: impl Into<String>) -> Result<(), DomainError> {
        if self.status != SowStatus::InReview {
            return Err(DomainError::policy(
                "SOW_NOT_IN_REVIEW",
                "only an in-review SOW can be approved",
            ));
        }
        self.status = SowStatus::Approved;
        self.approved_by = Some(actor.into());
        Ok(())
    }

    /// Reject the current version.
    pub fn reject(&mut self, _reason: impl Into<String>) -> Result<(), DomainError> {
        if self.status != SowStatus::InReview {
            return Err(DomainError::policy(
                "SOW_NOT_IN_REVIEW",
                "only an in-review SOW can be rejected",
            ));
        }
        self.status = SowStatus::Rejected;
        Ok(())
    }

    /// Create a new immutable version and return to draft.
    pub fn revise(
        &mut self,
        document: SowDocument,
        change_note: impl Into<String>,
    ) -> Result<u32, DomainError> {
        let change_note = change_note.into();
        if change_note.trim().is_empty() {
            return Err(DomainError::validation(
                "sow.change_note",
                "must not be empty",
            ));
        }
        let Some(current) = self.current() else {
            return Err(DomainError::validation(
                "sow.versions",
                "missing current version",
            ));
        };
        let next = current.version + 1;
        let content_hash = document.content_hash();
        self.versions.push(SowVersion {
            version: next,
            document,
            content_hash,
            change_note,
        });
        self.status = SowStatus::Draft;
        self.approved_by = None;
        Ok(next)
    }

    /// Supersede an approved SOW.
    pub fn supersede(&mut self) -> Result<(), DomainError> {
        if self.status != SowStatus::Approved {
            return Err(DomainError::policy(
                "SOW_NOT_APPROVED",
                "only an approved SOW can be superseded",
            ));
        }
        self.status = SowStatus::Superseded;
        Ok(())
    }
}

/// WBS task.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanTask {
    /// Stable key within the plan.
    pub task_key: String,
    /// Title.
    pub title: String,
    /// Role.
    pub role: AgentRole,
    /// Estimate.
    pub estimate: Option<Estimate>,
    /// Dependencies.
    pub depends_on: Vec<String>,
    /// Deliverables.
    pub deliverables: Vec<String>,
    /// Acceptance criteria.
    pub acceptance_criteria: Vec<String>,
}

/// Plan lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanStatus {
    /// Draft.
    Draft,
    /// Approved.
    Approved,
    /// Materialized.
    Materialized,
    /// Superseded.
    Superseded,
}

/// Card preview produced by materialization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterializedCard {
    /// Source task.
    pub task_key: String,
    /// Title.
    pub title: String,
    /// Story.
    pub story: CardStory,
    /// Estimate.
    pub estimate: Option<Estimate>,
    /// Role.
    pub role: AgentRole,
    /// Dependencies.
    pub depends_on: Vec<String>,
    /// SOW traceability.
    pub sow_id: SowId,
}

/// Plan aggregate with an acyclic task graph.
#[derive(Debug, Clone)]
pub struct Plan {
    id: PlanId,
    project_id: ProjectId,
    sow_id: SowId,
    title: String,
    status: PlanStatus,
    tasks: Vec<PlanTask>,
    version: u32,
}

impl Plan {
    /// Create a validated plan.
    pub fn create(
        id: PlanId,
        project_id: ProjectId,
        sow_id: SowId,
        title: impl Into<String>,
        tasks: Vec<PlanTask>,
    ) -> Result<Self, DomainError> {
        let title = title.into();
        if title.trim().is_empty() {
            return Err(DomainError::validation("plan.title", "must not be empty"));
        }
        validate_tasks(&tasks)?;
        let plan = Self {
            id,
            project_id,
            sow_id,
            title,
            status: PlanStatus::Draft,
            tasks,
            version: 1,
        };
        plan.topological_order()?;
        Ok(plan)
    }

    /// Identity.
    #[must_use]
    pub const fn id(&self) -> PlanId {
        self.id
    }

    /// Project scope.
    #[must_use]
    pub const fn project_id(&self) -> ProjectId {
        self.project_id
    }

    /// Title.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// SOW traceability.
    #[must_use]
    pub const fn sow_id(&self) -> SowId {
        self.sow_id
    }

    /// Status.
    #[must_use]
    pub const fn status(&self) -> PlanStatus {
        self.status
    }

    /// Tasks.
    #[must_use]
    pub fn tasks(&self) -> &[PlanTask] {
        &self.tasks
    }

    /// Approve the plan.
    pub fn approve(&mut self) -> Result<(), DomainError> {
        if self.status != PlanStatus::Draft {
            return Err(DomainError::policy(
                "PLAN_NOT_DRAFT",
                "only a draft plan can be approved",
            ));
        }
        self.status = PlanStatus::Approved;
        Ok(())
    }

    /// Topological order of task keys.
    pub fn topological_order(&self) -> Result<Vec<String>, DomainError> {
        let mut indegree: BTreeMap<String, usize> = self
            .tasks
            .iter()
            .map(|task| (task.task_key.clone(), task.depends_on.len()))
            .collect();
        let mut dependents: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for task in &self.tasks {
            for dependency in &task.depends_on {
                dependents
                    .entry(dependency.clone())
                    .or_default()
                    .push(task.task_key.clone());
            }
        }
        let mut queue: Vec<String> = indegree
            .iter()
            .filter(|(_, count)| **count == 0)
            .map(|(key, _)| key.clone())
            .collect();
        let mut order = Vec::new();
        while let Some(key) = queue.pop() {
            order.push(key.clone());
            if let Some(children) = dependents.get(&key) {
                for child in children {
                    if let Some(count) = indegree.get_mut(child) {
                        *count = count.saturating_sub(1);
                        if *count == 0 {
                            queue.push(child.clone());
                        }
                    }
                }
            }
        }
        if order.len() != self.tasks.len() {
            return Err(DomainError::validation(
                "plan.graph",
                "cycle detected in the plan",
            ));
        }
        Ok(order)
    }

    /// WBS leaves: tasks no other task depends on.
    #[must_use]
    pub fn leaves(&self) -> Vec<&PlanTask> {
        let depended: BTreeSet<&str> = self
            .tasks
            .iter()
            .flat_map(|task| task.depends_on.iter().map(String::as_str))
            .collect();
        self.tasks
            .iter()
            .filter(|task| !depended.contains(task.task_key.as_str()))
            .collect()
    }

    /// Preview materialization without mutating the plan.
    pub fn preview(&self) -> Result<Vec<MaterializedCard>, DomainError> {
        let order = self.topological_order()?;
        let leaves: BTreeSet<&str> = self
            .leaves()
            .iter()
            .map(|task| task.task_key.as_str())
            .collect();
        let mut cards = Vec::new();
        for key in order {
            if !leaves.contains(key.as_str()) {
                continue;
            }
            let Some(task) = self.tasks.iter().find(|task| task.task_key == key) else {
                continue;
            };
            cards.push(MaterializedCard {
                task_key: task.task_key.clone(),
                title: task.title.clone(),
                story: CardStory {
                    acceptance_criteria: task.acceptance_criteria.clone(),
                    deliverables: task.deliverables.clone(),
                    needs_refinement: false,
                },
                estimate: task.estimate,
                role: task.role,
                depends_on: task.depends_on.clone(),
                sow_id: self.sow_id,
            });
        }
        Ok(cards)
    }

    /// Mark the plan materialized after the cards are created.
    pub fn mark_materialized(&mut self) -> Result<(), DomainError> {
        if self.status != PlanStatus::Approved {
            return Err(DomainError::policy(
                "PLAN_NOT_APPROVED",
                "only an approved plan can be materialized",
            ));
        }
        self.status = PlanStatus::Materialized;
        self.version += 1;
        Ok(())
    }
}

fn validate_tasks(tasks: &[PlanTask]) -> Result<(), DomainError> {
    if tasks.is_empty() {
        return Err(DomainError::validation("plan.tasks", "must not be empty"));
    }
    let mut keys = BTreeSet::new();
    for task in tasks {
        if task.title.trim().is_empty() {
            return Err(DomainError::validation(
                "plan.task.title",
                "must not be empty",
            ));
        }
        if !keys.insert(task.task_key.clone()) {
            return Err(DomainError::validation(
                "plan.task_key",
                format!("duplicate task key: {}", task.task_key),
            ));
        }
        if task.estimate.is_none() {
            return Err(DomainError::validation("plan.task.estimate", "required"));
        }
        if task.deliverables.is_empty() || task.acceptance_criteria.is_empty() {
            return Err(DomainError::validation(
                "plan.task.content",
                "deliverables and acceptance criteria are required",
            ));
        }
    }
    for task in tasks {
        for dependency in &task.depends_on {
            if !keys.contains(dependency) {
                return Err(DomainError::validation(
                    "plan.task.depends_on",
                    format!("unknown dependency: {dependency}"),
                ));
            }
            if dependency == &task.task_key {
                return Err(DomainError::validation(
                    "plan.task.depends_on",
                    "a task cannot depend on itself",
                ));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::{LabId, PlanId, ProjectId, SowId};

    fn sections() -> BTreeMap<SowSection, String> {
        REQUIRED_SECTIONS
            .iter()
            .map(|section| (*section, format!("{} content", section.label())))
            .collect()
    }

    fn document() -> SowDocument {
        SowDocument::new(sections()).expect("document")
    }

    fn task(key: &str, depends_on: Vec<&str>) -> PlanTask {
        PlanTask {
            task_key: key.to_owned(),
            title: format!("Task {key}"),
            role: AgentRole::Dev,
            estimate: Some(Estimate::new(3).expect("estimate")),
            depends_on: depends_on.into_iter().map(str::to_owned).collect(),
            deliverables: vec![format!("docs/{key}.md")],
            acceptance_criteria: vec![format!("{key} works")],
        }
    }

    #[test]
    fn sow_requires_sections_and_unique_requirement_ids() {
        let mut missing = sections();
        missing.remove(&SowSection::ScopeOut);
        assert!(SowDocument::new(missing).is_err());
        let mut duplicate = sections();
        duplicate.insert(
            SowSection::FunctionalRequirements,
            "FR-001 first FR-001 second".to_owned(),
        );
        assert!(SowDocument::new(duplicate).is_err());
        assert_eq!(document().markdown(), document().markdown());
    }

    #[test]
    fn sow_lifecycle_and_versions() {
        let mut sow = Sow::create(
            SowId::generate(),
            ProjectId::generate(),
            LabId::generate(),
            "Demo SOW",
            document(),
        )
        .expect("sow");
        assert_eq!(sow.status(), SowStatus::Draft);
        sow.submit().expect("submit");
        sow.approve("operator").expect("approve");
        assert_eq!(sow.status(), SowStatus::Approved);
        let version = sow.revise(document(), "second version").expect("revise");
        assert_eq!(version, 2);
        assert_eq!(sow.status(), SowStatus::Draft);
        assert_eq!(sow.versions().len(), 2);
        assert_eq!(sow.current().expect("current").version, 2);
    }

    #[test]
    fn plan_rejects_cycles_unknown_dependencies_and_duplicates() {
        let project = ProjectId::generate();
        let sow = SowId::generate();
        assert!(
            Plan::create(
                PlanId::generate(),
                project,
                sow,
                "Cycle",
                vec![task("T1", vec!["T2"]), task("T2", vec!["T1"])]
            )
            .is_err()
        );
        assert!(
            Plan::create(
                PlanId::generate(),
                project,
                sow,
                "Unknown",
                vec![task("T1", vec!["T9"])]
            )
            .is_err()
        );
        assert!(
            Plan::create(
                PlanId::generate(),
                project,
                sow,
                "Duplicate",
                vec![task("T1", vec![]), task("T1", vec![])]
            )
            .is_err()
        );
    }

    #[test]
    fn plan_materializes_leaves_with_traceability() {
        let project = ProjectId::generate();
        let sow = SowId::generate();
        let mut plan = Plan::create(
            PlanId::generate(),
            project,
            sow,
            "Plan",
            vec![
                task("T1", vec![]),
                task("T2", vec!["T1"]),
                task("T3", vec!["T1"]),
            ],
        )
        .expect("plan");
        assert_eq!(plan.topological_order().expect("order")[0], "T1");
        let preview = plan.preview().expect("preview");
        assert_eq!(preview.len(), 2);
        assert!(preview.iter().all(|card| card.sow_id == sow));
        assert_eq!(preview[0].story.deliverables.len(), 1);
        plan.approve().expect("approve");
        plan.mark_materialized().expect("materialized");
        assert_eq!(plan.status(), PlanStatus::Materialized);
    }
}
