//! Codex / ChatGPT auth provider (protocol-baseline.md, task.md §6).
//!
//! Implements native browser PKCE + device-code login without the Codex CLI.
//! The device flow is provider-specific (NOT RFC 8628, D7).

pub mod browser;
pub mod callback;
pub mod claims;
pub mod config;
pub mod device;
pub mod errors;
pub mod model_cache;
pub mod models;
pub mod pkce;
pub mod request_auth;
pub mod token;

use std::collections::BTreeMap;

use async_trait::async_trait;
use chrono::Utc;
use url::Url;

use xai_grok_auth::{
    AuthFailureClass, AuthFailureResponse, AuthProvider, LoginCompletion, LoginFlowId, LoginInput,
    LoginRequest, LoginStart, LoginTransport, LogoutOutcome, LogoutRequest, ModelCatalog,
    ModelListRequest, ProviderAccountInfo, ProviderCapabilities, ProviderCredentialUpdate,
    ProviderDescriptor, ProviderEndpointRequest, ProviderError, ProviderId, ProviderRequestAuth,
    RefreshRequest, RequestAuthContext, StoredCredential, TokenRequest, TokenResolution,
};

pub use config::CodexOAuthConfig;
pub use errors::{RefreshFailureKind, classify_refresh_failure};

use crate::kill_switch;

/// The Codex provider ID.
pub fn codex_provider_id() -> ProviderId {
    ProviderId::new_unchecked("codex")
}

/// In-flight browser flow state (memory-only, never persisted).
#[derive(Clone)]
struct BrowserFlow {
    state: browser::BrowserFlowState,
    config: CodexOAuthConfig,
}

impl std::fmt::Debug for BrowserFlow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BrowserFlow")
            .field("state", &self.state)
            .field("config_client_id", &"<redacted-or-config>")
            .finish()
    }
}

/// In-flight device flow state (memory-only, never persisted).
#[derive(Clone)]
struct DeviceFlow {
    device_auth_id: String,
    user_code: String,
    interval: std::time::Duration,
    config: CodexOAuthConfig,
}

impl std::fmt::Debug for DeviceFlow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Never print user_code / device_auth_id — incidental Debug dumps must
        // not leak the device authorization code shown to the user (audit HIGH).
        f.debug_struct("DeviceFlow")
            .field("device_auth_id", &"<redacted>")
            .field("user_code", &"<redacted>")
            .field("interval", &self.interval)
            .finish()
    }
}

/// Codex / ChatGPT auth provider.
pub struct CodexAuthProvider {
    id: ProviderId,
    config: CodexOAuthConfig,
    http_client: reqwest::Client,
    /// In-flight flows keyed by LoginFlowId (memory-only).
    flows: parking_lot::RwLock<std::collections::HashMap<LoginFlowId, FlowKind>>,
}

impl std::fmt::Debug for CodexAuthProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CodexAuthProvider")
            .field("id", &self.id)
            .field("flows_count", &self.flows.read().len())
            .finish()
    }
}

#[derive(Clone)]
enum FlowKind {
    Browser(BrowserFlow),
    Device(DeviceFlow),
}

impl std::fmt::Debug for FlowKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FlowKind::Browser(b) => f.debug_tuple("Browser").field(b).finish(),
            FlowKind::Device(d) => f.debug_tuple("Device").field(d).finish(),
        }
    }
}

impl CodexAuthProvider {
    /// Create with default config.
    pub fn new() -> Self {
        Self::with_config(CodexOAuthConfig::default())
    }

    /// Create with custom config.
    pub fn with_config(config: CodexOAuthConfig) -> Self {
        Self {
            id: codex_provider_id(),
            config,
            http_client: reqwest::Client::new(),
            flows: parking_lot::RwLock::new(std::collections::HashMap::new()),
        }
    }

    /// Create with a custom HTTP client (for testing with mockito).
    pub fn with_client(config: CodexOAuthConfig, client: reqwest::Client) -> Self {
        Self {
            id: codex_provider_id(),
            config,
            http_client: client,
            flows: parking_lot::RwLock::new(std::collections::HashMap::new()),
        }
    }
}

impl Default for CodexAuthProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AuthProvider for CodexAuthProvider {
    fn id(&self) -> &ProviderId {
        &self.id
    }

