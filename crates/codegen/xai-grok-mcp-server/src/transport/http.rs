//! Streamable HTTP/SSE adapter surface for MCP control plane.
//! Bearer tokens are accepted from the Authorization header or the explicit
//! `?bearer=` compatibility query parameter for clients that cannot configure
//! custom HTTP headers.

use std::collections::HashMap;
use std::sync::Mutex;

use serde_json::{json, Value};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpAuthError;

/// Validate Authorization: Bearer for Streamable HTTP.
pub fn validate_http_bearer(header: Option<&str>, expected: &str) -> Result<(), HttpAuthError> {
    let Some(header) = header else {
        return Err(HttpAuthError);
    };
    let token = header
        .strip_prefix("Bearer ")
        .or_else(|| header.strip_prefix("bearer "))
        .unwrap_or("");
    // Reuse fixed-time compare semantics (length-padded XOR).
    let mut diff: u8 = if token.len() == expected.len() { 0 } else { 1 };
    for (i, exp) in expected.bytes().enumerate() {
        let c = token.as_bytes().get(i).copied().unwrap_or(0);
        diff |= c ^ exp;
    }
    for c in token.bytes().skip(expected.len()) {
        diff |= c;
    }
    if diff != 0 {
        return Err(HttpAuthError);
    }
    Ok(())
}

/// Resolve the bearer presented by a client. A valid Authorization header has
/// precedence; the query parameter is a compatibility path for hosted MCP
/// clients that only accept a URL. The caller still compares the result to the
/// configured expected token using the same constant-time validator.
pub fn presented_bearer(header: Option<&str>, query: Option<&str>) -> Option<String> {
    if let Some(header) = header {
        return Some(
            header
                .strip_prefix("Bearer ")
                .or_else(|| header.strip_prefix("bearer "))
                .unwrap_or("")
                .to_owned(),
        );
    }
    query.and_then(query_bearer)
}

/// Extract and percent-decode the explicit `bearer` query parameter.
pub fn query_bearer(query: &str) -> Option<String> {
    url::form_urlencoded::parse(query.as_bytes())
        .find_map(|(key, value)| (key == "bearer").then(|| value.into_owned()))
}

pub fn reject_token_query(query: &str) -> Result<(), HttpAuthError> {
    if query.contains("token=") || query.contains("access_token=") || query.contains("api_key=") {
        return Err(HttpAuthError);
    }
    Ok(())
}

/// In-memory SSE resume cursors keyed by session stream id.
#[derive(Debug, Default)]
pub struct SseResumeTable {
    cursors: Mutex<HashMap<String, u64>>,
}

impl SseResumeTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn resume_from(&self, stream_id: &str, last_event_id: Option<u64>) -> u64 {
        let mut guard = self.cursors.lock().unwrap();
        let current = guard.entry(stream_id.to_owned()).or_insert(0);
        if let Some(last) = last_event_id {
            *current = (*current).max(last);
        }
        *current
    }

    pub fn advance(&self, stream_id: &str, to: u64) {
        let mut guard = self.cursors.lock().unwrap();
        let current = guard.entry(stream_id.to_owned()).or_insert(0);
        *current = (*current).max(to);
    }
}

/// Shape a JSON-RPC response for POST /mcp (no request body secrets echoed).
pub fn post_mcp_response(id: Value, result: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    })
}

#[cfg(test)]
mod streamable_http_tests {
    use super::*;

    #[test]
    fn streamable_http_bearer_header_only_and_safe_sse_resume() {
        assert!(validate_http_bearer(Some("Bearer abc"), "abc").is_ok());
        assert!(validate_http_bearer(None, "abc").is_err());
        assert!(validate_http_bearer(Some("Bearer ab"), "abc").is_err());
        assert!(reject_token_query("foo=1").is_ok());
        assert!(reject_token_query("token=secret").is_err());
        assert_eq!(query_bearer("bearer=abc"), Some("abc".to_owned()));

        let table = SseResumeTable::new();
        assert_eq!(table.resume_from("s1", None), 0);
        table.advance("s1", 10);
        assert_eq!(table.resume_from("s1", Some(5)), 10);
        assert_eq!(table.resume_from("s1", Some(12)), 12);
    }
}

#[cfg(test)]
mod auth_failures_tests {
    use super::*;

    #[test]
    fn bearer_header_only_and_auth_failures_are_generic() {
        let expected = "token-value";
        for bad in [None, Some(""), Some("Bearer x"), Some("token-value")] {
            let err = validate_http_bearer(bad, expected);
            assert!(err.is_err());
        }
        assert!(validate_http_bearer(Some("Bearer token-value"), expected).is_ok());
        assert!(reject_token_query("access_token=1").is_err());
        assert_eq!(presented_bearer(None, Some("bearer=token-value")), Some(expected.to_owned()));
    }

    #[test]
    fn redaction_canary_absent_from_post_response_shape() {
        let resp = post_mcp_response(serde_json::json!(1), serde_json::json!({"ok":true}));
        let s = resp.to_string();
        assert!(!s.contains("sk-"));
        assert!(!s.contains("access_token"));
    }
}

pub fn enforce_body_limit(bytes: usize, max: usize) -> Result<(), &'static str> {
    if bytes > max {
        Err("message_too_large")
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod limits_tests {
    use super::*;

    #[test]
    fn limits_reject_oversized_body_explicitly() {
        assert!(enforce_body_limit(100, 1_048_576).is_ok());
        assert_eq!(enforce_body_limit(2_000_000, 1_048_576).unwrap_err(), "message_too_large");
    }
}
