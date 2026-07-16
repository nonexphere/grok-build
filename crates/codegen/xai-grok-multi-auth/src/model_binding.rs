//! Model binding and credential binding helpers.

use xai_grok_auth::{
    AccountFingerprint, CredentialBinding, CredentialId, CredentialKey, ModelBinding, ProviderId,
};

/// Resolve a `CredentialKey` from a `ModelBinding` and a default credential.
///
/// If the binding has an explicit credential, use it; otherwise return the
/// provided default.
pub fn resolve_credential_key(
    binding: &ModelBinding,
    default: Option<&CredentialKey>,
) -> Option<CredentialKey> {
    binding
        .credential
        .map(|id| CredentialKey {
            provider: binding.provider.clone(),
            credential_id: id,
        })
        .or_else(|| default.cloned())
}

/// Create a `CredentialBinding` from a key and its expected fingerprint.
pub fn bind_credential(
    key: CredentialKey,
    expected_account: AccountFingerprint,
) -> CredentialBinding {
    CredentialBinding {
        key,
        expected_account,
    }
}

/// Create a `ModelBinding` for a provider-backed model.
pub fn model_binding(
    provider: ProviderId,
    credential: Option<CredentialId>,
    model: impl Into<String>,
) -> ModelBinding {
    ModelBinding::new(provider, credential, model)
}
