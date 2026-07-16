//! Account fingerprint computation (protocol-baseline.md §11).
//!
//! `AccountFingerprint::sha256(provider_id || issuer || chatgpt_user_id || chatgpt_account_id)`

use sha2::{Digest, Sha256};

use xai_grok_auth::{AccountFingerprint, ProviderAccountInfo, ProviderId};

/// Compute a stable 32-byte fingerprint of an account identity.
///
/// Per protocol-baseline.md §11:
/// ```text
/// AccountFingerprint::sha256(
///     provider_id
///     || issuer
///     || chatgpt_user_id
///     || chatgpt_account_id
/// )
/// ```
pub fn compute_fingerprint(
    provider_id: &ProviderId,
    issuer: &str,
    account: &ProviderAccountInfo,
) -> AccountFingerprint {
    let mut hasher = Sha256::new();
    hasher.update(provider_id.as_str().as_bytes());
    hasher.update(b"\0");
    hasher.update(issuer.as_bytes());
    hasher.update(b"\0");
    // chatgpt_user_id from metadata or subject
    let chatgpt_user_id = account
        .metadata
        .get("chatgpt_user_id")
        .or_else(|| account.subject.as_ref())
        .map(|s| s.as_str())
        .unwrap_or("");
    hasher.update(chatgpt_user_id.as_bytes());
    hasher.update(b"\0");
    // chatgpt_account_id from metadata or provider_account_id
    let chatgpt_account_id = account
        .metadata
        .get("chatgpt_account_id")
        .or_else(|| account.provider_account_id.as_ref())
        .map(|s| s.as_str())
        .unwrap_or("");
    hasher.update(chatgpt_account_id.as_bytes());

    let result = hasher.finalize();
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&result);
    AccountFingerprint::from(bytes)
}

/// Compute a fingerprint from raw string components (useful for tests).
pub fn fingerprint_from_parts(
    provider_id: &str,
    issuer: &str,
    chatgpt_user_id: &str,
    chatgpt_account_id: &str,
) -> AccountFingerprint {
    let mut hasher = Sha256::new();
    hasher.update(provider_id.as_bytes());
    hasher.update(b"\0");
    hasher.update(issuer.as_bytes());
    hasher.update(b"\0");
    hasher.update(chatgpt_user_id.as_bytes());
    hasher.update(b"\0");
    hasher.update(chatgpt_account_id.as_bytes());
    let result = hasher.finalize();
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&result);
    AccountFingerprint::from(bytes)
}
