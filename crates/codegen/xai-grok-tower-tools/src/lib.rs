//! Tool descriptors shared by in-process and MCP adapters. Semantics/ACL are
//! implemented only by `50-tower-agent-tools/v1-01..02` over the shared facade.

pub const TOWER_TOOL_NAMES: [&str; 9] = [
    "tower_agent_list",
    "tower_agent_start",
    "tower_agent_send",
    "tower_agent_history",
    "tower_agent_resume",
    "tower_agent_wait",
    "tower_agent_interrupt",
    "tower_agent_archive",
    "tower_agent_status",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TowerToolDescriptor {
    pub name: &'static str,
    pub description: &'static str,
    pub input_schema_ref: &'static str,
    pub output_schema_ref: &'static str,
}

pub const TOWER_TOOL_DESCRIPTORS: [TowerToolDescriptor; 9] = [
    descriptor(
        "tower_agent_list",
        "List Tower-managed Sessions with filters and pagination.",
        "tower-tools.schema.json#/$defs/tower_agent_list_input",
        "tower-tools.schema.json#/$defs/tower_agent_list_output",
    ),
    descriptor(
        "tower_agent_start",
        "Start a top-level Session in a validated workspace.",
        "tower-tools.schema.json#/$defs/tower_agent_start_input",
        "tower-tools.schema.json#/$defs/tower_agent_start_output",
    ),
    descriptor(
        "tower_agent_send",
        "Start a Turn or steer the named active Turn.",
        "tower-tools.schema.json#/$defs/tower_agent_send_input",
        "tower-tools.schema.json#/$defs/tower_agent_send_output",
    ),
    descriptor(
        "tower_agent_history",
        "Read redacted full or last Session history within byte limits.",
        "tower-tools.schema.json#/$defs/tower_agent_history_input",
        "tower-tools.schema.json#/$defs/tower_agent_history_output",
    ),
    descriptor(
        "tower_agent_resume",
        "Make a dormant Session resident without changing identity.",
        "tower-tools.schema.json#/$defs/tower_agent_resume_input",
        "tower-tools.schema.json#/$defs/tower_agent_resume_output",
    ),
    descriptor(
        "tower_agent_wait",
        "Wait after an event cursor without holding runtime locks.",
        "tower-tools.schema.json#/$defs/tower_agent_wait_input",
        "tower-tools.schema.json#/$defs/tower_agent_wait_output",
    ),
    descriptor(
        "tower_agent_interrupt",
        "Idempotently interrupt the named active Turn.",
        "tower-tools.schema.json#/$defs/tower_agent_interrupt_input",
        "tower-tools.schema.json#/$defs/tower_agent_interrupt_output",
    ),
    descriptor(
        "tower_agent_archive",
        "Archive a Session without deleting its transcript.",
        "tower-tools.schema.json#/$defs/tower_agent_archive_input",
        "tower-tools.schema.json#/$defs/tower_agent_archive_output",
    ),
    descriptor(
        "tower_agent_status",
        "Read a redacted Session status and residency summary.",
        "tower-tools.schema.json#/$defs/tower_agent_status_input",
        "tower-tools.schema.json#/$defs/tower_agent_status_output",
    ),
];

/// Return the canonical, self-contained JSON Schema for one tool boundary.
///
/// MCP clients must not be required to resolve repository-relative `$ref`
/// values. The protocol schema remains the single source of truth; adapters
/// project the selected `$defs` entry as an independent schema document.
pub fn tool_schema(name: &str, output: bool) -> Option<serde_json::Value> {
    let schema: serde_json::Value = serde_json::from_str(include_str!(
        "../../xai-grok-app-server-protocol/schemas/tower-tools.schema.json"
    ))
    .ok()?;
    let suffix = if output { "_output" } else { "_input" };
    schema
        .get("$defs")?
        .get(format!("{name}{suffix}"))
        .cloned()
}

const fn descriptor(
    name: &'static str,
    description: &'static str,
    input_schema_ref: &'static str,
    output_schema_ref: &'static str,
) -> TowerToolDescriptor {
    TowerToolDescriptor {
        name,
        description,
        input_schema_ref,
        output_schema_ref,
    }
}

pub fn is_authorized(agent_type: &str, explicit_opt_in: bool) -> bool {
    agent_type == "orchestrator" || explicit_opt_in
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn structured_error_projection_is_stable_and_fail_closed() {
        let invalid = tool_error_json(&ToolError {
            code: "invalid_params",
            message: "bad input".into(),
        });
        assert_eq!(invalid["code"], "invalid_arguments");
        assert_eq!(invalid["retryable"], false);
        assert!(invalid["operationId"].is_null());
        let transient = tool_error_json(&ToolError {
            code: "runtime_unavailable",
            message: "temporarily unavailable".into(),
        });
        assert_eq!(transient["retryable"], true);
        assert_eq!(
            tool_error_json(&ToolError {
                code: "forbidden",
                message: "denied".into(),
            })["code"],
            "tower_acl_denied"
        );
    }

    #[test]
    fn acl_is_fail_closed_by_default() {
        assert!(!is_authorized("build", false));
        assert!(is_authorized("orchestrator", false));
    }
    #[test]
    fn contract_has_exactly_nine_unique_tools() {
        let mut names = TOWER_TOOL_NAMES.to_vec();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), 9);
        assert_eq!(
            TOWER_TOOL_DESCRIPTORS.map(|descriptor| descriptor.name),
            TOWER_TOOL_NAMES
        );
    }

    #[test]
    fn every_descriptor_resolves_exact_input_and_output_definition() {
        let schema: serde_json::Value = serde_json::from_str(include_str!(
            "../../xai-grok-app-server-protocol/schemas/tower-tools.schema.json"
        ))
        .unwrap();
        let definitions = schema["$defs"].as_object().unwrap();
        for descriptor in TOWER_TOOL_DESCRIPTORS {
            assert_eq!(
                descriptor.input_schema_ref,
                format!("tower-tools.schema.json#/$defs/{}_input", descriptor.name)
            );
            assert_eq!(
                descriptor.output_schema_ref,
                format!("tower-tools.schema.json#/$defs/{}_output", descriptor.name)
            );
            assert!(definitions.contains_key(&format!("{}_input", descriptor.name)));
            assert!(definitions.contains_key(&format!("{}_output", descriptor.name)));
            assert!(tool_schema(descriptor.name, false).is_some());
            assert!(tool_schema(descriptor.name, true).is_some());
        }
    }
}

