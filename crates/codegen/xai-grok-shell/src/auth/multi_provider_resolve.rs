//! Request-time multi-provider token resolution for shell sampling seams.
//!
//! Catalog entries use credential-scoped keys (`codex/{credential_id}/{slug}`)
//! with **no** OAuth token in `ModelEntry.api_key`. This module:
//! - parses the binding from the catalog identity;
//! - holds a typed [`ModelBinding`] for the session/turn (A4);
//! - resolves a fresh access token via `xai_grok_multi_auth::token_resolve`;
//! - exposes a [`xai_grok_sampler::BearerResolver`] that records a
//!   request-scoped [`SentCredentialStamp`] for generation-aware 401 recovery (A1).

use std::sync::Arc;

use xai_grok_auth::{CredentialId, ModelBinding, ProviderId, SentCredentialStamp};
use xai_grok_multi_auth::provider_model_key::{
    parse_provider_model_key, ProviderModelKey,
};
use xai_grok_multi_auth::token_resolve;
use xai_grok_sampler::{BearerResolver, SharedBearerResolver};

use crate::agent::config::ModelEntry;

/// Session/turn multi-provider auth: typed binding + the resolver that holds
/// the stamp for the last successful bearer resolve (A1 + A4).
#[derive(Clone)]
pub struct MultiProviderSessionAuth {
    binding: ModelBinding,
    resolver: Arc<MultiProviderBearerResolver>,
}

impl MultiProviderSessionAuth {
    pub fn new(provider: ProviderId, credential_id: CredentialId, model: impl Into<String>) -> Self {
        let home = token_resolve::grok_home();
        let resolver = Arc::new(MultiProviderBearerResolver::new(
            provider.clone(),
            credential_id,
            home,
        ));
        Self {
            binding: ModelBinding::new(provider, Some(credential_id), model),
            resolver,
        }
    }

    pub fn from_provider_model_key(key: ProviderModelKey) -> Self {
        Self::new(key.provider, key.credential_id, key.model)
    }

    /// Typed model/account binding fixed for this session (A4).
    pub fn binding(&self) -> &ModelBinding {
        &self.binding
    }

    pub fn provider(&self) -> &ProviderId {
        &self.binding.provider
    }

    pub fn credential_id(&self) -> Option<CredentialId> {
        self.binding.credential
    }

    /// Shared resolver for `SamplerConfig::bearer_resolver`.
    pub fn shared_bearer_resolver(&self) -> SharedBearerResolver {
        self.resolver.clone() as SharedBearerResolver
    }

    pub fn resolver(&self) -> &Arc<MultiProviderBearerResolver> {
        &self.resolver
    }

    /// Generation-aware 401 recovery using the stamp from the resolve attempt
    /// that failed (A1 / AUD-006: FIFO of attempt stamps, not last-wins).
    pub fn try_recover_unauthorized(&self) -> bool {
        let Some(credential_id) = self.binding.credential else {
            return false;
        };
        // Prefer oldest unrecovered attempt stamp (matches sequential request order).
        let stamp = self.resolver.take_stamp_for_recovery();
        let home = token_resolve::grok_home();
        match token_resolve::recover_unauthorized_with_stamp_blocking(
            &home,
            &self.binding.provider,
            credential_id,
            stamp,
        ) {
            Ok(true) => {
                tracing::info!(
                    provider = %self.binding.provider,
                    credential_id = %credential_id,
                    used_request_stamp = true,
                    "multi-provider 401 recovery succeeded; resubmit once"
                );
                true
            }
            Ok(false) => {
                tracing::warn!(
                    provider = %self.binding.provider,
                    credential_id = %credential_id,
                    "multi-provider 401 recovery requires re-authentication"
                );
                false
            }
            Err(e) => {
                tracing::warn!(
                    provider = %self.binding.provider,
                    credential_id = %credential_id,
                    error = %e,
                    "multi-provider 401 recovery failed"
                );
                false
            }
        }
    }
}

