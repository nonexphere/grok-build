//! Login coordinator: orchestrates start/complete/persist via the store.
//!
//! Provides both low-level start/complete APIs and high-level
//! [`LoginCoordinator::run_login`] that drives browser loopback or device
//! polling until a credential is persisted.

use std::io::{self, Write};
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use thiserror::Error;
use tokio::net::TcpListener;
use url::Url;

use xai_grok_auth::{
    CredentialKey, CredentialMetadata, CredentialSecret, CredentialStore, LoginCompletion,
    LoginFlowId, LoginInput, LoginRequest, LoginStart, LoginTransport, NewCredentialRecord,
    ProviderAccountInfo, ProviderCapabilities, ProviderError, ProviderId, ProviderRegistry,
    SecretBackendKind, SecretString,
};

use crate::providers::codex::callback::{self, CallbackError};
use crate::providers::codex::config::DEFAULT_BROWSER_CALLBACK_PATH;

/// Default max wait for browser callback / device approval.
const LOGIN_TIMEOUT: Duration = Duration::from_secs(600);

/// Errors from the login coordinator.
#[derive(Debug, Error)]
pub enum LoginCoordinatorError {
    #[error("provider error: {0}")]
    Provider(#[from] ProviderError),

    #[error("store error: {0}")]
    Store(#[from] xai_grok_auth::StoreError),

    #[error("login is still pending")]
    Pending,

    #[error("callback error: {0}")]
    Callback(#[from] CallbackError),

    #[error("login timed out")]
    TimedOut,

    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("{0}")]
    Message(String),
}

/// UI events emitted during login.
#[derive(Debug, Clone)]
pub enum LoginUiEvent {
    OpenBrowser {
        url: Url,
    },
    ShowDeviceCode {
        verification_uri: Url,
        user_code: String,
        expires_at: chrono::DateTime<Utc>,
    },
    WaitingForApproval,
    ExchangingToken,
    Completed {
        key: CredentialKey,
    },
    Failed {
        error: String,
    },
}

/// Login coordinator that drives a provider's login flow and persists the
/// result credential.
pub struct LoginCoordinator {
    store: Arc<dyn CredentialStore>,
    registry: Arc<ProviderRegistry>,
}

impl std::fmt::Debug for LoginCoordinator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoginCoordinator").finish_non_exhaustive()
    }
}

impl LoginCoordinator {
    pub fn new(store: Arc<dyn CredentialStore>, registry: Arc<ProviderRegistry>) -> Self {
        Self { store, registry }
    }

    /// Start a login flow for a provider.
    pub async fn start_login(
        &self,
        provider_id: &ProviderId,
        transport: LoginTransport,
        requested_alias: Option<String>,
    ) -> Result<(LoginFlowId, Vec<LoginUiEvent>), LoginCoordinatorError> {
        let provider = self
            .registry
            .get(provider_id)
            .map_err(|e| ProviderError::InvalidConfig(e.to_string()))?;

        let request = LoginRequest {
            transport,
            requested_alias,
            force_reauthentication: false,
            open_browser: true,
            account_policy: Default::default(),
            client_surface: xai_grok_auth::ClientSurface::Cli,
        };

        let start = provider.start_login(request).await?;
        let mut events = Vec::new();

        let flow_id = match start {
            LoginStart::Browser {
                flow_id,
                authorization_url,
                ..
            } => {
                events.push(LoginUiEvent::OpenBrowser {
                    url: authorization_url,
                });
                flow_id
            }
            LoginStart::Device {
                flow_id,
                verification_uri,
                user_code,
                expires_at,
                ..
            } => {
                events.push(LoginUiEvent::ShowDeviceCode {
                    verification_uri,
                    user_code,
                    expires_at,
                });
                flow_id
            }
        };

        Ok((flow_id, events))
    }

