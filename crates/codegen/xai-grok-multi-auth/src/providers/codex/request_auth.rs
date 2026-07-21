//! Request auth header construction for Codex (protocol-baseline.md §6.3).

use http::header::AUTHORIZATION;
use http::{HeaderMap, HeaderValue};
use url::Url;

use xai_grok_auth::{
    AuthFailureClass, AuthFailureResponse, ProviderEndpointKind, ProviderEndpointRequest,
    ProviderError, StoredCredential,
};

use super::config::{CODEX_BASE_URL, CodexOAuthConfig};

/// Build the request headers for a Codex inference request.
///
/// Per §6.3, required headers:
/// ```http
/// Authorization: Bearer <access-token>
/// ChatGPT-Account-ID: <chatgpt-account-id>
/// ```
///
/// When the account claims require FedRAMP routing (§6.3):
/// ```http
/// X-OpenAI-Fedramp: true
/// ```
pub fn build_codex_request_headers(
    credential: &StoredCredential,
) -> Result<HeaderMap, ProviderError> {
    let mut headers = HeaderMap::new();

    // Authorization: Bearer <access-token>
    let bearer = format!("Bearer {}", credential.secret.access_token.expose());
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&bearer).map_err(|e| {
            ProviderError::InvalidConfig(format!("invalid access token header: {e}"))
        })?,
    );

    // ChatGPT-Account-ID: prefer stored metadata, then re-parse tokens.
    let enriched = super::claims::enrich_account_from_tokens(
        &credential.metadata.account,
        credential.secret.id_token.as_ref().map(|s| s.expose()),
        Some(credential.secret.access_token.expose()),
    );
    let account_id = enriched
        .metadata
        .get("chatgpt_account_id")
        .cloned()
        .or(enriched.provider_account_id.clone())
        .ok_or_else(|| {
            ProviderError::InvalidConfig("missing ChatGPT account ID for Codex credential".into())
        })?;
    headers.insert(
        "ChatGPT-Account-ID",
        HeaderValue::from_str(&account_id)
            .map_err(|e| ProviderError::InvalidConfig(format!("invalid account ID header: {e}")))?,
    );

    // X-OpenAI-Fedramp: true (when fedramp)
    if credential.metadata.account.fedramp {
        headers.insert("X-OpenAI-Fedramp", HeaderValue::from_static("true"));
    }

    Ok(headers)
}

/// Resolve a Codex endpoint URL (§6.1, §6.2, §7).
pub fn resolve_codex_endpoint(
    _config: &CodexOAuthConfig,
    request: &ProviderEndpointRequest<'_>,
) -> Result<Url, ProviderError> {
    let base = Url::parse(CODEX_BASE_URL)
        .map_err(|e| ProviderError::InvalidConfig(format!("invalid Codex base URL: {e}")))?;

    let path = match request.kind {
        ProviderEndpointKind::Inference => "/responses",
        ProviderEndpointKind::Models => "/models",
        ProviderEndpointKind::Account => "/me",
    };

    let mut url = base;
    url.set_path(path);
    Ok(url)
}

/// Classify a Codex auth failure response (§9).
pub fn classify_codex_auth_failure(response: &AuthFailureResponse) -> AuthFailureClass {
    match response.status.as_u16() {
        401 => AuthFailureClass::ReauthenticationRequired,
        403 => AuthFailureClass::PermissionDenied,
        429 => AuthFailureClass::Transient,
        500..=599 => AuthFailureClass::Transient,
        _ => AuthFailureClass::NotAuthentication,
    }
}
