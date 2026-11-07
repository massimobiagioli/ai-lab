//! Central secret redaction for logs, errors and telemetry.

const SECRET_PREFIXES: &[&str] = &["sk-", "ghp_", "github_pat_", "AKIA", "AIza", "xoxb-"];

/// Redact known secret shapes from a log line.
///
/// The redactor covers the provider, SCM, cloud and webhook token prefixes used
/// by v1. Prompt and memory content redaction is extended by WP-11.
#[must_use]
pub fn redact(input: &str) -> String {
    let mut output: Vec<&str> = Vec::new();
    let mut redact_next = false;
    for token in input.split_whitespace() {
        if redact_next {
            output.push("[REDACTED]");
            redact_next = false;
            continue;
        }
        if token == "Bearer" {
            output.push(token);
            redact_next = true;
            continue;
        }
        if SECRET_PREFIXES
            .iter()
            .any(|prefix| token.starts_with(prefix))
            || token.contains("AILAB_MASTER_KEY=")
            || token.contains("password=")
            || token.contains("api_key=")
        {
            output.push("[REDACTED]");
        } else {
            output.push(token);
        }
    }
    output.join(" ")
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
        assert_eq!(
            redact("Authorization: Bearer abc.def"),
            "Authorization: Bearer [REDACTED]"
        );
    }

    #[test]
    fn redacts_master_key_assignments() {
        assert_eq!(redact("AILAB_MASTER_KEY=abc"), "[REDACTED]");
    }

    #[test]
    fn keeps_normal_text() {
        assert_eq!(
            redact("card AIL-42 moved to READY"),
            "card AIL-42 moved to READY"
        );
    }
}
