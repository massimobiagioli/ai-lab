//! Request trace propagation (WP-20).

use axum::extract::Request;
use axum::http::HeaderMap;
use axum::middleware::Next;
use axum::response::Response;

tokio::task_local! {
    static TRACE_ID: String;
}

/// The current request trace id, if any.
#[must_use]
pub fn current() -> Option<String> {
    TRACE_ID.try_with(std::clone::Clone::clone).ok()
}

/// Extract the trace id from a W3C `traceparent` header, else `x-trace-id`/`x-request-id`,
/// else generate a new one.
#[must_use]
pub fn trace_id(headers: &HeaderMap) -> String {
    if let Some(parent) = headers
        .get("traceparent")
        .and_then(|value| value.to_str().ok())
        && let Some(id) = parse_traceparent(parent)
    {
        return id;
    }
    for header in ["x-trace-id", "x-request-id"] {
        if let Some(value) = headers.get(header).and_then(|value| value.to_str().ok())
            && !value.trim().is_empty()
        {
            return value.trim().to_owned();
        }
    }
    format!("{:032x}", uuid::Uuid::now_v7().as_u128())
}

/// Parse `00-<32 hex trace id>-<16 hex span id>-<flags>`.
#[must_use]
pub fn parse_traceparent(value: &str) -> Option<String> {
    let mut parts = value.split('-');
    let version = parts.next()?;
    let trace = parts.next()?;
    let span = parts.next()?;
    let flags = parts.next()?;
    if version.len() != 2 || trace.len() != 32 || span.len() != 16 || flags.len() != 2 {
        return None;
    }
    if !trace.chars().all(|character| character.is_ascii_hexdigit())
        || !span.chars().all(|character| character.is_ascii_hexdigit())
        || trace.chars().all(|character| character == '0')
    {
        return None;
    }
    Some(trace.to_ascii_lowercase())
}

/// Middleware that scopes every request to a trace id and echoes it.
pub async fn trace_layer(request: Request, next: Next) -> Response {
    let id = trace_id(request.headers());
    let mut response = TRACE_ID.scope(id.clone(), next.run(request)).await;
    if let Ok(value) = axum::http::HeaderValue::from_str(&id) {
        response.headers_mut().insert("x-trace-id", value);
    }
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_w3c_traceparent() {
        let value = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";
        assert_eq!(
            parse_traceparent(value).as_deref(),
            Some("4bf92f3577b34da6a3ce929d0e0e4736")
        );
        assert!(
            parse_traceparent("00-00000000000000000000000000000000-00f067aa0ba902b7-01").is_none()
        );
        assert!(parse_traceparent("bad").is_none());
    }

    #[test]
    fn falls_back_to_headers() {
        let mut headers = HeaderMap::new();
        headers.insert("x-request-id", "req-1".parse().expect("header"));
        assert_eq!(trace_id(&headers), "req-1");
    }
}