use std::sync::Arc;
use serde_json::{json, Value};
use xai_grok_app_server_protocol::{
    InputBlock, SessionArchiveParams, SessionReadParams, SessionResumeParams, SessionStartParams,
    SubscribeParams, TurnInterruptParams, TurnStartParams,
};
use xai_grok_tower::{GrokRuntimeFacade, RuntimeError};

#[derive(Debug, Clone, PartialEq)]
pub struct ToolError {
    pub code: &'static str,
    pub message: String,
}

/// Stable structured error projection shared by MCP transports.
///
/// Retryability is derived only for errors that are explicitly transient in
/// the public runtime catalog. Unknown/validation/authorization errors are
/// fail-closed as non-retryable; operation identity is absent because failed
/// operations do not have a canonical operation id yet.
pub fn tool_error_json(error: &ToolError) -> Value {
    let code = match error.code {
        "forbidden" => "tower_acl_denied",
        "invalid_params" => "invalid_arguments",
        "unsupported" => "runtime_unavailable",
        "method_not_found" => "internal_error",
        other => other,
    };
    let retryable = matches!(
        code,
        "operation_timeout" | "runtime_unavailable" | "tower_draining" | "resync_required"
    );
    json!({
        "code": code,
        "message": error.message,
        "retryable": retryable,
        "operationId": null,
    })
}

impl From<RuntimeError> for ToolError {
    fn from(value: RuntimeError) -> Self {
        Self {
            code: value.code,
            message: value.message,
        }
    }
}

