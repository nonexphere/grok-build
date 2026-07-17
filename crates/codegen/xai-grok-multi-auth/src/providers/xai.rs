//! Minimal xAI auth provider stub.
//!
//! The xAI provider's full login flow is handled by the legacy
//! `AuthManager` in `xai-grok-shell`. This stub exists so the
//! `ProviderRegistry` can list xAI as a known provider. Login methods
//! return `ProviderError::InvalidConfig` pointing to the legacy path.

use async_trait::async_trait;
use url::Url;

use xai_grok_auth::{
    AuthFailureClass, AuthFailureResponse, AuthProvider, CredentialMetadata, LoginCompletion,
    LoginFlowId, LoginInput, LoginRequest, LoginStart, LogoutOutcome, LogoutRequest, ModelCatalog,
    ModelListRequest, ProviderAccountInfo, ProviderCapabilities, ProviderCredentialUpdate,
    ProviderDescriptor, ProviderEndpointRequest, ProviderError,
    ProviderId, ProviderRequestAuth, RefreshRequest, RequestAuthContext, StoredCredential,
    TokenRequest, TokenResolution,
};

/// The xAI provider ID.
pub fn xai_provider_id() -> ProviderId {
    ProviderId::new_unchecked("xai")
}

/// Minimal xAI auth provider for registry listing.
#[derive(Debug)]
pub struct XaiAuthProvider {
    id: ProviderId,
}

impl XaiAuthProvider {
    pub fn new() -> Self {
        Self {
            id: xai_provider_id(),
        }
    }
}

impl Default for XaiAuthProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AuthProvider for XaiAuthProvider {
    fn id(&self) -> &ProviderId {
        &self.id
    }

    fn descriptor(&self) -> ProviderDescriptor {
        // Honest capabilities: lifecycle methods on this type are stubs that
        // defer to legacy AuthManager. Advertising PKCE/device/refresh here
        // was review B4 (registered stub). Keep empty until a real adapter
        // implements those methods; CLI still routes bare `login` to legacy xAI.
        let caps = ProviderCapabilities::empty();

        ProviderDescriptor {
            id: self.id.clone(),
            display_name: "Grok (xAI)".to_string(),
            short_name: "xai".to_string(),
            icon_key: Some("xai".to_string()),
            capabilities: caps,
            default_priority: 0,
        }
    }

    fn validate_config(&self) -> Result<(), ProviderError> {
        Ok(())
    }

    async fn start_login(&self, _request: LoginRequest) -> Result<LoginStart, ProviderError> {
        Err(ProviderError::InvalidConfig(
            "xAI login is handled by the legacy AuthManager. Use `grok login` without --provider."
                .into(),
        ))
    }

    async fn complete_login(
        &self,
        _flow_id: LoginFlowId,
        _input: LoginInput,
    ) -> Result<LoginCompletion, ProviderError> {
        Err(ProviderError::InvalidConfig(
            "xAI login is handled by the legacy AuthManager.".into(),
        ))
    }

    async fn cancel_login(&self, _flow_id: LoginFlowId) -> Result<(), ProviderError> {
        Ok(())
    }

    async fn refresh(
        &self,
        _request: RefreshRequest<'_>,
    ) -> Result<ProviderCredentialUpdate, ProviderError> {
        Err(ProviderError::InvalidConfig(
            "xAI refresh is handled by the legacy AuthManager.".into(),
        ))
    }

    async fn get_valid_token(
        &self,
        _request: TokenRequest<'_>,
    ) -> Result<TokenResolution, ProviderError> {
        Err(ProviderError::InvalidConfig(
            "xAI token resolution is handled by the legacy AuthManager.".into(),
        ))
    }

    async fn logout(
        &self,
        _request: LogoutRequest<'_>,
    ) -> Result<LogoutOutcome, ProviderError> {
        Err(ProviderError::InvalidConfig(
            "xAI logout is handled by the legacy AuthManager.".into(),
        ))
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
        Ok(ModelCatalog {
            models: Vec::new(),
            etag: None,
            fetched_at: chrono::Utc::now(),
            source: xai_grok_auth::ModelCatalogSource::Unknown,
            is_stale: false,
        })
    }

    fn resolve_endpoint(
        &self,
        _request: ProviderEndpointRequest<'_>,
    ) -> Result<Url, ProviderError> {
        Err(ProviderError::InvalidConfig(
            "xAI endpoint resolution is handled by the legacy AuthManager.".into(),
        ))
    }

    fn build_request_auth(
        &self,
        _request: RequestAuthContext<'_>,
    ) -> Result<ProviderRequestAuth, ProviderError> {
        Err(ProviderError::InvalidConfig(
            "xAI request auth is handled by the legacy AuthManager.".into(),
        ))
    }

    fn classify_auth_failure(&self, _response: &AuthFailureResponse) -> AuthFailureClass {
        AuthFailureClass::NotAuthentication
    }

    fn supports_credential(&self, metadata: &CredentialMetadata) -> bool {
        metadata.key.provider == self.id
    }
}
