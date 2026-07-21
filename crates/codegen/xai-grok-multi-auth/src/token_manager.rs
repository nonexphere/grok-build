//! Token manager: single-flight refresh, generation-aware 401, fingerprint check.
//!
//! Implements the refresh algorithm from task.md §5.4:
//! 1. Load cached credential.
//! 2. Verify provider and account identity.
//! 3. If token is outside the early-refresh window, return it.
//! 4. Acquire the per-credential in-process mutex.
//! 5. Reload from persistent storage.
//! 6. If another process already refreshed, adopt and return.
//! 7. Call AuthProvider::refresh.
//! 8. Verify fingerprint did not change.
//! 9. Compare-and-swap using the previous generation.
//! 10. Release locks and return.

use std::collections::BTreeMap;
use std::sync::Arc;

use chrono::Utc;
use dashmap::DashMap;
use thiserror::Error;
use tokio::sync::Mutex;

use xai_grok_auth::{
    CredentialBinding, CredentialKey, CredentialLockPurpose, CredentialSecret, CredentialStore,
    CredentialUpdate, ProviderError, ProviderRegistry, SecretString, SentCredentialStamp,
    StoredCredential, TokenUseReason, UnauthorizedRecovery, ValidToken,
};

use crate::fingerprint;

/// Errors from the token manager.
#[derive(Debug, Error)]
pub enum TokenManagerError {
    #[error("credential not found")]
    NotFound,

    #[error("account identity changed")]
    AccountMismatch,

    #[error("provider error: {0}")]
    Provider(#[from] ProviderError),

    #[error("store error: {0}")]
    Store(#[from] xai_grok_auth::StoreError),

    #[error("CAS error: {0}")]
    Cas(#[from] xai_grok_auth::CompareAndSwapError),

    #[error("provider not registered: {0}")]
    ProviderNotRegistered(String),
}

/// Token manager with per-credential single-flight refresh.
pub struct TokenManager {
    store: Arc<dyn CredentialStore>,
    registry: Arc<ProviderRegistry>,
    /// Per-credential in-process mutex for single-flight refresh.
    locks: DashMap<CredentialKey, Arc<Mutex<()>>>,
    /// Issuer string used for fingerprint computation.
    /// In production this would come from the provider's config.
    issuer: String,
}

impl std::fmt::Debug for TokenManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TokenManager")
            .field("registry", &self.registry)
            .finish_non_exhaustive()
    }
}

impl TokenManager {
    /// Create a new token manager.
    pub fn new(store: Arc<dyn CredentialStore>, registry: Arc<ProviderRegistry>) -> Self {
        Self {
            store,
            registry,
            locks: DashMap::new(),
            issuer: "https://auth.openai.com".to_string(),
        }
    }

    /// Create with a custom issuer (for fingerprint computation).
    pub fn with_issuer(
        store: Arc<dyn CredentialStore>,
        registry: Arc<ProviderRegistry>,
        issuer: String,
    ) -> Self {
        Self {
            store,
            registry,
            locks: DashMap::new(),
            issuer,
        }
    }

