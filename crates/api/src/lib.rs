//! AI-Lab API contracts: pagination, rate limits, origin/JSON guards and the
//! WebSocket envelope with resumable sequencing and event backfill.
#![forbid(unsafe_code)]

pub mod postgres_store;
pub mod server;

use ailab_domain::{EventId, LabId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Default page size.
pub const DEFAULT_LIMIT: u32 = 50;
/// Maximum page size.
pub const MAX_LIMIT: u32 = 200;

/// Cursor pagination query.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct PageQuery {
    /// Requested limit.
    pub limit: Option<u32>,
    /// Opaque cursor.
    pub cursor: Option<String>,
}

impl PageQuery {
    /// Normalize the limit to `1..=200`, defaulting to 50.
    pub fn normalize(&self) -> Result<u32, String> {
        match self.limit {
            None => Ok(DEFAULT_LIMIT),
            Some(0) => Err("limit must be at least 1".to_owned()),
            Some(limit) if limit > MAX_LIMIT => Err(format!("limit must be at most {MAX_LIMIT}")),
            Some(limit) => Ok(limit),
        }
    }
}

/// A page of results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page<T> {
    /// Data.
    pub data: Vec<T>,
    /// Next cursor.
    pub next_cursor: Option<String>,
    /// Total estimate.
    pub total_estimate: u64,
}

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

/// Persisted event envelope.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
pub struct EventEnvelope {
    /// Event id.
    pub event_id: EventId,
    /// Event name.
    pub event: String,
    /// Per-lab sequence.
    pub seq: i64,
    /// Lab id.
    pub lab_id: LabId,
    /// Occurred at.
    pub occurred_at: DateTime<Utc>,
    /// Payload.
    pub payload: serde_json::Value,
}

/// Return events strictly after `since`, in order, with duplicate ids removed.
#[must_use]
pub fn events_after(events: &[EventEnvelope], since: i64) -> Vec<EventEnvelope> {
    let mut seen = std::collections::HashSet::new();
    let mut output: Vec<EventEnvelope> = events
        .iter()
        .filter(|event| event.seq > since && seen.insert(event.event_id))
        .cloned()
        .collect();
    output.sort_by_key(|event| event.seq);
    output
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
    fn rate_limits_are_class_specific() {
        let policy = RateLimitPolicy::default();
        assert!(policy.allowed(RateLimitKind::Read, 119));
        assert!(!policy.allowed(RateLimitKind::Read, 120));
        assert!(policy.allowed(RateLimitKind::Write, 29));
        assert!(!policy.allowed(RateLimitKind::Write, 30));
        assert!(policy.allowed(RateLimitKind::Idea, 4));
    }

    #[test]
    fn origin_and_json_guards() {
        let policy = OriginPolicy {
            allowed: vec!["https://app.test".to_owned()],
        };
        assert!(policy.is_allowed(None));
        assert!(policy.is_allowed(Some("https://app.test")));
        assert!(!policy.is_allowed(Some("https://evil.test")));
        assert!(OriginPolicy::require_json(Some(
            "application/json; charset=utf-8"
        )));
        assert!(!OriginPolicy::require_json(Some("text/plain")));
        assert!(!OriginPolicy::require_json(None));
    }

    fn event(seq: i64, id: u128) -> EventEnvelope {
        EventEnvelope {
            event_id: EventId::new(uuid_from(id)).expect("event id"),
            event: "kanban.card.moved".to_owned(),
            seq,
            lab_id: LabId::generate(),
            occurred_at: Utc::now(),
            payload: json!({"seq": seq}),
        }
    }

    fn uuid_from(value: u128) -> uuid::Uuid {
        uuid::Uuid::from_u128(value)
    }

    #[test]
    fn ws_protocol_messages_match_the_contract() {
        let ack = serde_json::to_value(WsServerMessage::Ack {
            topics: vec!["system".to_owned()],
        })
        .expect("ack");
        assert_eq!(ack["type"], "ack");
        let ping: WsClientMessage =
            serde_json::from_str(r#"{"type":"ping","ts":123}"#).expect("ping");
        assert_eq!(ping, WsClientMessage::Ping { ts: 123 });
        let pong_value = serde_json::to_value(WsServerMessage::Pong { ts: 123 }).expect("pong");
        assert_eq!(pong_value["ts"], 123);
        let resync = serde_json::to_value(WsServerMessage::Resync { after: 5 }).expect("resync");
        assert_eq!(resync["after"], 5);
        let error = serde_json::to_value(WsServerMessage::Error {
            code: "BAD_MESSAGE".to_owned(),
            detail: "invalid".to_owned(),
        })
        .expect("error");
        assert_eq!(error["detail"], "invalid");
    }

    #[test]
    fn events_after_filters_dedupes_and_sorts() {
        let events = vec![event(3, 1), event(1, 2), event(3, 1), event(4, 3)];
        let filtered = events_after(&events, 1);
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].seq, 3);
        assert_eq!(filtered[1].seq, 4);
    }

    #[test]
    fn ws_messages_round_trip() {
        let subscribe = WsClientMessage::Subscribe {
            topics: vec!["lab:1".to_owned()],
        };
        let json = serde_json::to_string(&subscribe).expect("serialize");
        assert!(json.contains("subscribe"));
        let parsed: WsClientMessage = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed, subscribe);
        let hello = hello(Utc::now(), 10, vec!["system".to_owned()]);
        assert!(
            serde_json::to_string(&hello)
                .expect("hello")
                .contains("hello")
        );
    }
}

#[cfg(test)]
mod contract_tests {
    use serde_json::Value;

    fn read(path: &str) -> Value {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let body = std::fs::read_to_string(root.join(path)).expect("contract file");
        serde_json::from_str(&body).expect("valid JSON")
    }

    #[test]
    fn openapi_and_asyncapi_contracts_are_valid() {
        let openapi = read(".docs/product/api/openapi.json");
        assert_eq!(openapi["openapi"], "3.1.0");
        assert!(openapi["paths"]["/cards/{id}/transition"]["post"].is_object());
        assert!(openapi["paths"]["/runs/{id}"]["get"].is_object());
        assert!(openapi["paths"]["/runs/{id}/cancel"]["post"].is_object());
        assert!(openapi["components"]["schemas"]["Problem"].is_object());

        let asyncapi = read(".docs/product/api/asyncapi.json");
        assert_eq!(asyncapi["asyncapi"], "3.0.0");
        assert!(asyncapi["channels"]["kanban.card.moved"].is_object());
        assert!(asyncapi["components"]["schemas"]["EventEnvelope"].is_object());
    }
}
