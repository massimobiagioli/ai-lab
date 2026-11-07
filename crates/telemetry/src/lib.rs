//! AI-Lab telemetry metric names.
//!
//! The names are the stable contract used by the API, worker, runner and the
//! Prometheus alerts. The OpenTelemetry exporter is wired by WP-20.
#![forbid(unsafe_code)]

/// HTTP request duration histogram.
pub const HTTP_REQUEST_DURATION_SECONDS: &str = "ailab_http_request_duration_seconds";
/// HTTP request counter.
pub const HTTP_REQUESTS_TOTAL: &str = "ailab_http_requests_total";
/// Database pool connections.
pub const DB_POOL_CONNECTIONS: &str = "ailab_db_pool_connections";
/// Redis latency.
pub const REDIS_LATENCY_SECONDS: &str = "ailab_redis_latency_seconds";
/// Run queue depth.
pub const QUEUE_DEPTH: &str = "ailab_run_queue_depth";
/// Lease age.
pub const LEASE_AGE_SECONDS: &str = "ailab_lease_age_seconds";
/// Run duration.
pub const RUN_DURATION_SECONDS: &str = "ailab_run_duration_seconds";
/// LLM tokens.
pub const LLM_TOKENS_TOTAL: &str = "ailab_llm_tokens_total";
/// LLM cost in micro-USD.
pub const LLM_COST_MICROUSD: &str = "ailab_llm_cost_microusd";
/// Budget remaining in micro-USD.
pub const BUDGET_REMAINING_MICROUSD: &str = "ailab_budget_remaining_microusd";
/// WebSocket connections.
pub const WS_CONNECTIONS: &str = "ailab_ws_connections";
/// Event lag.
pub const EVENT_LAG_SECONDS: &str = "ailab_event_lag_seconds";
/// Office FPS.
pub const OFFICE_FPS: &str = "ailab_office_fps";

/// All metric names.
#[must_use]
pub const fn metric_names() -> &'static [&'static str] {
    &[
        HTTP_REQUEST_DURATION_SECONDS,
        HTTP_REQUESTS_TOTAL,
        DB_POOL_CONNECTIONS,
        REDIS_LATENCY_SECONDS,
        QUEUE_DEPTH,
        LEASE_AGE_SECONDS,
        RUN_DURATION_SECONDS,
        LLM_TOKENS_TOTAL,
        LLM_COST_MICROUSD,
        BUDGET_REMAINING_MICROUSD,
        WS_CONNECTIONS,
        EVENT_LAG_SECONDS,
        OFFICE_FPS,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metric_names_are_prefixed_and_unique() {
        let names = metric_names();
        assert_eq!(names.len(), 13);
        let unique: std::collections::BTreeSet<&str> = names.iter().copied().collect();
        assert_eq!(unique.len(), names.len());
        assert!(names.iter().all(|name| name.starts_with("ailab_")));
    }
}
