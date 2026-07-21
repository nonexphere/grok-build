//! The `AuthProvider` control-plane trait, provider capabilities,
//! `ProviderDescriptor`, and the `ProviderRegistry`. This is the high-level
//! seam distinct from the low-level `AuthCredentialProvider` HTTP seam in
//! [`crate::auth_provider`]. Mirrors `task.md` Appendix A.

use std::collections::BTreeMap;
use std::fmt;
use std::sync::Arc;

use async_trait::async_trait;

use crate::credential::{CredentialMetadata, StoredCredential};
use crate::errors::{ProviderError, ProviderRegistrationError};
use crate::login::{LoginCompletion, LoginInput, LoginRequest, LoginStart};
use crate::request_auth::{
    AuthFailureClass, AuthFailureResponse, LogoutOutcome, LogoutRequest, ModelCatalog,
    ModelListRequest, ProviderCredentialUpdate, ProviderEndpointRequest, ProviderRequestAuth,
    RefreshRequest, RequestAuthContext, TokenRequest, TokenResolution,
};
use crate::types::ProviderId;
use url::Url;

/// Capabilities a provider advertises. Implemented as a `u32` bitset with
/// associated constants rather than pulling in the `bitflags` crate (which
/// is not a workspace dependency). Supports the bitwise operators needed for
/// capability checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ProviderCapabilities(u32);

impl ProviderCapabilities {
    pub const BROWSER_PKCE: Self = Self(1 << 0);
    pub const DEVICE_CODE: Self = Self(1 << 1);
    pub const REFRESH_TOKEN: Self = Self(1 << 2);
    pub const TOKEN_REVOCATION: Self = Self(1 << 3);
    pub const MULTI_ACCOUNT: Self = Self(1 << 4);
    pub const MODEL_DISCOVERY: Self = Self(1 << 5);
    pub const ACCOUNT_INFO: Self = Self(1 << 6);
    pub const WORKSPACE_ACCOUNTS: Self = Self(1 << 7);
    pub const API_KEY_LOGIN: Self = Self(1 << 8);
    pub const ENTERPRISE_SSO: Self = Self(1 << 9);

    /// Empty capability set.
    pub const fn empty() -> Self {
        Self(0)
    }

    /// Whether the set is empty.
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// The raw bits.
    pub const fn bits(self) -> u32 {
        self.0
    }

    /// Construct from raw bits.
    pub const fn from_bits(bits: u32) -> Self {
        Self(bits)
    }

    /// Whether `self` contains all of `other`.
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    /// Whether `self` intersects `other`.
    pub const fn intersects(self, other: Self) -> bool {
        (self.0 & other.0) != 0
    }

    /// Set union.
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Set intersection.
    pub const fn intersection(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }

    /// Set difference.
    pub const fn difference(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }

    /// Insert `other` into this set in place.
    pub fn insert(&mut self, other: Self) {
        self.0 |= other.0;
    }

    /// Remove `other` from this set in place.
    pub fn remove(&mut self, other: Self) {
        self.0 &= !other.0;
    }
}

impl std::ops::BitOr for ProviderCapabilities {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        self.union(rhs)
    }
}

impl std::ops::BitOrAssign for ProviderCapabilities {
    fn bitor_assign(&mut self, rhs: Self) {
        self.insert(rhs);
    }
}

impl std::ops::BitAnd for ProviderCapabilities {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        self.intersection(rhs)
    }
}

impl std::ops::Sub for ProviderCapabilities {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        self.difference(rhs)
    }
}

impl std::ops::Not for ProviderCapabilities {
    type Output = Self;
    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

/// Static description of a registered provider.
#[derive(Debug, Clone)]
pub struct ProviderDescriptor {
    pub id: ProviderId,
    pub display_name: String,
    pub short_name: String,
    pub icon_key: Option<String>,
    pub capabilities: ProviderCapabilities,
    pub default_priority: i32,
}

/// The control-plane trait implemented by each auth provider (e.g. xAI,
/// Codex). Object-safe so callers can hold `Arc<dyn AuthProvider>`.
///
/// This is deliberately named `AuthProvider` to distinguish it from the
/// low-level HTTP seam [`crate::auth_provider::AuthCredentialProvider`],
/// which remains unchanged.
#[async_trait]
pub trait AuthProvider: fmt::Debug + Send + Sync {
    fn id(&self) -> &ProviderId;

    fn descriptor(&self) -> ProviderDescriptor;

    fn validate_config(&self) -> Result<(), ProviderError>;

    async fn start_login(&self, request: LoginRequest) -> Result<LoginStart, ProviderError>;

    async fn complete_login(
        &self,
        flow_id: crate::login::LoginFlowId,
        input: LoginInput,
    ) -> Result<LoginCompletion, ProviderError>;

