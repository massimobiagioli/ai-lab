//! RFC 9457 Problem Details mapping.

use ailab_domain::DomainError;
use serde::Serialize;

/// A machine-readable violation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Violation {
    /// Rule identifier.
    pub rule: String,
    /// Field that failed.
    pub field: String,
    /// Human-readable message.
    pub message: String,
}

/// RFC 9457 problem payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProblemDetails {
    /// Problem type URI.
    #[serde(rename = "type")]
    pub type_uri: String,
    /// Short title.
    pub title: String,
    /// HTTP status.
    pub status: u16,
    /// Stable machine code.
    pub code: String,
    /// Detail.
    pub detail: String,
    /// Violations.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub violations: Vec<Violation>,
}

impl From<DomainError> for ProblemDetails {
    fn from(error: DomainError) -> Self {
        let code = error.code();
        let violations = match &error {
            DomainError::Validation { field, message } => vec![Violation {
                rule: code.to_owned(),
                field: (*field).to_owned(),
                message: message.clone(),
            }],
            DomainError::Violation {
                code: violation_code,
                field,
                message,
            } => vec![Violation {
                rule: (*violation_code).to_owned(),
                field: (*field).to_owned(),
                message: message.clone(),
            }],
            _ => Vec::new(),
        };
        Self {
            type_uri: format!("https://ai-lab.dev/errors/{}", code.to_lowercase()),
            title: title_for(code).to_owned(),
            status: error.status(),
            code: code.to_owned(),
            detail: error.to_string(),
            violations,
        }
    }
}

fn title_for(code: &str) -> &'static str {
    match code {
        "VALIDATION_FAILED" => "Validation failed",
        "INVALID_TRANSITION" => "Invalid Kanban transition",
        "VERSION_CONFLICT" => "Version conflict",
        "SEPARATION_OF_DUTIES" => "Separation of duties violated",
        "NOT_FOUND" => "Not found",
        "CONFLICT" => "Conflict",
        _ => "Rule violation",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validation_maps_to_422_with_violation() {
        let problem = ProblemDetails::from(DomainError::validation("slug", "bad slug"));
        assert_eq!(problem.status, 422);
        assert_eq!(problem.code, "VALIDATION_FAILED");
        assert_eq!(problem.violations.len(), 1);
        assert!(problem.type_uri.ends_with("/validation_failed"));
    }

    #[test]
    fn version_conflict_maps_to_409() {
        let problem = ProblemDetails::from(DomainError::VersionConflict {
            expected: 1,
            current: 2,
        });
        assert_eq!(problem.status, 409);
        assert_eq!(problem.title, "Version conflict");
        assert_eq!(problem.violations, Vec::<Violation>::new());
    }
}
