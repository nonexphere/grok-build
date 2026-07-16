//! Request-time multi-provider token resolution for shell sampling seams.
//!
//! Catalog entries use credential-scoped keys (`codex/{credential_id}/{slug}`)
//! with **no** OAuth token in `ModelEntry.api_key`. This module:
//! - parses the binding from the catalog identity;
//! - resolves a fresh access token via `xai_grok_multi_auth::token_resolve`;
//! - exposes a [`xai_grok_sampler::BearerResolver`] for per-request refresh.

use xai_grok_auth::{CredentialId, ProviderId};
use xai_grok_multi_auth::provider_model_key::{
    parse_provider_model_key, ProviderModelKey,
};
use xai_grok_multi_auth::token_resolve;
use xai_grok_sampler::{BearerResolver, SharedBearerResolver};

use crate::agent::config::ModelEntry;

/// Extract a multi-provider binding from a model catalog entry.
///
/// Prefers `info.id` (full catalog key); falls back to scanning is unnecessary
/// because merge always sets `info.id` to the credential-scoped key.
pub fn binding_from_model_entry(model: &ModelEntry) -> Option<ProviderModelKey> {
    let key = model
        .info
        .id
        .as_deref()
        .filter(|s| !s.is_empty())
        .or_else(|| {
            // Some call sites pass entries where only `info.model` was set to
            // the full key (defensive).
            let m = model.info.model.as_str();
            if m.contains('/') { Some(m) } else { None }
        })?;
    parse_provider_model_key(key)
}

/// Resolve a live access token for a multi-provider model entry.
pub fn resolve_token_for_model_entry(model: &ModelEntry) -> Option<String> {
    let binding = binding_from_model_entry(model)?;
    let home = token_resolve::grok_home();
    match token_resolve::resolve_access_token_blocking(
        &home,
        &binding.provider,
        binding.credential_id,
    ) {
        Ok(token) => Some(token),
        Err(e) => {
            tracing::warn!(
                provider = %binding.provider,
                credential_id = %binding.credential_id,
                model = %binding.model,
                error = %e,
                "multi-provider token resolve failed"
            );
            None
        }
    }
}

/// Build a per-request bearer resolver for a multi-provider model, if bound.
pub fn bearer_resolver_for_model_entry(model: &ModelEntry) -> Option<SharedBearerResolver> {
    let binding = binding_from_model_entry(model)?;
    Some(std::sync::Arc::new(MultiProviderBearerResolver {
        provider: binding.provider,
        credential_id: binding.credential_id,
        home: token_resolve::grok_home(),
    }) as SharedBearerResolver)
}

/// Also build from a raw catalog model id string (session reconstruct path).
pub fn bearer_resolver_for_model_id(model_id: &str) -> Option<SharedBearerResolver> {
    let binding = parse_provider_model_key(model_id)?;
    Some(std::sync::Arc::new(MultiProviderBearerResolver {
        provider: binding.provider,
        credential_id: binding.credential_id,
        home: token_resolve::grok_home(),
    }) as SharedBearerResolver)
}

/// Session reconstruct path: chat_state keeps the **wire** model slug, not the
/// credential-scoped catalog key. Recover the credential via the
/// `ChatGPT-Account-ID` header that merge injects into multi-provider entries.
pub fn bearer_resolver_for_codex_account_header(
    account_id: &str,
) -> Option<SharedBearerResolver> {
    if account_id.is_empty() {
        return None;
    }
    let home = token_resolve::grok_home();
    let provider = ProviderId::new_unchecked("codex");
    let credential_id = find_codex_credential_for_account(&home, &provider, account_id)?;
    Some(std::sync::Arc::new(MultiProviderBearerResolver {
        provider,
        credential_id,
        home,
    }) as SharedBearerResolver)
}

/// Prefer catalog-key resolver; else credential-id header; else account lookup.
pub fn bearer_resolver_for_sampling_hints(
    model_id_or_slug: &str,
    base_url: &str,
    extra_headers: &indexmap::IndexMap<String, String>,
) -> Option<SharedBearerResolver> {
    if let Some(r) = bearer_resolver_for_model_id(model_id_or_slug) {
        return Some(r);
    }
    if let Some((provider, credential_id)) =
        credential_from_sampling_hints(model_id_or_slug, base_url, extra_headers)
    {
        return Some(std::sync::Arc::new(MultiProviderBearerResolver {
            provider,
            credential_id,
            home: token_resolve::grok_home(),
        }) as SharedBearerResolver);
    }
    None
}

/// Credential binding for a sampling config hint (catalog key, credential header,
/// or Codex account header fallback).
pub fn credential_from_sampling_hints(
    model_id_or_slug: &str,
    base_url: &str,
    extra_headers: &indexmap::IndexMap<String, String>,
) -> Option<(ProviderId, CredentialId)> {
    if let Some(b) = parse_provider_model_key(model_id_or_slug) {
        return Some((b.provider, b.credential_id));
    }
    // Prefer merge-injected credential id (no store I/O).
    if let Some(cred_raw) = extra_headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("x-goblin-credential-id"))
        .map(|(_, v)| v.as_str())
    {
        if let Ok(uuid) = uuid::Uuid::parse_str(cred_raw) {
            return Some((
                ProviderId::new_unchecked("codex"),
                CredentialId::from_uuid(uuid),
            ));
        }
    }
    let is_codex = base_url.contains("chatgpt.com") || base_url.contains("/codex");
    if !is_codex {
        return None;
    }
    let account_id = extra_headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("ChatGPT-Account-ID"))
        .map(|(_, v)| v.as_str())?;
    let home = token_resolve::grok_home();
    let provider = ProviderId::new_unchecked("codex");
    let credential_id = find_codex_credential_for_account(&home, &provider, account_id)?;
    Some((provider, credential_id))
}

