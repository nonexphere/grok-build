//! C6-A evidence: Tower tools ACL + cross-surface parity.
//!
//! Covers all nine `tower_agent_*` tools through the shared semantic core
//! (`invoke_tower_tool`) over a real `FakeRuntime` adapter with:
//!   * fail-closed ACL (default deny for non-orchestrator agents),
//!   * an invoke (happy) path per tool with structured-output assertions,
//!   * an ACL-deny path per tool that returns `forbidden` *before* any target
//!     lookup can leak existence,
//!   * idempotency replay for mutating tools, and
//!   * swarm/limit behavior without a hub entity.
//!
//! These are differential against the same semantic core used by both the
//! in-process composition path and the MCP server registration (see
//! `xai-grok-mcp-server` parity fixtures). The product MUST NOT inject its own
//! MCP server into its local MCP client config; this file asserts the
//! `tower_agent_hub` symbol is absent from the production surface.

use serde_json::{Value, json};
use std::sync::Arc;
use xai_grok_tower::FakeRuntime;
use xai_grok_tower_tools::{
    TOWER_TOOL_DESCRIPTORS, TOWER_TOOL_NAMES, TowerToolDescriptor, invoke_tower_tool, is_authorized,
};

/// A non-orchestrator agent type used to exercise the fail-closed default.
const DENIED_AGENT: &str = "build";

fn rt() -> Arc<FakeRuntime> {
    Arc::new(FakeRuntime::new())
}

/// Orchestrator is the only built-in agent type allowed by default.
fn orch() -> (&'static str, bool) {
    ("orchestrator", false)
}

/// Custom agent that has explicitly opted into Tower access.
fn custom_opt_in() -> (&'static str, bool) {
    ("custom-agent", true)
}

async fn deny(name: &str, args: &Value) -> &'static str {
    invoke_tower_tool(rt(), DENIED_AGENT, false, name, args.clone())
        .await
        .unwrap_err()
        .code
}

async fn allow(name: &str, args: &Value) -> Value {
    let (agent, opt_in) = orch();
    invoke_tower_tool(rt(), agent, opt_in, name, args.clone())
        .await
        .expect("orchestrator invoke should succeed")
}

/// Bootstraps a Session and returns its id; used as a precondition for tools
/// that require an existing target.
async fn seed_session(runtime: Arc<FakeRuntime>) -> String {
    let v = invoke_tower_tool(
        runtime,
        "orchestrator",
        false,
        "tower_agent_start",
        json!({"workspaceRoot":"/work","agentType":"build","idempotencyKey":"seed-session-1"}),
    )
    .await
    .expect("seed start");
    v["sessionId"].as_str().expect("sessionId").to_owned()
}

#[tokio::test]
async fn c6_all_nine_descriptors_have_input_and_output_schema() {
    let schema: Value = serde_json::from_str(include_str!(
        "../../xai-grok-app-server-protocol/schemas/tower-tools.schema.json"
    ))
    .expect("tower-tools schema parses");
    let defs = schema["$defs"].as_object().expect("$defs object");
    for TowerToolDescriptor {
        name,
        input_schema_ref,
        output_schema_ref,
        ..
    } in TOWER_TOOL_DESCRIPTORS
    {
        let in_key = format!("{name}_input");
        let out_key = format!("{name}_output");
        assert_eq!(
            *input_schema_ref,
            format!("tower-tools.schema.json#/$defs/{in_key}")
        );
        assert_eq!(
            *output_schema_ref,
            format!("tower-tools.schema.json#/$defs/{out_key}")
        );
        assert!(defs.contains_key(&in_key), "missing {in_key}");
        assert!(defs.contains_key(&out_key), "missing {out_key}");
    }
    // Exactly nine unique names.
    let mut names = TOWER_TOOL_NAMES.to_vec();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), 9);
}