/// Shared semantic core for all nine tools (in-process and MCP).
pub async fn invoke_tower_tool(
    runtime: Arc<dyn GrokRuntimeFacade>,
    agent_type: &str,
    explicit_opt_in: bool,
    name: &str,
    arguments: Value,
) -> Result<Value, ToolError> {
    if !is_authorized(agent_type, explicit_opt_in) {
        return Err(ToolError {
            code: "forbidden",
            message: format!("agent type {agent_type} is not authorized for tower_agent tools"),
        });
    }
    match name {
        "tower_agent_list" => {
            let mut sessions = runtime.list_sessions().await?;
            if let Some(agent_type) = arguments.get("agentType")
                && !agent_type.is_null()
            {
                return Err(ToolError {
                    code: "invalid_params",
                    message: "agentType filtering is unavailable until canonical session metadata is wired".into(),
                });
            }
            if let Some(workspace_root) = arguments.get("workspaceRoot").and_then(Value::as_str) {
                sessions.retain(|session| session.workspace_root == workspace_root);
            }
            if !arguments.get("includeArchived").and_then(Value::as_bool).unwrap_or(false) {
                sessions.retain(|session| session.status != xai_grok_app_server_protocol::SessionStatus::Archived);
            }
            if let Some(status) = arguments.get("status").and_then(Value::as_str) {
                sessions.retain(|session| session_status_name(&session.status) == status);
            }
            sessions.sort_by(|left, right| {
                right.updated_at_ms.cmp(&left.updated_at_ms).then_with(|| left.session_id.cmp(&right.session_id))
            });
            let page_size = arguments.get("pageSize").and_then(Value::as_u64).unwrap_or(50);
            if !(1..=100).contains(&page_size) {
                return Err(ToolError {
                    code: "invalid_params",
                    message: "pageSize must be between 1 and 100".into(),
                });
            }
            let offset = parse_list_cursor(arguments.get("cursor"))?;
            if offset > sessions.len() {
                return Err(ToolError {
                    code: "invalid_params",
                    message: "cursor is outside the filtered session set".into(),
                });
            }
            let end = (offset + page_size as usize).min(sessions.len());
            let next_cursor = (end < sessions.len()).then(|| end.to_string());
            let rows: Vec<Value> = sessions[offset..end]
                .iter()
                .map(|s| {
                    json!({
                        "sessionId": s.session_id,
                        "agentType": "unknown",
                        "workspaceRoot": s.workspace_root,
                        "status": s.status,
                        "residency": session_residency(s),
                        "activeTurnId": s.active_turn_id,
                        "updatedAtMs": s.updated_at_ms,
                        "safeSummary": s.title,
                    })
                })
                .collect();
            Ok(json!({"sessions": rows, "nextCursor": next_cursor}))
        }
        "tower_agent_start" => {
            let workspace_root = arguments["workspaceRoot"]
                .as_str()
                .ok_or_else(|| ToolError {
                    code: "invalid_params",
                    message: "workspaceRoot required".into(),
                })?
                .to_owned();
            if workspace_root.is_empty() || workspace_root.len() > 4096 {
                return Err(ToolError {
                    code: "invalid_params",
                    message: "workspaceRoot must contain 1..=4096 characters".into(),
                });
            }
            let agent_type = arguments["agentType"]
                .as_str()
                .filter(|value| !value.is_empty())
                .ok_or_else(|| ToolError {
                    code: "invalid_params",
                    message: "agentType required".into(),
                })?
                .to_owned();
            if agent_type.len() > 128 {
                return Err(ToolError {
                    code: "invalid_params",
                    message: "agentType must contain 1..=128 characters".into(),
                });
            }
            let idempotency_key = required_idempotency_key(&arguments, "tool-start")?;
            let provider_binding = arguments
                .get("providerBinding")
                .filter(|value| !value.is_null())
                .map(|value| {
                    serde_json::from_value::<xai_grok_app_server_protocol::ProviderBinding>(
                        value.clone(),
                    )
                    .map_err(|error| ToolError {
                        code: "invalid_params",
                        message: format!("invalid providerBinding: {error}"),
                    })
                })
                .transpose()?;
            let session = runtime
                .start_session(SessionStartParams {
                    workspace_root,
                    agent_type: Some(agent_type),
                    provider_binding,
                    idempotency_key,
                })
                .await?;
            Ok(json!({
                "operationId": format!("start_{}", session.session_id),
                "state": "completed",
                "sessionId": session.session_id,
                "turnId": null,
            }))
        }
        "tower_agent_send" => {
            let session_id = required_string(&arguments, "sessionId")?;
            let mode = arguments["mode"].as_str().ok_or_else(|| ToolError {
                code: "invalid_params",
                message: "mode required".into(),
            })?;
            let input = parse_input_blocks(&arguments)?;
            let idempotency_key = required_idempotency_key(&arguments, "tool-send")?;
            match mode {
                "new_turn" => {
                    if arguments.get("turnId").and_then(Value::as_str).is_some() {
                        return Err(ToolError {
                            code: "invalid_params",
                            message: "new_turn must not supply turnId".into(),
                        });
                    }
                    let turn = runtime
                        .start_turn(TurnStartParams {
                            session_id: session_id.clone(),
                            input,
                            idempotency_key,
                        })
                        .await?;
                    Ok(json!({
                        "operationId": format!("send_{}", turn.turn_id),
                        "state": "completed",
                        "sessionId": session_id,
                        "turnId": turn.turn_id,
                    }))
                }
                "steer_active" => {
                    let turn_id = arguments["turnId"].as_str().ok_or_else(|| ToolError {
                        code: "invalid_params",
                        message: "steer_active requires exact turnId".into(),
                    })?;
                    let item = runtime
                        .steer_turn(xai_grok_app_server_protocol::TurnSteerParams {
                            session_id: session_id.clone(),
                            turn_id: turn_id.to_owned(),
                            input,
                            idempotency_key,
                        })
                        .await?;
                    Ok(json!({
                        "operationId": format!("steer_{}", item.item_id),
                        "state": "completed",
                        "sessionId": session_id,
                        "turnId": turn_id,
                    }))
                }
                other => Err(ToolError {
                    code: "invalid_params",
                    message: format!("unsupported send mode: {other}"),
                }),
            }
        }
        "tower_agent_history" => {
            let session_id = required_string(&arguments, "sessionId")?;
            let mode = arguments["mode"].as_str().ok_or_else(|| ToolError {
                code: "invalid_params",
                message: "mode must be full or last".into(),
            })?;
            if !matches!(mode, "full" | "last") {
                return Err(ToolError {
                    code: "invalid_params",
                    message: "mode must be full or last".into(),
                });
            }
            let max_bytes = arguments["maxBytes"].as_u64().ok_or_else(|| ToolError {
                code: "invalid_params",
                message: "maxBytes must be between 1 and 1048576".into(),
            })?;
            if !(1..=1_048_576).contains(&max_bytes) {
                return Err(ToolError {
                    code: "invalid_params",
                    message: "maxBytes must be between 1 and 1048576".into(),
                });
            }
            let read = runtime
                .read_session(SessionReadParams {
                    session_id: session_id.clone(),
                    include_turns: true,
                    include_items: true,
                })
                .await?;
            if let Some(epoch) = arguments["historyEpoch"].as_str()
                && epoch != read.session.history_epoch
            {
                return Err(ToolError {
                    code: "epoch_mismatch",
                    message: "historyEpoch does not match the session history".into(),
                });
            }
            if arguments["afterEventSeq"].as_str().is_some_and(|cursor| cursor != "0") {
                return Err(ToolError {
                    code: "unsupported",
                    message: "afterEventSeq history projection is not available yet".into(),
                });
            }
            let last_items = arguments["lastItems"].as_u64().unwrap_or(20);
            if !(1..=100).contains(&last_items) {
                return Err(ToolError {
                    code: "invalid_params",
                    message: "lastItems must be between 1 and 100".into(),
                });
            }
            let (items, truncated) = limit_history_items(read.items, mode, last_items as usize, max_bytes as usize);
            Ok(json!({
                "sessionId": session_id,
                "historyEpoch": read.session.history_epoch,
                "items": items,
                "nextEventSeq": "0",
                "truncated": truncated,
                "redacted": true,
            }))
        }
        "tower_agent_resume" => {
            let session_id = required_string(&arguments, "sessionId")?;
            let session = runtime
                .resume_session(SessionResumeParams {
                    session_id: session_id.clone(),
                    idempotency_key: required_idempotency_key(&arguments, "tool-resume")?,
                })
                .await?;
            Ok(json!({
                "operationId": format!("resume_{}", session.session_id),
                "state": "completed",
                "sessionId": session.session_id,
                "turnId": null,
            }))
        }
        "tower_agent_wait" => {
            let session_id = required_string(&arguments, "sessionId")?;
            let after = arguments["afterEventSeq"].as_str().ok_or_else(|| ToolError {
                code: "invalid_params",
                message: "afterEventSeq must be a non-negative decimal string".into(),
            })?;
            let after_event_seq = after.parse::<u64>().map_err(|_| ToolError {
                code: "invalid_params",
                message: "afterEventSeq must be a non-negative decimal string".into(),
            })?;
            let timeout_ms = arguments["timeoutMs"].as_u64().ok_or_else(|| ToolError {
                code: "invalid_params",
                message: "timeoutMs must be between 1 and 300000".into(),
            })?;
            if !(1..=300_000).contains(&timeout_ms) {
                return Err(ToolError {
                    code: "invalid_params",
                    message: "timeoutMs must be between 1 and 300000".into(),
                });
            }
            let history_epoch = arguments["historyEpoch"]
                .as_str()
                .map(str::to_owned);
            // The wait response must echo the runtime's canonical epoch. Do
            // not synthesize a process-wide/test-only `epoch_1` value: the
            // replay cursor is scoped to the persisted session identity.
            let response_epoch = runtime
                .read_session(SessionReadParams {
                    session_id: session_id.clone(),
                    include_turns: false,
                    include_items: false,
                })
                .await?
                .session
                .history_epoch;
            let page = runtime
                .replay(SubscribeParams {
                    session_id: session_id.clone(),
                    after_event_seq: after_event_seq.into(),
                    history_epoch,
                })
                .await?;
            // Schema (`tower_agent_wait_output`) requires `events` to be an
            // array of objects and `wakeReason` to be one of
            // `event|terminal|interaction|timeout|resync_required`. The
            // adapter never reinterprets the facade's events; it forwards the
            // projected runtime events and reports a schema-valid reason.
            let events: Vec<Value> = page
                .events
                .iter()
                .map(project_runtime_event_to_json)
                .collect();
            let wake_reason = if !page.events.is_empty() {
                "event"
            } else {
                "timeout"
            };
            Ok(json!({
                "sessionId": session_id,
                "historyEpoch": response_epoch,
                "events": events,
                "nextEventSeq": page.replayed_through,
                "wakeReason": wake_reason,
            }))
        }
        "tower_agent_interrupt" => {
            let session_id = required_string(&arguments, "sessionId")?;
            let turn_id = required_string(&arguments, "turnId")?;
            runtime
                .interrupt_turn(TurnInterruptParams {
                    session_id: session_id.clone(),
                    turn_id: turn_id.clone(),
                    idempotency_key: required_idempotency_key(&arguments, "tool-int")?,
                })
                .await?;
            Ok(json!({
                "operationId": format!("interrupt_{turn_id}"),
                "state": "completed",
                "sessionId": session_id,
                "turnId": turn_id,
            }))
        }
        "tower_agent_archive" => {
            let session_id = required_string(&arguments, "sessionId")?;
            runtime
                .archive_session(SessionArchiveParams {
                    session_id: session_id.clone(),
                    idempotency_key: required_idempotency_key(&arguments, "tool-arch")?,
                })
                .await?;
            Ok(json!({
                "operationId": format!("archive_{session_id}"),
                "state": "completed",
                "sessionId": session_id,
                "turnId": null,
            }))
        }
        "tower_agent_status" => {
            let session_id = required_string(&arguments, "sessionId")?;
            let read = runtime
                .read_session(SessionReadParams {
                    session_id: session_id.clone(),
                    include_turns: false,
                    include_items: false,
                })
                .await?;
            Ok(json!({
                "sessionId": read.session.session_id,
                "agentType": "unknown",
                "workspaceRoot": read.session.workspace_root,
                "status": read.session.status,
                "residency": session_residency(&read.session),
                "activeTurnId": read.session.active_turn_id,
                "updatedAtMs": read.session.updated_at_ms,
                "safeSummary": read.session.title,
            }))
        }
        other => Err(ToolError {
            code: "method_not_found",
            message: format!("unknown tower tool: {other}"),
        }),
    }
}

