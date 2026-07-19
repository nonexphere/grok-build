//! BYOK API-key `AuthProvider` driven by a static [`ByokProviderSpec`].
//!
//! OpenRouter, Groq, and Cloudflare are "bring-your-own-key" verticals: the
//! credential is a static API key supplied by the user, there is no OAuth
//! flow, no refresh token, and no remote revocation. Login is driven directly
//! by [`LoginCoordinator::run_api_key_login`](crate::login_coordinator::LoginCoordinator::run_api_key_login)
//! after the coordinator validates the provider is registered and
//! advertises [`ProviderCapabilities::API_KEY_LOGIN`]. This type therefore
//! does **not** implement `start_login`/`complete_login` for the API-key
//! transport — those return `ProviderError::InvalidConfig` so the coordinator
//! remains the single composition root for API-key persistence.
//!
//! What this provider **does** own:
//!   - `descriptor()` advertises `API_KEY_LOGIN` (and only that);
//!   - `resolve_endpoint()` builds the per-vertical inference / models URL;
//!   - `build_request_auth()` emits the static `Authorization: Bearer <key>`;
//!   - `get_valid_token()` returns the stored key (API keys do not expire
//!     unless the caller recorded an `expires_at`);
//!   - `logout()` is honest: never claims `remote_revoked` for a static key;
//!   - `classify_auth_failure()` maps 401/403/429/5xx.

use url::Url;

use xai_grok_auth::{
    AuthFailureClass, AuthFailureResponse, AuthProvider, CredentialMetadata, LoginCompletion,
    LoginFlowId, LoginInput, LoginRequest, LoginStart, LogoutOutcome, LogoutRequest, ModelCatalog,
    ModelCatalogSource, ModelListRequest, ProviderAccountInfo, ProviderCapabilities,
    ProviderCredentialUpdate, ProviderDescriptor, ProviderEndpointKind, ProviderEndpointRequest,
    ProviderError, ProviderId, ProviderRequestAuth, RefreshRequest, RequestAuthContext,
    StoredCredential, TokenRequest, TokenResolution,
};

use async_trait::async_trait;
use http::header::AUTHORIZATION;
use http::{HeaderMap, HeaderValue};

use super::ByokProviderSpec;

/// BYOK API-key auth provider.
#[derive(Debug)]
pub struct ByokAuthProvider {
    spec: ByokProviderSpec,
    id: ProviderId,
}

impl ByokAuthProvider {
    /// Build a provider from a static BYOK spec.
    pub fn new(spec: ByokProviderSpec) -> Self {
        Self {
            id: ProviderId::new_unchecked(spec.id),
            spec,
        }
    }

    /// The static spec driving this provider.
    pub fn spec(&self) -> &ByokProviderSpec {
        &self.spec
    }
}

#[async_trait]
impl AuthProvider for ByokAuthProvider {
    fn id(&self) -> &ProviderId {
        &self.id
    }

    fn descriptor(&self) -> ProviderDescriptor {
        ProviderDescriptor {
            id: self.id.clone(),
            display_name: self.spec.display_name.to_string(),
            short_name: self.spec.id.to_string(),
            icon_key: None,
            // Honest capabilities: only API-key login. No OAuth, no refresh,
            // no remote revocation, no model discovery through this seam.
            capabilities: ProviderCapabilities::API_KEY_LOGIN,
            default_priority: 50,
        }
    }

    fn validate_config(&self) -> Result<(), ProviderError> {
        if self.spec.id.is_empty() {
            return Err(ProviderError::InvalidConfig(
                "BYOK provider spec id must not be empty".into(),
            ));
        }
        if Url::parse(self.spec.base_url).is_err() {
            return Err(ProviderError::InvalidConfig(format!(
                "BYOK provider `{}` has an invalid base_url",
                self.spec.id
            )));
        }
        Ok(())
    }

    async fn start_login(&self, _request: LoginRequest) -> Result<LoginStart, ProviderError> {
        // API-key login is driven by `LoginCoordinator::run_api_key_login`,
        // not via start/complete. Browser/device are not supported by BYOK.
        Err(ProviderError::InvalidConfig(
            "BYOK providers only support API-key login; use `grok login --provider <id>` \
             with an API key (handled by LoginCoordinator::run_api_key_login)"
                .into(),
        ))
    }

    async fn complete_login(
        &self,
        _flow_id: LoginFlowId,
        _input: LoginInput,
    ) -> Result<LoginCompletion, ProviderError> {
        Err(ProviderError::InvalidConfig(
            "BYOK providers do not use complete_login".into(),
        ))
    }

    async fn cancel_login(&self, _flow_id: LoginFlowId) -> Result<(), ProviderError> {
        Ok(())
    }

    async fn refresh(
        &self,
        _request: RefreshRequest<'_>,
    ) -> Result<ProviderCredentialUpdate, ProviderError> {
        Err(ProviderError::ReauthenticationRequired(
            "BYOK API-key credentials do not support refresh; re-import the API key".into(),
        ))
    }

    async fn get_valid_token(
        &self,
        request: TokenRequest<'_>,
    ) -> Result<TokenResolution, ProviderError> {
        // API keys are static; return the stored access token as-is. We do
        // not consult `expires_at` for proactive refresh because there is no
        // refresh endpoint — the caller must re-import the key on expiry.
        Ok(TokenResolution {
            token: request.credential.secret.access_token.clone(),
            expires_at: request.credential.metadata.expires_at,
            update: None,
        })
    }

