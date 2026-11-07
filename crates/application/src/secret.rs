//! A wrapper that never prints or serializes its value.

use serde::{Serialize, Serializer};

/// A secret value that is redacted in `Debug`, `Display` and `Serialize`.
#[derive(Clone, PartialEq, Eq)]
pub struct Secret<T>(T);

impl<T> Secret<T> {
    /// Wrap a secret.
    pub const fn new(value: T) -> Self {
        Self(value)
    }

    /// Explicitly expose the value to an adapter.
    pub const fn expose(&self) -> &T {
        &self.0
    }
}

impl<T> std::fmt::Debug for Secret<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Secret([REDACTED])")
    }
}

impl<T> std::fmt::Display for Secret<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[REDACTED]")
    }
}

impl<T> Serialize for Secret<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str("[REDACTED]")
    }
}

impl<T> From<T> for Secret<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_and_display_do_not_leak() {
        let secret = Secret::new("super-secret");
        assert!(!format!("{secret:?}").contains("super-secret"));
        assert!(!format!("{secret}").contains("super-secret"));
        assert_eq!(secret.expose(), &"super-secret");
    }

    #[test]
    fn serialize_does_not_leak() {
        let secret = Secret::new("super-secret");
        let json = serde_json::to_string(&secret).expect("serialize");
        assert!(!json.contains("super-secret"));
        assert!(json.contains("[REDACTED]"));
    }
}
