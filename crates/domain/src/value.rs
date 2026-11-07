//! Validated value objects for the domain kernel.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;

use crate::error::DomainError;
use crate::ids::AgentId;

/// Lowercase ASCII slug.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Slug(String);

impl Slug {
    /// Validate and build a slug.
    pub fn new(value: &str) -> Result<Self, DomainError> {
        if !(2..=64).contains(&value.len()) {
            return Err(DomainError::validation("slug", "length must be 2-64"));
        }
        if value.starts_with('-') || value.ends_with('-') {
            return Err(DomainError::validation(
                "slug",
                "must not start or end with '-'",
            ));
        }
        if !value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        {
            return Err(DomainError::validation(
                "slug",
                "only lowercase letters, digits and '-' are allowed",
            ));
        }
        Ok(Self(value.to_owned()))
    }

    /// Borrow the slug.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for Slug {
    type Error = DomainError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(&value)
    }
}

impl From<Slug> for String {
    fn from(value: Slug) -> Self {
        value.0
    }
}

impl fmt::Display for Slug {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Non-negative money in micro-USD (`1 USD = 1_000_000`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(try_from = "i64", into = "i64")]
pub struct Money(i64);

impl Money {
    /// Zero micro-USD.
    pub const ZERO: Self = Self(0);

    /// Build from an integer amount; rejects negatives.
    pub fn from_microusd(value: i64) -> Result<Self, DomainError> {
        if value < 0 {
            return Err(DomainError::validation(
                "money",
                "amount must not be negative",
            ));
        }
        Ok(Self(value))
    }

    /// Return the integer amount.
    #[must_use]
    pub const fn microusd(self) -> i64 {
        self.0
    }

    /// Checked addition.
    pub fn checked_add(self, other: Self) -> Result<Self, DomainError> {
        self.0
            .checked_add(other.0)
            .map(Self)
            .ok_or_else(|| DomainError::validation("money", "addition overflow"))
    }

    /// Checked subtraction; the result must remain non-negative.
    pub fn checked_sub(self, other: Self) -> Result<Self, DomainError> {
        self.0
            .checked_sub(other.0)
            .filter(|value| *value >= 0)
            .map(Self)
            .ok_or_else(|| DomainError::validation("money", "subtraction underflow"))
    }
}

impl TryFrom<i64> for Money {
    type Error = DomainError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        Self::from_microusd(value)
    }
}

impl From<Money> for i64 {
    fn from(value: Money) -> Self {
        value.0
    }
}

/// Monotonic aggregate version starting at 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "u32", into = "u32")]
pub struct Version(u32);

impl Version {
    /// The first version of a new aggregate.
    pub const ONE: Self = Self(1);

    /// Build a version; rejects zero.
    pub fn new(value: u32) -> Result<Self, DomainError> {
        if value == 0 {
            return Err(DomainError::validation("version", "must be at least 1"));
        }
        Ok(Self(value))
    }

    /// Return the integer version.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }

    /// Increment the version.
    pub fn next(self) -> Result<Self, DomainError> {
        self.0
            .checked_add(1)
            .ok_or_else(|| DomainError::validation("version", "overflow"))
            .and_then(Self::new)
    }
}

impl TryFrom<u32> for Version {
    type Error = DomainError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<Version> for u32 {
    fn from(value: Version) -> Self {
        value.0
    }
}

/// Fixed agent roles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentRole {
    /// Exactly one active per lab.
    TechLead,
    /// SOW and planning.
    Analyst,
    /// Implementation.
    Dev,
    /// Independent verification.
    Qa,
}

impl AgentRole {
    /// Stable wire name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TechLead => "tech_lead",
            Self::Analyst => "analyst",
            Self::Dev => "dev",
            Self::Qa => "qa",
        }
    }
}

/// Kanban column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum KanbanColumn {
    /// Backlog.
    Todo,
    /// Refined and ready.
    Ready,
    /// In progress.
    Doing,
    /// Awaiting review.
    Review,
    /// Complete.
    Done,
    /// Parked.
    Icebox,
}

impl KanbanColumn {
    /// Stable wire name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Todo => "TODO",
            Self::Ready => "READY",
            Self::Doing => "DOING",
            Self::Review => "REVIEW",
            Self::Done => "DONE",
            Self::Icebox => "ICEBOX",
        }
    }
}

/// Card priority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    /// Must have.
    Must,
    /// Should have.
    Should,
    /// Could have.
    Could,
    /// Explicitly out of scope.
    Wont,
}