    fn descriptor(&self) -> ProviderDescriptor {
        let mut caps = ProviderCapabilities::empty();
        caps |= ProviderCapabilities::BROWSER_PKCE;
        caps |= ProviderCapabilities::DEVICE_CODE;
        caps |= ProviderCapabilities::REFRESH_TOKEN;
        caps |= ProviderCapabilities::TOKEN_REVOCATION;
        caps |= ProviderCapabilities::MULTI_ACCOUNT;
        caps |= ProviderCapabilities::MODEL_DISCOVERY;
        caps |= ProviderCapabilities::ACCOUNT_INFO;

        ProviderDescriptor {
            id: self.id.clone(),
            display_name: "Codex (ChatGPT)".to_string(),
            short_name: "codex".to_string(),
            icon_key: Some("codex".to_string()),
            capabilities: caps,
            default_priority: 10,
        }
    }

    fn validate_config(&self) -> Result<(), ProviderError> {
        if kill_switch::codex_auth_disabled() {
            return Err(ProviderError::Disabled);
        }
        if self.config.client_id.is_empty() {
            return Err(ProviderError::InvalidConfig(
                "client_id must not be empty".into(),
            ));
        }
        Ok(())
    }

    async fn start_login(&self, request: LoginRequest) -> Result<LoginStart, ProviderError> {
        match request.transport {
            LoginTransport::BrowserPkce => {
                if kill_switch::codex_browser_login_disabled() {
                    return Err(ProviderError::Disabled);
                }
                let flow_state = browser::start_browser_flow(&self.config);
                let url = browser::build_authorization_url(
                    &self.config,
                    &flow_state.pkce,
                    &flow_state.state,
                    flow_state.port,
                );
                let flow_id = LoginFlowId::new();
                self.flows.write().insert(
                    flow_id,
                    FlowKind::Browser(BrowserFlow {
                        state: flow_state,
                        config: self.config.clone(),
                    }),
                );
                Ok(LoginStart::Browser {
                    flow_id,
                    authorization_url: url,
                    expires_at: Utc::now() + chrono::Duration::minutes(10),
                })
            }
            LoginTransport::DeviceCode => {
                if kill_switch::codex_device_login_disabled() {
                    return Err(ProviderError::Disabled);
                }
                let resp = device::request_user_code(
                    &self.http_client,
                    &self.config.device_usercode_url(),
                    &self.config.client_id,
                )
                .await
                .map_err(|e| ProviderError::Transport(e.to_string()))?;

                let flow_id = LoginFlowId::new();
                let interval = std::time::Duration::from_secs(resp.interval_secs());
                let verification_uri = self.config.device_verification_url();

                self.flows.write().insert(
                    flow_id,
                    FlowKind::Device(DeviceFlow {
                        device_auth_id: resp.device_auth_id.clone(),
                        user_code: resp.user_code.clone(),
                        interval,
                        config: self.config.clone(),
                    }),
                );

                Ok(LoginStart::Device {
                    flow_id,
                    verification_uri,
                    verification_uri_complete: None,
                    user_code: resp.user_code,
                    expires_at: Utc::now() + chrono::Duration::minutes(15),
                    interval,
                })
            }
            LoginTransport::ApiKey => Err(ProviderError::InvalidConfig(
                "Codex provider does not support API key login".into(),
            )),
        }
    }

