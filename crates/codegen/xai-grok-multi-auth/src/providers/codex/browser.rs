//! Browser PKCE login flow for Codex (protocol-baseline.md §3).

use std::collections::HashMap;

use url::Url;

use super::config::{CodexOAuthConfig, DEFAULT_SCOPES};
use super::pkce::{generate_state, PkceVerifier};

/// In-memory state for a browser PKCE login flow.
///
/// `Debug` redacts PKCE and OAuth state secrets.
#[derive(Clone)]
pub struct BrowserFlowState {
    pub pkce: PkceVerifier,
    pub state: String,
    pub port: u16,
}

impl std::fmt::Debug for BrowserFlowState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BrowserFlowState")
            .field("pkce", &self.pkce)
            .field("state", &"<redacted>")
            .field("port", &self.port)
            .finish()
    }
}

/// Build the authorization URL for the browser PKCE flow (§3.1).
pub fn build_authorization_url(
    config: &CodexOAuthConfig,
    pkce: &PkceVerifier,
    state: &str,
    port: u16,
) -> Url {
    let mut url = config.authorize_url();
    url.query_pairs_mut()
        .append_pair("response_type", "code")
        .append_pair("client_id", &config.client_id)
        .append_pair(
            "redirect_uri",
            &config.redirect_uri_for_port(port),
        )
        .append_pair("scope", &DEFAULT_SCOPES.join(" "))
        .append_pair("code_challenge", pkce.challenge())
        .append_pair("code_challenge_method", "S256")
        .append_pair("id_token_add_organizations", "true")
        .append_pair("codex_cli_simplified_flow", "true")
        .append_pair("state", state)
        .append_pair("originator", &config.originator);
    url
}

/// Start a new browser flow: generate PKCE + state, pick the first
/// configured redirect port that can bind on `127.0.0.1`.
pub fn start_browser_flow(config: &CodexOAuthConfig) -> BrowserFlowState {
    let pkce = PkceVerifier::new();
    let state = generate_state();
    let port = pick_free_loopback_port(&config.browser_redirect_ports)
        .unwrap_or(config.browser_redirect_ports[0]);
    BrowserFlowState { pkce, state, port }
}

/// Probe which configured callback port is free on loopback.
fn pick_free_loopback_port(ports: &[u16]) -> Option<u16> {
    for &port in ports {
        if std::net::TcpListener::bind(("127.0.0.1", port)).is_ok() {
            return Some(port);
        }
    }
    None
}

/// Parse the callback URL and extract `code` and `state` parameters.
pub fn parse_callback(url: &Url) -> Option<CallbackParams> {
    let params: HashMap<String, String> = url
        .query_pairs()
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect();
    let code = params.get("code")?.clone();
    let state = params.get("state")?.clone();
    let error = params.get("error").cloned();
    Some(CallbackParams { code, state, error })
}

/// Parsed callback parameters.
#[derive(Debug, Clone)]
pub struct CallbackParams {
    pub code: String,
    pub state: String,
    pub error: Option<String>,
}

/// Validate that the callback state matches the expected state.
///
/// Returns `Ok(code)` if the state matches and there is no error.
/// Returns `Err` with a descriptive message otherwise.
pub fn validate_callback(
    expected_state: &str,
    params: &CallbackParams,
) -> Result<String, String> {
    if let Some(ref error) = params.error {
        return Err(format!("OAuth error: {error}"));
    }
    if !constant_time_eq(expected_state.as_bytes(), params.state.as_bytes()) {
        return Err("state mismatch".to_string());
    }
    Ok(params.code.clone())
}

/// Constant-time comparison to mitigate timing attacks.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut result = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }
    result == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth_url_contains_required_query_params() {
        let config = CodexOAuthConfig::default();
        let pkce = PkceVerifier::new();
        let state = "test-state-123";
        let url = build_authorization_url(&config, &pkce, state, 1455);

        let query: HashMap<String, String> = url
            .query_pairs()
            .map(|(k, v)| (k.into_owned(), v.into_owned()))
            .collect();
        assert_eq!(query.get("response_type").map(|s| s.as_str()), Some("code"));
        assert_eq!(
            query.get("client_id").map(|s| s.as_str()),
            Some("app_EMoamEEZ73f0CkXaXp7hrann")
        );
        assert_eq!(
            query.get("redirect_uri").map(|s| s.as_str()),
            Some("http://localhost:1455/auth/callback")
        );
        assert!(query.get("scope").unwrap().contains("openid"));
        assert!(query.get("scope").unwrap().contains("offline_access"));
        assert_eq!(
            query.get("code_challenge_method").map(|s| s.as_str()),
            Some("S256")
        );
        assert_eq!(query.get("state").map(|s| s.as_str()), Some("test-state-123"));
        assert_eq!(
            query.get("id_token_add_organizations").map(|s| s.as_str()),
            Some("true")
        );
        assert_eq!(
            query.get("codex_cli_simplified_flow").map(|s| s.as_str()),
            Some("true")
        );
        assert!(query.get("originator").is_some());
        // Challenge should be present and non-empty.
        assert!(!query.get("code_challenge").unwrap().is_empty());
    }

    #[test]
    fn callback_state_mismatch_rejected() {
        let expected = "correct-state";
        let params = CallbackParams {
            code: "test-code".to_string(),
            state: "wrong-state".to_string(),
            error: None,
        };
        let result = validate_callback(expected, &params);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "state mismatch");
    }

    #[test]
    fn callback_state_match_accepted() {
        let expected = "correct-state";
        let params = CallbackParams {
            code: "test-code".to_string(),
            state: "correct-state".to_string(),
            error: None,
        };
        let result = validate_callback(expected, &params);
        assert_eq!(result.unwrap(), "test-code");
    }

    #[test]
    fn callback_error_rejected() {
        let params = CallbackParams {
            code: String::new(),
            state: "correct-state".to_string(),
            error: Some("access_denied".to_string()),
        };
        let result = validate_callback("correct-state", &params);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("access_denied"));
    }
}