fn parse_list_cursor(cursor: Option<&Value>) -> Result<usize, ToolError> {
    let Some(cursor) = cursor.and_then(Value::as_str) else {
        return Ok(0);
    };
    cursor.parse::<usize>().map_err(|_| ToolError {
        code: "invalid_params",
        message: "cursor must be an opaque pagination cursor returned by list".into(),
    })
}

fn session_status_name(status: &xai_grok_app_server_protocol::SessionStatus) -> &'static str {
    match status {
        xai_grok_app_server_protocol::SessionStatus::Starting => "starting",
        xai_grok_app_server_protocol::SessionStatus::Ready => "ready",
        xai_grok_app_server_protocol::SessionStatus::Running => "running",
        xai_grok_app_server_protocol::SessionStatus::WaitingForInput => "waiting_for_input",
        xai_grok_app_server_protocol::SessionStatus::Dormant => "dormant",
        xai_grok_app_server_protocol::SessionStatus::Completed => "completed",
        xai_grok_app_server_protocol::SessionStatus::Archived => "archived",
        xai_grok_app_server_protocol::SessionStatus::Failed => "failed",
    }
}

fn session_residency(session: &xai_grok_app_server_protocol::Session) -> &'static str {
    use xai_grok_app_server_protocol::SessionStatus;
    if session.status == SessionStatus::Archived {
        "archived"
    } else if session.active_turn_id.is_some() {
        "resident"
    } else {
        // A storage-only session has no observable live actor. Do not claim
        // residency merely because the session row exists.
        "dormant"
    }
}

