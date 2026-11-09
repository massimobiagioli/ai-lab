//! Prometheus metrics and redacted structured request logs (WP-20).

use std::fmt::Write as _;
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};

const BUCKETS: [u64; 7] = [10, 25, 50, 100, 250, 500, 1_000];

/// Process metrics for the edge server.
#[derive(Debug, Default)]
pub struct Metrics {
    requests: AtomicU64,
    errors: AtomicU64,
    in_flight: AtomicI64,
    latency: [AtomicU64; BUCKETS.len()],
}

impl Metrics {
    /// Shared registry.
    #[must_use]
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Mark a request as started.
    pub fn start(&self) {
        self.in_flight.fetch_add(1, Ordering::Relaxed);
    }

    /// Mark a request as finished and record its status/latency.
    pub fn finish(&self, status: u16, elapsed_ms: u64) {
        self.in_flight.fetch_sub(1, Ordering::Relaxed);
        self.requests.fetch_add(1, Ordering::Relaxed);
        if status >= 500 {
            self.errors.fetch_add(1, Ordering::Relaxed);
        }
        let index = BUCKETS
            .iter()
            .position(|bucket| elapsed_ms <= *bucket)
            .unwrap_or(BUCKETS.len() - 1);
        self.latency[index].fetch_add(1, Ordering::Relaxed);
    }

    /// Render the Prometheus text exposition.
    #[must_use]
    pub fn render(&self) -> String {
        let mut output = String::new();
        output.push_str("# HELP ailab_http_requests_total Total HTTP requests\n");
        output.push_str("# TYPE ailab_http_requests_total counter\n");
        let _ = writeln!(
            output,
            "ailab_http_requests_total {}",
            self.requests.load(Ordering::Relaxed)
        );
        output.push_str("# HELP ailab_http_errors_total Total 5xx responses\n");
        output.push_str("# TYPE ailab_http_errors_total counter\n");
        let _ = writeln!(
            output,
            "ailab_http_errors_total {}",
            self.errors.load(Ordering::Relaxed)
        );
        output.push_str("# HELP ailab_http_in_flight In-flight requests\n");
        output.push_str("# TYPE ailab_http_in_flight gauge\n");
        let _ = writeln!(
            output,
            "ailab_http_in_flight {}",
            self.in_flight.load(Ordering::Relaxed)
        );
        output.push_str("# HELP ailab_http_request_duration_ms Request latency\n");
        output.push_str("# TYPE ailab_http_request_duration_ms histogram\n");
        let mut cumulative = 0_u64;
        for (index, bucket) in BUCKETS.iter().enumerate() {
            cumulative = cumulative.saturating_add(self.latency[index].load(Ordering::Relaxed));
            let _ = writeln!(
                output,
                "ailab_http_request_duration_ms_bucket{{le=\"{bucket}\"}} {cumulative}"
            );
        }
        let _ = writeln!(
            output,
            "ailab_http_request_duration_ms_count {}",
            self.requests.load(Ordering::Relaxed)
        );
        output
    }
}

/// Redact bearer tokens, `token=` and `api_key=` values from a string.
#[must_use]
pub fn redact(value: &str) -> String {
    let mut result = value.to_owned();
    if let Some(index) = result.to_ascii_lowercase().find("bearer ") {
        let start = index + 7;
        let end = result[start..]
            .find(char::is_whitespace)
            .map_or(result.len(), |offset| start + offset);
        result.replace_range(start..end, "[REDACTED]");
    }
    result
        .split('&')
        .map(|segment| {
            let lower = segment.to_ascii_lowercase();
            for key in ["token=", "api_key=", "apikey="] {
                if let Some(index) = lower.find(key) {
                    let end = index + key.len();
                    return format!("{}[REDACTED]", &segment[..end]);
                }
            }
            segment.to_owned()
        })
        .collect::<Vec<_>>()
        .join("&")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_counters_and_buckets() {
        let metrics = Metrics::default();
        metrics.start();
        metrics.start();
        metrics.finish(200, 12);
        metrics.finish(500, 120);
        let text = metrics.render();
        assert!(text.contains("ailab_http_requests_total 2"));
        assert!(text.contains("ailab_http_errors_total 1"));
        assert!(text.contains("ailab_http_request_duration_ms_bucket{le=\"25\"} 1"));
        assert!(text.contains("ailab_http_in_flight 0"));
    }

    #[test]
    fn redacts_tokens() {
        assert_eq!(
            redact("Authorization: Bearer abc.def"),
            "Authorization: Bearer [REDACTED]"
        );
        assert_eq!(redact("/ws?token=secret&x=1"), "/ws?token=[REDACTED]&x=1");
        assert_eq!(redact("/events?api_key=k"), "/events?api_key=[REDACTED]");
    }
}
