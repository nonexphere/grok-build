//! Stable numeric/string error catalog for experimental-v2.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// One row from the protocol error catalog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ErrorSpec {
    pub numeric: i64,
    pub code: &'static str,
    pub retryable: bool,
    pub message: &'static str,
}

/// Domain error payload carried in `error.data`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DomainErrorData {
    pub code: String,
    pub retryable: bool,
    #[serde(default)]
    pub operation_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
}

macro_rules! catalog {
    ($($name:ident => ($num:expr, $code:expr, $retry:expr, $msg:expr)),* $(,)?) => {
        $(
            pub const $name: ErrorSpec = ErrorSpec {
                numeric: $num,
                code: $code,
                retryable: $retry,
                message: $msg,
            };
        )*
        pub const ALL: &[ErrorSpec] = &[$($name),*];
    };
}

catalog! {
    PARSE_ERROR => (-32700, "parse_error", false, "Parse error"),
    INVALID_REQUEST => (-32600, "invalid_request", false, "Invalid Request"),
    METHOD_NOT_FOUND => (-32601, "method_not_found", false, "Method not found"),
    INVALID_PARAMS => (-32602, "invalid_params", false, "Invalid params"),
    INTERNAL_ERROR => (-32603, "internal_error", true, "Internal error"),
    UNAUTHORIZED => (-32001, "unauthorized", false, "Authentication required."),
    NOT_INITIALIZED => (-32002, "not_initialized", true, "Connection is not initialized."),
    ALREADY_INITIALIZED => (-32003, "already_initialized", false, "Connection is already initialized."),
    PROTOCOL_VERSION_UNSUPPORTED => (
        -32004,
        "protocol_version_unsupported",
        false,
        "Protocol version is not supported."
    ),
    SESSION_NOT_FOUND => (-32010, "session_not_found", false, "Session was not found."),
    TURN_NOT_FOUND => (-32011, "turn_not_found", false, "Turn was not found."),
    EPOCH_MISMATCH => (-32012, "epoch_mismatch", true, "History epoch does not match."),
    CURSOR_TOO_OLD => (-32013, "cursor_too_old", true, "Cursor is too old; resubscribe."),
    RESYNC_REQUIRED => (-32014, "resync_required", true, "Subscription must resync."),
    IDEMPOTENCY_CONFLICT => (
        -32015,
        "idempotency_conflict",
        false,
        "The idempotency key was already used with different input."
    ),
    INVALID_STATE => (-32016, "invalid_state", false, "The resource is not in a valid state."),
    INTERACTION_PENDING => (-32017, "interaction_pending", true, "An interaction is pending."),
    CONTROLLER_LEASE_REQUIRED => (
        -32018,
        "controller_lease_required",
        true,
        "A controller lease is required."
    ),
    INTERACTION_ALREADY_RESOLVED => (
        -32019,
        "interaction_already_resolved",
        false,
        "The interaction was already resolved."
    ),
    INVALID_WORKSPACE => (-32020, "invalid_workspace", false, "The workspace cannot be opened."),
    MESSAGE_TOO_LARGE => (-32021, "message_too_large", false, "Message exceeds the size limit."),
    BACKPRESSURE => (-32022, "backpressure", true, "Outbound queue is saturated."),
    TOWER_DRAINING => (-32023, "tower_draining", true, "Tower is draining."),
    RUNTIME_UNAVAILABLE => (-32024, "runtime_unavailable", true, "Runtime is unavailable."),
}

/// Default protocol limits used by initialize and backpressure policy.
pub mod defaults {
    pub const MAX_MESSAGE_BYTES: u64 = 1_048_576;
    pub const MAX_PAGE_SIZE: u32 = 100;
    pub const REPLAY_WINDOW_EVENTS: u64 = 10_000;
    pub const REPLAY_WINDOW_BYTES: u64 = 16 * 1024 * 1024;
    pub const OUTBOUND_QUEUE_EVENTS: u64 = 1_024;
    pub const INITIALIZE_TIMEOUT_MS: u64 = 10_000;
    pub const WS_PING_MS: u64 = 30_000;
    pub const WS_PONG_TIMEOUT_MS: u64 = 10_000;
    pub const TOOL_WAIT_MAX_MS: u64 = 300_000;
}

pub fn lookup(code: &str) -> Option<&'static ErrorSpec> {
    ALL.iter().find(|spec| spec.code == code)
}