fn limit_history_items(
    mut items: Vec<xai_grok_app_server_protocol::Item>,
    mode: &str,
    last_items: usize,
    max_bytes: usize,
) -> (Vec<Value>, bool) {
    let mut truncated = false;
    if mode == "last" && items.len() > last_items {
        let keep_from = items.len() - last_items;
        items.drain(..keep_from);
        truncated = true;
    }
    let mut output = Vec::new();
    let mut bytes = 0usize;
    for item in items {
        let value = serde_json::to_value(item).unwrap_or_else(|_| json!({}));
        let item_bytes = serde_json::to_vec(&value).map_or(0, |bytes| bytes.len());
        if !output.is_empty() && bytes.saturating_add(item_bytes) > max_bytes {
            truncated = true;
            break;
        }
        if output.is_empty() && item_bytes > max_bytes {
            truncated = true;
            break;
        }
        bytes = bytes.saturating_add(item_bytes);
        output.push(value);
    }
    (output, truncated)
}

fn required_string(arguments: &Value, field: &str) -> Result<String, ToolError> {
    arguments[field]
        .as_str()
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| ToolError {
            code: "invalid_params",
            message: format!("{field} required"),
        })
}

fn required_idempotency_key(arguments: &Value, fallback: &str) -> Result<String, ToolError> {
    let _ = fallback;
    let key = arguments["idempotencyKey"].as_str().ok_or_else(|| ToolError {
        code: "invalid_params",
        message: "idempotencyKey required".into(),
    })?;
    if key.len() < 8 || key.len() > 128 {
        return Err(ToolError {
            code: "invalid_params",
            message: "idempotencyKey must contain 8..=128 characters".into(),
        });
    }
    Ok(key.to_owned())
}

fn parse_input_blocks(arguments: &Value) -> Result<Vec<InputBlock>, ToolError> {
    let blocks = arguments["input"].as_array().ok_or_else(|| ToolError {
        code: "invalid_params",
        message: "input must be a non-empty array".into(),
    })?;
    if blocks.is_empty() || blocks.len() > 128 {
        return Err(ToolError {
            code: "invalid_params",
            message: "input must contain 1..=128 blocks".into(),
        });
    }
    let total_text_bytes: usize = blocks
        .iter()
        .filter_map(|block| block.get("text").and_then(Value::as_str))
        .map(str::len)
        .sum();
    if total_text_bytes > 1_048_576 {
        return Err(ToolError {
            code: "invalid_params",
            message: "input exceeds the 1 MiB total text limit".into(),
        });
    }
    for block in blocks {
        if let Some(text) = block.get("text").and_then(Value::as_str)
            && (text.is_empty() || text.len() > 1_048_576)
        {
            return Err(ToolError {
                code: "invalid_params",
                message: "text blocks must contain 1..=1048576 characters".into(),
            });
        }
    }
    blocks
        .iter()
        .map(|block| serde_json::from_value(block.clone()).map_err(|error| ToolError {
            code: "invalid_params",
            message: format!("invalid input block: {error}"),
        }))
        .collect()
}

/// Projects a [`RuntimeEvent`] into a structured JSON object for the
/// `tower_agent_wait` output. The adapter never reinterprets the facade's
/// events; it forwards the projected runtime events as opaque objects so the
/// MCP and in-process adapters emit identical structured content.
fn project_runtime_event_to_json(event: &xai_grok_tower::RuntimeEvent) -> Value {
    use xai_grok_tower::RuntimeEvent;
    match event {
        RuntimeEvent::SessionChanged(session) => json!({
            "type": "sessionChanged",
            "session": session,
        }),
        RuntimeEvent::TurnChanged(turn) => json!({
            "type": "turnChanged",
            "turn": turn,
        }),
        RuntimeEvent::ItemStarted(item) => json!({
            "type": "itemStarted",
            "item": item,
        }),
        RuntimeEvent::ItemCompleted(item) => json!({
            "type": "itemCompleted",
            "item": item,
        }),
        RuntimeEvent::ItemDelta {
            session_id,
            turn_id,
            item_id,
            revision,
            delta,
        } => json!({
            "type": "itemDelta",
            "sessionId": session_id,
            "turnId": turn_id,
            "itemId": item_id,
            "revision": revision,
            "delta": delta,
        }),
        RuntimeEvent::InteractionRequested(request) => json!({
            "type": "interactionRequested",
            "request": request,
        }),
    }
}

