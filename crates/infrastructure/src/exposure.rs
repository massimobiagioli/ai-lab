//! Exposure-mode guard and constant-time token comparison.

use subtle::ConstantTimeEq;

/// Why a bind configuration is refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExposureError {
    /// A non-loopback bind requires a bearer token.
    NonLoopbackWithoutToken,
    /// A remote token must be at least 32 bytes.
    TokenTooShort,
}

impl std::fmt::Display for ExposureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::NonLoopbackWithoutToken => {
                "refusing to bind to a non-loopback address without AILAB_API_TOKEN"
            }
            Self::TokenTooShort => "AILAB_API_TOKEN must be at least 32 bytes",
        };
        f.write_str(message)
    }
}

impl std::error::Error for ExposureError {}

/// True for the loopback bind addresses supported by v1.
#[must_use]
pub fn is_loopback(bind: &str) -> bool {
    matches!(bind, "127.0.0.1" | "::1" | "localhost")
}

/// Validate the bind/token combination.
pub fn validate_exposure(bind: &str, token: Option<&str>) -> Result<(), ExposureError> {
    if is_loopback(bind) {
        return Ok(());
    }
    match token {
        Some(value) if value.len() >= 32 => Ok(()),
        Some(_) => Err(ExposureError::TokenTooShort),
        None => Err(ExposureError::NonLoopbackWithoutToken),
    }
}

/// Compare two byte slices in constant time.
#[must_use]
pub fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    left.ct_eq(right).into()
}

/// Compare a provided bearer token with the expected value in constant time.
#[must_use]
pub fn token_matches(expected: &str, provided: Option<&str>) -> bool {
    provided.is_some_and(|value| constant_time_eq(expected.as_bytes(), value.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loopback_needs_no_token() {
        assert!(validate_exposure("127.0.0.1", None).is_ok());
        assert!(validate_exposure("::1", None).is_ok());
        assert!(validate_exposure("localhost", None).is_ok());
    }

    #[test]
    fn non_loopback_requires_a_token() {
        assert_eq!(
            validate_exposure("0.0.0.0", None),
            Err(ExposureError::NonLoopbackWithoutToken)
        );
        assert_eq!(
            validate_exposure("0.0.0.0", Some("short")),
            Err(ExposureError::TokenTooShort)
        );
        assert!(validate_exposure("0.0.0.0", Some(&"x".repeat(32))).is_ok());
    }

    #[test]
    fn constant_time_compare() {
        assert!(constant_time_eq(b"token", b"token"));
        assert!(!constant_time_eq(b"token", b"tokeN"));
        assert!(!constant_time_eq(b"token", b"token-longer"));
    }

    #[test]
    fn token_match_requires_a_value() {
        assert!(token_matches("secret-token", Some("secret-token")));
        assert!(!token_matches("secret-token", Some("other-token")));
        assert!(!token_matches("secret-token", None));
    }
}
