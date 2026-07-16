//! Codex OAuth configuration (protocol-baseline.md §1, §2).

use url::Url;

/// Protocol-default OAuth client ID (protocol reference only, D10).
pub const DEFAULT_CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann";

/// Default issuer.
pub const DEFAULT_ISSUER: &str = "https://auth.openai.com";

/// Default browser callback ports (1455 preferred, 1457 fallback).
pub const DEFAULT_BROWSER_PORTS: &[u16] = &[1455, 1457];

/// Default browser callback path.
pub const DEFAULT_BROWSER_CALLBACK_PATH: &str = "/auth/callback";

/// Default scopes.
pub const DEFAULT_SCOPES: &[&str] = &[
    "openid",
    "profile",
    "email",
    "offline_access",
    "api.connectors.read",
    "api.connectors.invoke",
];

/// Default originator (pending provider approval, OQ3).
pub const DEFAULT_ORIGINATOR: &str = "grok-build";

/// Device verification URI path.
pub const DEVICE_VERIFICATION_PATH: &str = "/codex/device";

/// Device user-code endpoint path.
pub const DEVICE_USERCODE_PATH: &str = "/api/accounts/deviceauth/usercode";

/// Device polling endpoint path.
pub const DEVICE_TOKEN_PATH: &str = "/api/accounts/deviceauth/token";

/// Device exchange redirect URI.
pub const DEVICE_REDIRECT_URI: &str = "https://auth.openai.com/deviceauth/callback";

/// Authorization endpoint path.
pub const AUTHORIZE_PATH: &str = "/oauth/authorize";

/// Token endpoint path.
pub const TOKEN_PATH: &str = "/oauth/token";

/// Revocation endpoint path.
pub const REVOKE_PATH: &str = "/oauth/revoke";

/// ChatGPT Codex base URL.
pub const CODEX_BASE_URL: &str = "https://chatgpt.com/backend-api/codex";

/// Configurable Codex OAuth configuration.
#[derive(Debug, Clone)]
pub struct CodexOAuthConfig {
    pub issuer: Url,
    pub client_id: String,
    pub browser_redirect_ports: Vec<u16>,
    pub browser_callback_path: String,
    pub originator: String,
}

impl Default for CodexOAuthConfig {
    fn default() -> Self {
        Self {
            issuer: Url::parse(DEFAULT_ISSUER).unwrap(),
            client_id: DEFAULT_CLIENT_ID.to_string(),
            browser_redirect_ports: DEFAULT_BROWSER_PORTS.to_vec(),
            browser_callback_path: DEFAULT_BROWSER_CALLBACK_PATH.to_string(),
            originator: DEFAULT_ORIGINATOR.to_string(),
        }
    }
}

impl CodexOAuthConfig {
    /// Full authorization endpoint URL.
    pub fn authorize_url(&self) -> Url {
        self.issuer_url_with_path(AUTHORIZE_PATH)
    }

    /// Full token endpoint URL.
    pub fn token_url(&self) -> Url {
        self.issuer_url_with_path(TOKEN_PATH)
    }

    /// Full revocation endpoint URL.
    pub fn revoke_url(&self) -> Url {
        self.issuer_url_with_path(REVOKE_PATH)
    }

    /// Full device user-code endpoint URL.
    pub fn device_usercode_url(&self) -> Url {
        self.issuer_url_with_path(DEVICE_USERCODE_PATH)
    }

    /// Full device token polling endpoint URL.
    pub fn device_token_url(&self) -> Url {
        self.issuer_url_with_path(DEVICE_TOKEN_PATH)
    }

    /// Device verification URL.
    pub fn device_verification_url(&self) -> Url {
        self.issuer_url_with_path(DEVICE_VERIFICATION_PATH)
    }

    /// Loopback redirect URI for a given port.
    pub fn redirect_uri_for_port(&self, port: u16) -> String {
        format!("http://localhost:{port}{}", self.browser_callback_path)
    }

    fn issuer_url_with_path(&self, path: &str) -> Url {
        let mut url = self.issuer.clone();
        url.set_path(path);
        url
    }
}