impl Priority {
    /// Stable wire name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Must => "must",
            Self::Should => "should",
            Self::Could => "could",
            Self::Wont => "wont",
        }
    }
}

/// Fibonacci estimate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub struct Estimate(u8);

impl Estimate {
    const ALLOWED: [u8; 6] = [1, 2, 3, 5, 8, 13];

    /// Build an estimate from the allowed Fibonacci set.
    pub fn new(value: u8) -> Result<Self, DomainError> {
        if Self::ALLOWED.contains(&value) {
            Ok(Self(value))
        } else {
            Err(DomainError::validation(
                "estimate",
                "must be one of 1,2,3,5,8,13",
            ))
        }
    }

    /// Numeric points.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }

    /// A 13-point card must be split before READY.
    #[must_use]
    pub const fn requires_split(self) -> bool {
        self.0 == 13
    }
}

impl TryFrom<u8> for Estimate {
    type Error = DomainError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<Estimate> for u8 {
    fn from(value: Estimate) -> Self {
        value.0
    }
}

/// Autonomy level A0 through A3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum AutonomyLevel {
    /// Human approves every transition and merge.
    #[serde(rename = "A0")]
    A0,
    /// Agents move to REVIEW; humans merge.
    #[serde(rename = "A1")]
    A1,
    /// Agents merge with QA approval, green CI and budget.
    #[serde(rename = "A2")]
    A2,
    /// Agents may also create projects and pick dependencies.
    #[serde(rename = "A3")]
    A3,
}

impl AutonomyLevel {
    /// Effective autonomy is the minimum of lab, card and agent.
    #[must_use]
    pub fn effective(lab: Self, card: Self, agent: Self) -> Self {
        lab.min(card).min(agent)
    }

    /// Stable wire name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::A0 => "A0",
            Self::A1 => "A1",
            Self::A2 => "A2",
            Self::A3 => "A3",
        }
    }
}

/// Future due date.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DueDate(DateTime<Utc>);

impl DueDate {
    /// Borrow the timestamp.
    #[must_use]
    pub const fn at(self) -> DateTime<Utc> {
        self.0
    }

    /// True when the due date is strictly after `now`.
    #[must_use]
    pub fn is_future(self, now: DateTime<Utc>) -> bool {
        self.0 > now
    }
}

/// Actor kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActorType {
    /// Human operator.
    Human,
    /// Agent.
    Agent,
    /// System.
    System,
}

/// Actor of a mutation or event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Actor {
    /// Actor kind.
    #[serde(rename = "type")]
    pub actor_type: ActorType,
    /// Required for agent actors.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<AgentId>,
}

impl Actor {
    /// Human actor.
    #[must_use]
    pub const fn human() -> Self {
        Self {
            actor_type: ActorType::Human,
            id: None,
        }
    }

    /// System actor.
    #[must_use]
    pub const fn system() -> Self {
        Self {
            actor_type: ActorType::System,
            id: None,
        }
    }

    /// Agent actor; the ID is mandatory.
    #[must_use]
    pub const fn agent(id: AgentId) -> Self {
        Self {
            actor_type: ActorType::Agent,
            id: Some(id),
        }
    }

    /// Validate the actor shape.
    pub fn validate(&self) -> Result<(), DomainError> {
        if self.actor_type == ActorType::Agent && self.id.is_none() {
            return Err(DomainError::validation(
                "actor",
                "agent actors require an agent id",
            ));
        }
        Ok(())
    }
}

/// SHA-256 content hash in the `sha256:<hex>` form.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct ContentHash(String);

impl ContentHash {
    /// Validate an existing hash string.
    pub fn new(value: &str) -> Result<Self, DomainError> {
        let Some(hex) = value.strip_prefix("sha256:") else {
            return Err(DomainError::validation(
                "content_hash",
                "missing sha256: prefix",
            ));
        };
        if hex.len() != 64 || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(DomainError::validation(
                "content_hash",
                "expected 64 hexadecimal characters",
            ));
        }
        Ok(Self(value.to_ascii_lowercase()))
    }

    /// Hash bytes with SHA-256.
    #[must_use]
    pub fn from_bytes(bytes: &[u8]) -> Self {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let digest = Sha256::digest(bytes);
        let mut hex = String::with_capacity(64);
        for byte in digest {
            hex.push(char::from(HEX[usize::from(byte >> 4)]));
            hex.push(char::from(HEX[usize::from(byte & 0x0f)]));
        }
        Self(format!("sha256:{hex}"))
    }

    /// Borrow the hash.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for ContentHash {
    type Error = DomainError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(&value)
    }
}

