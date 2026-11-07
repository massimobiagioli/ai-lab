//! Security headers used by the nginx edge and, from WP-16, the API middleware.

/// Content Security Policy: same-origin scripts and styles, no inline scripts,
/// no eval, same-origin WebSocket, no framing and same-origin form posts.
pub const CONTENT_SECURITY_POLICY: &str = "default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self' data:; connect-src 'self'; font-src 'self'; object-src 'none'; base-uri 'self'; form-action 'self'; frame-ancestors 'none'";

/// Static security headers for API and HTML responses.
pub const SECURITY_HEADERS: [(&str, &str); 9] = [
    ("Content-Security-Policy", CONTENT_SECURITY_POLICY),
    ("X-Content-Type-Options", "nosniff"),
    ("X-Frame-Options", "DENY"),
    ("Referrer-Policy", "no-referrer"),
    (
        "Strict-Transport-Security",
        "max-age=31536000; includeSubDomains",
    ),
    (
        "Permissions-Policy",
        "camera=(), microphone=(), geolocation=(), payment=()",
    ),
    ("Cross-Origin-Opener-Policy", "same-origin"),
    ("Cross-Origin-Resource-Policy", "same-origin"),
    ("Cache-Control", "no-store"),
];

/// Return the security header set.
#[must_use]
pub const fn security_headers() -> &'static [(&'static str, &'static str)] {
    &SECURITY_HEADERS
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header(name: &str) -> Option<&'static str> {
        security_headers()
            .iter()
            .find(|(key, _)| *key == name)
            .map(|(_, value)| *value)
    }

    #[test]
    fn csp_blocks_inline_scripts_and_remote_frames() {
        assert!(CONTENT_SECURITY_POLICY.contains("script-src 'self'"));
        assert!(CONTENT_SECURITY_POLICY.contains("style-src 'self'"));
        assert!(CONTENT_SECURITY_POLICY.contains("connect-src 'self'"));
        assert!(CONTENT_SECURITY_POLICY.contains("form-action 'self'"));
        assert!(CONTENT_SECURITY_POLICY.contains("frame-ancestors 'none'"));
        assert!(!CONTENT_SECURITY_POLICY.contains("unsafe-inline"));
        assert!(!CONTENT_SECURITY_POLICY.contains("unsafe-eval"));
        assert!(!CONTENT_SECURITY_POLICY.contains("ws:"));
    }

    #[test]
    fn headers_include_hardening() {
        assert_eq!(header("X-Content-Type-Options"), Some("nosniff"));
        assert_eq!(header("X-Frame-Options"), Some("DENY"));
        assert_eq!(header("Referrer-Policy"), Some("no-referrer"));
        assert_eq!(
            header("Permissions-Policy"),
            Some("camera=(), microphone=(), geolocation=(), payment=()")
        );
        assert_eq!(header("Cross-Origin-Opener-Policy"), Some("same-origin"));
        assert_eq!(header("Cross-Origin-Resource-Policy"), Some("same-origin"));
        assert_eq!(header("Cache-Control"), Some("no-store"));
    }
}
