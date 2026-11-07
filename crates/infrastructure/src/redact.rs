//! Central secret redaction for logs, errors and telemetry.
//!
//! The redactor removes Bearer tokens, provider/SCM/cloud token prefixes,
//! JWTs, PEM private-key blocks, sensitive header values and `key=value` /
//! `"key": "value"` pairs whose key looks secret.

use std::sync::LazyLock;

use regex::Regex;

static PEM: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?s)-----BEGIN [A-Z ]*PRIVATE KEY-----.*?-----END [A-Z ]*PRIVATE KEY-----")
        .expect("valid PEM regex")
});

static HEADER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b(authorization|cookie|set-cookie|x-api-key)\s*:\s*[^\r\n]+")
        .expect("valid header regex")
});

static BEARER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\bBearer\s+[A-Za-z0-9._~+/=-]+").expect("valid bearer regex")
});

static JWT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\beyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\b").expect("valid JWT regex")
});

static PREFIX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(?:sk-|ghp_|github_pat_|xoxb-|AKIA|AIza)[A-Za-z0-9_.\-]+")
        .expect("valid prefix regex")
});

static KEY_VALUE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?i)("?(?:api[_-]?key|password|passwd|secret|token|access[_-]?token|refresh[_-]?token|session[_-]?id|session|master[_-]?key|private[_-]?key|authorization)"?\s*[:=]\s*"?)([^"\s,;&}]+)"#,
    )
    .expect("valid key/value regex")
});

/// Redact known secret shapes from a log line.
#[must_use]
pub fn redact(input: &str) -> String {
    let value = PEM.replace_all(input, "[REDACTED]").into_owned();
    let value = HEADER
        .replace_all(&value, |caps: &regex::Captures<'_>| {
            format!("{}: [REDACTED]", &caps[1])
        })
        .into_owned();
    let value = BEARER.replace_all(&value, "Bearer [REDACTED]").into_owned();
    let value = JWT.replace_all(&value, "[REDACTED]").into_owned();
    let value = PREFIX.replace_all(&value, "[REDACTED]").into_owned();
    KEY_VALUE
        .replace_all(&value, |caps: &regex::Captures<'_>| {
            format!("{}[REDACTED]", &caps[1])
        })
        .into_owned()
}

#[cfg(test)]
mod tests {
    use super::redact;

    #[test]
    fn redacts_github_tokens() {
        assert_eq!(redact("token ghp_abc123"), "token [REDACTED]");
    }

    #[test]
    fn redacts_bearer_tokens() {
        assert_eq!(redact("Bearer abc.def"), "Bearer [REDACTED]");
    }

    #[test]
    fn redacts_master_key_assignments() {
        assert_eq!(
            redact("AILAB_MASTER_KEY=abc"),
            "AILAB_MASTER_KEY=[REDACTED]"
        );
    }

    #[test]
    fn redacts_secrets_inside_json() {
        assert_eq!(
            redact(r#"{"api_key": "ghp_abc123"}"#),
            r#"{"api_key": "[REDACTED]"}"#
        );
        assert_eq!(
            redact(r#"{"token":"sk-secretvalue"}"#),
            r#"{"token":"[REDACTED]"}"#
        );
    }

    #[test]
    fn redacts_secrets_inside_urls() {
        assert_eq!(
            redact("https://host/?key=sk-secretvalue"),
            "https://host/?key=[REDACTED]"
        );
    }

    #[test]
    fn redacts_sensitive_headers() {
        assert_eq!(
            redact("Authorization: Bearer abc.def"),
            "Authorization: [REDACTED]"
        );
        assert_eq!(redact("Cookie: session=abcd1234"), "Cookie: [REDACTED]");
        assert_eq!(redact("Set-Cookie: sid=abcd1234"), "Set-Cookie: [REDACTED]");
    }

    #[test]
    fn redacts_jwts_and_pem_blocks() {
        assert_eq!(
            redact("jwt eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxIn0.c2lnbmF0dXJl"),
            "jwt [REDACTED]"
        );
        let pem = "-----BEGIN RSA PRIVATE KEY-----\nMIIE\n-----END RSA PRIVATE KEY-----";
        assert_eq!(redact(pem), "[REDACTED]");
    }

    #[test]
    fn keeps_normal_text() {
        assert_eq!(
            redact("card AIL-42 moved to READY"),
            "card AIL-42 moved to READY"
        );
    }
}