    async fn complete_login(
        &self,
        flow_id: LoginFlowId,
        input: LoginInput,
    ) -> Result<LoginCompletion, ProviderError> {
        // Peek (do not remove yet). Device Pending must leave the flow in place
        // so subsequent polls can continue; only success/error removes it.
        let flow_snapshot = {
            let flows = self.flows.read();
            flows
                .get(&flow_id)
                .cloned()
                .ok_or_else(|| ProviderError::InvalidConfig("unknown login flow".into()))?
        };

        match (flow_snapshot, input) {
            (FlowKind::Browser(browser_flow), LoginInput::BrowserCallback { url }) => {
                let params = browser::parse_callback(&url).ok_or(ProviderError::InvalidCallback)?;
                let code = browser::validate_callback(&browser_flow.state.state, &params)
                    .map_err(|_e| ProviderError::InvalidCallback)?;

                // Commit: drop the flow before network exchange so retries
                // cannot reuse a one-shot PKCE verifier.
                self.flows.write().remove(&flow_id);

                let token_resp = token::exchange_authorization_code(
                    &self.http_client,
                    &browser_flow.config.token_url(),
                    &code,
                    &browser_flow
                        .config
                        .redirect_uri_for_port(browser_flow.state.port),
                    &browser_flow.config.client_id,
                    browser_flow.state.pkce.verifier(),
                )
                .await
                .map_err(|e| ProviderError::TokenExchange(e.to_string()))?;

                let claims =
                    claims::parse_id_token_claims(token_resp.id_token.as_deref().unwrap_or(""))
                        .ok();
                let account = claims
                    .as_ref()
                    .map(claims::claims_to_account_info)
                    .unwrap_or_default();

                let secret = token_resp.to_credential_secret();
                let expires_at = claims
                    .as_ref()
                    .and_then(claims::extract_expiration)
                    .or_else(|| {
                        token_resp
                            .expires_in
                            .map(|s| Utc::now() + chrono::Duration::seconds(s as i64))
                    });

                Ok(LoginCompletion::Complete {
                    credential: xai_grok_auth::NewCredentialRecord {
                        provider: self.id.clone(),
                        requested_alias: None,
                        account,
                        secret,
                        expires_at,
                        backend: xai_grok_auth::SecretBackendKind::File,
                    },
                })
            }
            (FlowKind::Device(device_flow), LoginInput::Poll) => {
                let poll = device::poll_device_token(
                    &self.http_client,
                    &device_flow.config.device_token_url(),
                    &device_flow.device_auth_id,
                    &device_flow.user_code,
                )
                .await
                .map_err(|e| ProviderError::Transport(e.to_string()))?;

                match poll {
                    Some(poll_resp) => {
                        // Success path: remove flow, then exchange.
                        self.flows.write().remove(&flow_id);

                        let token_resp = device::complete_device_flow(
                            &self.http_client,
                            &device_flow.config.token_url(),
                            &poll_resp,
                            &device_flow.config.client_id,
                            device::DEVICE_REDIRECT_URI,
                        )
                        .await
                        .map_err(|e| ProviderError::TokenExchange(e.to_string()))?;

                        let claims = claims::parse_id_token_claims(
                            token_resp.id_token.as_deref().unwrap_or(""),
                        )
                        .ok();
                        let account = claims
                            .as_ref()
                            .map(claims::claims_to_account_info)
                            .unwrap_or_default();
                        let secret = token_resp.to_credential_secret();
                        let expires_at = claims
                            .as_ref()
                            .and_then(claims::extract_expiration)
                            .or_else(|| {
                                token_resp
                                    .expires_in
                                    .map(|s| Utc::now() + chrono::Duration::seconds(s as i64))
                            });

                        Ok(LoginCompletion::Complete {
                            credential: xai_grok_auth::NewCredentialRecord {
                                provider: self.id.clone(),
                                requested_alias: None,
                                account,
                                secret,
                                expires_at,
                                backend: xai_grok_auth::SecretBackendKind::File,
                            },
                        })
                    }
                    // CRITICAL: keep the flow so the next Poll can retry.
                    None => Ok(LoginCompletion::Pending {
                        retry_after: device_flow.interval,
                    }),
                }
            }
            _ => Err(ProviderError::InvalidConfig(
                "login input does not match flow type".into(),
            )),
        }
    }

    async fn cancel_login(&self, flow_id: LoginFlowId) -> Result<(), ProviderError> {
        self.flows.write().remove(&flow_id);
        Ok(())
    }

    async fn refresh(
        &self,
        request: RefreshRequest<'_>,
    ) -> Result<ProviderCredentialUpdate, ProviderError> {
        let refresh_token_str = request
            .credential
            .secret
            .refresh_token
            .as_ref()
            .map(|s| s.expose().to_string())
            .ok_or_else(|| {
                ProviderError::ReauthenticationRequired("no refresh token available".into())
            })?;

        let token_resp = token::refresh_token(
            &self.http_client,
            &self.config.token_url(),
            &self.config.client_id,
            &refresh_token_str,
        )
        .await
        .map_err(|e| {
            let status = e.status_code().unwrap_or(0);
            let code = e.error_code();
            let (_kind, provider_err) = classify_refresh_failure(status, code);
            provider_err
        })?;

        // Parse claims if available.
        let claims = token_resp
            .id_token
            .as_deref()
            .and_then(|t| claims::parse_id_token_claims(t).ok());
        let account = claims.as_ref().map(claims::claims_to_account_info);
        let expires_at = claims
            .as_ref()
            .and_then(claims::extract_expiration)
            .or_else(|| {
                token_resp
                    .expires_in
                    .map(|s| Utc::now() + chrono::Duration::seconds(s as i64))
            });

        Ok(ProviderCredentialUpdate {
            account,
            access_token: Some(xai_grok_auth::SecretString::from_str(
                &token_resp.access_token,
            )),
            refresh_token: token_resp
                .refresh_token
                .map(|s| xai_grok_auth::SecretString::from_str(&s)),
            id_token: token_resp
                .id_token
                .map(|s| xai_grok_auth::SecretString::from_str(&s)),
            fields: BTreeMap::new(),
            expires_at,
        })
    }

