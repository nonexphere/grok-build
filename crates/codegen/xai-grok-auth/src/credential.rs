//! Credential record types and the async `CredentialStore` trait. Mirrors
//! `task.md` Appendix A.

use std::collections::BTreeMap;
use std::fmt;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::secret::SecretString;
use crate::types::{
    CredentialKey, CredentialStatus, ProviderAccountInfo, ProviderId, SecretBackendKind,
};

/// Persisted metadata describing a credential, separate from its secret
/// material. The `generation` counter supports optimistic compare-and-swap
/// updates against the store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialMetadata {
    pub schema_version: u32,
    pub key: CredentialKey,
    pub alias: String,
    pub account: ProviderAccountInfo,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub status: CredentialStatus,
    pub generation: u64,
    pub secret_backend: SecretBackendKind,
}

/// Secret material for a credential. The `Debug` impl redacts all token
/// fields so logs and error reports never leak secrets.
///
/// `Serialize`/`Deserialize` emit/consume plaintext (via [`SecretString`]'s
/// own serde impls) so secret backends can round-trip secret material to
/// disk. Never serialize into logs, telemetry, or status surfaces — use
/// `Debug` (always redacted) for diagnostics.
#[derive(Clone, Serialize, Deserialize)]
pub struct CredentialSecret {
    pub access_token: SecretString,
    pub refresh_token: Option<SecretString>,
    pub id_token: Option<SecretString>,
    pub fields: BTreeMap<String, SecretString>,
}

impl fmt::Debug for CredentialSecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CredentialSecret")
            .field("access_token", &"<redacted>")
            .field(
                "refresh_token",
                &self.refresh_token.as_ref().map(|_| "<redacted>"),
            )
            .field("id_token", &self.id_token.as_ref().map(|_| "<redacted>"))
            .field("field_names", &self.fields.keys().collect::<Vec<_>>())
            .finish()
    }
}

/// A credential together with its metadata, as loaded from the store.
#[derive(Debug, Clone)]
pub struct StoredCredential {
    pub metadata: CredentialMetadata,
    pub secret: CredentialSecret,
}

/// A request to create a new credential. Consumed by `CredentialStore::create`.
#[derive(Debug)]
pub struct NewCredentialRecord {
    pub provider: ProviderId,
    pub requested_alias: Option<String>,
    pub account: ProviderAccountInfo,
    pub secret: CredentialSecret,
    pub expires_at: Option<DateTime<Utc>>,
    pub backend: SecretBackendKind,
}

/// A partial update to an existing credential applied via compare-and-swap.
/// `Option<Option<_>>` fields distinguish "leave unchanged" from "clear".
#[derive(Debug)]
pub struct CredentialUpdate {
    pub key: CredentialKey,
    pub account: Option<ProviderAccountInfo>,
    pub secret: Option<CredentialSecret>,
    pub expires_at: Option<Option<DateTime<Utc>>>,
    pub status: Option<CredentialStatus>,
    pub updated_at: DateTime<Utc>,
}

/// Why a credential lock is being acquired.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CredentialLockPurpose {
    Refresh,
    Replace,
    Logout,
    Migrate,
}

/// A guard returned by `CredentialStore::acquire_lock`. Dropping the guard
/// releases the lock. Implementations store backend-specific state inside.
pub trait CredentialLockGuard: fmt::Debug + Send + Sync {}

/// Errors raised by a `CredentialStore` backend.
#[derive(Debug, Error)]
pub enum StoreError {
    #[error("credential store is unavailable: {0}")]
    Unavailable(String),

    #[error("credential data is corrupt: {0}")]
    Corrupt(String),

    #[error("credential backend rejected the operation: {0}")]
    Backend(String),

    #[error("credential lock timed out")]
    LockTimeout,

    #[error("credential was not found")]
    NotFound,
}

/// Errors raised by `CredentialStore::compare_and_swap`.
#[derive(Debug, Error)]
pub enum CompareAndSwapError {
    #[error("credential generation changed")]
    GenerationChanged,

    #[error(transparent)]
    Store(#[from] StoreError),
}

/// Async credential store abstraction. Backends (keyring, encrypted file,
/// ...) implement this; the control plane holds an `Arc<dyn
/// CredentialStore>`.
#[async_trait]
pub trait CredentialStore: fmt::Debug + Send + Sync {
    async fn list_providers(&self) -> Result<Vec<ProviderId>, StoreError>;

    async fn list_accounts(
        &self,
        provider: &ProviderId,
    ) -> Result<Vec<CredentialMetadata>, StoreError>;

    async fn resolve_alias(
        &self,
        provider: &ProviderId,
        alias: &str,
    ) -> Result<Option<CredentialKey>, StoreError>;

    async fn default_account(
        &self,
        provider: &ProviderId,
    ) -> Result<Option<CredentialKey>, StoreError>;

    async fn set_default_account(&self, key: &CredentialKey) -> Result<(), StoreError>;

    async fn load_metadata(
        &self,
        key: &CredentialKey,
    ) -> Result<Option<CredentialMetadata>, StoreError>;

    async fn load_secret(
        &self,
        key: &CredentialKey,
    ) -> Result<Option<CredentialSecret>, StoreError>;

    /// Load a full credential. Default implementation combines
    /// `load_metadata` and `load_secret`, returning `Corrupt` if metadata
    /// exists without a secret.
    async fn load(&self, key: &CredentialKey) -> Result<Option<StoredCredential>, StoreError> {
        let Some(metadata) = self.load_metadata(key).await? else {
            return Ok(None);
        };

        let Some(secret) = self.load_secret(key).await? else {
            return Err(StoreError::Corrupt(format!(
                "metadata exists without secret for {:?}",
                key
            )));
        };

        Ok(Some(StoredCredential { metadata, secret }))
    }

    async fn create(&self, record: NewCredentialRecord) -> Result<CredentialMetadata, StoreError>;

    async fn compare_and_swap(
        &self,
        expected_generation: u64,
        update: CredentialUpdate,
    ) -> Result<CredentialMetadata, CompareAndSwapError>;

    async fn delete(&self, key: &CredentialKey) -> Result<bool, StoreError>;

    async fn acquire_lock(
        &self,
        key: &CredentialKey,
        purpose: CredentialLockPurpose,
    ) -> Result<Box<dyn CredentialLockGuard>, StoreError>;
}

#[cfg(test)]
mod tests {
    use super::CredentialSecret;
    use crate::secret::SecretString;
    use std::collections::BTreeMap;

    #[test]
    fn credential_secret_debug_redacts_tokens() {
        let mut fields = BTreeMap::new();
        fields.insert("api_key".to_string(), SecretString::from_str("shh"));
        let secret = CredentialSecret {
            access_token: SecretString::from_str("access-secret-value"),
            refresh_token: Some(SecretString::from_str("refresh-secret-value")),
            id_token: Some(SecretString::from_str("id-secret-value")),
            fields,
        };
        let debug = format!("{:?}", secret);
        assert!(!debug.contains("access-secret-value"));
        assert!(!debug.contains("refresh-secret-value"));
        assert!(!debug.contains("id-secret-value"));
        assert!(!debug.contains("shh"));
        assert!(debug.contains("redacted"));
        // field names are visible (non-secret metadata)
        assert!(debug.contains("api_key"));
    }
}
