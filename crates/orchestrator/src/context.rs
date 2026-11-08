//! Deterministic, token-budgeted context assembly (WP-13 §3.4).

/// One context section.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextSection {
    /// Section name (repo map, card, memory, files, conventions, feedback, state).
    pub name: &'static str,
    /// Section content.
    pub content: String,
    /// Higher priority sections are kept first.
    pub priority: u8,
}

impl ContextSection {
    /// Build a section.
    #[must_use]
    pub fn new(name: &'static str, content: impl Into<String>, priority: u8) -> Self {
        Self {
            name,
            content: content.into(),
            priority,
        }
    }

    /// Approximate token count (4 characters per token).
    #[must_use]
    pub fn tokens(&self) -> u32 {
        u32::try_from(self.content.chars().count().div_ceil(4)).unwrap_or(u32::MAX)
    }
}

/// The assembled context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextBundle {
    /// Kept sections, highest priority first.
    pub sections: Vec<ContextSection>,
    /// Sections that were summarized or dropped.
    pub omitted: Vec<&'static str>,
    /// Configured token budget.
    pub token_budget: u32,
    /// Approximate tokens used.
    pub used_tokens: u32,
    /// Source citations backing the packet.
    pub citations: Vec<String>,
}

impl ContextBundle {
    /// Render the bundle for the model.
    #[must_use]
    pub fn render(&self) -> String {
        let mut output = String::new();
        for section in &self.sections {
            output.push_str("## ");
            output.push_str(section.name);
            output.push_str("\n\n");
            output.push_str(&section.content);
            output.push_str("\n\n");
        }
        output
    }

    /// Attach source citations.
    #[must_use]
    pub fn with_citations(mut self, citations: Vec<String>) -> Self {
        self.citations = citations;
        self
    }

    /// True when the task packet is present.
    #[must_use]
    pub fn has_task(&self) -> bool {
        self.sections.iter().any(|section| section.name == "task")
    }
}

/// Build a context bundle under a token budget.
///
/// The `task` section is always kept; other sections are included by priority
/// while the budget allows, then summarized (priority >= 5) or omitted.
#[must_use]
pub fn build_context(mut sections: Vec<ContextSection>, token_budget: u32) -> ContextBundle {
    sections.sort_by_key(|section| section.priority);
    let mut kept = Vec::new();
    let mut omitted = Vec::new();
    let mut used = 0_u32;
    for mut section in sections {
        let tokens = section.tokens();
        let is_task = section.name == "task";
        if is_task || used.saturating_add(tokens) <= token_budget {
            used = used.saturating_add(tokens);
            kept.push(section);
        } else if section.priority >= 5 {
            let original = section.content.chars().count();
            section.content = format!("[summarized: {original} characters omitted]");
            used = used.saturating_add(section.tokens());
            if used <= token_budget {
                kept.push(section);
            } else {
                omitted.push(section.name);
            }
        } else {
            omitted.push(section.name);
        }
    }
    ContextBundle {
        sections: kept,
        omitted,
        token_budget,
        used_tokens: used,
        citations: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn section(name: &'static str, content: &str, priority: u8) -> ContextSection {
        ContextSection::new(name, content, priority)
    }

    #[test]
    fn task_is_never_dropped_and_budget_is_respected() {
        let sections = vec![
            section("task", "implement the card", 0),
            section("card", &"c".repeat(4_000), 1),
            section("files", &"f".repeat(4_000), 3),
            section("memory", &"m".repeat(4_000), 4),
        ];
        let bundle = build_context(sections, 1_200);
        assert!(bundle.has_task());
        assert!(bundle.used_tokens <= bundle.token_budget);
        assert_eq!(bundle.sections[0].name, "task");
    }

    #[test]
    fn high_priority_overflow_is_summarized() {
        let sections = vec![
            section("task", "task", 0),
            section("conventions", &"x".repeat(8_000), 6),
        ];
        let bundle = build_context(sections, 100);
        assert_eq!(bundle.omitted, Vec::<&str>::new());
        assert!(
            bundle
                .sections
                .iter()
                .any(|section| section.content.contains("[summarized:"))
        );
    }

    #[test]
    fn low_priority_overflow_is_omitted() {
        let sections = vec![
            section("task", "task", 0),
            section("files", &"f".repeat(8_000), 2),
        ];
        let bundle = build_context(sections, 100);
        assert_eq!(bundle.omitted, vec!["files"]);
    }

    #[test]
    fn rendering_is_deterministic() {
        let sections = vec![section("card", "card", 1), section("task", "task", 0)];
        let first = build_context(sections.clone(), 1_000);
        let second = build_context(sections, 1_000);
        assert_eq!(first, second);
        assert!(
            first.render().find("## task").expect("task")
                < first.render().find("## card").expect("card")
        );
    }
}
