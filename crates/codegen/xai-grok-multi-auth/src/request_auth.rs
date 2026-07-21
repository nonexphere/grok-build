//! Request auth resolver: produces headers + SentCredentialStamp from
//! a ModelBinding / CredentialBinding.

use std::sync::Arc;

use http::HeaderMap;
use thiserror::Error;

use xai_grok_auth::{
    CredentialBinding, CredentialKey, CredentialStore, ModelBinding, ProviderRegistry,
    SentCredentialStamp,
};

use crate::fingerprint;
use crate::token_manager::{TokenManager, TokenManagerError};

/// Errors from the request auth resolver.
#[derive(Debug, Error)]
pub enum RequestAuthError {
    #[error("credential not found")]
    NotFound,

    #[error("token manager error: {0}")]
    TokenManager(#[from] TokenManagerError),

    #[error("provider not registered: {0}")]
    ProviderNotRegistered(String),

    #[error("provider error: {0}")]
    Provider(#[from] xai_grok_auth::ProviderError),

    #[error("store error: {0}")]
    Store(#[from] xai_grok_auth::StoreError),
}

/// The resolved auth for a request: headers + stamp.
#[derive(Debug)]
pub struct ResolvedRequestAuth {
    pub headers: HeaderMap,
    pub stamp: SentCredentialStamp,
}

/// Request auth resolver that uses the TokenManager and provider's
/// `build_request_auth` to produce headers.
pub struct RequestAuthResolver {
    store: Arc<dyn CredentialStore>,
    registry: Arc<ProviderRegistry>,
    token_manager: Arc<TokenManager>,
    issuer: String,
}

impl std::fmt::Debug for RequestAuthResolver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RequestAuthResolver")
            .finish_non_exhaustive()
    }
}

impl RequestAuthResolver {
    pub fn new(
        store: Arc<dyn CredentialStore>,
        registry: Arc<ProviderRegistry>,
        token_manager: Arc<TokenManager>,
    ) -> Self {
        Self {
            store,
            registry,
            token_manager,
            issuer: "https://auth.openai.com".to_string(),
        }
    }

    /// Resolve request auth from a model binding.
    pub async fn resolve(
        &self,
        binding: &ModelBinding,
        default_credential: Option<&CredentialKey>,
    ) -> Result<ResolvedRequestAuth, RequestAuthError> {
        // Resolve the credential key.
        let key = binding
            .credential
            .map(|id| CredentialKey {
                provider: binding.provider.clone(),
                credential_id: id,
            })
            .or_else(|| default_credential.cloned())
            .ok_or(RequestAuthError::NotFound)?;

        // Load the credential.
        let credential = self
            .store
            .load(&key)
            .await?
            .ok_or(RequestAuthError::NotFound)?;

        // Compute fingerprint.
        let fp = fingerprint::compute_fingerprint(
            &key.provider,
            &self.issuer,
            &credential.metadata.account,
        );

        // Get a valid token.
        let cred_binding = CredentialBinding {
            key: key.clone(),
            expected_account: fp,
        };
        let token = self
            .token_manager
            .get_valid_token(&cred_binding, xai_grok_auth::TokenUseReason::Inference)
            .await?;

        // Build request headers via the provider.
        let provider = self
            .registry
            .get(&binding.provider)
            .map_err(|e| RequestAuthError::ProviderNotRegistered(e.to_string()))?;

        // Reload the credential (it may have been refreshed).
        let credential = self
            .store
            .load(&key)
            .await?
            .ok_or(RequestAuthError::NotFound)?;

        let endpoint = url::Url::parse("https://chatgpt.com/backend-api/codex/responses").unwrap();
        let method = http::Method::POST;
        let request_auth = provider.build_request_auth(xai_grok_auth::RequestAuthContext {
            endpoint: &endpoint,
            method: &method,
            credential: Some(&credential),
            request_kind: xai_grok_auth::RequestKind::Inference,
        })?;

        Ok(ResolvedRequestAuth {
            headers: request_auth.headers,
            stamp: SentCredentialStamp {
                key,
                generation: token.generation,
                account_fingerprint: token.account_fingerprint,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry;
    use crate::store::ephemeral::EphemeralCredentialStore;
    use crate::token_manager::TokenManager;
    use chrono::Utc;
    use std::collections::BTreeMap;
    use xai_grok_auth::{
        CredentialSecret, NewCredentialRecord, ProviderAccountInfo, ProviderId, SecretBackendKind,
        SecretString,
    };

    /// Test 15: request auth headers Authorization + ChatGPT-Account-ID.
    #[tokio::test]
    #[serial_test::serial]
    async fn request_auth_headers_authorization_and_account_id() {
        let store = Arc::new(EphemeralCredentialStore::new());

        // Create a Codex credential with account metadata.
        let provider_id = ProviderId::new_unchecked("codex");
        let mut account = ProviderAccountInfo::default();
        account
            .metadata
            .insert("chatgpt_account_id".to_string(), "acct-123".to_string());

        let metadata = store
            .create(NewCredentialRecord {
                provider: provider_id.clone(),
                requested_alias: Some("personal".to_string()),
                account: account.clone(),
                secret: CredentialSecret {
                    access_token: SecretString::from_str("test-access-token"),
                    refresh_token: Some(SecretString::from_str("test-refresh-token")),
                    id_token: None,
                    fields: BTreeMap::new(),
                },
                expires_at: Some(Utc::now() + chrono::Duration::hours(1)),
                backend: SecretBackendKind::Ephemeral,
            })
            .await
            .unwrap();

        let key = metadata.key.clone();

        // Build registry with Codex provider.
        let registry = Arc::new(registry::build_registry(false));
        let token_manager = Arc::new(TokenManager::with_issuer(
            store.clone(),
            registry.clone(),
            "https://auth.openai.com".to_string(),
        ));
        let resolver = RequestAuthResolver::new(store, registry, token_manager);

        let binding = ModelBinding::new(
            provider_id.clone(),
            Some(key.credential_id),
            "codex-model".to_string(),
        );

        let resolved = resolver.resolve(&binding, None).await.unwrap();

        // Authorization header must be present.
        assert_eq!(
            resolved
                .headers
                .get("authorization")
                .unwrap()
                .to_str()
                .unwrap(),
            "Bearer test-access-token"
        );

        // ChatGPT-Account-ID header must be present.
        assert_eq!(
            resolved
                .headers
                .get("chatgpt-account-id")
                .unwrap()
                .to_str()
                .unwrap(),
            "acct-123"
        );

        // Stamp must have the correct key.
        assert_eq!(resolved.stamp.key, key);
    }
}