    /// Complete a login flow (browser callback or device poll) and, on
    /// success, persist the credential to the store.
    pub async fn complete_login(
        &self,
        provider_id: &ProviderId,
        flow_id: LoginFlowId,
        input: LoginInput,
    ) -> Result<Vec<LoginUiEvent>, LoginCoordinatorError> {
        let provider = self
            .registry
            .get(provider_id)
            .map_err(|e| ProviderError::InvalidConfig(e.to_string()))?;

        let completion = provider.complete_login(flow_id, input).await?;

        match completion {
            LoginCompletion::Pending { .. } => Ok(vec![LoginUiEvent::WaitingForApproval]),
            LoginCompletion::Complete { credential } => {
                let mut events = vec![LoginUiEvent::ExchangingToken];
                let metadata = self.store.create(credential).await?;
                events.push(LoginUiEvent::Completed {
                    key: metadata.key.clone(),
                });
                Ok(events)
            }
        }
    }

    /// Cancel a login flow.
    pub async fn cancel_login(
        &self,
        provider_id: &ProviderId,
        flow_id: LoginFlowId,
    ) -> Result<(), LoginCoordinatorError> {
        let provider = self
            .registry
            .get(provider_id)
            .map_err(|e| ProviderError::InvalidConfig(e.to_string()))?;
        provider.cancel_login(flow_id).await?;
        Ok(())
    }

    /// Full interactive login for a provider: start → (loopback | poll) → persist.
    pub async fn run_login(
        &self,
        provider_id: &ProviderId,
        transport: LoginTransport,
        open_browser: bool,
    ) -> Result<CredentialMetadata, LoginCoordinatorError> {
        match transport {
            LoginTransport::BrowserPkce => self.run_browser_login(provider_id, open_browser).await,
            LoginTransport::DeviceCode => self.run_device_login(provider_id).await,
            LoginTransport::ApiKey => self.run_api_key_login(provider_id, None).await,
        }
    }

    /// Persist an API-key credential for a BYOK provider.
    ///
    /// Secret source precedence: explicit `api_key` argument, then
    /// `GROK_BYOK_API_KEY` env (non-TTY). The secret never appears in argv
    /// of this function's return value or UI events.
    ///
    /// **Registry contract:** the provider must be registered in the
    /// coordinator's registry and advertise
    /// [`ProviderCapabilities::API_KEY_LOGIN`]. Unknown/unregistered ids and
    /// registered-but-non-API-key providers (e.g. xAI, Codex) are rejected.
    /// Third-party BYOK never falls back to `XAI_API_KEY`.
    pub async fn run_api_key_login(
        &self,
        provider_id: &ProviderId,
        api_key: Option<SecretString>,
    ) -> Result<CredentialMetadata, LoginCoordinatorError> {
        // Honor the registry: reject unknown/unregistered provider ids and
        // providers that do not advertise API_KEY_LOGIN. This closes the
        // foundation gap that previously accepted any well-formed id.
        let provider = self.registry.get(provider_id).map_err(|e| {
            LoginCoordinatorError::Provider(ProviderError::InvalidConfig(e.to_string()))
        })?;
        let caps = provider.descriptor().capabilities;
        if !caps.contains(ProviderCapabilities::API_KEY_LOGIN) {
            return Err(LoginCoordinatorError::Provider(
                ProviderError::InvalidConfig(format!(
                    "provider `{}` does not support API-key login \
                     (missing API_KEY_LOGIN capability)",
                    provider_id.as_str()
                )),
            ));
        }

        // Track which env var supplied the secret so the xAI fallback guard
        // can reject third-party use of XAI_API_KEY. Explicit args use an
        // opaque sentinel that never matches a real env name.
        let (key, used_env) = match api_key {
            Some(s) => (s.expose().to_owned(), "<explicit>"),
            None => match std::env::var("GROK_BYOK_API_KEY") {
                Ok(v) => (v, "GROK_BYOK_API_KEY"),
                Err(_) => {
                    return Err(LoginCoordinatorError::Message(
                        "API key login requires GROK_BYOK_API_KEY or an explicit secret".into(),
                    ));
                }
            },
        };
        if key.trim().is_empty() {
            return Err(LoginCoordinatorError::Message(
                "API key must not be empty".into(),
            ));
        }
        // Never allow third-party bindings to fall back to XAI_API_KEY.
        crate::providers::byok::reject_xai_api_key_fallback(provider_id.as_str(), used_env)
            .map_err(LoginCoordinatorError::Message)?;

        let mut secret_fields = std::collections::BTreeMap::new();
        secret_fields.insert("api_key".to_string(), SecretString::from_str(&key));
        let metadata = self
            .store
            .create(NewCredentialRecord {
                provider: provider_id.clone(),
                requested_alias: Some("default".into()),
                account: ProviderAccountInfo::default(),
                secret: CredentialSecret {
                    access_token: SecretString::from_str(&key),
                    refresh_token: None,
                    id_token: None,
                    fields: secret_fields,
                },
                expires_at: None,
                backend: SecretBackendKind::Ephemeral,
            })
            .await?;
        // Metadata Debug/status must not include raw key material.
        let rendered = format!("{metadata:?}");
        if rendered.contains(&key) {
            return Err(LoginCoordinatorError::Message(
                "credential metadata leaked secret material".into(),
            ));
        }
        Ok(metadata)
    }