    async fn cancel_login(&self, flow_id: crate::login::LoginFlowId) -> Result<(), ProviderError>;

    async fn refresh(
        &self,
        request: RefreshRequest<'_>,
    ) -> Result<ProviderCredentialUpdate, ProviderError>;

    async fn get_valid_token(
        &self,
        request: TokenRequest<'_>,
    ) -> Result<TokenResolution, ProviderError>;

    async fn logout(&self, request: LogoutRequest<'_>) -> Result<LogoutOutcome, ProviderError>;

    async fn get_account_info(
        &self,
        credential: &StoredCredential,
    ) -> Result<crate::types::ProviderAccountInfo, ProviderError>;

    async fn list_models(
        &self,
        request: ModelListRequest<'_>,
    ) -> Result<ModelCatalog, ProviderError>;

    fn resolve_endpoint(&self, request: ProviderEndpointRequest<'_>) -> Result<Url, ProviderError>;

    fn build_request_auth(
        &self,
        request: RequestAuthContext<'_>,
    ) -> Result<ProviderRequestAuth, ProviderError>;

    fn classify_auth_failure(&self, response: &AuthFailureResponse) -> AuthFailureClass;

    fn supports_credential(&self, metadata: &CredentialMetadata) -> bool;

    fn redact_error(&self, error: ProviderError) -> ProviderError {
        error
    }
}

/// The registry of registered providers. Held by the control plane and used
/// to dispatch per-provider operations.
#[derive(Debug, Default)]
pub struct ProviderRegistry {
    providers: BTreeMap<ProviderId, Arc<dyn AuthProvider>>,
}

impl ProviderRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            providers: BTreeMap::new(),
        }
    }

    /// Register a provider. Returns `Duplicate` if a provider with the same
    /// id is already registered, or `Invalid` if the provider's config fails
    /// validation.
    pub fn register(
        &mut self,
        provider: Arc<dyn AuthProvider>,
    ) -> Result<(), ProviderRegistrationError> {
        let id = provider.id().clone();

        if self.providers.contains_key(&id) {
            return Err(ProviderRegistrationError::Duplicate(id));
        }

        provider
            .validate_config()
            .map_err(ProviderRegistrationError::Invalid)?;

        self.providers.insert(id, provider);
        Ok(())
    }

    /// Look up a registered provider by id.
    pub fn get(&self, id: &ProviderId) -> Result<Arc<dyn AuthProvider>, ProviderRegistrationError> {
        self.providers
            .get(id)
            .cloned()
            .ok_or_else(|| ProviderRegistrationError::Unknown(id.clone()))
    }

    /// List descriptors for all registered providers.
    pub fn list(&self) -> Vec<ProviderDescriptor> {
        self.providers
            .values()
            .map(|provider| provider.descriptor())
            .collect()
    }

    /// Number of registered providers.
    pub fn len(&self) -> usize {
        self.providers.len()
    }

    /// Whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.providers.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::login::LoginFlowId;
    use crate::types::ProviderAccountInfo;
    use chrono::Utc;

    /// A minimal mock provider used to exercise object-safety and the
    /// registry. All async methods return `ProviderError::Disabled` so the
    /// trait is fully implemented without real I/O.
    #[derive(Debug)]
    struct MockProvider {
        id: ProviderId,
        descriptor: ProviderDescriptor,
    }

    impl MockProvider {
        fn new(id: &str) -> Self {
            let id = ProviderId::new_unchecked(id);
            Self {
                id: id.clone(),
                descriptor: ProviderDescriptor {
                    id: id.clone(),
                    display_name: id.as_str().to_owned(),
                    short_name: id.as_str().to_owned(),
                    icon_key: None,
                    capabilities: ProviderCapabilities::empty(),
                    default_priority: 0,
                },
            }
        }
    }

    #[async_trait]
    impl AuthProvider for MockProvider {
        fn id(&self) -> &ProviderId {
            &self.id
        }

        fn descriptor(&self) -> ProviderDescriptor {
            self.descriptor.clone()
        }

        fn validate_config(&self) -> Result<(), ProviderError> {
            Ok(())
        }

        async fn start_login(&self, _request: LoginRequest) -> Result<LoginStart, ProviderError> {
            Err(ProviderError::Disabled)
        }

        async fn complete_login(
            &self,
            _flow_id: LoginFlowId,
            _input: LoginInput,
        ) -> Result<LoginCompletion, ProviderError> {
            Err(ProviderError::Disabled)
        }

        async fn cancel_login(&self, _flow_id: LoginFlowId) -> Result<(), ProviderError> {
            Err(ProviderError::Disabled)
        }