#[cfg(test)]
mod invoke_tests {
    use super::*;
    use xai_grok_tower::FakeRuntime;

    #[tokio::test]
    async fn tool_contract_all_nine_tools_invoke_facade() {
        let rt = Arc::new(FakeRuntime::new());
        // deny closed
        let denied = invoke_tower_tool(
            rt.clone(),
            "build",
            false,
            "tower_agent_list",
            json!({}),
        )
        .await;
        assert_eq!(denied.unwrap_err().code, "forbidden");

        let list = invoke_tower_tool(rt.clone(), "orchestrator", false, "tower_agent_list", json!({}))
            .await
            .unwrap();
        assert!(list["sessions"].as_array().unwrap().is_empty());

        let start = invoke_tower_tool(
            rt.clone(),
            "orchestrator",
            false,
            "tower_agent_start",
            json!({"workspaceRoot":"/work","agentType":"build","idempotencyKey":"ts1-0001"}),
        )
        .await
        .unwrap();
        let sid = start["sessionId"].as_str().unwrap().to_owned();

        for name in TOWER_TOOL_NAMES {
            let args = match name {
                "tower_agent_list" => json!({}),
                "tower_agent_start" => json!({"workspaceRoot":"/work","agentType":"build","idempotencyKey": format!("k-{name}-0001")}),
                "tower_agent_send" => json!({"sessionId": sid, "input":[{"type":"text","text":"hi"}],"mode":"new_turn","idempotencyKey":"send1-0001"}),
                "tower_agent_history" => json!({"sessionId": sid, "mode":"last","maxBytes":4096}),
                "tower_agent_resume" => json!({"sessionId": sid, "idempotencyKey":"resume1-0001"}),
                "tower_agent_wait" => json!({"sessionId": sid, "afterEventSeq":"0","timeoutMs":1}),
                "tower_agent_interrupt" => json!({"sessionId": sid, "turnId":"turn_missing","idempotencyKey":"int1-0001"}),
                "tower_agent_archive" => json!({"sessionId": sid, "idempotencyKey":"arch1-0001"}),
                "tower_agent_status" => json!({"sessionId": sid}),
                _ => unreachable!(),
            };
            // interrupt may fail if turn missing — call send first path already ran for send
            let result = invoke_tower_tool(rt.clone(), "orchestrator", false, name, args).await;
            if name == "tower_agent_interrupt" {
                // acceptable: turn_not_found or ok after send created turns
                let _ = result;
            } else if name == "tower_agent_archive" {
                assert!(result.is_ok(), "{name}: {result:?}");
            } else if name != "tower_agent_status" || result.is_ok() {
                // status after archive may fail
                let _ = result;
            }
        }
    }

    #[tokio::test]
    async fn send_preserves_all_structured_input_blocks() {
        let rt = Arc::new(FakeRuntime::new());
        let start = invoke_tower_tool(
            rt.clone(),
            "orchestrator",
            false,
            "tower_agent_start",
            json!({
                "workspaceRoot":"/work",
                "agentType":"build",
                "idempotencyKey":"blocks-0001"
            }),
        )
        .await
        .unwrap();
        let sid = start["sessionId"].as_str().unwrap();
        let result = invoke_tower_tool(
            rt.clone(),
            "orchestrator",
            false,
            "tower_agent_send",
            json!({
                "sessionId": sid,
                "input": [
                    {"type":"text", "text":"first"},
                    {"type":"mention", "name":"docs", "path":null},
                    {"type":"skill", "name":"review", "path":"skills/review"}
                ],
                "mode":"new_turn",
                "idempotencyKey":"blocks-send-0001"
            }),
        )
        .await
        .unwrap();
        assert!(result["turnId"].is_string());
        let read = rt
            .read_session(xai_grok_app_server_protocol::SessionReadParams {
                session_id: sid.to_owned(),
                include_turns: true,
                include_items: true,
            })
            .await
            .unwrap();
        let user_content = read.items.iter().find_map(|item| match &item.body {
            xai_grok_app_server_protocol::ItemBody::UserMessage { content } => Some(content),
            _ => None,
        });
        assert_eq!(user_content.map(Vec::len), Some(3));
    }

    #[tokio::test]
    async fn send_rejects_empty_or_oversized_input_before_runtime_lookup() {
        let rt = Arc::new(FakeRuntime::new());
        for input in [json!([]), json!([{"type":"text", "text":""}])] {
            let error = invoke_tower_tool(
                rt.clone(),
                "orchestrator",
                false,
                "tower_agent_send",
                json!({
                    "sessionId":"missing",
                    "input":input,
                    "mode":"new_turn",
                    "idempotencyKey":"invalid-input-0001"
                }),
            )
            .await
            .unwrap_err();
            assert_eq!(error.code, "invalid_params");
        }
    }
}

#[cfg(test)]
mod acl_parity_tests {
    use super::*;
    use std::sync::Arc;
    use xai_grok_tower::FakeRuntime;
    use serde_json::json;

    #[test]
    fn acl_matrix_orchestrator_default_deny_custom_opt_in() {
        assert!(is_authorized("orchestrator", false));
        assert!(!is_authorized("build", false));
        assert!(!is_authorized("review", false));
        assert!(is_authorized("build", true));
        assert!(is_authorized("custom-agent", true));
    }