    async fn run_browser_login(
        &self,
        provider_id: &ProviderId,
        open_browser: bool,
    ) -> Result<CredentialMetadata, LoginCoordinatorError> {
        let (flow_id, events) = self
            .start_login(provider_id, LoginTransport::BrowserPkce, None)
            .await?;

        let auth_url = events.iter().find_map(|ev| match ev {
            LoginUiEvent::OpenBrowser { url } => Some(url.clone()),
            _ => None,
        });

        let Some(auth_url) = auth_url else {
            return Err(LoginCoordinatorError::Message(
                "provider did not return a browser authorization URL".into(),
            ));
        };

        // Port in the authorize URL's redirect_uri must match the loopback bind
        // and the PKCE token exchange redirect_uri stored in the flow.
        let port = redirect_port_from_auth_url(&auth_url).unwrap_or(1455);

        let listener = TcpListener::bind(("127.0.0.1", port)).await.map_err(|e| {
            CallbackError::Bind(format!(
                "cannot bind 127.0.0.1:{port} for OAuth callback: {e}"
            ))
        })?;

        println!("Open this URL to authorize:\n  {auth_url}");
        if open_browser {
            let _ = webbrowser::open(auth_url.as_str());
        }
        println!(
            "Waiting for browser callback on 127.0.0.1:{port}{DEFAULT_BROWSER_CALLBACK_PATH} ..."
        );

        let callback_url =
            callback::await_callback(listener, DEFAULT_BROWSER_CALLBACK_PATH, LOGIN_TIMEOUT)
                .await?;

        let events = self
            .complete_login(
                provider_id,
                flow_id,
                LoginInput::BrowserCallback { url: callback_url },
            )
            .await?;

        self.metadata_from_completed_events(events).await
    }

    async fn run_device_login(
        &self,
        provider_id: &ProviderId,
    ) -> Result<CredentialMetadata, LoginCoordinatorError> {
        let (flow_id, events) = self
            .start_login(provider_id, LoginTransport::DeviceCode, None)
            .await?;

        let mut interval = Duration::from_secs(5);
        for ev in events {
            if let LoginUiEvent::ShowDeviceCode {
                verification_uri,
                user_code,
                expires_at,
            } = ev
            {
                println!(
                    "Visit {verification_uri} and enter code: {user_code}\n\
                     Code expires at {expires_at}"
                );
            }
        }

        let deadline = tokio::time::Instant::now() + LOGIN_TIMEOUT;
        loop {
            if tokio::time::Instant::now() >= deadline {
                let _ = self.cancel_login(provider_id, flow_id).await;
                return Err(LoginCoordinatorError::TimedOut);
            }

            print!(".");
            let _ = io::stdout().flush();

            let events = self
                .complete_login(provider_id, flow_id, LoginInput::Poll)
                .await?;

            let mut completed_key = None;
            let mut pending = false;
            for ev in events {
                match ev {
                    LoginUiEvent::WaitingForApproval => pending = true,
                    LoginUiEvent::Completed { key } => completed_key = Some(key),
                    LoginUiEvent::ExchangingToken => {
                        println!("\nExchanging authorization code...");
                    }
                    other => {
                        eprintln!("\nlogin: {other:?}");
                    }
                }
            }

            if let Some(key) = completed_key {
                println!();
                return self.store.load_metadata(&key).await?.ok_or_else(|| {
                    LoginCoordinatorError::Message("credential vanished after create".into())
                });
            }

            if pending {
                tokio::time::sleep(interval).await;
                if interval < Duration::from_secs(15) {
                    interval += Duration::from_secs(1);
                }
                continue;
            }

            return Err(LoginCoordinatorError::Message(
                "unexpected login completion state".into(),
            ));
        }
    }