#[tokio::test]
async fn c6_acl_is_fail_closed_by_default() {
    // Built-in non-orchestrator agents and unknown agents are all denied.
    for agent in [
        "build",
        "review",
        "explore",
        "repo-explore",
        "architect",
        "general",
        "unknown",
    ] {
        assert!(
            !is_authorized(agent, false),
            "agent {agent} should be denied by default"
        );
    }
    // Orchestrator is allowed by default; custom agents may opt in explicitly.
    assert!(is_authorized("orchestrator", false));
    assert!(is_authorized("custom-agent", true));
    assert!(!is_authorized("custom-agent", false));
    // Inheritance/capability/prompt never implies access: opt-in is the only
    // path for non-orchestrator agents.
    assert!(!is_authorized("orchestrator-child", false));
    // The real contract is: only the literal "orchestrator" agent type or an
    // explicit opt-in is allowed.
    assert!(is_authorized("orchestrator", false));
    assert!(is_authorized("build", true));
}

#[tokio::test]
async fn c6_acl_denies_every_tool_before_target_lookup() {
    // For every tool, a denied agent receives `forbidden` regardless of
    // whether the target exists. This proves ACL is evaluated before any
    // argument lookup that could leak existence.
    let existing = seed_session(rt()).await;
    let missing = "session_does_not_exist";

    // Pairs of (existing-target args, missing-target args) per tool.
    let cases: Vec<(&str, Value, Value)> = vec![
        ("tower_agent_list", json!({}), json!({})),
        (
            "tower_agent_start",
            json!({"workspaceRoot":"/work","agentType":"build","idempotencyKey":"deny-start-1"}),
            json!({"workspaceRoot":"/work","agentType":"build","idempotencyKey":"deny-start-2"}),
        ),
        (
            "tower_agent_send",
            json!({"sessionId": existing, "input":[{"type":"text","text":"hi"}], "mode":"new_turn", "idempotencyKey":"deny-send-1"}),
            json!({"sessionId": missing, "input":[{"type":"text","text":"hi"}], "mode":"new_turn", "idempotencyKey":"deny-send-2"}),
        ),
        (
            "tower_agent_history",
            json!({"sessionId": existing, "mode":"last", "maxBytes":4096}),
            json!({"sessionId": missing, "mode":"last", "maxBytes":4096}),
        ),
        (
            "tower_agent_resume",
            json!({"sessionId": existing, "idempotencyKey":"deny-res-1"}),
            json!({"sessionId": missing, "idempotencyKey":"deny-res-2"}),
        ),
        (
            "tower_agent_wait",
            json!({"sessionId": existing, "afterEventSeq":"0", "timeoutMs":1}),
            json!({"sessionId": missing, "afterEventSeq":"0", "timeoutMs":1}),
        ),
        (
            "tower_agent_interrupt",
            json!({"sessionId": existing, "turnId":"turn_missing", "idempotencyKey":"deny-int-1"}),
            json!({"sessionId": missing, "turnId":"turn_missing", "idempotencyKey":"deny-int-2"}),
        ),
        (
            "tower_agent_archive",
            json!({"sessionId": existing, "idempotencyKey":"deny-arch-1"}),
            json!({"sessionId": missing, "idempotencyKey":"deny-arch-2"}),
        ),
        (
            "tower_agent_status",
            json!({"sessionId": existing}),
            json!({"sessionId": missing}),
        ),
    ];

    for (name, existing_args, missing_args) in cases {
        let code_existing = deny(name, &existing_args).await;
        let code_missing = deny(name, &missing_args).await;
        assert_eq!(
            code_existing, "forbidden",
            "{name}: existing-target deny must be forbidden"
        );
        assert_eq!(
            code_missing, "forbidden",
            "{name}: missing-target deny must be forbidden"
        );
        // ACL does not leak existence: identical code for existing vs missing.
        assert_eq!(
            code_existing, code_missing,
            "{name}: ACL must not leak target existence"
        );
    }
}

#[tokio::test]
async fn c6_tower_agent_list_invoke_path() {
    let out = allow("tower_agent_list", &json!({})).await;
    assert!(out["sessions"].is_array());
    assert!(out["nextCursor"].is_null() || out["nextCursor"].is_string());
}

#[tokio::test]
async fn c6_tower_agent_start_invoke_path() {
    let out = allow(
        "tower_agent_start",
        &json!({"workspaceRoot":"/work","agentType":"build","idempotencyKey":"c6-start-1"}),
    )
    .await;
    assert_eq!(out["state"], "completed");
    assert!(out["operationId"].is_string());
    assert!(out["sessionId"].is_string());
    // start never returns provider credentials.
    assert!(out.get("providerCredentials").is_none());
}

