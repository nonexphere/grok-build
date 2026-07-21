//! WebSocket transport adapter — reuses the same processor as stdio/in-process.
//! Authentication: bearer token in Authorization header only (no query credentials).

use std::sync::Arc;

use crate::ProcessorError;
use crate::processor::FacadeProcessor;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebSocketAuth {
    pub bearer_token: String,
}

/// Validate Authorization header for remote/local WS. Tokens never appear in URLs.
/// Comparison is constant-time over the full expected token length so that
/// early-exit timing does not enumerate valid prefixes.
pub fn validate_bearer_header(header: Option<&str>, expected: &str) -> Result<(), ProcessorError> {
    let unauthorized = || ProcessorError {
        code: -32001,
        message: "Authentication required.".into(),
    };
    let Some(header) = header else {
        return Err(unauthorized());
    };
    let token = header
        .strip_prefix("Bearer ")
        .or_else(|| header.strip_prefix("bearer "))
        .unwrap_or("");
    if !constant_time_eq(token.as_bytes(), expected.as_bytes()) {
        return Err(unauthorized());
    }
    Ok(())
}

/// Fixed-time equality: always scans `expected.len()` bytes (padding candidate
/// when shorter) so length and prefix mismatches do not short-circuit early.
fn constant_time_eq(candidate: &[u8], expected: &[u8]) -> bool {
    let mut diff: u8 = if candidate.len() == expected.len() {
        0
    } else {
        1
    };
    for (i, exp) in expected.iter().enumerate() {
        let c = candidate.get(i).copied().unwrap_or(0);
        diff |= c ^ exp;
    }
    // Also fold remaining candidate bytes so longer candidates are not free.
    for (i, c) in candidate.iter().enumerate().skip(expected.len()) {
        let _ = i;
        diff |= *c;
    }
    diff == 0
}

pub fn reject_credentials_in_url(url: &str) -> Result<(), ProcessorError> {
    if url.contains('@') || url.contains("token=") || url.contains("access_token=") {
        return Err(ProcessorError {
            code: -32600,
            message: "WebSocket URL must not contain credentials".into(),
        });
    }
    Ok(())
}

/// Process one text WebSocket message (JSON object) through the shared processor.
pub async fn handle_ws_text(
    processor: Arc<FacadeProcessor>,
    text: &str,
) -> Result<Option<String>, ProcessorError> {
    validate_ws_text_frame(text)?;
    processor.handle_line(text).await
}

/// Reject binary framing stand-ins, batches, and oversized text frames (1 MiB).
pub fn validate_ws_text_frame(text: &str) -> Result<(), ProcessorError> {
    const MAX: usize = 1_048_576;
    if text.len() > MAX {
        return Err(ProcessorError {
            code: -32021,
            message: "Message exceeds the size limit.".into(),
        });
    }
    let trimmed = text.trim_start();
    if trimmed.starts_with('[') {
        return Err(ProcessorError {
            code: -32600,
            message: "JSON-RPC batches are unsupported".into(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod websocket_tests {
    use super::*;
    use serde_json::json;
    use std::sync::Arc;
    use xai_grok_app_server_protocol::PROTOCOL_VERSION;
    use xai_grok_tower::FakeRuntime;

    #[test]
    fn websocket_auth_header_only_rejects_missing_and_url_tokens() {
        assert!(validate_bearer_header(None, "secret").is_err());
        assert!(validate_bearer_header(Some("Bearer secret"), "secret").is_ok());
        assert!(validate_bearer_header(Some("Bearer wrong"), "secret").is_err());
        assert!(reject_credentials_in_url("ws://127.0.0.1:8080/").is_ok());
        assert!(reject_credentials_in_url("ws://user:pass@host/").is_err());
        assert!(reject_credentials_in_url("ws://host/?token=abc").is_err());
    }

    #[test]
    fn bearer_auth_rejects_prefix_and_length_mismatches_identically() {
        let expected = "super-secret-token-value";
        for bad in [
            None,
            Some("Bearer super-secret-token-valu"),
            Some("Bearer super-secret-token-valueX"),
            Some("Bearer wrong"),
            Some("super-secret-token-value"),
        ] {
            let err = validate_bearer_header(bad, expected).unwrap_err();
            assert_eq!(err.code, -32001);
            assert_eq!(err.message, "Authentication required.");
        }
        assert!(validate_bearer_header(Some("Bearer super-secret-token-value"), expected).is_ok());
    }

    #[tokio::test]
    async fn websocket_reuses_processor_parity_with_stdio() {
        let processor = Arc::new(FacadeProcessor::new(Arc::new(FakeRuntime::new())));
        let init = json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{
            "protocolVersion": PROTOCOL_VERSION,
            "clientInfo":{"name":"ws","version":"0"},
            "capabilities":{}
        }})
        .to_string();
        let resp = handle_ws_text(processor, &init).await.unwrap().unwrap();
        let v: serde_json::Value = serde_json::from_str(&resp).unwrap();
        assert_eq!(v["result"]["protocolVersion"], PROTOCOL_VERSION);
    }

    #[test]
    fn websocket_transport_rejects_batch_and_oversized_frames() {
        assert!(validate_ws_text_frame(r#"{"jsonrpc":"2.0"}"#).is_ok());
        assert_eq!(validate_ws_text_frame("[1,2]").unwrap_err().code, -32600);
        let big = "x".repeat(1_048_577);
        assert_eq!(validate_ws_text_frame(&big).unwrap_err().code, -32021);
    }
}
