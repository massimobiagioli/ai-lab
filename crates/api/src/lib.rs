//! AI-Lab API contracts: pagination, rate limits, origin/JSON guards and the
//! WebSocket envelope with resumable sequencing and event backfill.
#![forbid(unsafe_code)]

pub mod metrics;
pub mod server;
pub mod trace;

pub use ailab_application::{
    DEFAULT_LIMIT, EventEnvelope, MAX_LIMIT, Page, PageQuery, RunCursor, RunStore, RunView,
    StepView, StoreError, events_after,
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Rate-limit class.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RateLimitKind {
    /// Read.
    Read,
    /// Write.
    Write,
    /// Idea or SOW generation.
    Idea,
}

/// Rate-limit policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::struct_field_names)]
pub struct RateLimitPolicy {
    /// Reads per minute.
    pub read_rpm: u32,
    /// Writes per minute.
    pub write_rpm: u32,
    /// Idea generations per minute.
    pub idea_rpm: u32,
}

impl Default for RateLimitPolicy {
    fn default() -> Self {
        Self {
            read_rpm: 120,
            write_rpm: 30,
            idea_rpm: 5,
        }
    }
}

impl RateLimitPolicy {
    /// Limit for a class.
    #[must_use]
    pub const fn limit(self, kind: RateLimitKind) -> u32 {
        match kind {
            RateLimitKind::Read => self.read_rpm,
            RateLimitKind::Write => self.write_rpm,
            RateLimitKind::Idea => self.idea_rpm,
        }
    }

    /// True when `used` is still below the limit.
    #[must_use]
    pub const fn allowed(self, kind: RateLimitKind, used: u32) -> bool {
        used < self.limit(kind)
    }
}

/// Origin policy.
#[derive(Debug, Clone, Default)]
pub struct OriginPolicy {
    /// Exact allowed origins.
    pub allowed: Vec<String>,
}

impl OriginPolicy {
    /// Same-origin by default; exact-match allowlist when configured.
    #[must_use]
    pub fn is_allowed(&self, origin: Option<&str>) -> bool {
        origin.is_none_or(|origin| self.allowed.iter().any(|allowed| allowed == origin))
    }

    /// Writes require `application/json`.
    #[must_use]
    pub fn require_json(content_type: Option<&str>) -> bool {
        content_type.is_some_and(|value| value.to_ascii_lowercase().starts_with("application/json"))
    }
}

/// Client WebSocket message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsClientMessage {
    /// Subscribe to topics.
    Subscribe {
        /// Topics.
        topics: Vec<String>,
    },
    /// Unsubscribe from topics.
    Unsubscribe {
        /// Topics.
        topics: Vec<String>,
    },
    /// Heartbeat ping.
    Ping {
        /// Client timestamp.
        ts: i64,
    },
    /// Acknowledge receipt.
    Ack {
        /// Acknowledged sequence.
        seq: i64,
    },
}

/// Server WebSocket message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[allow(clippy::derive_partial_eq_without_eq)]
pub enum WsServerMessage {
    /// Initial handshake.
    Hello {
        /// Server time.
        server_time: DateTime<Utc>,
        /// Current sequence.
        seq: i64,
        /// Topics.
        topics: Vec<String>,
    },
    /// State event (flat envelope).
    Event {
        /// Event id.
        event_id: String,
        /// Event name.
        event: String,
        /// Per-lab sequence.
        seq: i64,
        /// Lab id.
        lab_id: String,
        /// Occurred at.
        occurred_at: DateTime<Utc>,
        /// Payload.
        payload: serde_json::Value,
    },
    /// Resync request after a lag.
    Resync {
        /// Last delivered sequence.
        after: i64,
    },
    /// Heartbeat pong.
    Pong {
        /// Echoed client timestamp.
        ts: i64,
    },
    /// Subscription acknowledged.
    Ack {
        /// Topics.
        topics: Vec<String>,
    },
    /// Protocol error.
    Error {
        /// Code.
        code: String,
        /// Detail.
        detail: String,
    },
    /// Health snapshot.
    Health {
        /// Components.
        components: serde_json::Value,
    },
}