pub fn lookup_numeric(numeric: i64) -> Option<&'static ErrorSpec> {
    ALL.iter().find(|spec| spec.numeric == numeric)
}

impl ErrorSpec {
    pub fn domain_data(&self) -> DomainErrorData {
        DomainErrorData {
            code: self.code.to_owned(),
            retryable: self.retryable,
            operation_id: None,
            field: None,
        }
    }

    pub fn rpc_error_value(&self) -> Value {
        json!({
            "code": self.numeric,
            "message": self.message,
            "data": self.domain_data(),
        })
    }
}

/// Canonical domain projection for a numeric JSON-RPC error. Unknown runtime
/// numbers fail closed to the catalog's internal error row so every adapter
/// emits the same typed shape.
pub fn domain_data_for_numeric(numeric: i64) -> DomainErrorData {
    lookup_numeric(numeric)
        .unwrap_or(&INTERNAL_ERROR)
        .domain_data()
}

/// Pure classify helper for pre-initialize method classes (connection gate).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitializeGateClass {
    AllowedInitialize,
    AlreadyInitialized,
    NotInitialized,
    HealthAllowed,
}

pub fn classify_pre_init(method: &str, already_initialized: bool) -> InitializeGateClass {
    match method {
        "initialize" if already_initialized => InitializeGateClass::AlreadyInitialized,
        "initialize" => InitializeGateClass::AllowedInitialize,
        "health" | "health/check" => InitializeGateClass::HealthAllowed,
        _ if !already_initialized => InitializeGateClass::NotInitialized,
        _ => InitializeGateClass::AllowedInitialize, // post-init methods pass this pure gate
    }
}

pub fn gate_error(class: InitializeGateClass) -> Option<&'static ErrorSpec> {
    match class {
        InitializeGateClass::AlreadyInitialized => Some(&ALREADY_INITIALIZED),
        InitializeGateClass::NotInitialized => Some(&NOT_INITIALIZED),
        InitializeGateClass::AllowedInitialize | InitializeGateClass::HealthAllowed => None,
    }
}

#[cfg(test)]
mod errors_tests {
    use super::*;

    #[test]
    fn errors_catalog_covers_jsonrpc_and_domain_codes() {
        assert_eq!(PARSE_ERROR.numeric, -32700);
        assert_eq!(NOT_INITIALIZED.numeric, -32002);
        assert_eq!(BACKPRESSURE.numeric, -32022);
        assert!(NOT_INITIALIZED.retryable);
        assert!(!UNAUTHORIZED.retryable);
        assert_eq!(ALL.len(), 24);
        for spec in ALL {
            assert_eq!(lookup(spec.code).unwrap().numeric, spec.numeric);
            assert_eq!(lookup_numeric(spec.numeric).unwrap().code, spec.code);
            let data = spec.domain_data();
            assert_eq!(data.code, spec.code);
            assert_eq!(data.retryable, spec.retryable);
            let wire = spec.rpc_error_value();
            assert_eq!(wire["code"], spec.numeric);
            assert_eq!(wire["data"]["code"], spec.code);
            assert_eq!(wire["data"]["retryable"], spec.retryable);
            assert_eq!(wire["data"]["operationId"], Value::Null);
        }
    }

    #[test]
    fn errors_defaults_match_contract_backpressure_policy() {
        assert_eq!(defaults::MAX_MESSAGE_BYTES, 1_048_576);
        assert_eq!(defaults::MAX_PAGE_SIZE, 100);
        assert_eq!(defaults::REPLAY_WINDOW_EVENTS, 10_000);
        assert_eq!(defaults::OUTBOUND_QUEUE_EVENTS, 1_024);
        assert_eq!(defaults::INITIALIZE_TIMEOUT_MS, 10_000);
    }

    #[test]
    fn errors_initialize_gate_maps_pre_init_classes() {
        assert_eq!(
            classify_pre_init("initialize", false),
            InitializeGateClass::AllowedInitialize
        );
        assert_eq!(
            classify_pre_init("initialize", true),
            InitializeGateClass::AlreadyInitialized
        );
        assert_eq!(
            classify_pre_init("session/start", false),
            InitializeGateClass::NotInitialized
        );
        assert!(gate_error(InitializeGateClass::NotInitialized).is_some());
        assert!(gate_error(InitializeGateClass::AllowedInitialize).is_none());
        assert_eq!(
            gate_error(InitializeGateClass::AlreadyInitialized)
                .unwrap()
                .code,
            "already_initialized"
        );
    }
}