    async fn get_valid_token(
        &self,
        request: TokenRequest<'_>,
    ) -> Result<TokenResolution, ProviderError> {
        // Return current token if not expiring.
        let now = request.now;
        let early_window = request.early_refresh_window;
        if let Some(expires_at) = request.credential.metadata.expires_at
            && expires_at > now + early_window
        {
            return Ok(TokenResolution {
                token: request.credential.secret.access_token.clone(),
                expires_at: Some(expires_at),
                update: None,
            });
        }

        // Needs refresh.
        let update = self
            .refresh(xai_grok_auth::RefreshRequest {
                credential: request.credential,
                reason: request.reason,
            })
            .await?;

        let token = update
            .access_token
            .clone()
            .unwrap_or_else(|| request.credential.secret.access_token.clone());

        Ok(TokenResolution {
            token,
            expires_at: update.expires_at,
            update: Some(update),
        })
    }

    async fn logout(&self, request: LogoutRequest<'_>) -> Result<LogoutOutcome, ProviderError> {
        let mut remote_revoked = false;
        let mut warning = None;
        if request.revoke {
            let revoke_result = if let Some(ref rt) = request.credential.secret.refresh_token {
                token::revoke_token(
                    &self.http_client,
                    &self.config.revoke_url(),
                    rt.expose(),
                    "refresh_token",
                    &self.config.client_id,
                )
                .await
            } else {
                token::revoke_token(
                    &self.http_client,
                    &self.config.revoke_url(),
                    request.credential.secret.access_token.expose(),
                    "access_token",
                    &self.config.client_id,
                )
                .await
            };
            match revoke_result {
                Ok(()) => remote_revoked = true,
                Err(e) => {
                    // Local delete still proceeds at the store layer; do not claim
                    // remote revoke when upstream failed (audit HIGH).
                    warning = Some(format!(
                        "remote token revoke failed; local credential will still be removed: {e}"
                    ));
                }
            }
        }
        Ok(LogoutOutcome {
            remote_revoked,
            warning,
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
        request: ModelListRequest<'_>,
    ) -> Result<ModelCatalog, ProviderError> {
        let credential = request.credential.ok_or_else(|| {
            ProviderError::ModelDiscovery(
                "Codex model listing requires an authenticated credential".into(),
            )
        })?;
        // Re-enrich account from tokens if ChatGPT-Account-ID was not stored
        // (older logins with broken nested-claim parsing).
        let mut credential = credential.clone();
        credential.metadata.account = claims::enrich_account_from_tokens(
            &credential.metadata.account,
            credential.secret.id_token.as_ref().map(|s| s.expose()),
            Some(credential.secret.access_token.expose()),
        );
        let version = if request.client_version.is_empty() {
            models::DEFAULT_MODELS_CLIENT_VERSION
        } else {
            request.client_version
        };

        // M7/D9: per-credential disk cache + stale/bundled fallback.
        let home = crate::token_resolve::grok_home();
        let cache_path = model_cache::cache_path(&home, credential.metadata.key.credential_id);
        let now = Utc::now();
        let prior_etag = model_cache::load_cache(&cache_path).and_then(|c| {
            if c.is_fresh(now) {
                None // fresh: return without network
            } else {
                c.etag
            }
        });
        if let Some(cached) = model_cache::load_cache(&cache_path)
            && cached.is_fresh(now)
        {
            return Ok(cached.into_model_catalog(xai_grok_auth::ModelCatalogSource::FreshDisk));
        }

        let fetch = models::fetch_codex_models_with_etag(
            &self.http_client,
            &credential,
            version,
            prior_etag.as_deref(),
        )
        .await
        .map_err(|e| e.to_string());
        let (catalog, source) = model_cache::resolve_after_fetch(fetch, &cache_path, now);
        if source == model_cache::CacheSource::AuthFailure {
            return Err(ProviderError::ModelDiscovery(
                "Codex model list failed: auth/identity error (not serving stale catalog)".into(),
            ));
        }
        Ok(catalog)
    }

    fn resolve_endpoint(&self, request: ProviderEndpointRequest<'_>) -> Result<Url, ProviderError> {
        request_auth::resolve_codex_endpoint(&self.config, &request)
    }

    fn build_request_auth(
        &self,
        request: RequestAuthContext<'_>,
    ) -> Result<ProviderRequestAuth, ProviderError> {
        let credential = request.credential.ok_or_else(|| {
            ProviderError::InvalidConfig("no credential for Codex request".into())
        })?;
        let headers = request_auth::build_codex_request_headers(credential)?;
        Ok(ProviderRequestAuth { headers })
    }

    fn classify_auth_failure(&self, response: &AuthFailureResponse) -> AuthFailureClass {
        request_auth::classify_codex_auth_failure(response)
    }

    fn supports_credential(&self, metadata: &xai_grok_auth::CredentialMetadata) -> bool {
        metadata.key.provider == self.id
    }
}

#[cfg(test)]
mod audit_remediation_tests {
    use super::*;
    use chrono::Utc;
    use std::collections::BTreeMap;
    use xai_grok_auth::{
        CredentialId, CredentialKey, CredentialMetadata, CredentialSecret, CredentialStatus,
        LogoutRequest, ProviderAccountInfo, SecretBackendKind, SecretString, StoredCredential,
    };

    #[test]
    fn device_flow_debug_redacts_user_code_and_device_auth_id() {
        let flow = DeviceFlow {
            device_auth_id: "device-auth-SECRET-123".into(),
            user_code: "WXYZ-ABCD".into(),
            interval: std::time::Duration::from_secs(5),
            config: CodexOAuthConfig::default(),
        };
        let dbg = format!("{flow:?}");
        assert!(
            !dbg.contains("WXYZ-ABCD"),
            "user_code must not appear in Debug: {dbg}"
        );
        assert!(
            !dbg.contains("device-auth-SECRET-123"),
            "device_auth_id must not appear in Debug: {dbg}"
        );
        assert!(
            dbg.contains("<redacted>"),
            "expected redaction markers: {dbg}"
        );
    }

    /// Drive the real [`AuthProvider::logout`] path with a mockito revoke 5xx.
    #[tokio::test]
    async fn logout_remote_revoked_false_when_upstream_revoke_fails() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/oauth/revoke")
            .with_status(500)
            .with_body("upstream revoke failed")
            .create_async()
            .await;

        let mut config = CodexOAuthConfig::default();
        config.issuer = url::Url::parse(&server.url()).expect("issuer url");
        let provider = CodexAuthProvider::with_client(config, reqwest::Client::new());

        let now = Utc::now();
        let key = CredentialKey {
            provider: codex_provider_id(),
            credential_id: CredentialId::new(),
        };
        let stored = StoredCredential {
            metadata: CredentialMetadata {
                schema_version: 1,
                key,
                alias: "test".into(),
                account: ProviderAccountInfo::default(),
                created_at: now,
                updated_at: now,
                last_used_at: None,
                expires_at: Some(now + chrono::Duration::hours(1)),
                status: CredentialStatus::Ready,
                generation: 1,
                secret_backend: SecretBackendKind::Ephemeral,
            },
            secret: CredentialSecret {
                access_token: SecretString::from_str("access-token-canary"),
                refresh_token: Some(SecretString::from_str("refresh-token-canary")),
                id_token: None,
                fields: BTreeMap::new(),
            },
        };

        let outcome = AuthProvider::logout(
            &provider,
            LogoutRequest {
                credential: &stored,
                revoke: true,
            },
        )
        .await
        .expect("logout must return Ok outcome even when remote revoke fails");

        assert!(
            !outcome.remote_revoked,
            "remote_revoked must be false when revoke HTTP fails"
        );
        assert!(
            outcome
                .warning
                .as_deref()
                .is_some_and(|w| w.contains("revoke") || w.contains("failed")),
            "expected typed warning, got {:?}",
            outcome.warning
        );
        mock.assert_async().await;
    }
}
