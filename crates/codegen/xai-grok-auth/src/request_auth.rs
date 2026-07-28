//! Request-time auth supporting types: token use reasons, valid-token
//! snapshots, request auth context, and auth-failure classification. These
//! types appear in `AuthProvider` method signatures. Mirrors `task.md`
//! Appendix A.

use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, Utc};
use http::{HeaderMap, Method, StatusCode};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::credential::StoredCredential;
use crate::secret::SecretString;
use crate::types::{AccountFingerprint, CredentialKey, ProviderAccountInfo};

/// Why a token is being requested. Drives refresh policy and telemetry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenUseReason {
    Inference,
    ModelDiscovery,
    AccountInfo,
    ProactiveRefresh,
    UnauthorizedRecovery,
}

/// A request for a valid token, passed to `AuthProvider::get_valid_token`.
#[derive(Debug)]
pub struct TokenRequest<'a> {
    pub credential: &'a StoredCredential,
    pub reason: TokenUseReason,
    pub now: DateTime<Utc>,
    pub early_refresh_window: chrono::Duration,
}

/// The result of `get_valid_token`: a usable token plus an optional update
/// to persist.
#[derive(Debug)]
pub struct TokenResolution {
    pub token: SecretString,
    pub expires_at: Option<DateTime<Utc>>,
    pub update: Option<ProviderCredentialUpdate>,
}

/// A partial update to a provider credential produced by a refresh.
#[derive(Debug)]
pub struct ProviderCredentialUpdate {
    pub account: Option<ProviderAccountInfo>,
    pub access_token: Option<SecretString>,
    pub refresh_token: Option<SecretString>,
    pub id_token: Option<SecretString>,
    pub fields: BTreeMap<String, SecretString>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// A request to refresh a credential, passed to `AuthProvider::refresh`.
#[derive(Debug)]
pub struct RefreshRequest<'a> {
    pub credential: &'a StoredCredential,
    pub reason: TokenUseReason,
}

/// A request to log out a credential, passed to `AuthProvider::logout`.
#[derive(Debug)]
pub struct LogoutRequest<'a> {
    pub credential: &'a StoredCredential,
    pub revoke: bool,
}

/// The outcome of `AuthProvider::logout`.
#[derive(Debug)]
pub struct LogoutOutcome {
    pub remote_revoked: bool,
    pub warning: Option<String>,
}

/// A request to list a provider's models, passed to `AuthProvider::list_models`.
#[derive(Debug, Clone)]
pub struct ModelListRequest<'a> {
    pub credential: Option<&'a StoredCredential>,
    pub client_version: &'a str,
    pub etag: Option<&'a str>,
}

/// A single model advertised by a provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderModel {
    pub id: String,
    pub display_name: String,
    pub description: Option<String>,
    pub context_window: Option<u64>,
    pub priority: i32,
    pub capabilities: BTreeSet<String>,
    pub raw_metadata: serde_json::Value,
}

/// Provenance of a [`ModelCatalog`] payload (M7 / AUD-010).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ModelCatalogSource {
    /// Not classified (legacy constructors).
    #[default]
    Unknown,
    /// Fresh network fetch.
    Network,
    /// Served from disk within TTL.
    FreshDisk,
    /// Served from disk past TTL after a transient fetch error.
    StaleDisk,
    /// Built-in fallback when no cache exists.
    Bundled,
    /// Auth/identity failure — must not be treated as a healthy catalog.
    AuthFailure,
}

/// A fetched catalog of provider models.
#[derive(Debug)]
pub struct ModelCatalog {
    pub models: Vec<ProviderModel>,
    pub etag: Option<String>,
    pub fetched_at: DateTime<Utc>,
    /// Where this catalog came from (AUD-010). Defaults to [`ModelCatalogSource::Unknown`].
    pub source: ModelCatalogSource,
    /// True when served past freshness TTL (stale-disk path).
    pub is_stale: bool,
}

/// Which kind of provider endpoint is being resolved.
#[derive(Debug, Clone)]
pub enum ProviderEndpointKind {
    Inference,
    Models,
    Account,
}

/// A request to resolve a provider endpoint URL.
#[derive(Debug, Clone)]
pub struct ProviderEndpointRequest<'a> {
    pub kind: ProviderEndpointKind,
    pub credential: Option<&'a StoredCredential>,
}

/// The kind of outbound request, used to pick the right auth strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestKind {
    Inference,
    ModelList,
    AccountInfo,
}

/// Context for building per-request auth headers.
#[derive(Debug, Clone)]
pub struct RequestAuthContext<'a> {
    pub endpoint: &'a Url,
    pub method: &'a Method,
    pub credential: Option<&'a StoredCredential>,
    pub request_kind: RequestKind,
}

/// The headers a provider wants attached to a request.
#[derive(Debug)]
pub struct ProviderRequestAuth {
    pub headers: HeaderMap,
}

/// A captured auth-failure response, classified by the provider.
#[derive(Debug)]
pub struct AuthFailureResponse {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub provider_error_code: Option<String>,
    pub provider_error_message: Option<String>,
}

/// How a 401/403-class failure should be handled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthFailureClass {
    NotAuthentication,
    Refreshable,
    ReauthenticationRequired,
    PermissionDenied,
    AccountMismatch,
    Transient,
}

/// A binding that pins a credential to an expected account identity.
#[derive(Debug, Clone)]
pub struct CredentialBinding {
    pub key: CredentialKey,
    pub expected_account: AccountFingerprint,
}

/// A validated token ready to attach to a request, with the generation and
/// fingerprint it was derived from.
#[derive(Debug, Clone)]
pub struct ValidToken {
    pub access_token: SecretString,
    pub expires_at: Option<DateTime<Utc>>,
    pub generation: u64,
    pub account_fingerprint: AccountFingerprint,
}

/// A record of the credential stamp sent on a request, used by
/// unauthorized-recovery to detect generation drift.
#[derive(Debug, Clone)]
pub struct SentCredentialStamp {
    pub key: CredentialKey,
    pub generation: u64,
    pub account_fingerprint: AccountFingerprint,
}

/// The recommended recovery action for an unauthorized response.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnauthorizedRecovery {
    RetryWithCurrentCredential,
    RetryAfterRefresh,
    ReauthenticationRequired,
    NotAuthenticationFailure,
}
