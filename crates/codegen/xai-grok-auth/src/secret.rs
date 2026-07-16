//! A local `SecretString` newtype that redacts its contents in `Debug` and
//! `Display` output. Implemented in-tree to avoid pulling in the `secrecy`
//! crate (which is not a workspace dependency). The inner value is exposed
//! only through an explicit `ExposeSecret`-style accessor so that secret
//! leakage remains a deliberate act.

use std::fmt;

/// A wrapper around a `String` whose sensitive contents are never printed
/// via `Debug` or `Display`. Use [`SecretString::expose`] to access the
/// underlying value.
#[derive(Clone, Default)]
pub struct SecretString(String);

impl SecretString {
    /// Wrap an existing `String` as a secret.
    pub fn new(value: String) -> Self {
        Self(value)
    }

    /// Wrap an existing `&str` as a secret.
    pub fn from_str(value: &str) -> Self {
        Self(value.to_owned())
    }

    /// Access the inner secret value. Callers must take care not to log or
    /// serialize the returned reference.
    pub fn expose(&self) -> &str {
        &self.0
    }

    /// Consume the wrapper and return the inner `String`.
    pub fn into_inner(self) -> String {
        self.0
    }

    /// Whether the secret is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Debug for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SecretString(<redacted>)")
    }
}

impl fmt::Display for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<redacted>")
    }
}

impl From<String> for SecretString {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for SecretString {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

impl AsRef<str> for SecretString {
    fn as_ref(&self) -> &str {
        self.expose()
    }
}

impl PartialEq for SecretString {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for SecretString {}

impl std::hash::Hash for SecretString {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl PartialOrd for SecretString {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SecretString {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

// The `secret` module is only compiled under the `native-multi-provider-auth`
// feature, which enables the `serde` dependency, so these impls are always
// compilable when this module is in the build graph.

impl serde::Serialize for SecretString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // PERSISTENCE ONLY: Serialize emits the plaintext secret so secret
        // backends can round-trip. Never serialize SecretString into logs,
        // telemetry, JSON status, or TUI error surfaces — use Debug/Display
        // (always redacted) for diagnostics.
        serializer.serialize_str(&self.0)
    }
}

impl<'de> serde::Deserialize<'de> for SecretString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Ok(Self(value))
    }
}

#[cfg(test)]
mod tests {
    use super::SecretString;

    #[test]
    fn debug_redacts_value() {
        let secret = SecretString::from_str("super-secret-token");
        let debug = format!("{:?}", secret);
        assert!(!debug.contains("super-secret-token"));
        assert!(debug.contains("redacted"));
    }

    #[test]
    fn display_redacts_value() {
        let secret = SecretString::from_str("super-secret-token");
        let display = format!("{}", secret);
        assert!(!display.contains("super-secret-token"));
    }

    #[test]
    fn expose_returns_inner() {
        let secret = SecretString::from_str("super-secret-token");
        assert_eq!(secret.expose(), "super-secret-token");
    }
}
