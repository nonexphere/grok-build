//! Request-time access-token resolution for multi-provider credentials.
//!
//! Used by shell/sampler seams so OAuth tokens are never snapshotted into
//! static model `api_key` fields (review B1 / add-provider skill §F/G).
//!
//! Each successful resolve records a [`SentCredentialStamp`] so a later 401
//! can call [`recover_unauthorized`] with generation-aware one-retry semantics.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use dashmap::DashMap;
use once_cell::sync::Lazy;
use xai_grok_auth::{
    CredentialBinding, CredentialId, CredentialKey, CredentialStore, ProviderId,
    SentCredentialStamp, TokenUseReason, UnauthorizedRecovery,
};

use crate::fingerprint;
use crate::registry;
use crate::store::FileCredentialStore;
use crate::token_manager::TokenManager;

const OPENAI_ISSUER: &str = "https://auth.openai.com";

/// Last stamp observed when a bearer was resolved for inference (process-local).
static LAST_SENT_STAMPS: Lazy<DashMap<CredentialKey, SentCredentialStamp>> =
    Lazy::new(DashMap::new);

fn make_store_and_manager(
    home: &Path,
) -> Result<(Arc<dyn CredentialStore>, TokenManager), String> {
    let store: Arc<dyn CredentialStore> = Arc::new(FileCredentialStore::new(home.to_path_buf()));
    let registry = Arc::new(registry::build_default_registry());
    let token_manager =
        TokenManager::with_issuer(store.clone(), registry, OPENAI_ISSUER.to_string());
    Ok((store, token_manager))
}

async fn credential_binding(
    store: &Arc<dyn CredentialStore>,
    provider: &ProviderId,
    credential_id: CredentialId,
) -> Result<CredentialBinding, String> {
    let key = CredentialKey {
        provider: provider.clone(),
        credential_id,
    };
    let credential = store
        .load(&key)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("credential not found: {provider}/{credential_id}"))?;
    let fp = fingerprint::compute_fingerprint(provider, OPENAI_ISSUER, &credential.metadata.account);
    Ok(CredentialBinding {
        key,
        expected_account: fp,
    })
}

/// Resolve a fresh access token for `(provider, credential_id)` via the store
/// + [`TokenManager`] (refresh when near expiry). Records a generation stamp.
pub async fn resolve_access_token(
    home: &Path,
    provider: &ProviderId,
    credential_id: CredentialId,
) -> Result<String, String> {
    let (token, _stamp) = resolve_access_token_with_stamp(home, provider, credential_id).await?;
    Ok(token)
}

/// Like [`resolve_access_token`], but also returns the stamp sent with the token.
pub async fn resolve_access_token_with_stamp(
    home: &Path,
    provider: &ProviderId,
    credential_id: CredentialId,
) -> Result<(String, SentCredentialStamp), String> {
    if crate::kill_switch::codex_auth_disabled() && provider.as_str() == "codex" {
        return Err("Codex auth is disabled (GROK_DISABLE_CODEX_AUTH)".into());
    }

    let (store, token_manager) = make_store_and_manager(home)?;
    let binding = credential_binding(&store, provider, credential_id).await?;
    let token = token_manager
        .get_valid_token(&binding, TokenUseReason::Inference)
        .await
        .map_err(|e| e.to_string())?;
    let stamp = SentCredentialStamp {
        key: binding.key.clone(),
        generation: token.generation,
        account_fingerprint: token.account_fingerprint.clone(),
    };
    LAST_SENT_STAMPS.insert(binding.key, stamp.clone());
    Ok((token.access_token.expose().to_string(), stamp))
}