    async fn logout(&self, _request: LogoutRequest<'_>) -> Result<LogoutOutcome, ProviderError> {
        // No remote revoke endpoint for a static API key. Local deletion is
        // handled by the store layer. Never claim remote_revoked (audit HIGH).
        Ok(LogoutOutcome {
            remote_revoked: false,
            warning: None,
        })
    }

    async fn get_account_info(
        &self,
        credential: &StoredCredential,
    ) -> Result<ProviderAccountInfo, ProviderError> {
        Ok(credential.metadata.account.clone())
    }

    async fn list_models(
        &self,
        _request: ModelListRequest<'_>,
    ) -> Result<ModelCatalog, ProviderError> {
        // Model discovery for BYOK providers is handled via the shell catalog
        // (TOML config / remote fetch) rather than the AuthProvider seam in
        // this slice. Return an empty catalog with Unknown provenance so the
        // registry listing does not claim a capability it cannot honour.
        Ok(ModelCatalog {
            models: Vec::new(),
            etag: None,
            fetched_at: chrono::Utc::now(),
            source: ModelCatalogSource::Unknown,
            is_stale: false,
        })
    }

    fn resolve_endpoint(&self, request: ProviderEndpointRequest<'_>) -> Result<Url, ProviderError> {
        resolve_byok_endpoint(&self.spec, &request)
    }

    fn build_request_auth(
        &self,
        request: RequestAuthContext<'_>,
    ) -> Result<ProviderRequestAuth, ProviderError> {
        let credential = request.credential.ok_or_else(|| {
            ProviderError::InvalidConfig("BYOK request auth requires a credential".into())
        })?;
        let mut headers = HeaderMap::new();
        let bearer = format!("Bearer {}", credential.secret.access_token.expose());
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&bearer).map_err(|e| {
                ProviderError::InvalidConfig(format!("invalid API key header value: {e}"))
            })?,
        );
        Ok(ProviderRequestAuth { headers })
    }

    fn classify_auth_failure(&self, response: &AuthFailureResponse) -> AuthFailureClass {
        match response.status.as_u16() {
            401 => AuthFailureClass::ReauthenticationRequired,
            403 => AuthFailureClass::PermissionDenied,
            429 => AuthFailureClass::Transient,
            500..=599 => AuthFailureClass::Transient,
            _ => AuthFailureClass::NotAuthentication,
        }
    }

    fn supports_credential(&self, metadata: &CredentialMetadata) -> bool {
        metadata.key.provider == self.id
    }
}

/// Resolve a BYOK inference / models / account endpoint URL for a vertical.
///
/// - OpenRouter and Groq are OpenAI-compatible: `{base}/chat/completions` and
///   `{base}/models`.
/// - Cloudflare Workers AI requires the account id in the path; it is read
///   from the credential's `cloudflare_account_id` metadata, falling back to
///   `provider_account_id`. Resolution fails loudly when the account id is
///   missing so a request never silently targets the wrong account.
pub fn resolve_byok_endpoint(
    spec: &ByokProviderSpec,
    request: &ProviderEndpointRequest<'_>,
) -> Result<Url, ProviderError> {
    let base = Url::parse(spec.base_url)
        .map_err(|e| ProviderError::InvalidConfig(format!("invalid {} base URL: {e}", spec.id)))?;
    match spec.id {
        "cloudflare" => {
            let account_id = request
                .credential
                .and_then(|c| {
                    c.metadata
                        .account
                        .metadata
                        .get("cloudflare_account_id")
                        .cloned()
                })
                .or_else(|| {
                    request
                        .credential
                        .and_then(|c| c.metadata.account.provider_account_id.clone())
                })
                .ok_or_else(|| {
                    ProviderError::InvalidConfig(
                        "Cloudflare BYOK requires account metadata `cloudflare_account_id` \
                         (or provider_account_id) for endpoint resolution"
                            .into(),
                    )
                })?;
            let mut url = base;
            match request.kind {
                ProviderEndpointKind::Inference => {
                    url.set_path(&format!(
                        "/client/v4/accounts/{account_id}/ai/v1/chat/completions"
                    ));
                }
                ProviderEndpointKind::Models => {
                    url.set_path(&format!(
                        "/client/v4/accounts/{account_id}/ai/models/search"
                    ));
                }
                ProviderEndpointKind::Account => {
                    url.set_path(&format!("/client/v4/accounts/{account_id}"));
                }
            }
            Ok(url)
        }
        "openrouter" | "groq" => {
            let mut url = base;
            // Preserve the base path prefix (e.g. `/api/v1`, `/openai/v1`)
            // and append the per-kind suffix.
            let base_path = url.path().trim_end_matches('/').to_string();
            let suffix = match request.kind {
                ProviderEndpointKind::Inference => "/chat/completions",
                ProviderEndpointKind::Models => "/models",
                ProviderEndpointKind::Account => {
                    return Err(ProviderError::InvalidConfig(format!(
                        "{}` BYOK does not expose an account endpoint",
                        spec.id
                    )));
                }
            };
            let full = if base_path.is_empty() {
                suffix.to_string()
            } else {
                format!("{base_path}{suffix}")
            };
            url.set_path(&full);
            Ok(url)
        }
        other => Err(ProviderError::InvalidConfig(format!(
            "unknown BYOK provider spec id: {other}"
        ))),
    }
}