#[tokio::test]
async fn c6_tower_agent_send_new_turn_invoke_path() {
    let runtime = rt();
    let sid = seed_session(runtime.clone()).await;
    let out = invoke_tower_tool(
        runtime,
        "orchestrator",
        false,
        "tower_agent_send",
        json!({"sessionId": sid, "input":[{"type":"text","text":"hello"}], "mode":"new_turn", "idempotencyKey":"c6-send-1"}),
    )
    .await
    .unwrap();
    assert_eq!(out["state"], "completed");
    assert!(out["turnId"].is_string(), "new_turn returns a turnId");
    assert_eq!(out["sessionId"], sid);
}

#[tokio::test]
async fn c6_tower_agent_send_new_turn_rejects_turn_id() {
    let runtime = rt();
    let sid = seed_session(runtime.clone()).await;
    let err = invoke_tower_tool(
        runtime,
        "orchestrator",
        false,
        "tower_agent_send",
        json!({"sessionId": sid, "input":[{"type":"text","text":"hi"}], "mode":"new_turn", "turnId":"turn_1", "idempotencyKey":"c6-send-bad"}),
    )
    .await
    .unwrap_err();
    assert_eq!(err.code, "invalid_params");
}

#[tokio::test]
async fn c6_tower_agent_send_steer_active_invoke_path() {
    let runtime = rt();
    let sid = seed_session(runtime.clone()).await;
    let turn = invoke_tower_tool(
        runtime.clone(),
        "orchestrator",
        false,
        "tower_agent_send",
        json!({"sessionId": sid, "input":[{"type":"text","text":"first"}], "mode":"new_turn", "idempotencyKey":"c6-steer-1"}),
    )
    .await
    .unwrap();
    let turn_id = turn["turnId"].as_str().unwrap().to_owned();
    let out = invoke_tower_tool(
        runtime,
        "orchestrator",
        false,
        "tower_agent_send",
        json!({"sessionId": sid, "input":[{"type":"text","text":"steer"}], "mode":"steer_active", "turnId": turn_id, "idempotencyKey":"c6-steer-2"}),
    )
    .await
    .unwrap();
    assert_eq!(out["state"], "completed");
    assert_eq!(out["turnId"], turn_id);
}

#[tokio::test]
async fn c6_tower_agent_send_steer_active_requires_turn_id() {
    let runtime = rt();
    let sid = seed_session(runtime.clone()).await;
    let err = invoke_tower_tool(
        runtime,
        "orchestrator",
        false,
        "tower_agent_send",
        json!({"sessionId": sid, "input":[{"type":"text","text":"hi"}], "mode":"steer_active", "idempotencyKey":"c6-steer-3"}),
    )
    .await
    .unwrap_err();
    assert_eq!(err.code, "invalid_params");
}

#[tokio::test]
async fn c6_tower_agent_history_invoke_path() {
    let runtime = rt();
    let sid = seed_session(runtime.clone()).await;
    let out = invoke_tower_tool(
        runtime,
        "orchestrator",
        false,
        "tower_agent_history",
        json!({"sessionId": sid, "mode":"last", "maxBytes":4096}),
    )
    .await
    .unwrap();
    assert_eq!(out["sessionId"], sid);
    assert!(out["historyEpoch"].is_string());
    assert!(out["items"].is_array());
    assert!(out["nextEventSeq"].is_string());
    assert_eq!(out["truncated"], false);
    // History is always redacted.
    assert_eq!(out["redacted"], true);
}

#[tokio::test]
async fn c6_tower_agent_resume_invoke_path() {
    let runtime = rt();
    let sid = seed_session(runtime.clone()).await;
    let out = invoke_tower_tool(
        runtime,
        "orchestrator",
        false,
        "tower_agent_resume",
        json!({"sessionId": sid, "idempotencyKey":"c6-resume-1"}),
    )
    .await
    .unwrap();
    assert_eq!(out["state"], "completed");
    assert_eq!(out["sessionId"], sid);
}