    async fn metadata_from_completed_events(
        &self,
        events: Vec<LoginUiEvent>,
    ) -> Result<CredentialMetadata, LoginCoordinatorError> {
        for ev in events {
            if let LoginUiEvent::Completed { key } = ev {
                return self.store.load_metadata(&key).await?.ok_or_else(|| {
                    LoginCoordinatorError::Message("credential vanished after create".into())
                });
            }
        }
        Err(LoginCoordinatorError::Message(
            "login completed without credential".into(),
        ))
    }
}

/// Extract the redirect port embedded in the OAuth authorize URL.
fn redirect_port_from_auth_url(auth_url: &Url) -> Option<u16> {
    let redirect = auth_url
        .query_pairs()
        .find(|(k, _)| k == "redirect_uri")
        .map(|(_, v)| v.into_owned())?;
    let ru = Url::parse(&redirect).ok()?;
    ru.port()
}

#[cfg(test)]
mod api_key_login_tests {
    use super::*;
    use crate::registry;
    use crate::store::ephemeral::EphemeralCredentialStore;

    #[tokio::test]
    async fn api_key_login_persists_without_leaking_secret() {
        let store = Arc::new(EphemeralCredentialStore::new());
        let registry = Arc::new(registry::build_default_registry());
        let coord = LoginCoordinator::new(store.clone(), registry);
        let provider = ProviderId::new_unchecked("openrouter");
        let secret = SecretString::from_str("sk-test-secret-value-never-log");
        let meta = coord
            .run_api_key_login(&provider, Some(secret))
            .await
            .unwrap();
        assert_eq!(meta.key.provider.as_str(), "openrouter");
        let debug = format!("{meta:?}");
        assert!(
            !debug.contains("sk-test-secret-value-never-log"),
            "leaked: {debug}"
        );
        let accounts = store.list_accounts(&provider).await.unwrap();
        assert_eq!(accounts.len(), 1);
    }

    #[tokio::test]
    async fn api_key_login_rejects_empty_and_missing_env() {
        let store = Arc::new(EphemeralCredentialStore::new());
        let coord = LoginCoordinator::new(store, Arc::new(registry::build_default_registry()));
        let provider = ProviderId::new_unchecked("groq");
        let err = coord
            .run_api_key_login(&provider, Some(SecretString::from_str("")))
            .await
            .unwrap_err();
        assert!(format!("{err}").contains("empty"));
        // clear env and require explicit secret or env
        let err2 = coord.run_api_key_login(&provider, None).await.unwrap_err();
        assert!(
            format!("{err2}").contains("GROK_BYOK_API_KEY")
                || format!("{err2}").contains("API key")
        );
    }

    #[tokio::test]
    async fn api_key_login_rejects_unregistered_provider() {
        let store = Arc::new(EphemeralCredentialStore::new());
        let coord = LoginCoordinator::new(store, Arc::new(registry::build_default_registry()));
        let unknown = ProviderId::new_unchecked("unknown-byok-provider");
        let err = coord
            .run_api_key_login(&unknown, Some(SecretString::from_str("sk-test")))
            .await
            .expect_err("unregistered provider must be rejected");
        assert!(
            format!("{err}").to_lowercase().contains("not registered")
                || format!("{err}").to_lowercase().contains("unknown"),
            "expected registry rejection, got: {err}"
        );
    }

    #[tokio::test]
    async fn api_key_login_rejects_provider_without_api_key_capability() {
        let store = Arc::new(EphemeralCredentialStore::new());
        let coord = LoginCoordinator::new(store, Arc::new(registry::build_default_registry()));
        // xAI advertises empty capabilities.
        let err = coord
            .run_api_key_login(
                &ProviderId::new_unchecked("xai"),
                Some(SecretString::from_str("sk-test")),
            )
            .await
            .expect_err("xAI must be rejected for API-key login");
        assert!(
            format!("{err}").to_lowercase().contains("api-key")
                || format!("{err}").to_lowercase().contains("api_key")
                || format!("{err}").to_lowercase().contains("does not support"),
            "expected capability rejection, got: {err}"
        );
    }
}
