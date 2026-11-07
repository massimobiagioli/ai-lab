//! Strongly typed identifiers and the human-facing card key.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

use crate::error::DomainError;

macro_rules! id_type {
    ($name:ident, $label:literal) => {
        /// Strongly typed non-nil identifier.
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
        )]
        #[serde(try_from = "Uuid", into = "Uuid")]
        pub struct $name(Uuid);

        impl $name {
            /// Validate and wrap a `UUID`.
            pub fn new(id: Uuid) -> Result<Self, DomainError> {
                if id.is_nil() {
                    return Err(DomainError::validation(
                        $label,
                        "identifier must not be nil",
                    ));
                }
                Ok(Self(id))
            }

            /// Generate a time-ordered `UUIDv7`.
            #[must_use]
            pub fn generate() -> Self {
                Self(Uuid::now_v7())
            }

            /// Return the inner UUID.
            #[must_use]
            pub const fn as_uuid(self) -> Uuid {
                self.0
            }
        }

        impl TryFrom<Uuid> for $name {
            type Error = DomainError;

            fn try_from(value: Uuid) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }

        impl From<$name> for Uuid {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl FromStr for $name {
            type Err = DomainError;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                let uuid = Uuid::parse_str(value)
                    .map_err(|_| DomainError::validation($label, "invalid UUID"))?;
                Self::new(uuid)
            }
        }
    };
}

id_type!(WorkspaceId, "workspace_id");
id_type!(LabId, "lab_id");
id_type!(AgentId, "agent_id");
id_type!(ProjectId, "project_id");
id_type!(CardId, "card_id");
id_type!(RunId, "run_id");
id_type!(SowId, "sow_id");
id_type!(PlanId, "plan_id");
id_type!(DocumentId, "document_id");
id_type!(ChunkId, "chunk_id");
id_type!(ProviderId, "provider_id");
id_type!(ModelId, "model_id");
id_type!(ScmCredentialId, "scm_credential_id");
id_type!(CvVersionId, "cv_version_id");
id_type!(EventId, "event_id");

/// Human-facing card key `{PREFIX}-{SEQ}`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct CardKey {
    prefix: String,
    seq: i64,
}

fn valid_prefix(prefix: &str) -> bool {
    let bytes = prefix.as_bytes();
    (2..=10).contains(&bytes.len())
        && bytes[0].is_ascii_uppercase()
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit())
}

impl CardKey {
    /// Build a card key from a project prefix and a positive sequence.
    pub fn new(prefix: &str, seq: i64) -> Result<Self, DomainError> {
        if !valid_prefix(prefix) {
            return Err(DomainError::validation(
                "card_key",
                "prefix must be 2-10 uppercase ASCII letters or digits",
            ));
        }
        if seq <= 0 {
            return Err(DomainError::validation(
                "card_key",
                "sequence must be positive",
            ));
        }
        Ok(Self {
            prefix: prefix.to_owned(),
            seq,
        })
    }

    /// Parse `{PREFIX}-{SEQ}`.
    pub fn parse(value: &str) -> Result<Self, DomainError> {
        let (prefix, seq) = value
            .rsplit_once('-')
            .ok_or_else(|| DomainError::validation("card_key", "expected PREFIX-SEQ"))?;
        let seq = seq
            .parse::<i64>()
            .map_err(|_| DomainError::validation("card_key", "sequence is not an integer"))?;
        Self::new(prefix, seq)
    }

    /// Project prefix.
    #[must_use]
    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    /// Project-local sequence.
    #[must_use]
    pub const fn seq(&self) -> i64 {
        self.seq
    }
}

impl fmt::Display for CardKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", self.prefix, self.seq)
    }
}

impl TryFrom<String> for CardKey {
    type Error = DomainError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(&value)
    }
}

impl From<CardKey> for String {
    fn from(value: CardKey) -> Self {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn card_key_round_trip() {
        let key = CardKey::new("AIL", 42).expect("valid");
        assert_eq!(key.to_string(), "AIL-42");
        assert_eq!(CardKey::parse("AIL-42").expect("parse"), key);
    }

    #[test]
    fn card_key_rejects_bad_prefix_and_seq() {
        assert!(CardKey::new("ail", 1).is_err());
        assert!(CardKey::new("A", 1).is_err());
        assert!(CardKey::new("AIL", 0).is_err());
        assert!(CardKey::parse("AIL").is_err());
        assert!(CardKey::parse("AIL-abc").is_err());
    }

    #[test]
    fn ids_reject_nil_and_parse() {
        assert!(LabId::new(Uuid::nil()).is_err());
        let id = LabId::generate();
        assert_eq!(LabId::from_str(&id.to_string()).expect("parse"), id);
    }
}
