//! Domain error taxonomy with stable codes and HTTP status mapping.

/// A domain-level failure with a stable machine-readable code.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DomainError {
    /// A value failed validation.
    #[error("validation failed for {field}: {message}")]
    Validation {
        /// Field that failed.
        field: &'static str,
        /// Human-readable reason.
        message: String,
    },
    /// A Kanban transition is not allowed.
    #[error("invalid transition")]
    InvalidTransition,
    /// Optimistic concurrency lost.
    #[error("version conflict: expected {expected}, current {current}")]
    VersionConflict {
        /// Expected version.
        expected: u32,
        /// Current version.
        current: u32,
    },
    /// Reviewer equals author or merger equals author.
    #[error("separation of duties violated")]
    SeparationOfDuties,
    /// The aggregate does not exist.
    #[error("not found")]
    NotFound,
    /// A generic conflict.
    #[error("conflict: {0}")]
    Conflict(String),
    /// A policy rule with a specific code.
    #[error("{message}")]
    Policy {
        /// Stable code.
        code: &'static str,
        /// Human-readable message.
        message: String,
    },
}

impl DomainError {
    /// Stable code used by the API error payload.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::Validation { .. } => "VALIDATION_FAILED",
            Self::InvalidTransition => "INVALID_TRANSITION",
            Self::VersionConflict { .. } => "VERSION_CONFLICT",
            Self::SeparationOfDuties => "SEPARATION_OF_DUTIES",
            Self::NotFound => "NOT_FOUND",
            Self::Conflict(_) => "CONFLICT",
            Self::Policy { code, .. } => code,
        }
    }

    /// HTTP status used by RFC 9457 mapping.
    #[must_use]
    pub const fn status(&self) -> u16 {
        match self {
            Self::Validation { .. }
            | Self::InvalidTransition
            | Self::SeparationOfDuties
            | Self::Policy { .. } => 422,
            Self::VersionConflict { .. } | Self::Conflict(_) => 409,
            Self::NotFound => 404,
        }
    }

    /// Constructor for a validation error.
    #[must_use]
    pub fn validation(field: &'static str, message: impl Into<String>) -> Self {
        Self::Validation {
            field,
            message: message.into(),
        }
    }

    /// Constructor for a policy error with a specific code.
    #[must_use]
    pub fn policy(code: &'static str, message: impl Into<String>) -> Self {
        Self::Policy {
            code,
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_and_status_are_stable() {
        assert_eq!(DomainError::InvalidTransition.code(), "INVALID_TRANSITION");
        assert_eq!(DomainError::InvalidTransition.status(), 422);
        assert_eq!(
            DomainError::VersionConflict {
                expected: 1,
                current: 2
            }
            .status(),
            409
        );
        assert_eq!(DomainError::NotFound.status(), 404);
    }
}