    /// Get a valid token, refreshing if necessary (single-flight per credential).
    pub async fn get_valid_token(
        &self,
        binding: &CredentialBinding,
        reason: TokenUseReason,
    ) -> Result<ValidToken, TokenManagerError> {
        let now = Utc::now();
        let early_window = chrono::Duration::minutes(5);

        // Step 1: Load credential.
        let credential = self
            .store
            .load(&binding.key)
            .await?
            .ok_or(TokenManagerError::NotFound)?;

        // Step 2: Verify fingerprint.
        let current_fp = fingerprint::compute_fingerprint(
            &binding.key.provider,
            &self.issuer,
            &credential.metadata.account,
        );
        if current_fp != binding.expected_account {
            return Err(TokenManagerError::AccountMismatch);
        }

        // Step 3: If token is valid, return it.
        if !needs_refresh(&credential, now, early_window) {
            return Ok(valid_token_from(&credential, &current_fp));
        }

        // Step 4: In-process single-flight, then cross-process credential lock
        // (file flock on FileCredentialStore) so two OS processes cannot both
        // consume a rotating refresh token (AUD-007 / A2+R5).
        let lock = self.get_or_create_lock(&binding.key);
        let _guard = lock.lock().await;
        let _xproc = self
            .store
            .acquire_lock(&binding.key, CredentialLockPurpose::Refresh)
            .await?;

        // Step 5: Reload from store (double-check).
        let credential = self
            .store
            .load(&binding.key)
            .await?
            .ok_or(TokenManagerError::NotFound)?;

        // Step 6: If already refreshed by another task/process, adopt.
        if !needs_refresh(&credential, now, early_window) {
            return Ok(valid_token_from(&credential, &current_fp));
        }

        // Step 7: Call provider refresh.
        let provider = self
            .registry
            .get(&binding.key.provider)
            .map_err(|e| TokenManagerError::ProviderNotRegistered(e.to_string()))?;

        let update = provider
            .refresh(xai_grok_auth::RefreshRequest {
                credential: &credential,
                reason,
            })
            .await?;

        // Step 8: Verify fingerprint didn't change.
        if let Some(ref new_account) = update.account {
            let new_fp =
                fingerprint::compute_fingerprint(&binding.key.provider, &self.issuer, new_account);
            if new_fp != current_fp {
                return Err(TokenManagerError::AccountMismatch);
            }
        }

        // Step 9: CAS persist.
        let new_secret = build_updated_secret(&credential.secret, &update);
        let cas_update = CredentialUpdate {
            key: binding.key.clone(),
            account: update.account.clone(),
            secret: Some(new_secret),
            expires_at: Some(update.expires_at),
            status: None,
            updated_at: Utc::now(),
        };

        let new_metadata = self
            .store
            .compare_and_swap(credential.metadata.generation, cas_update)
            .await?;

        // Step 10: Return valid token.
        Ok(ValidToken {
            access_token: update
                .access_token
                .unwrap_or_else(|| credential.secret.access_token.clone()),
            expires_at: update.expires_at.or(credential.metadata.expires_at),
            generation: new_metadata.generation,
            account_fingerprint: current_fp,
        })
    }

    /// Recover from a 401 Unauthorized response (generation-aware).
    ///
    /// If the sent generation is stale (current is newer), retry with the
    /// current credential without refreshing. Otherwise, refresh once.
    pub async fn recover_unauthorized(
        &self,
        binding: &CredentialBinding,
        sent: &SentCredentialStamp,
        _status: u16,
    ) -> Result<UnauthorizedRecovery, TokenManagerError> {
        // Load current credential to check generation.
        let credential = self
            .store
            .load(&binding.key)
            .await?
            .ok_or(TokenManagerError::NotFound)?;

        // If the sent generation is stale (current is newer), the token was
        // already refreshed by another task/process. Retry with current.
        if credential.metadata.generation > sent.generation {
            return Ok(UnauthorizedRecovery::RetryWithCurrentCredential);
        }

        // If the fingerprint changed, reauth is required.
        let current_fp = fingerprint::compute_fingerprint(
            &binding.key.provider,
            &self.issuer,
            &credential.metadata.account,
        );
        if current_fp != binding.expected_account {
            return Ok(UnauthorizedRecovery::ReauthenticationRequired);
        }

        // Same generation: need to refresh once under in-process + cross-process
        // locks (AUD-007).
        let lock = self.get_or_create_lock(&binding.key);
        let _guard = lock.lock().await;
        let _xproc = self
            .store
            .acquire_lock(&binding.key, CredentialLockPurpose::Refresh)
            .await?;

        // Reload after acquiring locks.
        let credential = self
            .store
            .load(&binding.key)
            .await?
            .ok_or(TokenManagerError::NotFound)?;

        // If generation changed while waiting for the lock, retry with current.
        if credential.metadata.generation > sent.generation {
            return Ok(UnauthorizedRecovery::RetryWithCurrentCredential);
        }

        // Refresh.
        let provider = self
            .registry
            .get(&binding.key.provider)
            .map_err(|e| TokenManagerError::ProviderNotRegistered(e.to_string()))?;

        let update = provider
            .refresh(xai_grok_auth::RefreshRequest {
                credential: &credential,
                reason: TokenUseReason::UnauthorizedRecovery,
            })
            .await?;

        // CAS persist.
        let new_secret = build_updated_secret(&credential.secret, &update);
        let cas_update = CredentialUpdate {
            key: binding.key.clone(),
            account: update.account.clone(),
            secret: Some(new_secret),
            expires_at: Some(update.expires_at),
            status: None,
            updated_at: Utc::now(),
        };

        self.store
            .compare_and_swap(credential.metadata.generation, cas_update)
            .await?;

        Ok(UnauthorizedRecovery::RetryAfterRefresh)
    }

