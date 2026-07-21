//! Request-time access-token resolution for multi-provider credentials.
//!
//! Used by shell/sampler seams so OAuth tokens are never snapshotted into
//! static model `api_key` fields (review B1 / add-provider skill §F/G).
//!
//! Each successful resolve **returns** a [`SentCredentialStamp`]. Callers
//! (e.g. `MultiProviderBearerResolver`) hold the stamp for the request that
//! used the token and pass it to [`recover_unauthorized_with_stamp`] so a
//! concurrent peer cannot overwrite generation-aware 401 recovery (A1).
//!
//! [`TokenManager`] is process-shared per grok home so in-process single-flight
//! refresh works across concurrent resolves (A2).

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

/// Shared store + TokenManager per absolute grok home (single-flight refresh).
type SharedManager = (Arc<dyn CredentialStore>, Arc<TokenManager>);

static SHARED_MANAGERS: Lazy<DashMap<PathBuf, SharedManager>> = Lazy::new(DashMap::new);

fn make_store_and_manager(
    home: &Path,
) -> Result<(Arc<dyn CredentialStore>, Arc<TokenManager>), String> {
    let home_key = dunce::canonicalize(home).unwrap_or_else(|_| home.to_path_buf());
    // Atomic entry: avoid check-then-act race that could insert two managers
    // for the same home and split in-process single-flight (AUD-007).
    use dashmap::mapref::entry::Entry;
    match SHARED_MANAGERS.entry(home_key) {
        Entry::Occupied(o) => {
            let (store, token_manager) = o.get();
            Ok((store.clone(), token_manager.clone()))
        }
        Entry::Vacant(v) => {
            let store: Arc<dyn CredentialStore> =
                Arc::new(FileCredentialStore::new(home.to_path_buf()));
            let registry = Arc::new(registry::build_default_registry());
            let token_manager = Arc::new(TokenManager::with_issuer(
                store.clone(),
                registry,
                OPENAI_ISSUER.to_string(),
            ));
            v.insert((store.clone(), token_manager.clone()));
            Ok((store, token_manager))
        }
    }
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
    let fp =
        fingerprint::compute_fingerprint(provider, OPENAI_ISSUER, &credential.metadata.account);
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
    Ok((token.access_token.expose().to_string(), stamp))
}

/// Generation-aware 401 recovery for a multi-provider credential.
///
/// Prefer [`recover_unauthorized_with_stamp`] with the stamp from the same
/// request that received the 401. This entry point loads the current
/// generation from the store (best-effort) when no request-scoped stamp is
/// available.
///
/// Returns `true` when the caller should retry the request once with a fresh
/// bearer (either after refresh or after adopting a concurrent rotation).
pub async fn recover_unauthorized(
    home: &Path,
    provider: &ProviderId,
    credential_id: CredentialId,
) -> Result<bool, String> {
    recover_unauthorized_with_stamp(home, provider, credential_id, None).await
}

/// Like [`recover_unauthorized`], but uses a **request-scoped** stamp from
/// the resolve that produced the failed request (A1: no process-global
/// last-wins map).
pub async fn recover_unauthorized_with_stamp(
    home: &Path,
    provider: &ProviderId,
    credential_id: CredentialId,
    sent_stamp: Option<SentCredentialStamp>,
) -> Result<bool, String> {
    if crate::kill_switch::codex_auth_disabled() && provider.as_str() == "codex" {
        return Err("Codex auth is disabled (GROK_DISABLE_CODEX_AUTH)".into());
    }

    let (store, token_manager) = make_store_and_manager(home)?;
    let binding = credential_binding(&store, provider, credential_id).await?;

    let stamp = if let Some(s) = sent_stamp {
        s
    } else {
        // No request-scoped stamp — use current generation so recover still
        // refreshes once rather than no-opping.
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
        UnauthorizedRecovery::RetryAfterRefresh
        | UnauthorizedRecovery::RetryWithCurrentCredential => Ok(true),
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
    recover_unauthorized_with_stamp_blocking(home, provider, credential_id, None)
}

/// Blocking recovery with a request-scoped stamp (A1).
pub fn recover_unauthorized_with_stamp_blocking(
    home: &Path,
    provider: &ProviderId,
    credential_id: CredentialId,
    sent_stamp: Option<SentCredentialStamp>,
) -> Result<bool, String> {
    let home = home.to_path_buf();
    let provider = provider.clone();
    block_on_safe(async move {
        recover_unauthorized_with_stamp(&home, &provider, credential_id, sent_stamp).await
    })
}

/// Like [`resolve_access_token_blocking`], but returns the generation stamp.
pub fn resolve_access_token_with_stamp_blocking(
    home: &Path,
    provider: &ProviderId,
    credential_id: CredentialId,
) -> Result<(String, SentCredentialStamp), String> {
    let home = home.to_path_buf();
    let provider = provider.clone();
    block_on_safe(
        async move { resolve_access_token_with_stamp(&home, &provider, credential_id).await },
    )
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

/// Product home for multi-provider credential storage (must match
/// `xai_grok_config::paths`: `GROK_OSS_HOME` → `GROK_HOME` → `~/.grok-oss`).
pub fn grok_home() -> PathBuf {
    if let Some(v) = std::env::var_os("GROK_OSS_HOME") {
        return PathBuf::from(v);
    }
    if let Some(v) = std::env::var_os("GROK_HOME") {
        return PathBuf::from(v);
    }
    dirs::home_dir()
        .map(|h| h.join(".grok-oss"))
        .unwrap_or_else(|| PathBuf::from(".grok-oss"))
}

#[cfg(test)]
mod home_tests {
    use super::grok_home;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn multi_auth_home_prefers_grok_oss_home_then_default_segment() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        // SAFETY: single-threaded under ENV_LOCK; test-only env mutation.
        unsafe {
            let prev_oss = std::env::var_os("GROK_OSS_HOME");
            let prev_legacy = std::env::var_os("GROK_HOME");
            std::env::remove_var("GROK_OSS_HOME");
            std::env::remove_var("GROK_HOME");
            let def = grok_home();
            assert!(
                def.ends_with(".grok-oss"),
                "default multi-auth home must be ~/.grok-oss, got {}",
                def.display()
            );
            std::env::set_var("GROK_OSS_HOME", "/tmp/oss-home-test-xyz");
            assert_eq!(
                grok_home(),
                std::path::PathBuf::from("/tmp/oss-home-test-xyz")
            );
            std::env::remove_var("GROK_OSS_HOME");
            std::env::set_var("GROK_HOME", "/tmp/legacy-home-test-xyz");
            assert_eq!(
                grok_home(),
                std::path::PathBuf::from("/tmp/legacy-home-test-xyz")
            );
            match prev_oss {
                Some(v) => std::env::set_var("GROK_OSS_HOME", v),
                None => std::env::remove_var("GROK_OSS_HOME"),
            }
            match prev_legacy {
                Some(v) => std::env::set_var("GROK_HOME", v),
                None => std::env::remove_var("GROK_HOME"),
            }
        }
    }
}

/// Drop shared managers (tests that rebind home paths).
#[cfg(test)]
pub fn clear_shared_managers_for_test() {
    SHARED_MANAGERS.clear();
}
