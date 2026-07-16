//! Provider-level errors raised by `AuthProvider` implementations and the
//! `ProviderRegistry`. Mirrors `task.md` Appendix A.

use thiserror::Error;

/// Errors produced by an `AuthProvider` while validating configuration,
/// running a login flow, refreshing tokens, discovering models, or building
/// request auth. These are control-plane errors distinct from the
/// low-level store errors in [`crate::credential`].
#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("provider is disabled")]
    Disabled,

    #[error("provider configuration is invalid: {0}")]
    InvalidConfig(String),

    #[error("login was denied")]
    LoginDenied,

    #[error("login flow expired")]
    LoginExpired,

    #[error("callback validation failed")]
    InvalidCallback,

    #[error("token exchange failed: {0}")]
    TokenExchange(String),

    #[error("refresh failed: {0}")]
    Refresh(String),

    #[error("account identity changed")]
    AccountMismatch,

    #[error("reauthentication required: {0}")]
    ReauthenticationRequired(String),

    #[error("model discovery failed: {0}")]
    ModelDiscovery(String),

    #[error("provider transport failed: {0}")]
    Transport(String),
}

/// Errors raised while registering a provider with the `ProviderRegistry`.
#[derive(Debug, Error)]
pub enum ProviderRegistrationError {
    #[error("provider is already registered: {0:?}")]
    Duplicate(crate::types::ProviderId),

    #[error("provider is not registered: {0:?}")]
    Unknown(crate::types::ProviderId),

    #[error("provider configuration is invalid: {0}")]
    Invalid(ProviderError),
}