    fn get_or_create_lock(&self, key: &CredentialKey) -> Arc<Mutex<()>> {
        self.locks
            .entry(key.clone())
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone()
    }
}

/// Check if a credential's token needs refresh.
fn needs_refresh(
    credential: &StoredCredential,
    now: chrono::DateTime<Utc>,
    early_window: chrono::Duration,
) -> bool {
    match credential.metadata.expires_at {
        Some(expires_at) => expires_at <= now + early_window,
        None => false, // No expiration → assume valid
    }
}

/// Build a `ValidToken` from a stored credential.
fn valid_token_from(
    credential: &StoredCredential,
    fingerprint: &xai_grok_auth::AccountFingerprint,
) -> ValidToken {
    ValidToken {
        access_token: credential.secret.access_token.clone(),
        expires_at: credential.metadata.expires_at,
        generation: credential.metadata.generation,
        account_fingerprint: fingerprint.clone(),
    }
}

/// Merge a `ProviderCredentialUpdate` into the existing secret.
fn build_updated_secret(
    old: &CredentialSecret,
    update: &xai_grok_auth::ProviderCredentialUpdate,
) -> CredentialSecret {
    CredentialSecret {
        access_token: update
            .access_token
            .clone()
            .unwrap_or_else(|| old.access_token.clone()),
        refresh_token: update
            .refresh_token
            .clone()
            .or_else(|| old.refresh_token.clone()),
        id_token: update.id_token.clone().or_else(|| old.id_token.clone()),
        fields: {
            let mut fields: BTreeMap<String, SecretString> = old.fields.clone();
            fields.extend(update.fields.clone());
            fields
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fingerprint;
    use crate::store::ephemeral::EphemeralCredentialStore;
    use async_trait::async_trait;
    use chrono::Utc;
    use std::collections::BTreeMap;
    use std::sync::atomic::{AtomicU32, Ordering};
    use xai_grok_auth::{
        AuthFailureClass, AuthFailureResponse, AuthProvider, CredentialId, CredentialKey,
        CredentialMetadata, CredentialSecret, LoginCompletion, LoginFlowId, LoginInput,
        LoginRequest, LoginStart, LogoutOutcome, LogoutRequest, ModelCatalog, ModelListRequest,
        NewCredentialRecord, ProviderAccountInfo, ProviderCapabilities, ProviderCredentialUpdate,
        ProviderDescriptor, ProviderEndpointRequest, ProviderError, ProviderId,
        ProviderRequestAuth, RefreshRequest, RequestAuthContext, StoredCredential, TokenRequest,
        TokenResolution,
    };

    /// A mock provider that counts refresh calls.
    #[derive(Debug)]
    struct CountingProvider {
        id: ProviderId,
        refresh_count: Arc<AtomicU32>,
    }

    #[async_trait]
    impl AuthProvider for CountingProvider {
        fn id(&self) -> &ProviderId {
            &self.id
        }
        fn descriptor(&self) -> ProviderDescriptor {
            ProviderDescriptor {
                id: self.id.clone(),
                display_name: "mock".into(),
                short_name: "mock".into(),
                icon_key: None,
                capabilities: ProviderCapabilities::REFRESH_TOKEN,
                default_priority: 0,
            }
        }
        fn validate_config(&self) -> Result<(), ProviderError> {
            Ok(())
        }
        async fn start_login(&self, _: LoginRequest) -> Result<LoginStart, ProviderError> {
            Err(ProviderError::Disabled)
        }
        async fn complete_login(
            &self,
            _: LoginFlowId,
            _: LoginInput,
        ) -> Result<LoginCompletion, ProviderError> {
            Err(ProviderError::Disabled)
        }
        async fn cancel_login(&self, _: LoginFlowId) -> Result<(), ProviderError> {
            Ok(())
        }
        async fn refresh(
            &self,
            _req: RefreshRequest<'_>,
        ) -> Result<ProviderCredentialUpdate, ProviderError> {
            self.refresh_count.fetch_add(1, Ordering::SeqCst);
            // Small delay to let concurrent callers pile up on the lock.
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            Ok(ProviderCredentialUpdate {
                account: None,
                access_token: Some(SecretString::from_str("refreshed-token")),
                refresh_token: None,
                id_token: None,
                fields: BTreeMap::new(),
                expires_at: Some(Utc::now() + chrono::Duration::hours(1)),
            })
        }
        async fn get_valid_token(
            &self,
            _: TokenRequest<'_>,
        ) -> Result<TokenResolution, ProviderError> {
            Err(ProviderError::Disabled)
        }
        async fn logout(&self, _: LogoutRequest<'_>) -> Result<LogoutOutcome, ProviderError> {
            Err(ProviderError::Disabled)
        }
        async fn get_account_info(
            &self,
            cred: &StoredCredential,
        ) -> Result<ProviderAccountInfo, ProviderError> {
            Ok(cred.metadata.account.clone())
        }
        async fn list_models(
            &self,
            _: ModelListRequest<'_>,
        ) -> Result<ModelCatalog, ProviderError> {
            Ok(ModelCatalog {
                models: vec![],
                etag: None,
                fetched_at: Utc::now(),
                source: xai_grok_auth::ModelCatalogSource::Unknown,
                is_stale: false,
            })
        }
        fn resolve_endpoint(
            &self,
            _: ProviderEndpointRequest<'_>,
        ) -> Result<url::Url, ProviderError> {
            Err(ProviderError::Disabled)
        }
        fn build_request_auth(
            &self,
            _: RequestAuthContext<'_>,
        ) -> Result<ProviderRequestAuth, ProviderError> {
            Err(ProviderError::Disabled)
        }
        fn classify_auth_failure(&self, _: &AuthFailureResponse) -> AuthFailureClass {
            AuthFailureClass::NotAuthentication
        }
        fn supports_credential(&self, _: &CredentialMetadata) -> bool {
            true
        }
    }

    fn make_credential(
        provider: &ProviderId,
        expired: bool,
    ) -> (CredentialKey, CredentialMetadata, CredentialSecret) {
        let key = CredentialKey {
            provider: provider.clone(),
            credential_id: CredentialId::new(),
        };
        let now = Utc::now();
        let expires_at = if expired {
            Some(now - chrono::Duration::minutes(1))
        } else {
            Some(now + chrono::Duration::hours(1))
        };
        let metadata = CredentialMetadata {
            schema_version: 1,
            key: key.clone(),
            alias: "test".to_string(),
            account: ProviderAccountInfo::default(),
            created_at: now,
            updated_at: now,
            last_used_at: None,
            expires_at,
            status: xai_grok_auth::CredentialStatus::Ready,
            generation: 1,
            secret_backend: xai_grok_auth::SecretBackendKind::Ephemeral,
        };
        let secret = CredentialSecret {
            access_token: SecretString::from_str("old-token"),
            refresh_token: Some(SecretString::from_str("refresh-token")),
            id_token: None,
            fields: BTreeMap::new(),
        };
        (key, metadata, secret)
    }

    /// Test 12: refresh single-flight 50 concurrent => 1 refresh.
    #[tokio::test]
    async fn refresh_single_flight_50_concurrent_one_refresh() {
        let store = Arc::new(EphemeralCredentialStore::new());
        let refresh_count = Arc::new(AtomicU32::new(0));
        let provider = Arc::new(CountingProvider {
            id: ProviderId::new_unchecked("mock"),
            refresh_count: refresh_count.clone(),
        });
        let mut registry = ProviderRegistry::new();
        registry.register(provider).unwrap();
        let registry = Arc::new(registry);

        // Create an expired credential.
        let provider_id = ProviderId::new_unchecked("mock");
        let (key, metadata, secret) = make_credential(&provider_id, true);

        // Manually insert into the ephemeral store.
        store
            .create(NewCredentialRecord {
                provider: provider_id.clone(),
                requested_alias: Some("test".to_string()),
                account: ProviderAccountInfo::default(),
                secret: secret.clone(),
                expires_at: metadata.expires_at,
                backend: xai_grok_auth::SecretBackendKind::Ephemeral,
            })
            .await
            .unwrap();

        // Get the actual key from the store (the create generates a new ID).
        let accounts = store.list_accounts(&provider_id).await.unwrap();
        let actual_key = accounts[0].key.clone();

        let fp = fingerprint::fingerprint_from_parts("mock", "https://auth.openai.com", "", "");
        let binding = CredentialBinding {
            key: actual_key.clone(),
            expected_account: fp,
        };

        let tm = Arc::new(TokenManager::new(store, registry));

        // Launch 50 concurrent get_valid_token calls.
        let mut handles = Vec::new();
        for _ in 0..50 {
            let tm = tm.clone();
            let binding = binding.clone();
            handles.push(tokio::spawn(async move {
                tm.get_valid_token(&binding, TokenUseReason::Inference)
                    .await
            }));
        }

        // Wait for all to complete.
        for handle in handles {
            handle.await.unwrap().unwrap();
        }

        // Only 1 refresh should have occurred.
        assert_eq!(
            refresh_count.load(Ordering::SeqCst),
            1,
            "expected exactly 1 refresh, got {}",
            refresh_count.load(Ordering::SeqCst)
        );
    }

    /// Two independent TokenManagers on the same file home (separate in-process
    /// lock maps) must still single-flight via cross-process credential flock.
    #[tokio::test]
    async fn two_managers_same_file_home_one_refresh_via_xproc_lock() {
        use crate::store::file::FileCredentialStore;

        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().to_path_buf();
        let refresh_count = Arc::new(AtomicU32::new(0));
        let provider = Arc::new(CountingProvider {
            id: ProviderId::new_unchecked("mock"),
            refresh_count: refresh_count.clone(),
        });
        let mut registry = ProviderRegistry::new();
        registry.register(provider).unwrap();
        let registry = Arc::new(registry);

        // Shared disk state; each manager gets its own FileCredentialStore Arc
        // (as two processes would).
        let store_a: Arc<dyn CredentialStore> = Arc::new(FileCredentialStore::new(home.clone()));
        let store_b: Arc<dyn CredentialStore> = Arc::new(FileCredentialStore::new(home.clone()));

        let provider_id = ProviderId::new_unchecked("mock");
        let meta = store_a
            .create(NewCredentialRecord {
                provider: provider_id.clone(),
                requested_alias: Some("xproc".into()),
                account: ProviderAccountInfo::default(),
                secret: CredentialSecret {
                    access_token: SecretString::from_str("old-token"),
                    refresh_token: Some(SecretString::from_str("refresh-token")),
                    id_token: None,
                    fields: BTreeMap::new(),
                },
                expires_at: Some(Utc::now() - chrono::Duration::hours(1)),
                backend: xai_grok_auth::SecretBackendKind::File,
            })
            .await
            .unwrap();

        let fp = fingerprint::fingerprint_from_parts("mock", "https://auth.openai.com", "", "");
        let binding = CredentialBinding {
            key: meta.key.clone(),
            expected_account: fp,
        };

        let tm_a = Arc::new(TokenManager::new(store_a, registry.clone()));
        let tm_b = Arc::new(TokenManager::new(store_b, registry));

        let mut handles = Vec::new();
        for tm in [tm_a, tm_b] {
            for _ in 0..8 {
                let tm = tm.clone();
                let binding = binding.clone();
                handles.push(tokio::spawn(async move {
                    tm.get_valid_token(&binding, TokenUseReason::Inference)
                        .await
                }));
            }
        }
        for handle in handles {
            handle.await.unwrap().unwrap();
        }
        assert_eq!(
            refresh_count.load(Ordering::SeqCst),
            1,
            "cross-process flock must keep refresh single-flight across managers; got {}",
            refresh_count.load(Ordering::SeqCst)
        );
    }

    /// Test 13: stale 401 generation => no refresh.
    #[tokio::test]
    async fn stale_401_generation_no_refresh() {
        let store = Arc::new(EphemeralCredentialStore::new());
        let refresh_count = Arc::new(AtomicU32::new(0));
        let provider = Arc::new(CountingProvider {
            id: ProviderId::new_unchecked("mock"),
            refresh_count: refresh_count.clone(),
        });
        let mut registry = ProviderRegistry::new();
        registry.register(provider).unwrap();
        let registry = Arc::new(registry);

        let provider_id = ProviderId::new_unchecked("mock");
        let (key, metadata, secret) = make_credential(&provider_id, false);

        store
            .create(NewCredentialRecord {
                provider: provider_id.clone(),
                requested_alias: Some("test".to_string()),
                account: ProviderAccountInfo::default(),
                secret: secret.clone(),
                expires_at: metadata.expires_at,
                backend: xai_grok_auth::SecretBackendKind::Ephemeral,
            })
            .await
            .unwrap();

        let accounts = store.list_accounts(&provider_id).await.unwrap();
        let actual_key = accounts[0].key.clone();

        // Bump the generation to 2 (as if another process refreshed).
        let update = CredentialUpdate {
            key: actual_key.clone(),
            account: None,
            secret: Some(CredentialSecret {
                access_token: SecretString::from_str("new-token-by-other"),
                refresh_token: Some(SecretString::from_str("new-refresh")),
                id_token: None,
                fields: BTreeMap::new(),
            }),
            // CredentialUpdate.expires_at is Option<Option<_>>: outer None =
            // leave unchanged, Some(None) = clear, Some(Some(t)) = set.
            expires_at: Some(Some(Utc::now() + chrono::Duration::hours(1))),
            status: None,
            updated_at: Utc::now(),
        };
        store.compare_and_swap(1, update).await.unwrap();

        let fp = fingerprint::fingerprint_from_parts("mock", "https://auth.openai.com", "", "");
        let binding = CredentialBinding {
            key: actual_key.clone(),
            expected_account: fp.clone(),
        };

        // Sent stamp has the OLD generation (1).
        let sent = SentCredentialStamp {
            key: actual_key.clone(),
            generation: 1,
            account_fingerprint: fp,
        };

        let tm = TokenManager::new(store, registry);

        // recover_unauthorized should see that generation 2 > 1 and retry
        // with current credential WITHOUT refreshing.
        let recovery = tm.recover_unauthorized(&binding, &sent, 401).await.unwrap();

        assert_eq!(
            recovery,
            UnauthorizedRecovery::RetryWithCurrentCredential,
            "stale 401 should retry with current credential, not refresh"
        );

        // No refresh should have occurred.
        assert_eq!(refresh_count.load(Ordering::SeqCst), 0);
    }

    /// Same-generation 401 → refresh once → CAS persist → RetryAfterRefresh.
    #[tokio::test]
    async fn same_generation_401_refreshes_persists_retry_after_refresh() {
        let store = Arc::new(EphemeralCredentialStore::new());
        let refresh_count = Arc::new(AtomicU32::new(0));
        let provider = Arc::new(CountingProvider {
            id: ProviderId::new_unchecked("mock"),
            refresh_count: refresh_count.clone(),
        });
        let mut registry = ProviderRegistry::new();
        registry.register(provider).unwrap();
        let registry = Arc::new(registry);

        let provider_id = ProviderId::new_unchecked("mock");
        let (_key, metadata, secret) = make_credential(&provider_id, false);
        store
            .create(NewCredentialRecord {
                provider: provider_id.clone(),
                requested_alias: Some("test".into()),
                account: ProviderAccountInfo::default(),
                secret: secret.clone(),
                expires_at: metadata.expires_at,
                backend: xai_grok_auth::SecretBackendKind::Ephemeral,
            })
            .await
            .unwrap();

        let accounts = store.list_accounts(&provider_id).await.unwrap();
        let actual_key = accounts[0].key.clone();
        let gen_before = accounts[0].generation;
        assert_eq!(gen_before, 1);

        let fp = fingerprint::fingerprint_from_parts("mock", "https://auth.openai.com", "", "");
        let binding = CredentialBinding {
            key: actual_key.clone(),
            expected_account: fp.clone(),
        };
        // Sent stamp matches store generation → must refresh once.
        let sent = SentCredentialStamp {
            key: actual_key.clone(),
            generation: gen_before,
            account_fingerprint: fp,
        };

        let tm = TokenManager::new(store.clone(), registry);
        let recovery = tm.recover_unauthorized(&binding, &sent, 401).await.unwrap();
        assert_eq!(
            recovery,
            UnauthorizedRecovery::RetryAfterRefresh,
            "same-generation 401 must refresh and return RetryAfterRefresh"
        );
        assert_eq!(
            refresh_count.load(Ordering::SeqCst),
            1,
            "exactly one provider.refresh"
        );

        let loaded = store.load(&actual_key).await.unwrap().unwrap();
        assert_eq!(loaded.metadata.generation, gen_before + 1);
        assert_eq!(loaded.secret.access_token.expose(), "refreshed-token");

        // Second recover with old stamp must NOT refresh again (stale stamp).
        let recovery2 = tm.recover_unauthorized(&binding, &sent, 401).await.unwrap();
        assert_eq!(recovery2, UnauthorizedRecovery::RetryWithCurrentCredential);
        assert_eq!(
            refresh_count.load(Ordering::SeqCst),
            1,
            "second recover with stale stamp must not re-refresh (no loop)"
        );
    }

    /// Account fingerprint mismatch → reauth required, no refresh loop.
    #[tokio::test]
    async fn account_mismatch_on_401_requires_reauth_no_refresh() {
        let store = Arc::new(EphemeralCredentialStore::new());
        let refresh_count = Arc::new(AtomicU32::new(0));
        let provider = Arc::new(CountingProvider {
            id: ProviderId::new_unchecked("mock"),
            refresh_count: refresh_count.clone(),
        });
        let mut registry = ProviderRegistry::new();
        registry.register(provider).unwrap();
        let registry = Arc::new(registry);

        let provider_id = ProviderId::new_unchecked("mock");
        let (_key, metadata, secret) = make_credential(&provider_id, false);
        store
            .create(NewCredentialRecord {
                provider: provider_id.clone(),
                requested_alias: Some("test".into()),
                account: ProviderAccountInfo::default(),
                secret,
                expires_at: metadata.expires_at,
                backend: xai_grok_auth::SecretBackendKind::Ephemeral,
            })
            .await
            .unwrap();

        let accounts = store.list_accounts(&provider_id).await.unwrap();
        let actual_key = accounts[0].key.clone();
        let wrong_fp =
            fingerprint::fingerprint_from_parts("mock", "https://auth.openai.com", "other", "acct");
        let binding = CredentialBinding {
            key: actual_key.clone(),
            expected_account: wrong_fp.clone(),
        };
        let sent = SentCredentialStamp {
            key: actual_key,
            generation: 1,
            account_fingerprint: wrong_fp,
        };

        let tm = TokenManager::new(store, registry);
        let recovery = tm.recover_unauthorized(&binding, &sent, 401).await.unwrap();
        assert_eq!(recovery, UnauthorizedRecovery::ReauthenticationRequired);
        assert_eq!(refresh_count.load(Ordering::SeqCst), 0);

        // Permanent: second call still reauth, still no refresh.
        let recovery2 = tm.recover_unauthorized(&binding, &sent, 401).await.unwrap();
        assert_eq!(recovery2, UnauthorizedRecovery::ReauthenticationRequired);
        assert_eq!(refresh_count.load(Ordering::SeqCst), 0);
    }
}