/// Extract a multi-provider binding from a model catalog entry.
pub fn binding_from_model_entry(model: &ModelEntry) -> Option<ProviderModelKey> {
    let key = model
        .info
        .id
        .as_deref()
        .filter(|s| !s.is_empty())
        .or_else(|| {
            let m = model.info.model.as_str();
            if m.contains('/') {
                Some(m)
            } else {
                None
            }
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

/// Build session auth (binding + stamp-holding resolver) from a catalog entry.
pub fn session_auth_for_model_entry(model: &ModelEntry) -> Option<MultiProviderSessionAuth> {
    let key = binding_from_model_entry(model)?;
    Some(MultiProviderSessionAuth::from_provider_model_key(key))
}

/// Build a per-request bearer resolver for a multi-provider model, if bound.
pub fn bearer_resolver_for_model_entry(model: &ModelEntry) -> Option<SharedBearerResolver> {
    session_auth_for_model_entry(model).map(|a| a.shared_bearer_resolver())
}

/// Session auth from a raw catalog model id string.
pub fn session_auth_for_model_id(model_id: &str) -> Option<MultiProviderSessionAuth> {
    let key = parse_provider_model_key(model_id)?;
    Some(MultiProviderSessionAuth::from_provider_model_key(key))
}

/// Also build from a raw catalog model id string (session reconstruct path).
pub fn bearer_resolver_for_model_id(model_id: &str) -> Option<SharedBearerResolver> {
    session_auth_for_model_id(model_id).map(|a| a.shared_bearer_resolver())
}

/// Session reconstruct path: chat_state keeps the **wire** model slug, not the
/// credential-scoped catalog key. Recover the credential via the
/// `ChatGPT-Account-ID` header that merge injects into multi-provider entries.
pub fn session_auth_for_codex_account_header(
    account_id: &str,
    model_slug: &str,
) -> Option<MultiProviderSessionAuth> {
    if account_id.is_empty() {
        return None;
    }
    let home = token_resolve::grok_home();
    let provider = ProviderId::new_unchecked("codex");
    let credential_id = find_codex_credential_for_account(&home, &provider, account_id)?;
    Some(MultiProviderSessionAuth::new(
        provider,
        credential_id,
        model_slug.to_string(),
    ))
}

pub fn bearer_resolver_for_codex_account_header(
    account_id: &str,
) -> Option<SharedBearerResolver> {
    session_auth_for_codex_account_header(account_id, "unknown")
        .map(|a| a.shared_bearer_resolver())
}

// A4 pin policy lives in multi-auth so unit tests run without shell harness.
pub use xai_grok_multi_auth::{session_pin_decision, SessionPinDecision};

/// Prefer catalog-key; else credential-id header; else account lookup.
/// Returns full session auth so the caller can pin binding + stamp holder.
pub fn session_auth_for_sampling_hints(
    model_id_or_slug: &str,
    base_url: &str,
    extra_headers: &indexmap::IndexMap<String, String>,
) -> Option<MultiProviderSessionAuth> {
    if let Some(a) = session_auth_for_model_id(model_id_or_slug) {
        return Some(a);
    }
    if let Some((provider, credential_id)) =
        credential_from_sampling_hints(model_id_or_slug, base_url, extra_headers)
    {
        // Wire model is the slug part when catalog key is unavailable.
        let model = model_id_or_slug
            .rsplit('/')
            .next()
            .unwrap_or(model_id_or_slug)
            .to_string();
        return Some(MultiProviderSessionAuth::new(provider, credential_id, model));
    }
    None
}

pub fn bearer_resolver_for_sampling_hints(
    model_id_or_slug: &str,
    base_url: &str,
    extra_headers: &indexmap::IndexMap<String, String>,
) -> Option<SharedBearerResolver> {
    session_auth_for_sampling_hints(model_id_or_slug, base_url, extra_headers)
        .map(|a| a.shared_bearer_resolver())
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
/// **Prefer** [`MultiProviderSessionAuth::try_recover_unauthorized`] with the
/// session-held auth (uses request stamp). This fallback only runs when the
/// session has not pinned multi-provider auth yet.
pub fn try_recover_unauthorized_for_sampling(
    model_id_or_slug: &str,
    base_url: &str,
    extra_headers: &indexmap::IndexMap<String, String>,
) -> bool {
    // Prefer building session auth so we still have a typed binding; stamp
    // will only be present if this same resolver instance was used for resolve.
    if let Some(auth) =
        session_auth_for_sampling_hints(model_id_or_slug, base_url, extra_headers)
    {
        // New resolver instance → no stamp from the failed request. Callers
        // must use session-held MultiProviderSessionAuth instead.
        return auth.try_recover_unauthorized();
    }
    false
}

/// Recover using an already-pinned session auth (A1 stamp + A4 binding).
pub fn try_recover_unauthorized_with_session_auth(auth: &MultiProviderSessionAuth) -> bool {
    auth.try_recover_unauthorized()
}

fn find_codex_credential_for_account(
    home: &std::path::Path,
    provider: &ProviderId,
    account_id: &str,
) -> Option<CredentialId> {
    use xai_grok_auth::CredentialStore as _;
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
///
/// **Stamp policy (A1 / AUD-006):**
/// - [`BearerResolver::current_bearer`] (HTTP send via `SamplingClient::post`)
///   resolves and **records** an attempt stamp for 401 recovery.
/// - [`BearerResolver::peek_bearer`] (auth_info / attribution prefix) resolves
///   or reuses the last token **without** enqueueing a recovery stamp.
///
/// Logging calls must never pollute the recovery FIFO.
pub struct MultiProviderBearerResolver {
    provider: ProviderId,
    credential_id: CredentialId,
    home: std::path::PathBuf,
    stamps: xai_grok_multi_auth::request_stamp::RequestScopedStamp,
    /// Last token string for peek without a second network resolve when possible.
    last_token: std::sync::Mutex<Option<String>>,
}

impl MultiProviderBearerResolver {
    pub fn new(
        provider: ProviderId,
        credential_id: CredentialId,
        home: std::path::PathBuf,
    ) -> Self {
        Self {
            provider,
            credential_id,
            home,
            stamps: xai_grok_multi_auth::request_stamp::RequestScopedStamp::new(),
            last_token: std::sync::Mutex::new(None),
        }
    }

    /// Last resolve stamp (peek only — prefer [`Self::take_stamp_for_recovery`]).
    pub fn last_stamp(&self) -> Option<SentCredentialStamp> {
        self.stamps.last()
    }

    /// Consume the oldest unrecovered attempt stamp for 401 recovery.
    pub fn take_stamp_for_recovery(&self) -> Option<SentCredentialStamp> {
        self.stamps.take_for_recovery()
    }

    pub fn take_last_stamp(&self) -> Option<SentCredentialStamp> {
        self.take_stamp_for_recovery()
    }

    /// Pending recovery stamps (tests).
    pub fn pending_recovery_len(&self) -> usize {
        self.stamps.ledger().pending_recovery_len()
    }

    /// Test/helper: inject a stamp without going through the network.
    #[cfg(test)]
    pub fn set_last_stamp_for_test(&self, stamp: SentCredentialStamp) {
        self.stamps.record(stamp);
    }

    /// Resolve token without recording a recovery stamp (log/attribution only).
    fn resolve_token_no_stamp(&self) -> Option<String> {
        match token_resolve::resolve_access_token_with_stamp_blocking(
            &self.home,
            &self.provider,
            self.credential_id,
        ) {
            Ok((t, _stamp)) => {
                if let Ok(mut g) = self.last_token.lock() {
                    *g = Some(t.clone());
                }
                Some(t)
            }
            Err(e) => {
                tracing::warn!(
                    provider = %self.provider,
                    credential_id = %self.credential_id,
                    error = %e,
                    "MultiProviderBearerResolver: peek resolve failed"
                );
                None
            }
        }
    }

    /// Resolve token + record attempt stamp (HTTP send path only).
    pub fn resolve_attempt(&self) -> Option<(String, SentCredentialStamp, u64)> {
        match token_resolve::resolve_access_token_with_stamp_blocking(
            &self.home,
            &self.provider,
            self.credential_id,
        ) {
            Ok((t, stamp)) => {
                let id = self.stamps.record(stamp.clone());
                if let Ok(mut g) = self.last_token.lock() {
                    *g = Some(t.clone());
                }
                Some((t, stamp, id))
            }
            Err(e) => {
                tracing::warn!(
                    provider = %self.provider,
                    credential_id = %self.credential_id,
                    error = %e,
                    "MultiProviderBearerResolver: resolve_attempt failed"
                );
                None
            }
        }
    }
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
    /// HTTP send path: record recovery stamp for this attempt only.
    fn current_bearer(&self) -> Option<String> {
        self.resolve_attempt().map(|(t, _, _)| t)
    }

    /// Logging / attribution: never enqueue a recovery stamp (AUD-006).
    fn peek_bearer(&self) -> Option<String> {
        if let Ok(g) = self.last_token.lock() {
            if let Some(t) = g.as_ref() {
                return Some(t.clone());
            }
        }
        self.resolve_token_no_stamp()
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

    #[test]
    fn session_auth_holds_typed_model_binding() {
        let provider = ProviderId::new_unchecked("codex");
        let cred = CredentialId::from_uuid(
            uuid::Uuid::parse_str("aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee").unwrap(),
        );
        let key = format_provider_model_key(&provider, cred, "gpt-5.6-luna");
        let auth = session_auth_for_model_id(&key).expect("session auth");
        assert_eq!(auth.binding().provider.as_str(), "codex");
        assert_eq!(auth.binding().credential, Some(cred));
        assert_eq!(auth.binding().model, "gpt-5.6-luna");
        // Mid-session flip prevention: binding is fixed on the auth object.
        let again = auth.binding().clone();
        assert_eq!(again.credential, Some(cred));
    }

    #[test]
    fn concurrent_resolvers_keep_distinct_stamps() {
        use xai_grok_auth::{AccountFingerprint, CredentialKey};

        let provider = ProviderId::new_unchecked("codex");
        let cred_a = CredentialId::from_uuid(
            uuid::Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap(),
        );
        let cred_b = CredentialId::from_uuid(
            uuid::Uuid::parse_str("22222222-2222-2222-2222-222222222222").unwrap(),
        );
        let home = std::path::PathBuf::from("/tmp/goblin-stamp-isolation-test");
        let ra = Arc::new(MultiProviderBearerResolver::new(
            provider.clone(),
            cred_a,
            home.clone(),
        ));
        let rb = Arc::new(MultiProviderBearerResolver::new(
            provider.clone(),
            cred_b,
            home,
        ));

        ra.set_last_stamp_for_test(SentCredentialStamp {
            key: CredentialKey {
                provider: provider.clone(),
                credential_id: cred_a,
            },
            generation: 1,
            account_fingerprint: AccountFingerprint::from([1u8; 32]),
        });
        rb.set_last_stamp_for_test(SentCredentialStamp {
            key: CredentialKey {
                provider: provider.clone(),
                credential_id: cred_b,
            },
            generation: 99,
            account_fingerprint: AccountFingerprint::from([2u8; 32]),
        });

        assert_eq!(ra.last_stamp().unwrap().generation, 1);
        assert_eq!(rb.last_stamp().unwrap().generation, 99);

        let auth_a = MultiProviderSessionAuth::new(provider.clone(), cred_a, "m");
        let auth_b = MultiProviderSessionAuth::new(provider, cred_b, "m");
        assert_eq!(auth_a.credential_id(), Some(cred_a));
        assert_eq!(auth_b.credential_id(), Some(cred_b));
    }

    #[test]
    fn same_resolver_sequential_stamps_recover_fifo() {
        use xai_grok_auth::{AccountFingerprint, CredentialKey};

        let provider = ProviderId::new_unchecked("codex");
        let cred = CredentialId::from_uuid(
            uuid::Uuid::parse_str("aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee").unwrap(),
        );
        let r = MultiProviderBearerResolver::new(
            provider.clone(),
            cred,
            std::path::PathBuf::from("/tmp/goblin-fifo-stamp"),
        );
        r.set_last_stamp_for_test(SentCredentialStamp {
            key: CredentialKey {
                provider: provider.clone(),
                credential_id: cred,
            },
            generation: 1,
            account_fingerprint: AccountFingerprint::from([1u8; 32]),
        });
        r.set_last_stamp_for_test(SentCredentialStamp {
            key: CredentialKey {
                provider,
                credential_id: cred,
            },
            generation: 2,
            account_fingerprint: AccountFingerprint::from([2u8; 32]),
        });
        // last() is attempt 2, but recovery must take attempt 1 first.
        assert_eq!(r.last_stamp().unwrap().generation, 2);
        assert_eq!(r.take_stamp_for_recovery().unwrap().generation, 1);
        assert_eq!(r.take_stamp_for_recovery().unwrap().generation, 2);
    }

    /// Production call order: auth_info/peek before post, attribution after.
    /// Only `current_bearer` (send) may enqueue recovery stamps.
    #[test]
    fn production_peek_send_order_does_not_pollute_recovery_fifo() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use xai_grok_auth::{AccountFingerprint, CredentialKey};

        /// Mirrors MultiProviderBearerResolver stamp policy without network.
        struct ProdOrderResolver {
            peeks: AtomicUsize,
            sends: AtomicUsize,
            stamps: xai_grok_multi_auth::request_stamp::RequestScopedStamp,
            provider: ProviderId,
            cred: CredentialId,
        }

        impl std::fmt::Debug for ProdOrderResolver {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("ProdOrderResolver")
            }
        }

        impl BearerResolver for ProdOrderResolver {
            fn current_bearer(&self) -> Option<String> {
                let n = self.sends.fetch_add(1, Ordering::SeqCst) + 1;
                self.stamps.record(SentCredentialStamp {
                    key: CredentialKey {
                        provider: self.provider.clone(),
                        credential_id: self.cred,
                    },
                    generation: n as u64,
                    account_fingerprint: AccountFingerprint::from([n as u8; 32]),
                });
                Some(format!("token-{n}"))
            }

            fn peek_bearer(&self) -> Option<String> {
                self.peeks.fetch_add(1, Ordering::SeqCst);
                // No stamp record — matches MultiProvider peek policy.
                Some("token-peek".into())
            }
        }

        let provider = ProviderId::new_unchecked("codex");
        let cred = CredentialId::from_uuid(
            uuid::Uuid::parse_str("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb").unwrap(),
        );
        let r = ProdOrderResolver {
            peeks: AtomicUsize::new(0),
            sends: AtomicUsize::new(0),
            stamps: xai_grok_multi_auth::request_stamp::RequestScopedStamp::new(),
            provider,
            cred,
        };

        // request_task: auth_info() before the loop
        let _ = r.peek_bearer();
        let _ = r.peek_bearer();
        // SamplingClient::post()
        let sent = r.current_bearer().expect("send");
        assert_eq!(sent, "token-1");
        // 401 attribution current_sent_bearer_prefix
        let _ = r.peek_bearer();
        // Retry post after recovery of first attempt
        let _ = r.current_bearer();

        assert_eq!(r.peeks.load(Ordering::SeqCst), 3);
        assert_eq!(r.sends.load(Ordering::SeqCst), 2);
        assert_eq!(
            r.stamps.ledger().pending_recovery_len(),
            2,
            "only send resolves enqueue stamps"
        );
        // Recovery for first 401 must be gen=1 (the POST), not a peek.
        assert_eq!(r.stamps.take_for_recovery().unwrap().generation, 1);
        assert_eq!(r.stamps.take_for_recovery().unwrap().generation, 2);
    }
}
