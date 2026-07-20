//! Core control-plane identifiers and account model types: `ProviderId`,
//! `CredentialId`, `CredentialKey`, account kinds/plans, and account
//! fingerprints. Mirrors `task.md` Appendix A.

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// A validated provider identifier. Must be non-empty and contain only
/// ASCII lowercase letters, digits, or one of `.`, `_`, `-`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProviderId(String);

impl ProviderId {
    /// Construct a new `ProviderId`, validating the allowed character set.
    pub fn new(value: impl Into<String>) -> Result<Self, InvalidProviderId> {
        let value = value.into();
        let valid = !value.is_empty()
            && value
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '.' | '_' | '-'));

        if !valid {
            return Err(InvalidProviderId(value));
        }

        Ok(Self(value))
    }

    /// Construct a `ProviderId` without validating. Caller is responsible for
    /// ensuring the value satisfies the documented invariants.
    pub fn new_unchecked(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// The underlying identifier string.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consume into the underlying `String`.
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl fmt::Display for ProviderId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for ProviderId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Error returned when a `ProviderId` fails validation.
#[derive(Debug, Error)]
#[error("invalid provider id: {0}")]
pub struct InvalidProviderId(pub String);

/// A unique identifier for a stored credential. Backed by a UUID v7
/// (time-ordered) when available.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CredentialId(Uuid);

impl CredentialId {
    /// Generate a new `CredentialId` (UUID v7).
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    /// Wrap an existing `Uuid`.
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// The underlying `Uuid`.
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for CredentialId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for CredentialId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

/// The composite key used to address a credential: the owning provider plus
/// the credential id.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CredentialKey {
    pub provider: ProviderId,
    pub credential_id: CredentialId,
}

/// The kind of account a credential grants access to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountKind {
    Personal,
    Workspace,
    Service,
    Unknown,
}

/// A provider-specific account plan, captured with both the raw token and a
/// human-readable display name when known.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountPlan {
    Known {
        raw: String,
        display_name: String,
    },
    Unknown(String),
}

/// Identity and account metadata reported by a provider for a credential.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderAccountInfo {
    pub subject: Option<String>,
    pub provider_account_id: Option<String>,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub workspace_id: Option<String>,
    pub workspace_name: Option<String>,
    pub plan: Option<AccountPlan>,
    pub account_kind: AccountKind,
    pub fedramp: bool,
    pub metadata: BTreeMap<String, String>,
}

impl Default for ProviderAccountInfo {
    fn default() -> Self {
        Self {
            subject: None,
            provider_account_id: None,
            email: None,
            display_name: None,
            workspace_id: None,
            workspace_name: None,
            plan: None,
            account_kind: AccountKind::Unknown,
            fedramp: false,
            metadata: BTreeMap::new(),
        }
    }
}

/// Lifecycle status of a stored credential.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CredentialStatus {
    #[default]
    Ready,
    Expiring,
    Refreshing,
    ReauthRequired,
    Disabled,
    Corrupt,
}

/// Which secret backend persists a credential's secret material.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SecretBackendKind {
    Keyring,
    EncryptedFile,
    #[default]
    File,
    Ephemeral,
    Legacy,
}

/// A stable 32-byte fingerprint of an account identity, used to detect
/// account-identity changes across refreshes and reauthentications.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AccountFingerprint(pub [u8; 32]);

impl AccountFingerprint {
    /// The raw fingerprint bytes.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl From<[u8; 32]> for AccountFingerprint {
    fn from(value: [u8; 32]) -> Self {
        Self(value)
    }
}

/// Immutable request/session binding of provider + optional credential + model.
///
/// Authentication for an in-flight request is resolved through this binding
/// rather than a process-global "current provider" (see `task.md` §1 / §3.3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelBinding {
    pub provider: ProviderId,
    pub credential: Option<CredentialId>,
    pub model: String,
    pub endpoint_profile: Option<String>,
}

impl ModelBinding {
    /// Construct a binding for a provider-backed model, optionally pinned to a
    /// credential.
    pub fn new(
        provider: ProviderId,
        credential: Option<CredentialId>,
        model: impl Into<String>,
    ) -> Self {
        Self {
            provider,
            credential,
            model: model.into(),
            endpoint_profile: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_invalid_provider_id() {
        assert!(ProviderId::new("").is_err());
        assert!(ProviderId::new("XAI").is_err(), "uppercase rejected");
        assert!(ProviderId::new("codex!").is_err(), "punctuation rejected");
        assert!(ProviderId::new("has space").is_err(), "spaces rejected");
    }

    #[test]
    fn accepts_valid_provider_ids() {
        assert!(ProviderId::new("xai").is_ok());
        assert!(ProviderId::new("codex").is_ok());
        assert!(ProviderId::new("foo.bar-1").is_ok());
        assert!(ProviderId::new("a_b.c-2").is_ok());
    }

    #[test]
    fn invalid_provider_id_error_carries_value() {
        let err = ProviderId::new("Bad!").unwrap_err();
        assert_eq!(err.0, "Bad!");
    }
}
