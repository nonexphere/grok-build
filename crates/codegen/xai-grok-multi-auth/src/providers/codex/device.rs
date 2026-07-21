//! Device-code flow for Codex (protocol-baseline.md §5).
//!
//! This is NOT standard RFC 8628. It uses provider-specific endpoints and a
//! non-standard exchange sequence (D7).

use serde::{Deserialize, Serialize};
use std::time::Duration;
use url::Url;

use super::errors::CodexTokenError;
use super::token::{TokenResponse, exchange_authorization_code};

// Device redirect URI is defined in `config::DEVICE_REDIRECT_URI`
// (`https://auth.openai.com/deviceauth/callback`) — re-exported for callers.
pub use super::config::DEVICE_REDIRECT_URI;

/// Response from the device user-code endpoint (§5.1).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceUserCodeResponse {
    pub device_auth_id: String,
    pub user_code: String,
    /// Interval may come as a string ("5") or number.
    #[serde(default)]
    pub interval: serde_json::Value,
}

impl DeviceUserCodeResponse {
    /// Parse the polling interval as seconds (default 5).
    pub fn interval_secs(&self) -> u64 {
        match &self.interval {
            serde_json::Value::Number(n) => n.as_u64().unwrap_or(5),
            serde_json::Value::String(s) => s.parse().unwrap_or(5),
            _ => 5,
        }
    }
}

/// Response from the device token polling endpoint (§5.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevicePollResponse {
    #[serde(default)]
    pub authorization_code: Option<String>,
    #[serde(default)]
    pub code_challenge: Option<String>,
    #[serde(default)]
    pub code_verifier: Option<String>,
}

/// Request a user code from the device endpoint (§5.1).
pub async fn request_user_code(
    client: &reqwest::Client,
    usercode_endpoint: &Url,
    client_id: &str,
) -> Result<DeviceUserCodeResponse, CodexTokenError> {
    let body = serde_json::json!({ "client_id": client_id });

    let resp = client
        .post(usercode_endpoint.as_str())
        .json(&body)
        .send()
        .await
        .map_err(|e| CodexTokenError::Transport(e.to_string()))?;

    let status = resp.status();
    let resp_body = resp
        .text()
        .await
        .map_err(|e| CodexTokenError::Transport(e.to_string()))?;

    if !status.is_success() {
        return Err(CodexTokenError::http_status(status.as_u16(), &resp_body));
    }

    serde_json::from_str(&resp_body).map_err(|e| CodexTokenError::InvalidJson(e.to_string()))
}

/// Poll the device token endpoint once (§5.3).
///
/// Returns `Ok(Some(response))` when approval is complete, `Ok(None)` when
/// still pending (403/404 treated as pending per §5.3).
pub async fn poll_device_token(
    client: &reqwest::Client,
    token_endpoint: &Url,
    device_auth_id: &str,
    user_code: &str,
) -> Result<Option<DevicePollResponse>, CodexTokenError> {
    let body = serde_json::json!({
        "device_auth_id": device_auth_id,
        "user_code": user_code,
    });

    let resp = client
        .post(token_endpoint.as_str())
        .json(&body)
        .send()
        .await
        .map_err(|e| CodexTokenError::Transport(e.to_string()))?;

    let status = resp.status();
    let resp_body = resp
        .text()
        .await
        .map_err(|e| CodexTokenError::Transport(e.to_string()))?;

    // Treat 403 and 404 as pending (§5.3).
    if status.as_u16() == 403 || status.as_u16() == 404 {
        return Ok(None);
    }

    if !status.is_success() {
        return Err(CodexTokenError::http_status(status.as_u16(), &resp_body));
    }

    let poll: DevicePollResponse = serde_json::from_str(&resp_body)
        .map_err(|e| CodexTokenError::InvalidJson(e.to_string()))?;

    if poll.authorization_code.is_some() {
        Ok(Some(poll))
    } else {
        Ok(None)
    }
}

/// Complete the device flow: exchange the authorization code returned by
/// the poll endpoint (§5.4).
pub async fn complete_device_flow(
    client: &reqwest::Client,
    token_endpoint: &Url,
    poll_response: &DevicePollResponse,
    client_id: &str,
    device_redirect_uri: &str,
) -> Result<TokenResponse, CodexTokenError> {
    let code = poll_response
        .authorization_code
        .as_ref()
        .ok_or_else(|| CodexTokenError::MissingField("authorization_code".into()))?;
    let verifier = poll_response
        .code_verifier
        .as_ref()
        .ok_or_else(|| CodexTokenError::MissingField("code_verifier".into()))?;

    exchange_authorization_code(
        client,
        token_endpoint,
        code,
        device_redirect_uri,
        client_id,
        verifier,
    )
    .await
}

/// Default polling interval.
pub const DEFAULT_POLL_INTERVAL: Duration = Duration::from_secs(5);

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn device_usercode_and_poll_with_mockito() {
        let mut server = mockito::Server::new_async().await;
        let base = server.url();

        // Mock the usercode endpoint.
        let _usercode_mock = server
            .mock("POST", "/api/accounts/deviceauth/usercode")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "device_auth_id": "device-123",
                    "user_code": "ABCD-EFGH",
                    "interval": "5"
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = reqwest::Client::new();

        // Step 1: request user code.
        let usercode_url =
            Url::parse(&format!("{}/api/accounts/deviceauth/usercode", base)).unwrap();
        let usercode_resp = request_user_code(&client, &usercode_url, "test-client-id")
            .await
            .unwrap();
        assert_eq!(usercode_resp.device_auth_id, "device-123");
        assert_eq!(usercode_resp.user_code, "ABCD-EFGH");
        assert_eq!(usercode_resp.interval_secs(), 5);

        // Mock the poll endpoint — first call pending (404), second call complete.
        let poll_url = Url::parse(&format!("{}/api/accounts/deviceauth/token", base)).unwrap();

        let pending_mock = server
            .mock("POST", "/api/accounts/deviceauth/token")
            .with_status(404)
            .create_async()
            .await;

        // First poll: pending.
        let poll1 = poll_device_token(&client, &poll_url, "device-123", "ABCD-EFGH")
            .await
            .unwrap();
        assert!(poll1.is_none());
        pending_mock.assert_async().await;

        // Second poll: complete.
        let complete_mock = server
            .mock("POST", "/api/accounts/deviceauth/token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "authorization_code": "auth-code-xyz",
                    "code_challenge": "challenge-abc",
                    "code_verifier": "verifier-abc"
                })
                .to_string(),
            )
            .create_async()
            .await;

        let poll2 = poll_device_token(&client, &poll_url, "device-123", "ABCD-EFGH")
            .await
            .unwrap();
        assert!(poll2.is_some());
        let poll_resp = poll2.unwrap();
        assert_eq!(
            poll_resp.authorization_code.as_deref(),
            Some("auth-code-xyz")
        );
        assert_eq!(poll_resp.code_verifier.as_deref(), Some("verifier-abc"));
        complete_mock.assert_async().await;
    }
}
