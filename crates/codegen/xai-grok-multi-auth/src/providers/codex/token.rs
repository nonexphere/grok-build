//! Token exchange for Codex OAuth (protocol-baseline.md §4).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use url::Url;

use xai_grok_auth::SecretString;

use super::errors::CodexTokenError;

/// Response from the token endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    #[serde(default)]
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub id_token: Option<String>,
    #[serde(default)]
    pub expires_in: Option<u64>,
    #[serde(default)]
    pub token_type: Option<String>,
}

impl TokenResponse {
    /// Convert into a `CredentialSecret` with `SecretString` wrappers.
    pub fn to_credential_secret(&self) -> xai_grok_auth::CredentialSecret {
        xai_grok_auth::CredentialSecret {
            access_token: SecretString::from_str(&self.access_token),
            refresh_token: self
                .refresh_token
                .as_ref()
                .map(|s| SecretString::from_str(s)),
            id_token: self.id_token.as_ref().map(|s| SecretString::from_str(s)),
            fields: BTreeMap::new(),
        }
    }
}

/// Exchange an authorization code for tokens (browser flow, §4.1).
///
/// Uses `application/x-www-form-urlencoded` body.
pub async fn exchange_authorization_code(
    client: &reqwest::Client,
    token_endpoint: &Url,
    code: &str,
    redirect_uri: &str,
    client_id: &str,
    code_verifier: &str,
) -> Result<TokenResponse, CodexTokenError> {
    let params = [
        ("grant_type", "authorization_code"),
        ("code", code),
        ("redirect_uri", redirect_uri),
        ("client_id", client_id),
        ("code_verifier", code_verifier),
    ];

    let resp = client
        .post(token_endpoint.as_str())
        .form(&params)
        .send()
        .await
        .map_err(|e| CodexTokenError::Transport(e.to_string()))?;

    let status = resp.status();
    let body = resp
        .text()
        .await
        .map_err(|e| CodexTokenError::Transport(e.to_string()))?;

    if !status.is_success() {
        return Err(CodexTokenError::http_status(status.as_u16(), &body));
    }

    serde_json::from_str(&body).map_err(|e| CodexTokenError::InvalidJson(e.to_string()))
}

/// Refresh a token (§4.2).
///
/// Uses JSON body. Missing fields mean "retain the previous value."
pub async fn refresh_token(
    client: &reqwest::Client,
    token_endpoint: &Url,
    client_id: &str,
    refresh_token: &str,
) -> Result<TokenResponse, CodexTokenError> {
    let body = serde_json::json!({
        "client_id": client_id,
        "grant_type": "refresh_token",
        "refresh_token": refresh_token,
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

    if !status.is_success() {
        return Err(CodexTokenError::http_status(status.as_u16(), &resp_body));
    }

    serde_json::from_str(&resp_body).map_err(|e| CodexTokenError::InvalidJson(e.to_string()))
}

/// Revoke a token (§4.3).
pub async fn revoke_token(
    client: &reqwest::Client,
    revoke_endpoint: &Url,
    token: &str,
    token_type_hint: &str,
    client_id: &str,
) -> Result<(), CodexTokenError> {
    let body = serde_json::json!({
        "token": token,
        "token_type_hint": token_type_hint,
        "client_id": client_id,
    });

    let resp = client
        .post(revoke_endpoint.as_str())
        .json(&body)
        .send()
        .await
        .map_err(|e| CodexTokenError::Transport(e.to_string()))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp
            .text()
            .await
            .map_err(|e| CodexTokenError::Transport(e.to_string()))?;
        return Err(CodexTokenError::http_status(status.as_u16(), &body));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn token_exchange_with_mockito_success() {
        let mut server = mockito::Server::new_async().await;
        let token_url = server.url();

        let mock = server
            .mock("POST", "/oauth/token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "access_token": "test-access-token",
                    "refresh_token": "test-refresh-token",
                    "id_token": "test-id-token",
                    "expires_in": 300,
                    "token_type": "Bearer"
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let endpoint = Url::parse(&format!("{}/oauth/token", token_url)).unwrap();
        let result = exchange_authorization_code(
            &client,
            &endpoint,
            "test-code",
            "http://localhost:1455/auth/callback",
            "test-client-id",
            "test-verifier",
        )
        .await
        .unwrap();

        assert_eq!(result.access_token, "test-access-token");
        assert_eq!(result.refresh_token.as_deref(), Some("test-refresh-token"));
        assert_eq!(result.id_token.as_deref(), Some("test-id-token"));
        assert_eq!(result.expires_in, Some(300));

        mock.assert_async().await;
    }
}