impl WsServerMessage {
    /// Flatten an event envelope for the wire.
    #[must_use]
    pub fn event(envelope: &EventEnvelope) -> Self {
        Self::Event {
            event_id: envelope.event_id.to_string(),
            event: envelope.event.clone(),
            seq: envelope.seq,
            lab_id: envelope.lab_id.to_string(),
            occurred_at: envelope.occurred_at,
            payload: envelope.payload.clone(),
        }
    }
}

/// Build the `hello` handshake.
#[must_use]
#[allow(clippy::missing_const_for_fn)]
pub fn hello(server_time: DateTime<Utc>, seq: i64, topics: Vec<String>) -> WsServerMessage {
    WsServerMessage::Hello {
        server_time,
        seq,
        topics,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ailab_domain::{EventId, LabId};
    use serde_json::json;

    #[test]
    fn pagination_defaults_and_bounds() {
        assert_eq!(PageQuery::default().normalize().expect("default"), 50);
        assert!(
            PageQuery {
                limit: Some(0),
                cursor: None
            }
            .normalize()
            .is_err()
        );
        assert!(
            PageQuery {
                limit: Some(201),
                cursor: None
            }
            .normalize()
            .is_err()
        );
        assert_eq!(
            PageQuery {
                limit: Some(200),
                cursor: None
            }
            .normalize()
            .expect("max"),
            200
        );
    }

    #[test]
    fn events_after_filters_dedupes_and_sorts() {
        let lab = LabId::generate();
        let event = |seq: i64| EventEnvelope {
            event_id: EventId::generate(),
            event: "kanban.card.moved".to_owned(),
            seq,
            lab_id: lab,
            occurred_at: Utc::now(),
            payload: json!({}),
        };
        let first = event(1);
        let duplicate = first.clone();
        let third = event(3);
        let filtered = events_after(&[third, duplicate, first], 1);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].seq, 3);
    }

    #[test]
    fn ws_messages_round_trip() {
        let hello = hello(Utc::now(), 7, vec!["system".to_owned()]);
        let text = serde_json::to_string(&hello).expect("serialize");
        let parsed: WsServerMessage = serde_json::from_str(&text).expect("parse");
        assert_eq!(parsed, hello);
    }

    #[test]
    fn ws_protocol_messages_match_the_contract() {
        let subscribe = r#"{"type":"subscribe","topics":["lab:one"]}"#;
        let parsed: WsClientMessage = serde_json::from_str(subscribe).expect("subscribe");
        assert!(matches!(parsed, WsClientMessage::Subscribe { .. }));
        let event = WsServerMessage::event(&EventEnvelope {
            event_id: EventId::generate(),
            event: "kanban.card.moved".to_owned(),
            seq: 1,
            lab_id: LabId::generate(),
            occurred_at: Utc::now(),
            payload: json!({}),
        });
        let text = serde_json::to_string(&event).expect("serialize");
        assert!(text.contains("\"type\":\"event\""));
    }

    #[test]
    fn origin_and_json_guards() {
        let policy = OriginPolicy::default();
        assert!(policy.is_allowed(None));
        assert!(!policy.is_allowed(Some("https://evil.example")));
        assert!(OriginPolicy::require_json(Some("application/json")));
        assert!(!OriginPolicy::require_json(Some("text/plain")));
    }

    #[test]
    fn rate_limits_are_class_specific() {
        let policy = RateLimitPolicy::default();
        assert!(policy.allowed(RateLimitKind::Read, 119));
        assert!(!policy.allowed(RateLimitKind::Read, 120));
        assert!(!policy.allowed(RateLimitKind::Idea, 5));
    }
}