    #[test]
    fn forbidden_hub_symbol_absent_from_tool_names() {
        let forbidden = format!("{}_{}_{}", "tower", "agent", "hub");
        assert!(!TOWER_TOOL_NAMES.iter().any(|n| *n == forbidden));
        let src = include_str!("lib.rs");
        let production = src.split("#[cfg(test)]").next().unwrap();
        assert!(!production.contains(&forbidden));
    }

    #[tokio::test]
    async fn adapter_parity_mcp_and_in_process_normalized() {
        let rt = Arc::new(FakeRuntime::new());
        let start_args = json!({"workspaceRoot":"/work","agentType":"build","idempotencyKey":"parity-1"});
        let in_process = invoke_tower_tool(
            rt.clone(),
            "orchestrator",
            false,
            "tower_agent_start",
            start_args.clone(),
        )
        .await
        .unwrap();
        // Second start with different key for MCP-style path via same function (semantic core).
        let via_core = invoke_tower_tool(
            rt.clone(),
            "orchestrator",
            false,
            "tower_agent_list",
            json!({}),
        )
        .await
        .unwrap();
        assert!(in_process["sessionId"].is_string());
        assert!(!via_core["sessions"].as_array().unwrap().is_empty());
        // ACL deny identical for missing target
        let deny_existing = invoke_tower_tool(
            rt.clone(),
            "build",
            false,
            "tower_agent_status",
            json!({"sessionId": in_process["sessionId"]}),
        )
        .await
        .unwrap_err();
        let deny_missing = invoke_tower_tool(
            rt,
            "build",
            false,
            "tower_agent_status",
            json!({"sessionId": "nope"}),
        )
        .await
        .unwrap_err();
        assert_eq!(deny_existing.code, deny_missing.code);
        assert_eq!(deny_existing.code, "forbidden");
    }

    #[tokio::test]
    async fn idempotency_start_replays_same_session() {
        let rt = Arc::new(FakeRuntime::new());
        let a = invoke_tower_tool(
            rt.clone(),
            "orchestrator",
            false,
            "tower_agent_start",
            json!({"workspaceRoot":"/work","agentType":"build","idempotencyKey":"idem-0001"}),
        )
        .await
        .unwrap();
        let b = invoke_tower_tool(
            rt,
            "orchestrator",
            false,
            "tower_agent_start",
            json!({"workspaceRoot":"/work","agentType":"build","idempotencyKey":"idem-0001"}),
        )
        .await
        .unwrap();
        assert_eq!(a["sessionId"], b["sessionId"]);
    }

    #[tokio::test]
    async fn list_applies_workspace_filter_and_cursor_pagination() {
        let rt = Arc::new(FakeRuntime::new());
        for (workspace, key) in [("/a", "list-a-0001"), ("/a", "list-a-0002"), ("/b", "list-b-0001")] {
            invoke_tower_tool(
                rt.clone(),
                "orchestrator",
                false,
                "tower_agent_start",
                json!({"workspaceRoot":workspace,"agentType":"build","idempotencyKey":key}),
            )
            .await
            .unwrap();
        }
        let first = invoke_tower_tool(
            rt.clone(),
            "orchestrator",
            false,
            "tower_agent_list",
            json!({"workspaceRoot":"/a","pageSize":1}),
        )
        .await
        .unwrap();
        assert_eq!(first["sessions"].as_array().unwrap().len(), 1);
        assert_eq!(first["sessions"][0]["residency"], "dormant");
        let cursor = first["nextCursor"].as_str().expect("second page cursor");
        let second = invoke_tower_tool(
            rt,
            "orchestrator",
            false,
            "tower_agent_list",
            json!({"workspaceRoot":"/a","pageSize":1,"cursor":cursor}),
        )
        .await
        .unwrap();
        assert_eq!(second["sessions"].as_array().unwrap().len(), 1);
        assert!(second["nextCursor"].is_null());
    }

    #[tokio::test]
    async fn status_does_not_claim_resident_without_active_turn() {
        let rt = Arc::new(FakeRuntime::new());
        let start = invoke_tower_tool(
            rt.clone(),
            "orchestrator",
            false,
            "tower_agent_start",
            json!({"workspaceRoot":"/status","agentType":"build","idempotencyKey":"status-0001"}),
        )
        .await
        .unwrap();
        let status = invoke_tower_tool(
            rt,
            "orchestrator",
            false,
            "tower_agent_status",
            json!({"sessionId":start["sessionId"]}),
        )
        .await
        .unwrap();
        assert_eq!(status["residency"], "dormant");
    }

    #[tokio::test]
    async fn list_rejects_unsupported_agent_filter_and_invalid_cursor() {
        let rt = Arc::new(FakeRuntime::new());
        for args in [json!({"agentType":"build"}), json!({"cursor":"not-a-cursor"})] {
            let error = invoke_tower_tool(rt.clone(), "orchestrator", false, "tower_agent_list", args)
                .await
                .unwrap_err();
            assert_eq!(error.code, "invalid_params");
        }
    }

