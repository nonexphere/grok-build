//! Multi-provider authentication implementation (goblin fork canonical).
//!
//! This crate holds the fork-friendly, fully-tested multi-provider auth
//! control plane: credential store backends, token manager, Codex/xAI
//! providers, login coordinator, request auth resolver, CLI helpers, and
//! kill switches. It depends on [`xai_grok_auth`] for trait definitions
//! and type aliases but does not modify them.
//!
//! The shell's `auth::store` modules are thin re-exports of this crate's
//! [`store`] module (see `xai-grok-shell/src/auth/store/mod.rs`).

pub mod cli;
pub mod fingerprint;
pub mod kill_switch; // product gates + env kill switches (B5/D10)
pub mod login_coordinator;
pub mod model_binding;
pub mod provider_model_key;
pub mod providers;
pub mod registry;
pub mod request_auth;
pub mod request_stamp;
pub mod session_pin;
pub mod store;
pub mod token_manager;
pub mod token_resolve;

pub use session_pin::{
    session_pin_decision, session_pin_decision_for_turn, SessionPinDecision,
};

// Re-export the key control-plane types from xai-grok-auth so downstream
// callers can reach everything from one crate.
pub use xai_grok_auth::{
    AccountFingerprint, AccountKind, AccountPlan, AuthFailureClass, AuthFailureResponse,
    AuthProvider, CompareAndSwapError, CredentialBinding, CredentialId, CredentialKey,
    CredentialLockGuard, CredentialLockPurpose, CredentialMetadata, CredentialSecret,
    CredentialStatus, CredentialStore, CredentialUpdate, InvalidProviderId, LoginCompletion,
    LoginFlowId, LoginInput, LoginRequest, LoginStart, LoginTransport, LogoutOutcome,
    LogoutRequest, ModelBinding, ModelCatalog, ModelListRequest, NewCredentialRecord,
    ProviderAccountInfo, ProviderCapabilities, ProviderCredentialUpdate, ProviderDescriptor,
    ProviderEndpointKind, ProviderEndpointRequest, ProviderError, ProviderId,
    ProviderRegistrationError, ProviderRequestAuth, RefreshRequest, RequestAuthContext,
    RequestKind, SecretBackendKind, SecretString, SentCredentialStamp, StoreError,
    StoredCredential, TokenRequest, TokenResolution, TokenUseReason, UnauthorizedRecovery,
    ValidToken,
};