/// Generation-aware 401 recovery for a multi-provider credential.
///
/// Uses the last stamp from [`resolve_access_token_with_stamp`] when present;
/// otherwise loads the current generation (best-effort).
///
/// Returns `true` when the caller should retry the request once with a fresh
/// bearer (either after refresh or after adopting a concurrent rotation).
pub async fn recover_unauthorized(
    home: &Path,
    provider: &ProviderId,
    credential_id: CredentialId,
) -> Result<bool, String> {
    if crate::kill_switch::codex_auth_disabled() && provider.as_str() == "codex" {
        return Err("Codex auth is disabled (GROK_DISABLE_CODEX_AUTH)".into());
    }

    let (store, token_manager) = make_store_and_manager(home)?;
    let binding = credential_binding(&store, provider, credential_id).await?;

    let stamp = if let Some(s) = LAST_SENT_STAMPS.get(&binding.key) {
        s.clone()
    } else {
        // No prior resolve in this process — use current generation so
        // recover still refreshes once rather than no-opping.
        let cred = store
            .load(&binding.key)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "credential not found".to_string())?;
        SentCredentialStamp {
            key: binding.key.clone(),
            generation: cred.metadata.generation,
            account_fingerprint: binding.expected_account.clone(),
        }
    };

    let outcome = token_manager
        .recover_unauthorized(&binding, &stamp, 401)
        .await
        .map_err(|e| e.to_string())?;

    match outcome {
        UnauthorizedRecovery::RetryAfterRefresh | UnauthorizedRecovery::RetryWithCurrentCredential => {
            // Drop stamp so the next resolve records the post-recovery generation.
            LAST_SENT_STAMPS.remove(&binding.key);
            Ok(true)
        }
        UnauthorizedRecovery::ReauthenticationRequired
        | UnauthorizedRecovery::NotAuthenticationFailure => Ok(false),
    }
}

/// Blocking wrapper for sync shell seams (`BearerResolver`, catalog resolve).
///
/// Safe on Tokio **current-thread** / LocalSet session runtimes: never uses
/// `block_in_place` there (that panics with "can call blocking only when
/// running on the multi-threaded runtime").
pub fn resolve_access_token_blocking(
    home: &Path,
    provider: &ProviderId,
    credential_id: CredentialId,
) -> Result<String, String> {
    let home = home.to_path_buf();
    let provider = provider.clone();
    block_on_safe(async move { resolve_access_token(&home, &provider, credential_id).await })
}

/// Blocking 401 recovery for session sampler path (current-thread safe).
pub fn recover_unauthorized_blocking(
    home: &Path,
    provider: &ProviderId,
    credential_id: CredentialId,
) -> Result<bool, String> {
    let home = home.to_path_buf();
    let provider = provider.clone();
    block_on_safe(async move { recover_unauthorized(&home, &provider, credential_id).await })
}

/// Run an async future from sync code without panicking on current-thread
/// Tokio runtimes (session LocalSet workers).
///
/// - **No runtime:** new current-thread runtime on this thread.
/// - **Multi-thread runtime:** `block_in_place` + `block_on` on the handle.
/// - **Current-thread runtime:** dedicated OS thread with its own runtime
///   (never `block_in_place`).
pub fn block_on_safe<F, T>(fut: F) -> Result<T, String>
where
    F: std::future::Future<Output = Result<T, String>> + Send + 'static,
    T: Send + 'static,
{
    match tokio::runtime::Handle::try_current() {
        Ok(handle) => match handle.runtime_flavor() {
            tokio::runtime::RuntimeFlavor::MultiThread => {
                tokio::task::block_in_place(|| handle.block_on(fut))
            }
            // CurrentThread (and any future flavors): never block_in_place.
            _ => {
                let (tx, rx) = std::sync::mpsc::sync_channel(1);
                std::thread::Builder::new()
                    .name("goblin-mp-auth".into())
                    .spawn(move || {
                        let result = (|| {
                            let rt = tokio::runtime::Builder::new_current_thread()
                                .enable_all()
                                .build()
                                .map_err(|e| format!("tokio runtime: {e}"))?;
                            rt.block_on(fut)
                        })();
                        let _ = tx.send(result);
                    })
                    .map_err(|e| format!("spawn multi-provider worker: {e}"))?;
                rx.recv()
                    .map_err(|_| "multi-provider worker disconnected".to_string())?
            }
        },
        Err(_) => {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| format!("tokio runtime: {e}"))?;
            rt.block_on(fut)
        }
    }
}

/// Grok home directory used by multi-provider credential storage.
pub fn grok_home() -> PathBuf {
    std::env::var_os("GROK_HOME")
        .map(PathBuf::from)
        .or_else(|| dirs::home_dir().map(|h| h.join(".grok")))
        .unwrap_or_else(|| PathBuf::from(".grok"))
}

/// Clear process-local stamps (tests).
#[cfg(test)]
pub fn clear_last_sent_stamps_for_test() {
    LAST_SENT_STAMPS.clear();
}
