//! Optional re-export facade for the multi-provider credential store.
//!
//! Collects the fork's multi-provider credential-store backends and the
//! relevant `xai_grok_auth` control-plane types behind a single
//! `crate::auth::multi_provider` path so downstream callers do not need to
//! reach into `auth::store::*` and `xai_grok_auth::*` separately.
//!
//! This is a thin re-export layer only — no behavior lives here. Gated
//! behind `native-multi-provider-auth` (default-on in this fork).

pub use super::store::{AutoCredentialStore, EphemeralCredentialStore, FileCredentialStore};

pub use xai_grok_auth::{
    CompareAndSwapError, CredentialId, CredentialKey, CredentialLockGuard, CredentialLockPurpose,
    CredentialMetadata, CredentialSecret, CredentialStore, CredentialUpdate, NewCredentialRecord,
    ProviderAccountInfo, ProviderId, SecretBackendKind, SecretString, StoreError,
};
