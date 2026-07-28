//! Codex provider error mapping (protocol-baseline.md §9).

use thiserror::Error;
use xai_grok_auth::ProviderError;

/// Classification of refresh failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefreshFailureKind {
    Expired,
    Reused,
    Revoked,
    InvalidGrant,
    Transient,
    Unknown,
}

/// Map a backend error code + HTTP status to a `ProviderError`.
pub fn classify_refresh_failure(
    status: u16,
    error_code: Option<&str>,
) -> (RefreshFailureKind, ProviderError) {
    match error_code {
        Some("refresh_token_expired") => (
            RefreshFailureKind::Expired,
            ProviderError::ReauthenticationRequired("ChatGPT session expired".into()),
        ),
        Some("refresh_token_reused") => (
            RefreshFailureKind::Reused,
            ProviderError::ReauthenticationRequired("Session token was already rotated".into()),
        ),
        Some("refresh_token_invalidated") => (
            RefreshFailureKind::Revoked,
            ProviderError::ReauthenticationRequired("ChatGPT session was revoked".into()),
        ),
        _ => {
            if status == 401 {
                (
                    RefreshFailureKind::InvalidGrant,
                    ProviderError::ReauthenticationRequired(
                        "ChatGPT rejected the saved session".into(),
                    ),
                )
            } else if status == 429 {
                (
                    RefreshFailureKind::Transient,
                    ProviderError::Refresh(
                        "Authentication service is rate-limiting requests".into(),
                    ),
                )
            } else if status >= 500 {
                (
                    RefreshFailureKind::Transient,
                    ProviderError::Refresh(
                        "Authentication service is temporarily unavailable".into(),
                    ),
                )
            } else {
                (
                    RefreshFailureKind::Unknown,
                    ProviderError::Refresh(format!("Unexpected error (HTTP {status})")),
                )
            }
        }
    }
}

/// Errors specific to Codex token operations.
///
/// HTTP bodies are stored only for classification; `Display` never dumps the
/// raw body (which may contain sensitive OAuth fields).
#[derive(Debug, Error)]
pub enum CodexTokenError {
    #[error("HTTP request failed: {0}")]
    Transport(String),

    /// Token/device endpoint returned a non-success status. Body is never stored
    /// (may contain sensitive fields); only a short OAuth error code is kept.
    #[error("token endpoint returned HTTP {status}{code}")]
    HttpStatus {
        status: u16,
        /// Formatted suffix for Display, e.g. ` (error=invalid_grant)`.
        code: String,
        /// Parsed OAuth `error` / `error_code` when present.
        error_code: Option<String>,
    },

    #[error("token response missing required field: {0}")]
    MissingField(String),

    #[error("token response was not valid JSON: {0}")]
    InvalidJson(String),
}

impl CodexTokenError {
    /// Build an `HttpStatus` error from a raw response body without embedding
    /// the full body. Extracts a short `error` code when present.
    pub fn http_status(status: u16, body: &str) -> Self {
        let error_code = serde_json::from_str::<serde_json::Value>(body)
            .ok()
            .and_then(|v| {
                v.get("error")
                    .and_then(|e| {
                        e.as_str().map(|s| s.to_string()).or_else(|| {
                            e.get("code")
                                .and_then(|c| c.as_str().map(|s| s.to_string()))
                        })
                    })
                    .or_else(|| {
                        v.get("error_code")
                            .and_then(|c| c.as_str().map(|s| s.to_string()))
                    })
            });
        let code = error_code
            .as_ref()
            .map(|c| format!(" (error={c})"))
            .unwrap_or_default();
        Self::HttpStatus {
            status,
            code,
            error_code,
        }
    }

    pub fn status_code(&self) -> Option<u16> {
        match self {
            Self::HttpStatus { status, .. } => Some(*status),
            _ => None,
        }
    }

    pub fn error_code(&self) -> Option<&str> {
        match self {
            Self::HttpStatus { error_code, .. } => error_code.as_deref(),
            _ => None,
        }
    }
}