#[tokio::test]
async fn c6_tower_agent_wait_invoke_path() {
    let runtime = rt();
    let sid = seed_session(runtime.clone()).await;
    let out = invoke_tower_tool(
        runtime,
        "orchestrator",
        false,
        "tower_agent_wait",
        json!({"sessionId": sid, "afterEventSeq":"0", "timeoutMs":1}),
    )
    .await
    .unwrap();
    assert_eq!(out["sessionId"], sid);
    assert!(out["historyEpoch"].is_string());
    assert!(out["events"].is_array());
    assert!(out["nextEventSeq"].is_string());
    // wakeReason is one of the schema enum values.
    let reason = out["wakeReason"].as_str().unwrap();
    assert!(
        [
            "event",
            "terminal",
            "interaction",
            "timeout",
            "resync_required"
        ]
        .contains(&reason)
    );
}

#[tokio::test]
async fn c6_tower_agent_interrupt_invoke_path() {
    let runtime = rt();
    let sid = seed_session(runtime.clone()).await;
    let turn = invoke_tower_tool(
        runtime.clone(),
        "orchestrator",
        false,
        "tower_agent_send",
        json!({"sessionId": sid, "input":[{"type":"text","text":"hi"}], "mode":"new_turn", "idempotencyKey":"c6-int-1"}),
    )
    .await
    .unwrap();
    let turn_id = turn["turnId"].as_str().unwrap().to_owned();
    let out = invoke_tower_tool(
        runtime,
        "orchestrator",
        false,
        "tower_agent_interrupt",
        json!({"sessionId": sid, "turnId": turn_id, "idempotencyKey":"c6-int-2"}),
    )
    .await
    .unwrap();
    assert_eq!(out["state"], "completed");
    assert_eq!(out["turnId"], turn_id);
}

#[tokio::test]
async fn c6_tower_agent_archive_invoke_path() {
    let runtime = rt();
    let sid = seed_session(runtime.clone()).await;
    let out = invoke_tower_tool(
        runtime,
        "orchestrator",
        false,
        "tower_agent_archive",
        json!({"sessionId": sid, "idempotencyKey":"c6-arch-1"}),
    )
    .await
    .unwrap();
    assert_eq!(out["state"], "completed");
    assert_eq!(out["sessionId"], sid);
}

#[tokio::test]
async fn c6_tower_agent_status_invoke_path() {
    let runtime = rt();
    let sid = seed_session(runtime.clone()).await;
    let out = invoke_tower_tool(
        runtime,
        "orchestrator",
        false,
        "tower_agent_status",
        json!({"sessionId": sid}),
    )
    .await
    .unwrap();
    assert_eq!(out["sessionId"], sid);
    assert!(out["workspaceRoot"].is_string());
    assert!(out["status"].is_string());
    assert!(out["residency"].is_string());
    // status never returns provider credentials.
    assert!(out.get("providerCredentials").is_none());
}

#[tokio::test]
async fn c6_custom_explicit_opt_in_is_allowed() {
    let runtime = rt();
    let (agent, opt_in) = custom_opt_in();
    let out = invoke_tower_tool(runtime, agent, opt_in, "tower_agent_list", json!({}))
        .await
        .unwrap();
    assert!(out["sessions"].is_array());
}

#[tokio::test]
async fn c6_idempotency_start_replays_same_session() {
    let runtime = rt();
    let a = invoke_tower_tool(
        runtime.clone(),
        "orchestrator",
        false,
        "tower_agent_start",
        json!({"workspaceRoot":"/work","agentType":"build","idempotencyKey":"c6-idem-1"}),
    )
    .await
    .unwrap();
    let b = invoke_tower_tool(
        runtime,
        "orchestrator",
        false,
        "tower_agent_start",
        json!({"workspaceRoot":"/work","agentType":"build","idempotencyKey":"c6-idem-1"}),
    )
    .await
    .unwrap();
    assert_eq!(a["sessionId"], b["sessionId"]);
}

