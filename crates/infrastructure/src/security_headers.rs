//! Security headers used by the nginx edge and, from WP-16, the API middleware.

/// Content Security Policy: same-origin scripts, no inline scripts, WebSocket allowed.
pub const CONTENT_SECURITY_POLICY: &str = "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self' ws: wss:; font-src 'self'; object-src 'none'; base-uri 'self'; frame-ancestors 'none'";

/// Static security headers.
pub const SECURITY_HEADERS: [(&str, &str); 5] = [
    ("Content-Security-Policy", CONTENT_SECURITY_POLICY),
    ("X-Content-Type-Options", "nosniff"),
    ("X-Frame-Options", "DENY"),
    ("Referrer-Policy", "strict-origin-when-cross-origin"),
    (
        "Strict-Transport-Security",
        "max-age=31536000; includeSubDomains",
    ),
];

/// Return the security header set.
#[must_use]
pub const fn security_headers() -> &'static [(&'static str, &'static str)] {
    &SECURITY_HEADERS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csp_blocks_inline_scripts() {
        assert!(CONTENT_SECURITY_POLICY.contains("script-src 'self'"));
        assert!(!CONTENT_SECURITY_POLICY.contains("unsafe-eval"));
    }

    #[test]
    fn headers_include_nosniff_and_frame_deny() {
        let headers = security_headers();
        assert!(
            headers
                .iter()
                .any(|(name, _)| *name == "X-Content-Type-Options")
        );
        assert!(
            headers
                .iter()
                .any(|(name, value)| *name == "X-Frame-Options" && *value == "DENY")
        );
    }
}