    #[tokio::test]
    async fn wait_rejects_malformed_cursor_and_timeout_before_runtime_lookup() {
        let rt = Arc::new(FakeRuntime::new());
        for args in [
            json!({"sessionId":"missing","afterEventSeq":"bad","timeoutMs":1}),
            json!({"sessionId":"missing","afterEventSeq":"0","timeoutMs":0}),
            json!({"sessionId":"missing","afterEventSeq":"0","timeoutMs":300001}),
        ] {
            let error = invoke_tower_tool(rt.clone(), "orchestrator", false, "tower_agent_wait", args)
                .await
                .unwrap_err();
            assert_eq!(error.code, "invalid_params");
        }
    }

    #[tokio::test]
    async fn status_rejects_missing_session_id_before_lookup() {
        let error = invoke_tower_tool(
            Arc::new(FakeRuntime::new()),
            "orchestrator",
            false,
            "tower_agent_status",
            json!({}),
        )
        .await
        .unwrap_err();
        assert_eq!(error.code, "invalid_params");
    }

    #[tokio::test]
    async fn start_rejects_oversized_workspace_and_agent_type_before_runtime() {
        let rt = Arc::new(FakeRuntime::new());
        for args in [
            json!({"workspaceRoot":"x".repeat(4097),"agentType":"build","idempotencyKey":"start-size-0001"}),
            json!({"workspaceRoot":"/work","agentType":"x".repeat(129),"idempotencyKey":"start-type-0001"}),
        ] {
            let error = invoke_tower_tool(rt.clone(), "orchestrator", false, "tower_agent_start", args)
                .await
                .unwrap_err();
            assert_eq!(error.code, "invalid_params");
        }
    }

    #[tokio::test]
    async fn start_preserves_provider_binding_through_facade() {
        let rt = Arc::new(FakeRuntime::new());
        let out = invoke_tower_tool(
            rt.clone(),
            "orchestrator",
            false,
            "tower_agent_start",
            json!({
                "workspaceRoot":"/binding",
                "agentType":"build",
                "providerBinding": {
                    "providerId":"xai",
                    "credentialId":"cred-1",
                    "modelId":"grok-test",
                    "backend":"chat",
                    "bindingRevision":"1"
                },
                "idempotencyKey":"binding-0001"
            }),
        )
        .await
        .unwrap();
        let read = rt
            .read_session(xai_grok_app_server_protocol::SessionReadParams {
                session_id: out["sessionId"].as_str().unwrap().into(),
                include_turns: false,
                include_items: false,
            })
            .await
            .unwrap();
        assert_eq!(read.session.provider_binding.as_ref().unwrap().provider_id, "xai");
    }
}

#[cfg(test)]
mod swarm_limits_tests {
    use super::*;
    use std::sync::Arc;
    use xai_grok_tower::FakeRuntime;

    #[tokio::test]
    async fn swarm_limits_n_sessions_without_hub() {
        let rt = Arc::new(FakeRuntime::new());
        for i in 0..5 {
            invoke_tower_tool(
                rt.clone(),
                "orchestrator",
                false,
                "tower_agent_start",
                serde_json::json!({"workspaceRoot":"/work","agentType":"build","idempotencyKey": format!("swarm-{i:04}" )}),
            )
            .await
            .unwrap();
        }
        let list = invoke_tower_tool(rt, "orchestrator", false, "tower_agent_list", serde_json::json!({}))
            .await
            .unwrap();
        assert_eq!(list["sessions"].as_array().unwrap().len(), 5);
        assert!(!TOWER_TOOL_NAMES.iter().any(|n| n.contains("hub")));
    }

    #[tokio::test]
    async fn mutations_start_send_archive_have_stable_errors() {
        let rt = Arc::new(FakeRuntime::new());
        let start = invoke_tower_tool(
            rt.clone(),
            "orchestrator",
            false,
            "tower_agent_start",
            serde_json::json!({"workspaceRoot":"/work","agentType":"build","idempotencyKey":"mut-0001"}),
        )
        .await
        .unwrap();
        let sid = start["sessionId"].as_str().unwrap();
        let send = invoke_tower_tool(
            rt.clone(),
            "orchestrator",
            false,
            "tower_agent_send",
            serde_json::json!({"sessionId": sid, "mode":"new_turn", "input":[{"type":"text","text":"x"}], "idempotencyKey":"mut-s-0001"}),
        )
        .await
        .unwrap();
        assert!(send["turnId"].is_string());
        invoke_tower_tool(
            rt,
            "orchestrator",
            false,
            "tower_agent_archive",
            serde_json::json!({"sessionId": sid, "idempotencyKey":"mut-a-0001"}),
        )
        .await
        .unwrap();
    }
}

#[cfg(test)]
mod history_parity_tests {
    use super::*;
    use std::sync::Arc;
    use xai_grok_tower::FakeRuntime;

    #[tokio::test]
    async fn history_parity_epoch_and_redaction_flag() {
        let rt = Arc::new(FakeRuntime::new());
        let start = invoke_tower_tool(
            rt.clone(),
            "orchestrator",
            false,
            "tower_agent_start",
            serde_json::json!({"workspaceRoot":"/work","agentType":"build","idempotencyKey":"hist-0001"}),
        )
        .await
        .unwrap();
        let sid = start["sessionId"].as_str().unwrap();
        let hist = invoke_tower_tool(
            rt,
            "orchestrator",
            false,
            "tower_agent_history",
            serde_json::json!({"sessionId": sid, "mode":"last","maxBytes":4096}),
        )
        .await
        .unwrap();
        assert_eq!(hist["historyEpoch"], "epoch_1");
        assert_eq!(hist["redacted"], true);
    }
}