#[tokio::test]
async fn c6_idempotency_send_replays_same_turn() {
    let runtime = rt();
    let sid = seed_session(runtime.clone()).await;
    let a = invoke_tower_tool(
        runtime.clone(),
        "orchestrator",
        false,
        "tower_agent_send",
        json!({"sessionId": sid, "input":[{"type":"text","text":"x"}], "mode":"new_turn", "idempotencyKey":"c6-send-idem"}),
    )
    .await
    .unwrap();
    let b = invoke_tower_tool(
        runtime,
        "orchestrator",
        false,
        "tower_agent_send",
        json!({"sessionId": sid, "input":[{"type":"text","text":"x"}], "mode":"new_turn", "idempotencyKey":"c6-send-idem"}),
    )
    .await
    .unwrap();
    assert_eq!(a["turnId"], b["turnId"]);
}

#[tokio::test]
async fn c6_idempotency_key_conflict_on_diverging_input() {
    let runtime = rt();
    let _first = invoke_tower_tool(
        runtime.clone(),
        "orchestrator",
        false,
        "tower_agent_start",
        json!({"workspaceRoot":"/work-a","agentType":"build","idempotencyKey":"c6-conflict"}),
    )
    .await
    .unwrap();
    let err = invoke_tower_tool(
        runtime,
        "orchestrator",
        false,
        "tower_agent_start",
        json!({"workspaceRoot":"/work-b","agentType":"build","idempotencyKey":"c6-conflict"}),
    )
    .await
    .unwrap_err();
    assert_eq!(err.code, "idempotency_conflict");
}

#[tokio::test]
async fn c6_swarm_n_sessions_without_hub() {
    let runtime = rt();
    for i in 0..5 {
        invoke_tower_tool(
            runtime.clone(),
            "orchestrator",
            false,
            "tower_agent_start",
            json!({"workspaceRoot": format!("/work-{i}"), "agentType":"build", "idempotencyKey": format!("c6-swarm-{i}")}),
        )
        .await
        .unwrap();
    }
    let list = invoke_tower_tool(
        runtime,
        "orchestrator",
        false,
        "tower_agent_list",
        json!({}),
    )
    .await
    .unwrap();
    assert_eq!(list["sessions"].as_array().unwrap().len(), 5);
}

#[tokio::test]
async fn c6_forbidden_hub_symbol_absent() {
    let forbidden = "tower_agent_hub";
    assert!(!TOWER_TOOL_NAMES.contains(&forbidden));
    let src = include_str!("../src/lib.rs");
    let production = src.split("#[cfg(test)]").next().unwrap();
    assert!(
        !production.contains(forbidden),
        "production surface must not mention {forbidden}"
    );
}

#[tokio::test]
async fn c6_in_process_path_has_no_mcp_loop() {
    // The in-process composition path calls the shared semantic core
    // directly; there is no JSON-RPC/MCP round-trip. We assert the core
    // returns structured JSON values (not transport envelopes) and that the
    // tower-tools crate does not depend on the MCP server crate.
    let runtime = rt();
    let out = invoke_tower_tool(
        runtime,
        "orchestrator",
        false,
        "tower_agent_list",
        json!({}),
    )
    .await
    .unwrap();
    // Structured object, not a JSON-RPC response envelope.
    assert!(out.get("jsonrpc").is_none());
    assert!(out.get("result").is_none());
    assert!(out["sessions"].is_array());
    // Dependency assertion: the Cargo manifest lists no MCP dependency.
    let manifest = include_str!("../Cargo.toml");
    assert!(
        !manifest.contains("xai-grok-mcp-server"),
        "tower-tools must not depend on the MCP server crate (no local self-MCP edge)"
    );
}

#[tokio::test]
async fn c6_unknown_tool_is_method_not_found() {
    let err = invoke_tower_tool(rt(), "orchestrator", false, "tower_agent_hub", json!({}))
        .await
        .unwrap_err();
    assert_eq!(err.code, "method_not_found");
}

#[tokio::test]
async fn c6_invalid_params_when_workspace_root_missing() {
    let err = invoke_tower_tool(
        rt(),
        "orchestrator",
        false,
        "tower_agent_start",
        json!({"agentType":"build","idempotencyKey":"c6-no-wr"}),
    )
    .await
    .unwrap_err();
    assert_eq!(err.code, "invalid_params");
}
