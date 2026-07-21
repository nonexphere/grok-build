//! Strict JSON-RPC 2.0 envelope validation for native experimental-v2 wire.

use serde_json::{Map, Value};

use crate::errors::{self, ErrorSpec};

#[derive(Debug, Clone, PartialEq)]
pub enum EnvelopeKind {
    Request {
        id: Value,
        method: String,
        params: Value,
    },
    Notification {
        method: String,
        params: Value,
    },
    Success {
        id: Value,
        result: Value,
    },
    Failure {
        id: Value,
        error: Value,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnvelopeError {
    pub spec: &'static ErrorSpec,
    pub detail: String,
}

/// Parse one top-level JSON object as a strict native JSON-RPC envelope.
/// Batch arrays are rejected.
pub fn parse_envelope(value: &Value) -> Result<EnvelopeKind, EnvelopeError> {
    let obj = value.as_object().ok_or_else(|| EnvelopeError {
        spec: &errors::INVALID_REQUEST,
        detail: "top-level value must be a JSON object".into(),
    })?;

    if value.is_array() {
        return Err(EnvelopeError {
            spec: &errors::INVALID_REQUEST,
            detail: "JSON-RPC batches are unsupported".into(),
        });
    }

    match obj.get("jsonrpc").and_then(Value::as_str) {
        Some("2.0") => {}
        Some(_) => {
            return Err(EnvelopeError {
                spec: &errors::INVALID_REQUEST,
                detail: "jsonrpc must be exactly \"2.0\"".into(),
            });
        }
        None => {
            return Err(EnvelopeError {
                spec: &errors::INVALID_REQUEST,
                detail: "jsonrpc field is required".into(),
            });
        }
    }

    let has_method = obj.contains_key("method");
    let has_result = obj.contains_key("result");
    let has_error = obj.contains_key("error");
    let has_id = obj.contains_key("id");

    if has_method && !has_result && !has_error {
        let method = obj
            .get("method")
            .and_then(Value::as_str)
            .filter(|m| !m.is_empty())
            .ok_or_else(|| EnvelopeError {
                spec: &errors::INVALID_REQUEST,
                detail: "method must be a non-empty string".into(),
            })?
            .to_owned();
        let params = obj.get("params").cloned().unwrap_or(Value::Null);
        if has_id {
            let id = obj.get("id").cloned().unwrap();
            validate_id(&id)?;
            Ok(EnvelopeKind::Request { id, method, params })
        } else {
            Ok(EnvelopeKind::Notification { method, params })
        }
    } else if has_result && !has_error && !has_method {
        if !has_id {
            return Err(EnvelopeError {
                spec: &errors::INVALID_REQUEST,
                detail: "success responses require id".into(),
            });
        }
        let id = obj.get("id").cloned().unwrap();
        validate_id(&id)?;
        Ok(EnvelopeKind::Success {
            id,
            result: obj.get("result").cloned().unwrap(),
        })
    } else if has_error && !has_result && !has_method {
        let id = obj.get("id").cloned().unwrap_or(Value::Null);
        if !id.is_null() {
            validate_id(&id)?;
        }
        let error = obj.get("error").cloned().unwrap();
        validate_error_object(&error)?;
        Ok(EnvelopeKind::Failure { id, error })
    } else {
        Err(EnvelopeError {
            spec: &errors::INVALID_REQUEST,
            detail: "message is not a valid request, notification, success, or failure".into(),
        })
    }
}

fn validate_id(id: &Value) -> Result<(), EnvelopeError> {
    match id {
        Value::String(_) | Value::Number(_) => Ok(()),
        Value::Null => Err(EnvelopeError {
            spec: &errors::INVALID_REQUEST,
            detail: "request id must not be null".into(),
        }),
        _ => Err(EnvelopeError {
            spec: &errors::INVALID_REQUEST,
            detail: "request id must be string or number".into(),
        }),
    }
}

fn validate_error_object(error: &Value) -> Result<(), EnvelopeError> {
    let obj = error.as_object().ok_or_else(|| EnvelopeError {
        spec: &errors::INVALID_REQUEST,
        detail: "error must be an object".into(),
    })?;
    if obj.get("code").and_then(Value::as_i64).is_none()
        && obj.get("code").and_then(Value::as_u64).is_none()
    {
        return Err(EnvelopeError {
            spec: &errors::INVALID_REQUEST,
            detail: "error.code must be a number".into(),
        });
    }
    if obj.get("message").and_then(Value::as_str).is_none() {
        return Err(EnvelopeError {
            spec: &errors::INVALID_REQUEST,
            detail: "error.message must be a string".into(),
        });
    }
    Ok(())
}

/// Allowed object keys for each envelope kind (unknown keys are invalid).
pub fn assert_no_unknown_request_keys(obj: &Map<String, Value>) -> Result<(), EnvelopeError> {
    for key in obj.keys() {
        if !matches!(key.as_str(), "jsonrpc" | "id" | "method" | "params") {
            return Err(EnvelopeError {
                spec: &errors::INVALID_REQUEST,
                detail: format!("unknown request field: {key}"),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod initialize_envelope_tests {
    use super::*;
    use crate::PROTOCOL_VERSION;
    use serde_json::json;

    #[test]
    fn initialize_envelope_accepts_strict_request_shape() {
        let value = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": PROTOCOL_VERSION,
                "clientInfo": {"name": "t", "version": "1"},
                "capabilities": {}
            }
        });
        match parse_envelope(&value).unwrap() {
            EnvelopeKind::Request { method, id, .. } => {
                assert_eq!(method, "initialize");
                assert_eq!(id, json!(1));
            }
            other => panic!("expected request, got {other:?}"),
        }
    }

    #[test]
    fn initialize_envelope_rejects_missing_jsonrpc_and_batches() {
        assert_eq!(
            parse_envelope(&json!({"id":1,"method":"initialize","params":{}}))
                .unwrap_err()
                .spec
                .code,
            "invalid_request"
        );
        assert_eq!(
            parse_envelope(&json!([{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}]))
                .unwrap_err()
                .spec
                .code,
            "invalid_request"
        );
        assert_eq!(
            parse_envelope(&json!({"jsonrpc":"1.0","id":1,"method":"initialize","params":{}}))
                .unwrap_err()
                .spec
                .code,
            "invalid_request"
        );
    }

    #[test]
    fn initialize_envelope_accepts_notification_and_failure_shapes() {
        let note = parse_envelope(&json!({
            "jsonrpc":"2.0",
            "method":"initialized",
            "params":{}
        }))
        .unwrap();
        assert!(
            matches!(note, EnvelopeKind::Notification { method, .. } if method == "initialized")
        );

        let fail = parse_envelope(&json!({
            "jsonrpc":"2.0",
            "id": 2,
            "error": {"code": -32002, "message": "Connection is not initialized.", "data": {"code":"not_initialized","retryable":true}}
        }))
        .unwrap();
        assert!(matches!(fail, EnvelopeKind::Failure { .. }));
    }
}