/// Generation-aware multi-provider 401 recovery for the sampler turn path.
///
/// Returns `true` when the caller should rebuild the sampling client and
/// resubmit exactly once (TokenManager refresh or adopt concurrent rotation).
pub fn try_recover_unauthorized_for_sampling(
    model_id_or_slug: &str,
    base_url: &str,
    extra_headers: &indexmap::IndexMap<String, String>,
) -> bool {
    let Some((provider, credential_id)) =
        credential_from_sampling_hints(model_id_or_slug, base_url, extra_headers)
    else {
        return false;
    };
    let home = token_resolve::grok_home();
    match token_resolve::recover_unauthorized_blocking(&home, &provider, credential_id) {
        Ok(true) => {
            tracing::info!(
                provider = %provider,
                credential_id = %credential_id,
                "multi-provider 401 recovery succeeded; resubmit once"
            );
            true
        }
        Ok(false) => {
            tracing::warn!(
                provider = %provider,
                credential_id = %credential_id,
                "multi-provider 401 recovery requires re-authentication"
            );
            false
        }
        Err(e) => {
            tracing::warn!(
                provider = %provider,
                credential_id = %credential_id,
                error = %e,
                "multi-provider 401 recovery failed"
            );
            false
        }
    }
}

fn find_codex_credential_for_account(
    home: &std::path::Path,
    provider: &ProviderId,
    account_id: &str,
) -> Option<CredentialId> {
    use xai_grok_auth::CredentialStore as _;
    // Current-thread / LocalSet safe: never block_in_place.
    let home = home.to_path_buf();
    let provider = provider.clone();
    let account_id = account_id.to_string();
    token_resolve::block_on_safe(async move {
        let store = xai_grok_multi_auth::store::FileCredentialStore::new(home);
        let accounts = match store.list_accounts(&provider).await {
            Ok(a) => a,
            Err(_) => return Ok(None),
        };
        for meta in accounts {
            let match_meta = meta.account.provider_account_id.as_deref()
                == Some(account_id.as_str())
                || meta
                    .account
                    .metadata
                    .get("chatgpt_account_id")
                    .map(|s| s.as_str())
                    == Some(account_id.as_str());
            if match_meta {
                return Ok(Some(meta.key.credential_id));
            }
            // Fall back to loaded secret metadata (enriched JWT claims).
            if let Ok(Some(cred)) = store.load(&meta.key).await {
                let acct = &cred.metadata.account;
                if acct.provider_account_id.as_deref() == Some(account_id.as_str())
                    || acct
                        .metadata
                        .get("chatgpt_account_id")
                        .map(|s| s.as_str())
                        == Some(account_id.as_str())
                {
                    return Ok(Some(meta.key.credential_id));
                }
            }
        }
        Ok(None)
    })
    .ok()
    .flatten()
}

/// Sync [`BearerResolver`] that reloads/refreshes via TokenManager each call.
pub struct MultiProviderBearerResolver {
    provider: ProviderId,
    credential_id: CredentialId,
    home: std::path::PathBuf,
}

impl std::fmt::Debug for MultiProviderBearerResolver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MultiProviderBearerResolver")
            .field("provider", &self.provider)
            .field("credential_id", &self.credential_id)
            .finish_non_exhaustive()
    }
}

impl BearerResolver for MultiProviderBearerResolver {
    fn current_bearer(&self) -> Option<String> {
        match token_resolve::resolve_access_token_blocking(
            &self.home,
            &self.provider,
            self.credential_id,
        ) {
            Ok(t) => Some(t),
            Err(e) => {
                tracing::warn!(
                    provider = %self.provider,
                    credential_id = %self.credential_id,
                    error = %e,
                    "MultiProviderBearerResolver: token resolve failed"
                );
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use xai_grok_multi_auth::provider_model_key::format_provider_model_key;
    use xai_grok_sampler::AuthScheme;

    fn entry_with_id(id: &str, model: &str) -> ModelEntry {
        let mut info = crate::agent::config::ModelInfo::fallback(model);
        info.id = Some(id.to_string());
        info.model = model.to_string();
        info.auth_scheme = AuthScheme::Bearer;
        info.base_url = "https://chatgpt.com/backend-api/codex".into();
        ModelEntry {
            info,
            api_key: None,
            env_key: None,
            api_base_url: None,
        }
    }

    #[test]
    fn binding_from_credential_scoped_id() {
        let provider = ProviderId::new_unchecked("codex");
        let cred = CredentialId::from_uuid(
            uuid::Uuid::parse_str("01234567-89ab-cdef-0123-456789abcdef").unwrap(),
        );
        let key = format_provider_model_key(&provider, cred, "gpt-5.4");
        let entry = entry_with_id(&key, "gpt-5.4");
        let binding = binding_from_model_entry(&entry).expect("binding");
        assert_eq!(binding.provider.as_str(), "codex");
        assert_eq!(binding.credential_id, cred);
        assert_eq!(binding.model, "gpt-5.4");
        assert!(entry.has_own_credentials());
    }

    #[test]
    fn no_binding_for_xai_models() {
        let entry = entry_with_id("grok-4.5", "grok-4.5");
        assert!(binding_from_model_entry(&entry).is_none());
        assert!(!entry.has_own_credentials());
    }
}
