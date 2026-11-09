//! Prometheus metrics and redacted structured request logs (WP-20).

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

const BUCKETS: [f64; 7] = [0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0];

#[derive(Debug, Default)]
struct Latency {
    bucket: [u64; BUCKETS.len()],
    sum_ms: u64,
    count: u64,
}

/// Process metrics for the edge server.
#[derive(Debug, Default)]
pub struct Metrics {
    in_flight: AtomicI64,
    requests: Mutex<BTreeMap<(String, u16), u64>>,
    errors: AtomicU64,
    latency: Mutex<Latency>,
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
    #[allow(clippy::cast_precision_loss)]
    pub fn finish(&self, method: &str, status: u16, elapsed_ms: u64) {
        self.in_flight.fetch_sub(1, Ordering::Relaxed);
        if let Ok(mut requests) = self.requests.lock() {
            *requests.entry((method.to_owned(), status)).or_insert(0) += 1;
        }
        if status >= 500 {
            self.errors.fetch_add(1, Ordering::Relaxed);
        }
        if let Ok(mut latency) = self.latency.lock() {
            let seconds = elapsed_ms as f64 / 1_000.0;
            let index = BUCKETS
                .iter()
                .position(|bucket| seconds <= *bucket)
                .unwrap_or(BUCKETS.len() - 1);
            latency.bucket[index] += 1;
            latency.sum_ms = latency.sum_ms.saturating_add(elapsed_ms);
            latency.count += 1;
        }
    }

    /// Render the Prometheus text exposition.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn render(&self) -> String {
        let mut output = String::new();
        output.push_str("# HELP ailab_http_requests_total Total HTTP requests\n");
        output.push_str("# TYPE ailab_http_requests_total counter\n");
        if let Ok(requests) = self.requests.lock() {
            for ((method, status), count) in requests.iter() {
                let _ = writeln!(
                    output,
                    "ailab_http_requests_total{{method=\"{method}\",status=\"{status}\"}} {count}"
                );
            }
        }
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
        output.push_str("# HELP ailab_http_request_duration_seconds Request latency\n");
        output.push_str("# TYPE ailab_http_request_duration_seconds histogram\n");
        if let Ok(latency) = self.latency.lock() {
            let mut cumulative = 0_u64;
            for (index, bucket) in BUCKETS.iter().enumerate() {
                cumulative = cumulative.saturating_add(latency.bucket[index]);
                let _ = writeln!(
                    output,
                    "ailab_http_request_duration_seconds_bucket{{le=\"{bucket}\"}} {cumulative}"
                );
            }
            let _ = writeln!(
                output,
                "ailab_http_request_duration_seconds_bucket{{le=\"+Inf\"}} {cumulative}"
            );
            let _ = writeln!(
                output,
                "ailab_http_request_duration_seconds_sum {}",
                latency.sum_ms as f64 / 1_000.0
            );
            let _ = writeln!(
                output,
                "ailab_http_request_duration_seconds_count {}",
                latency.count
            );
        }
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
    fn renders_counters_labels_and_seconds_buckets() {
        let metrics = Metrics::default();
        metrics.start();
        metrics.start();
        metrics.finish("GET", 200, 12);
        metrics.finish("GET", 500, 120);
        let text = metrics.render();
        assert!(text.contains("ailab_http_requests_total{method=\"GET\",status=\"200\"} 1"));
        assert!(text.contains("ailab_http_requests_total{method=\"GET\",status=\"500\"} 1"));
        assert!(text.contains("ailab_http_errors_total 1"));
        assert!(text.contains("ailab_http_request_duration_seconds_bucket{le=\"0.025\"} 1"));
        assert!(text.contains("ailab_http_request_duration_seconds_count 2"));
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