        async fn refresh(
            &self,
            _request: crate::request_auth::RefreshRequest<'_>,
        ) -> Result<ProviderCredentialUpdate, ProviderError> {
            Err(ProviderError::Disabled)
        }

        async fn get_valid_token(
            &self,
            _request: TokenRequest<'_>,
        ) -> Result<TokenResolution, ProviderError> {
            Err(ProviderError::Disabled)
        }

        async fn logout(
            &self,
            _request: LogoutRequest<'_>,
        ) -> Result<LogoutOutcome, ProviderError> {
            Err(ProviderError::Disabled)
        }

        async fn get_account_info(
            &self,
            _credential: &StoredCredential,
        ) -> Result<ProviderAccountInfo, ProviderError> {
            Err(ProviderError::Disabled)
        }

        async fn list_models(
            &self,
            _request: ModelListRequest<'_>,
        ) -> Result<ModelCatalog, ProviderError> {
            Err(ProviderError::Disabled)
        }

        fn resolve_endpoint(
            &self,
            _request: ProviderEndpointRequest<'_>,
        ) -> Result<Url, ProviderError> {
            Err(ProviderError::Disabled)
        }

        fn build_request_auth(
            &self,
            _request: RequestAuthContext<'_>,
        ) -> Result<ProviderRequestAuth, ProviderError> {
            Err(ProviderError::Disabled)
        }

        fn classify_auth_failure(&self, _response: &AuthFailureResponse) -> AuthFailureClass {
            AuthFailureClass::NotAuthentication
        }

        fn supports_credential(&self, _metadata: &CredentialMetadata) -> bool {
            false
        }
    }

    #[test]
    fn duplicate_registration_fails() {
        let mut registry = ProviderRegistry::new();
        let p: Arc<dyn AuthProvider> = Arc::new(MockProvider::new("xai"));
        registry.register(p).unwrap();
        let p2: Arc<dyn AuthProvider> = Arc::new(MockProvider::new("xai"));
        let err = registry.register(p2).unwrap_err();
        assert!(matches!(err, ProviderRegistrationError::Duplicate(_)));
    }

    #[test]
    fn unknown_provider_get_fails() {
        let registry = ProviderRegistry::new();
        let missing = ProviderId::new("xai").unwrap();
        let err = registry.get(&missing).unwrap_err();
        assert!(matches!(err, ProviderRegistrationError::Unknown(_)));
    }

    #[test]
    fn arc_dyn_auth_provider_compiles_and_runs() {
        // Object-safety check: build an `Arc<dyn AuthProvider>` and call a
        // synchronous method through it.
        let provider: Arc<dyn AuthProvider> = Arc::new(MockProvider::new("codex"));
        let registry_id = provider.id().clone();
        assert_eq!(registry_id.as_str(), "codex");
        let descriptor = provider.descriptor();
        assert_eq!(descriptor.id.as_str(), "codex");
        assert!(provider.validate_config().is_ok());
        assert!(!provider.supports_credential(&unused_metadata()));
    }

    fn unused_metadata() -> CredentialMetadata {
        use crate::types::{CredentialId, CredentialKey, CredentialStatus, SecretBackendKind};
        let key = CredentialKey {
            provider: ProviderId::new_unchecked("xai"),
            credential_id: CredentialId::new(),
        };
        let now = Utc::now();
        CredentialMetadata {
            schema_version: 1,
            key,
            alias: "test".to_string(),
            account: ProviderAccountInfo::default(),
            created_at: now,
            updated_at: now,
            last_used_at: None,
            expires_at: None,
            status: CredentialStatus::Ready,
            generation: 0,
            secret_backend: SecretBackendKind::File,
        }
    }

    #[test]
    fn registry_list_returns_descriptors() {
        let mut registry = ProviderRegistry::new();
        registry
            .register(Arc::new(MockProvider::new("xai")))
            .unwrap();
        registry
            .register(Arc::new(MockProvider::new("codex")))
            .unwrap();
        let list = registry.list();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn capabilities_bitset_ops() {
        let caps = ProviderCapabilities::BROWSER_PKCE | ProviderCapabilities::REFRESH_TOKEN;
        assert!(caps.contains(ProviderCapabilities::BROWSER_PKCE));
        assert!(caps.contains(ProviderCapabilities::REFRESH_TOKEN));
        assert!(!caps.contains(ProviderCapabilities::DEVICE_CODE));
        assert!(
            caps.intersects(
                ProviderCapabilities::DEVICE_CODE | ProviderCapabilities::REFRESH_TOKEN
            )
        );
        assert_eq!(
            caps.bits(),
            ProviderCapabilities::BROWSER_PKCE.bits() | ProviderCapabilities::REFRESH_TOKEN.bits()
        );
    }
}