impl From<ContentHash> for String {
    fn from(value: ContentHash) -> Self {
        value.0
    }
}

/// Idempotency key stored hashed by the API.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct IdempotencyKey(String);

impl IdempotencyKey {
    /// Validate the key length.
    pub fn new(value: &str) -> Result<Self, DomainError> {
        if !(8..=128).contains(&value.len()) {
            return Err(DomainError::validation(
                "idempotency_key",
                "length must be 8-128",
            ));
        }
        Ok(Self(value.to_owned()))
    }

    /// Borrow the key.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for IdempotencyKey {
    type Error = DomainError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(&value)
    }
}

impl From<IdempotencyKey> for String {
    fn from(value: IdempotencyKey) -> Self {
        value.0
    }
}

/// W3C trace id: 32 lowercase hexadecimal characters.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct TraceId(String);

impl TraceId {
    /// Validate a trace id.
    pub fn new(value: &str) -> Result<Self, DomainError> {
        if value.len() != 32 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(DomainError::validation(
                "trace_id",
                "expected 32 hexadecimal characters",
            ));
        }
        Ok(Self(value.to_ascii_lowercase()))
    }

    /// Borrow the trace id.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for TraceId {
    type Error = DomainError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(&value)
    }
}

impl From<TraceId> for String {
    fn from(value: TraceId) -> Self {
        value.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::CardKey;
    use proptest::prelude::*;

    #[test]
    fn slug_validation() {
        assert!(Slug::new("demo-lab").is_ok());
        assert!(Slug::new("Demo").is_err());
        assert!(Slug::new("-demo").is_err());
        assert!(Slug::new("a").is_err());
    }

    #[test]
    fn money_checked_arithmetic() {
        let five = Money::from_microusd(5).expect("valid");
        let three = Money::from_microusd(3).expect("valid");
        assert_eq!(five.checked_add(three).expect("add").microusd(), 8);
        assert_eq!(five.checked_sub(three).expect("sub").microusd(), 2);
        assert!(three.checked_sub(five).is_err());
        assert!(Money::from_microusd(-1).is_err());
    }

    #[test]
    fn version_rejects_zero_and_increments() {
        assert!(Version::new(0).is_err());
        let version = Version::new(1).expect("valid");
        assert_eq!(version.next().expect("next").get(), 2);
    }

    #[test]
    fn estimate_allowed_set() {
        assert!(Estimate::new(8).is_ok());
        assert!(Estimate::new(4).is_err());
        assert!(Estimate::new(13).expect("valid").requires_split());
    }

    #[test]
    fn autonomy_effective_is_minimum() {
        assert_eq!(
            AutonomyLevel::effective(AutonomyLevel::A3, AutonomyLevel::A1, AutonomyLevel::A2),
            AutonomyLevel::A1
        );
    }

    #[test]
    fn content_hash_round_trip() {
        let hash = ContentHash::from_bytes(b"abc");
        assert_eq!(hash.as_str().len(), 71);
        assert_eq!(ContentHash::new(hash.as_str()).expect("valid"), hash);
        assert!(ContentHash::new("md5:abc").is_err());
    }

    #[test]
    fn agent_actor_requires_id() {
        let actor = Actor {
            actor_type: ActorType::Agent,
            id: None,
        };
        assert!(actor.validate().is_err());
        assert!(Actor::human().validate().is_ok());
    }

    #[test]
    fn trace_and_idempotency_validation() {
        assert!(TraceId::new(&"a".repeat(32)).is_ok());
        assert!(TraceId::new("abc").is_err());
        assert!(IdempotencyKey::new("short").is_err());
        assert!(IdempotencyKey::new("long-enough-key").is_ok());
    }

    proptest! {
        #[test]
        fn slug_round_trip(prefix in "[a-z][a-z0-9-]{1,30}[a-z0-9]") {
            let slug = Slug::new(&prefix).expect("generated slug is valid");
            prop_assert_eq!(slug.to_string(), prefix);
        }

        #[test]
        fn money_round_trip(value in 0_i64..1_000_000_000) {
            let money = Money::from_microusd(value).expect("non-negative");
            prop_assert_eq!(money.microusd(), value);
        }

        #[test]
        fn card_key_round_trip(prefix in "[A-Z][A-Z0-9]{1,9}", seq in 1_i64..1_000_000) {
            let key = CardKey::new(&prefix, seq).expect("valid");
            prop_assert_eq!(CardKey::parse(&key.to_string()).expect("parse"), key);
        }
    }
}
