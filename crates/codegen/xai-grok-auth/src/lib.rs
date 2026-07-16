//! Auth dependency-inversion seam shared between `xai-file-utils`
//! (the holder) and `xai-grok-shell` (the implementer). Keeps shell types
//! out of data-collector's import graph while still letting refresh-aware
//! token resolution drive HTTP requests.
//!
//! # Multi-provider control plane (goblin fork)
//!
//! Behind the `native-multi-provider-auth` feature (enabled by default in
//! this fork) this crate also exposes the high-level control-plane seam:
//! the [`AuthProvider`] trait, [`ProviderRegistry`], credential store
//! traits, and supporting types defined in `task.md` Appendix A. These are
//! distinct from the low-level HTTP seam [`AuthCredentialProvider`],
//! which remains unchanged.
//!
//! [`AuthProvider`]: provider::AuthProvider
//! [`ProviderRegistry`]: provider::ProviderRegistry
//! [`AuthCredentialProvider`]: auth_provider::AuthCredentialProvider

pub mod auth_provider;
#[cfg(feature = "middleware")]
pub mod retry_middleware;
pub mod visibility;

// --- Multi-provider control plane (goblin fork) -----------------------------
// Gated behind `native-multi-provider-auth` (a default feature in this fork)
// so default builds of the goblin product always have the control plane
// available, while upstream builds stay close to the original crate.

#[cfg(feature = "native-multi-provider-auth")]
pub mod credential;
#[cfg(feature = "native-multi-provider-auth")]
pub mod errors;
#[cfg(feature = "native-multi-provider-auth")]
pub mod login;
#[cfg(feature = "native-multi-provider-auth")]
pub mod provider;
#[cfg(feature = "native-multi-provider-auth")]
pub mod request_auth;
#[cfg(feature = "native-multi-provider-auth")]
pub mod secret;
#[cfg(feature = "native-multi-provider-auth")]
pub mod types;

// --- Re-exports --------------------------------------------------------------

pub use auth_provider::{AuthCredentialProvider, CredentialSnapshot, StaticAuthCredentialProvider};
#[cfg(feature = "middleware")]
pub use retry_middleware::AuthRetryMiddleware;
pub use visibility::HttpAuth;

#[cfg(feature = "native-multi-provider-auth")]
pub use credential::{
    CompareAndSwapError, CredentialLockGuard, CredentialLockPurpose, CredentialMetadata,
    CredentialSecret, CredentialStore, CredentialUpdate, NewCredentialRecord, StoreError,
    StoredCredential,
};
#[cfg(feature = "native-multi-provider-auth")]
pub use errors::{ProviderError, ProviderRegistrationError};
#[cfg(feature = "native-multi-provider-auth")]
pub use login::{
    AccountPolicy, ClientSurface, LoginCompletion, LoginFlowId, LoginInput, LoginRequest,
    LoginStart, LoginTransport,
};
#[cfg(feature = "native-multi-provider-auth")]
pub use provider::{
    AuthProvider, ProviderCapabilities, ProviderDescriptor, ProviderRegistry,
};
#[cfg(feature = "native-multi-provider-auth")]
pub use request_auth::{
    AuthFailureClass, AuthFailureResponse, CredentialBinding, LogoutOutcome, LogoutRequest,
    ModelCatalog, ModelListRequest, ProviderCredentialUpdate, ProviderEndpointKind,
    ProviderEndpointRequest, ProviderModel, ProviderRequestAuth, RefreshRequest,
    RequestAuthContext, RequestKind, SentCredentialStamp, TokenRequest, TokenResolution,
    TokenUseReason, UnauthorizedRecovery, ValidToken,
};
#[cfg(feature = "native-multi-provider-auth")]
pub use secret::SecretString;
#[cfg(feature = "native-multi-provider-auth")]
pub use types::{
    AccountFingerprint, AccountKind, AccountPlan, CredentialId, CredentialKey, CredentialStatus,
    InvalidProviderId, ModelBinding, ProviderAccountInfo, ProviderId, SecretBackendKind,
};
