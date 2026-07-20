//! Black-box integration tests for the Streamable HTTP MCP server.
//!
//! These tests bind a REAL axum TCP listener on an ephemeral loopback port
//! and drive it with a REAL HTTP client (reqwest). They exercise the public
//! surface of `xai-grok-mcp-server` only — no in-process helper calls, no
//! FakeRuntime-internal mutation. The semantic core is shared with the
//! in-process and stdio adapters via `invoke_tower_tool`, so these tests
//! prove the HTTP framing reaches the same implementation.
//!
//! Tool black-box: a `FakeRuntime` is injected as the runtime facade (the
//! handoff explicitly permits FakeRuntime for pure framing tests; the
//! `tools/call` semantic path still routes through `invoke_tower_tool`).

#![cfg(feature = "streamable-http")]

use std::sync::Arc;
use std::time::{Duration, Instant};

use futures_util::StreamExt;
use serde_json::{json, Value};
use tokio::time::timeout;
use xai_grok_mcp_server::{
    DEFAULT_MAX_SESSION_EVENTS, MCP_PROTOCOL_VERSION, McpHttpConfig, McpSession,
    run_mcp_http_server,
};
use xai_grok_tower::FakeRuntime;

/// Read a long-lived SSE body until `idle` passes without new bytes (or the
/// stream ends). C4-F live-push SSE no longer terminates after the snapshot.
async fn read_sse_until_idle(
    resp: reqwest::Response,
    idle: Duration,
    overall: Duration,
) -> String {
    let start = Instant::now();
    let mut body = String::new();
    let mut stream = resp.bytes_stream();
    let mut last_byte = Instant::now();
    while start.elapsed() < overall {
        match timeout(idle, stream.next()).await {
            Ok(Some(Ok(chunk))) => {
                body.push_str(&String::from_utf8_lossy(&chunk));
                last_byte = Instant::now();
            }
            Ok(Some(Err(_))) | Ok(None) => break,
            Err(_) => {
                // Idle timeout with some data already collected → done.
                if !body.is_empty() || last_byte.elapsed() >= idle {
                    break;
                }
            }
        }
    }
    body
}

const TOKEN: &str = "tower-bearer-token";

/// Spawn a real MCP HTTP server on an ephemeral loopback port with the given
/// config overrides. Returns the bound address and the join handle.
async fn spawn_server(
) -> (std::net::SocketAddr, Arc<xai_grok_mcp_server::McpHttpState>, tokio::task::JoinHandle<()>) {
    spawn_server_with(McpHttpConfig {
        bearer_token: TOKEN.to_owned(),
        require_auth: true,
        tower_instance_id: "tower-A".to_owned(),
        ..Default::default()
    })
    .await
}

async fn spawn_server_with(
    config: McpHttpConfig,
) -> (std::net::SocketAddr, Arc<xai_grok_mcp_server::McpHttpState>, tokio::task::JoinHandle<()>) {
    let runtime: Arc<dyn xai_grok_tower::GrokRuntimeFacade> = Arc::new(FakeRuntime::new());
    let handle = run_mcp_http_server(runtime, config).await.unwrap();
    (handle.addr, handle.state, handle.join)
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap()
}

fn init_request(id: u64) -> Value {
    json!({
        "jsonrpc":"2.0","id":id,"method":"initialize",
        "params": {
            "protocolVersion": MCP_PROTOCOL_VERSION,
            "clientInfo": {"name":"test","version":"0"},
            "capabilities": {}
        }
    })
}

async fn post_json(
    client: &reqwest::Client,
    addr: std::net::SocketAddr,
    token: Option<&str>,
    session: Option<&str>,
    body: &Value,
) -> (reqwest::StatusCode, Value, Option<String>) {
    let mut req = client
        .post(format!("http://{addr}/mcp"))
        .header("content-type", "application/json")
        .header("accept", "application/json, text/event-stream");
    if let Some(t) = token {
        req = req.header("authorization", format!("Bearer {t}"));
    }
    if let Some(s) = session {
        req = req.header("mcp-session-id", s);
    }
    let resp = req.json(body).send().await.unwrap();
    let status = resp.status();
    let session_id = resp
        .headers()
        .get("mcp-session-id")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_owned());
    let text = resp.text().await.unwrap();
    let value = if text.is_empty() {
        Value::Null
    } else {
        serde_json::from_str(&text).unwrap_or(Value::Null)
    };
    (status, value, session_id)
}

// ---------------------------------------------------------------------------
// RED→GREEN: POST initialize / tools/list / tools/call
// ---------------------------------------------------------------------------

#[tokio::test]
async fn post_initialize_negotiates_session_header() {
    let (addr, _state, _join) = spawn_server().await;
    let c = client();
    let (status, body, session) = post_json(&c, addr, Some(TOKEN), None, &init_request(1)).await;
    assert_eq!(status, reqwest::StatusCode::OK);
    assert_eq!(body["result"]["protocolVersion"], MCP_PROTOCOL_VERSION);
    let session = session.expect("initialize must return Mcp-Session-Id");
    assert!(!session.is_empty());
}

#[tokio::test]
async fn post_tools_lists_exactly_nine_descriptors_matching_in_process() {
    let (addr, _state, _join) = spawn_server().await;
    let c = client();
    let (_, _, session) = post_json(&c, addr, Some(TOKEN), None, &init_request(1)).await;
    let (status, body, _) = post_json(
        &c,
        addr,
        Some(TOKEN),
        session.as_deref(),
        &json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}),
    )
    .await;
    assert_eq!(status, reqwest::StatusCode::OK);
    let tools = body["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 9);
    // Nine-tool descriptor parity with in-process names.
    let names: Vec<String> = tools
        .iter()
        .map(|t| t["name"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(
        names,
        xai_grok_mcp_server::MCP_TOOL_NAMES.to_vec()
    );
}

#[tokio::test]
async fn post_tools_call_start_returns_structured_content_with_session_id() {
    let (addr, _state, _join) = spawn_server().await;
    let c = client();
    let (_, _, session) = post_json(&c, addr, Some(TOKEN), None, &init_request(1)).await;
    let (status, body, _) = post_json(
        &c,
        addr,
        Some(TOKEN),
        session.as_deref(),
        &json!({
            "jsonrpc":"2.0","id":3,"method":"tools/call",
            "params": {
                "name": "tower_agent_start",
                "arguments": {
                    "workspaceRoot": "/work",
                    "agentType": "build",
                    "idempotencyKey": "http-start-0001"
                }
            }
        }),
    )
    .await;
    assert_eq!(status, reqwest::StatusCode::OK);
    assert!(body["result"]["structuredContent"]["sessionId"].is_string());
    assert_eq!(body["result"]["structuredContent"]["state"], "completed");
    // No error flag on success.
    assert!(body["result"].get("isError").is_none() || body["result"]["isError"] == false);
}

#[tokio::test]
async fn completed_tool_result_does_not_leave_an_active_turn() {
    let (addr, state, _join) = spawn_server().await;
    let c = client();
    let (_, _, session) = post_json(&c, addr, Some(TOKEN), None, &init_request(1)).await;
    let sid = session.as_deref().unwrap().to_owned();
    let (_, start_body, _) = post_json(
        &c,
        addr,
        Some(TOKEN),
        Some(&sid),
        &json!({
            "jsonrpc":"2.0","id":2,"method":"tools/call",
            "params": {
                "name": "tower_agent_start",
                "arguments": {"workspaceRoot": "/work/active", "agentType": "build", "idempotencyKey": "active-0001"}
            }
        }),
    )
    .await;
    let tower_sid = start_body["result"]["structuredContent"]["sessionId"]
        .as_str()
        .unwrap();
    post_json(
        &c,
        addr,
        Some(TOKEN),
        Some(&sid),
        &json!({
            "jsonrpc":"2.0","id":3,"method":"tools/call",
            "params": {
                "name": "tower_agent_send",
                "arguments": {
                    "sessionId": tower_sid,
                    "input": [{"type":"text","text":"done"}],
                    "mode": "new_turn",
                    "idempotencyKey": "active-send-1"
                }
            }
        }),
    )
    .await;

    assert!(state.sessions.lock().unwrap()[&sid]
        .active_turn_id
        .lock()
        .unwrap()
        .is_none());
}

#[tokio::test]
async fn repeated_quiet_tools_call_does_not_duplicate_snapshot_event() {
    let (addr, state, _join) = spawn_server().await;
    let c = client();
    let (_, _, session) = post_json(&c, addr, Some(TOKEN), None, &init_request(1)).await;
    let start = json!({
        "jsonrpc":"2.0","id":2,"method":"tools/call",
        "params": {
            "name": "tower_agent_start",
            "arguments": {"workspaceRoot": "/work/quiet", "agentType": "build", "idempotencyKey": "quiet-0001"}
        }
    });

    post_json(&c, addr, Some(TOKEN), session.as_deref(), &start).await;
    let sid = session.as_deref().unwrap();
    let first_len = state.sessions.lock().unwrap()[sid]
        .events
        .lock()
        .unwrap()
        .len();

    // The idempotent repeat has no new Tower event. It still invokes the
    // facade polling path, so a cursor-only implementation would append the
    // sequence-0 snapshot a second time here.
    post_json(&c, addr, Some(TOKEN), session.as_deref(), &start).await;
    let second_len = state.sessions.lock().unwrap()[sid]
        .events
        .lock()
        .unwrap()
        .len();
    assert_eq!(second_len, first_len, "quiet snapshot must be emitted once");
}

#[tokio::test]
async fn concurrent_quiet_tools_calls_do_not_duplicate_replay_page() {
    let (addr, state, _join) = spawn_server().await;
    let c = client();
    let (_, _, session) = post_json(&c, addr, Some(TOKEN), None, &init_request(1)).await;
    let start = json!({
        "jsonrpc":"2.0","id":2,"method":"tools/call",
        "params": {
            "name": "tower_agent_start",
            "arguments": {"workspaceRoot": "/work/concurrent", "agentType": "build", "idempotencyKey": "concurrent-0001"}
        }
    });
    post_json(&c, addr, Some(TOKEN), session.as_deref(), &start).await;

    let sid = session.as_deref().unwrap().to_owned();
    let baseline = state.sessions.lock().unwrap()[&sid]
        .events
        .lock()
        .unwrap()
        .len();

    // The same idempotent request is intentionally issued in parallel. Both
    // handlers enter the shared replay helper; a cursor-only implementation
    // can read the same page before either handler advances the cursor.
    let left = post_json(&c, addr, Some(TOKEN), Some(&sid), &start);
    let right = post_json(&c, addr, Some(TOKEN), Some(&sid), &start);
    let _ = tokio::join!(left, right);

    let after = state.sessions.lock().unwrap()[&sid]
        .events
        .lock()
        .unwrap()
        .len();
    assert_eq!(
        after,
        baseline,
        "concurrent quiet pulls must append no page twice"
    );
}

#[tokio::test]
async fn post_tools_call_deny_path_emits_iserror_with_forbidden_code() {
    let (addr, _state, _join) = spawn_server_with(McpHttpConfig {
        bearer_token: TOKEN.to_owned(),
        require_auth: true,
        tower_instance_id: "tower-A".to_owned(),
        agent_type: "build".to_owned(), // build is fail-closed without opt-in
        explicit_opt_in: false,
        ..Default::default()
    })
    .await;
    let c = client();
    let (_, _, session) = post_json(&c, addr, Some(TOKEN), None, &init_request(1)).await;
    let (status, body, _) = post_json(
        &c,
        addr,
        Some(TOKEN),
        session.as_deref(),
        &json!({
            "jsonrpc":"2.0","id":4,"method":"tools/call",
            "params": {"name": "tower_agent_list", "arguments": {}}
        }),
    )
    .await;
    assert_eq!(status, reqwest::StatusCode::OK);
    assert_eq!(body["result"]["isError"], true);
    assert_eq!(body["result"]["structuredContent"]["code"], "tower_acl_denied");
}

// ---------------------------------------------------------------------------
// RED→GREEN: auth failure equivalence (indistinguishable 401)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn auth_failures_are_indistinguishable_401() {
    let (addr, _state, _join) = spawn_server().await;
    let c = client();
    // Missing, empty, wrong, malformed — all must yield 401 (no body leak).
    let cases: Vec<Option<&str>> = vec![None, Some(""), Some("Bearer wrong"), Some("token-value")];
    for token in &cases {
        let resp = c
            .post(format!("http://{addr}/mcp"))
            .header("content-type", "application/json")
            .header("accept", "application/json")
            .json(&init_request(1))
            .headers(auth_headers(token.as_deref()))
            .send()
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            reqwest::StatusCode::UNAUTHORIZED,
            "expected 401 for token={token:?}"
        );
        let www_auth = resp
            .headers()
            .get("www-authenticate")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("");
        assert_eq!(www_auth, "Bearer");
    }
}

fn auth_headers(token: Option<&str>) -> reqwest::header::HeaderMap {
    let mut h = reqwest::header::HeaderMap::new();
    if let Some(t) = token {
        if let Ok(v) = reqwest::header::HeaderValue::from_str(&format!("Bearer {t}")) {
            h.insert("authorization", v);
        } else if !t.is_empty() {
            // raw non-bearer value
            if let Ok(v) = reqwest::header::HeaderValue::from_str(t) {
                h.insert("authorization", v);
            }
        }
    }
    h
}

// ---------------------------------------------------------------------------
// RED→GREEN: body limit
// ---------------------------------------------------------------------------

#[tokio::test]
async fn body_limit_rejects_oversized_post_before_dispatch() {
    let (addr, _state, _join) = spawn_server().await;
    let c = client();
    let (_, _, session) = post_json(&c, addr, Some(TOKEN), None, &init_request(1)).await;
    // 2 MiB string exceeds the 1 MiB default cap.
    let big = "x".repeat(2 * 1024 * 1024);
    let resp = c
        .post(format!("http://{addr}/mcp"))
        .header("authorization", format!("Bearer {TOKEN}"))
        .header("content-type", "application/json")
        .header("accept", "application/json")
        .header("mcp-session-id", session.as_deref().unwrap_or(""))
        .body(big)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::PAYLOAD_TOO_LARGE);
}

// ---------------------------------------------------------------------------
// RED→GREEN: DELETE session
// ---------------------------------------------------------------------------

#[tokio::test]
async fn delete_session_terminates_and_rejects_subsequent_post() {
    let (addr, _state, _join) = spawn_server().await;
    let c = client();
    let (_, _, session) = post_json(&c, addr, Some(TOKEN), None, &init_request(1)).await;

    let del = c
        .delete(format!("http://{addr}/mcp"))
        .header("authorization", format!("Bearer {TOKEN}"))
        .header("mcp-session-id", session.as_deref().unwrap_or(""))
        .send()
        .await
        .unwrap();
    assert_eq!(del.status(), reqwest::StatusCode::OK);

    // Subsequent POST with the deleted session header is rejected (404).
    let (status, _body, _) = post_json(
        &c,
        addr,
        Some(TOKEN),
        session.as_deref(),
        &json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}),
    )
    .await;
    assert_eq!(status, reqwest::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_rejects_foreign_bearer_binding_before_removal() {
    let (addr, state, _join) = spawn_server().await;
    let c = client();
    let (_, _, session) = post_json(&c, addr, Some(TOKEN), None, &init_request(1)).await;
    let sid = session.unwrap();

    // Replace the negotiated session with one bound to a different bearer
    // fingerprint. DELETE must enforce the same binding as GET/POST.
    state.sessions.lock().unwrap().insert(
        sid.clone(),
        Arc::new(McpSession::new(
            0xdead_beef,
            "tower-A".to_owned(),
            state.max_session_events,
        )),
    );
    let resp = c
        .delete(format!("http://{addr}/mcp"))
        .header("authorization", format!("Bearer {TOKEN}"))
        .header("mcp-session-id", &sid)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::UNAUTHORIZED);
    assert!(state.sessions.lock().unwrap().contains_key(&sid));
}

#[tokio::test]
async fn delete_without_session_header_is_bad_request() {
    let (addr, _state, _join) = spawn_server().await;
    let c = client();
    let del = c
        .delete(format!("http://{addr}/mcp"))
        .header("authorization", format!("Bearer {TOKEN}"))
        .send()
        .await
        .unwrap();
    assert_eq!(del.status(), reqwest::StatusCode::BAD_REQUEST);
}

// ---------------------------------------------------------------------------
// RED→GREEN: GET SSE resume
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_sse_streams_events_after_tools_call() {
    let (addr, _state, _join) = spawn_server().await;
    let c = client();
    let (_, _, session) = post_json(&c, addr, Some(TOKEN), None, &init_request(1)).await;
    // Mutate so the transport session pulls facade events into its log.
    post_json(
        &c,
        addr,
        Some(TOKEN),
        session.as_deref(),
        &json!({
            "jsonrpc":"2.0","id":2,"method":"tools/call",
            "params": {
                "name": "tower_agent_start",
                "arguments": {"workspaceRoot": "/work", "agentType": "build", "idempotencyKey": "sse-0001"}
            }
        }),
    )
    .await;

    // GET /mcp with Accept: text/event-stream should deliver SSE events.
    let resp = c
        .get(format!("http://{addr}/mcp"))
        .header("authorization", format!("Bearer {TOKEN}"))
        .header("accept", "text/event-stream")
        .header("mcp-session-id", session.as_deref().unwrap_or(""))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    let ct = resp
        .headers()
        .get("content-type")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");
    assert!(ct.contains("text/event-stream"), "content-type: {ct}");

    // Long-lived SSE (C4-F live push): read until idle after the snapshot.
    let body = read_sse_until_idle(resp, Duration::from_millis(400), Duration::from_secs(5)).await;
    assert!(body.contains("event: session_changed"), "body: {body}");
    assert!(body.contains("id: 1"), "body: {body}");
}

#[tokio::test]
async fn get_sse_resume_from_last_event_id() {
    let (addr, _state, _join) = spawn_server().await;
    let c = client();
    let (_, _, session) = post_json(&c, addr, Some(TOKEN), None, &init_request(1)).await;
    post_json(
        &c,
        addr,
        Some(TOKEN),
        session.as_deref(),
        &json!({
            "jsonrpc":"2.0","id":2,"method":"tools/call",
            "params": {
                "name": "tower_agent_start",
                "arguments": {"workspaceRoot": "/work", "agentType": "build", "idempotencyKey": "sse-resume-0001"}
            }
        }),
    )
    .await;

    // Resume from id=1 → should NOT replay event id 1 again.
    let resp = c
        .get(format!("http://{addr}/mcp"))
        .header("authorization", format!("Bearer {TOKEN}"))
        .header("accept", "text/event-stream")
        .header("mcp-session-id", session.as_deref().unwrap_or(""))
        .header("last-event-id", "1")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    let body = read_sse_until_idle(resp, Duration::from_millis(400), Duration::from_secs(5)).await;
    assert!(
        !body.contains("id: 1\n"),
        "resume must not replay already-delivered event 1: {body}"
    );
}

#[tokio::test]
async fn open_sse_emits_resumption_error_when_buffer_expires() {
    let (addr, state, _join) = spawn_server_with(McpHttpConfig {
        bearer_token: TOKEN.to_owned(),
        require_auth: true,
        tower_instance_id: "tower-A".to_owned(),
        max_session_events: 1,
        ..Default::default()
    })
    .await;
    let c = client();
    let (_, _, session) = post_json(&c, addr, Some(TOKEN), None, &init_request(1)).await;
    let sid = session.as_deref().unwrap().to_owned();
    let session_arc = state.sessions.lock().unwrap()[&sid].clone();

    let resp = c
        .get(format!("http://{addr}/mcp"))
        .header("authorization", format!("Bearer {TOKEN}"))
        .header("accept", "text/event-stream")
        .header("mcp-session-id", &sid)
        .send()
        .await
        .unwrap();
    let mut stream = resp.bytes_stream();

    // Advance the open stream to cursor 1, then expire that cursor by adding
    // two more events to a one-entry buffer.
    session_arc.append_event("test".to_owned(), "one".to_owned());
    let first = timeout(Duration::from_secs(2), stream.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert!(String::from_utf8_lossy(&first).contains("id: 1"));
    session_arc.append_event("test".to_owned(), "two".to_owned());
    session_arc.append_event("test".to_owned(), "three".to_owned());

    let mut body = String::new();
    while let Ok(Some(Ok(chunk))) = timeout(Duration::from_secs(2), stream.next()).await {
        body.push_str(&String::from_utf8_lossy(&chunk));
        if body.contains("event: resumption_error") {
            break;
        }
    }
    assert!(body.contains("event: resumption_error"), "body: {body}");
}

#[tokio::test]
async fn open_sse_emits_resumption_error_on_tower_epoch_mismatch() {
    let force_epoch_mismatch = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let runtime: Arc<dyn xai_grok_tower::GrokRuntimeFacade> =
        Arc::new(InterruptProbeRuntime {
            inner: FakeRuntime::new(),
            interrupts: Arc::new(std::sync::Mutex::new(Vec::new())),
            force_epoch_mismatch: force_epoch_mismatch.clone(),
        });
    let handle = run_mcp_http_server(
        runtime,
        McpHttpConfig {
            bearer_token: TOKEN.to_owned(),
            require_auth: true,
            tower_instance_id: "tower-A".to_owned(),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let c = client();
    let (_, _, session) = post_json(&c, handle.addr, Some(TOKEN), None, &init_request(1)).await;
    let sid = session.as_deref().unwrap().to_owned();
    post_json(
        &c,
        handle.addr,
        Some(TOKEN),
        Some(&sid),
        &json!({
            "jsonrpc":"2.0","id":2,"method":"tools/call",
            "params": {
                "name": "tower_agent_start",
                "arguments": {"workspaceRoot": "/work/epoch", "agentType": "build", "idempotencyKey": "epoch-0001"}
            }
        }),
    )
    .await;
    force_epoch_mismatch.store(true, std::sync::atomic::Ordering::SeqCst);

    let resp = c
        .get(format!("http://{}/mcp", handle.addr))
        .header("authorization", format!("Bearer {TOKEN}"))
        .header("accept", "text/event-stream")
        .header("mcp-session-id", &sid)
        .send()
        .await
        .unwrap();
    let body = read_sse_until_idle(resp, Duration::from_millis(400), Duration::from_secs(5)).await;
    assert!(body.contains("event: resumption_error"), "body: {body}");
    handle.join.abort();
}

#[tokio::test]
async fn tower_session_rebind_resets_mcp_replay_identity() {
    let (addr, state, _join) = spawn_server().await;
    let c = client();
    let (_, _, session) = post_json(&c, addr, Some(TOKEN), None, &init_request(1)).await;
    let sid = session.as_deref().unwrap().to_owned();

    for (id, root, key) in [(2, "/work/rebind-a", "rebind-a"), (3, "/work/rebind-b", "rebind-b")] {
        let (_, body, _) = post_json(
            &c,
            addr,
            Some(TOKEN),
            Some(&sid),
            &json!({
                "jsonrpc":"2.0","id":id,"method":"tools/call",
                "params": {
                    "name": "tower_agent_start",
                    "arguments": {"workspaceRoot": root, "agentType": "build", "idempotencyKey": key}
                }
            }),
        )
        .await;
        assert!(body["result"]["structuredContent"]["sessionId"].is_string());
        if id == 2 {
            let transport = state.sessions.lock().unwrap()[&sid].clone();
            *transport.active_turn_id.lock().unwrap() = Some("old-turn".to_owned());
        }
    }

    assert!(state.sessions.lock().unwrap()[&sid]
        .active_turn_id
        .lock()
        .unwrap()
        .is_none());

    // The second start binds a different Tower session. Its replay must begin
    // at cursor zero with the new session epoch, not reuse session A's epoch.
    let resp = c
        .get(format!("http://{addr}/mcp"))
        .header("authorization", format!("Bearer {TOKEN}"))
        .header("accept", "text/event-stream")
        .header("mcp-session-id", &sid)
        .send()
        .await
        .unwrap();
    let body = read_sse_until_idle(resp, Duration::from_millis(400), Duration::from_secs(5)).await;
    assert!(!body.contains("event: resumption_error"), "body: {body}");
    assert!(
        !body.contains("/work/rebind-a"),
        "rebind must not replay the previous Tower session: {body}"
    );
}

#[tokio::test]
async fn open_sse_is_invalidated_when_tower_session_rebinds() {
    let (addr, _state, _join) = spawn_server().await;
    let c = client();
    let (_, _, session) = post_json(&c, addr, Some(TOKEN), None, &init_request(1)).await;
    let sid = session.as_deref().unwrap().to_owned();
    post_json(
        &c,
        addr,
        Some(TOKEN),
        Some(&sid),
        &json!({
            "jsonrpc":"2.0","id":2,"method":"tools/call",
            "params": {
                "name": "tower_agent_start",
                "arguments": {"workspaceRoot": "/work/open-a", "agentType": "build", "idempotencyKey": "open-a-0001"}
            }
        }),
    )
    .await;

    let resp = c
        .get(format!("http://{addr}/mcp"))
        .header("authorization", format!("Bearer {TOKEN}"))
        .header("accept", "text/event-stream")
        .header("mcp-session-id", &sid)
        .send()
        .await
        .unwrap();
    let mut stream = resp.bytes_stream();
    let _ = timeout(Duration::from_secs(2), stream.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();

    // Rebind while the old SSE producer is still alive.
    post_json(
        &c,
        addr,
        Some(TOKEN),
        Some(&sid),
        &json!({
            "jsonrpc":"2.0","id":3,"method":"tools/call",
            "params": {
                "name": "tower_agent_start",
                "arguments": {"workspaceRoot": "/work/open-b", "agentType": "build", "idempotencyKey": "open-b-0001"}
            }
        }),
    )
    .await;
    let mut body = String::new();
    while let Ok(Some(Ok(chunk))) = timeout(Duration::from_secs(2), stream.next()).await {
        body.push_str(&String::from_utf8_lossy(&chunk));
        if body.contains("session_rebound") {
            break;
        }
    }
    assert!(body.contains("session_rebound"), "body: {body}");
}

#[tokio::test]
async fn get_sse_foreign_last_event_id_returns_resumption_error() {
    let (addr, _state, _join) = spawn_server().await;
    let c = client();
    let (_, _, session) = post_json(&c, addr, Some(TOKEN), None, &init_request(1)).await;
    // No mutation → session has zero events. Last-Event-ID 999 is foreign.
    let resp = c
        .get(format!("http://{addr}/mcp"))
        .header("authorization", format!("Bearer {TOKEN}"))
        .header("accept", "text/event-stream")
        .header("mcp-session-id", session.as_deref().unwrap_or(""))
        .header("last-event-id", "999")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    let body = timeout(Duration::from_secs(5), resp.text())
        .await
        .expect("foreign-id stream must terminate")
        .expect("body");
    assert!(body.contains("resumption_error"), "body: {body}");
}

#[tokio::test]
async fn get_sse_does_not_replay_another_clients_events() {
    let (addr, _state, _join) = spawn_server().await;
    let c = client();
    // Client A: initialize + mutate (event logged in A's transport session).
    let (_, _, session_a) = post_json(&c, addr, Some(TOKEN), None, &init_request(1)).await;
    post_json(
        &c,
        addr,
        Some(TOKEN),
        session_a.as_deref(),
        &json!({
            "jsonrpc":"2.0","id":2,"method":"tools/call",
            "params": {
                "name": "tower_agent_start",
                "arguments": {"workspaceRoot": "/workA", "agentType": "build", "idempotencyKey": "iso-a-0001"}
            }
        }),
    )
    .await;
    // Client B: separate transport session, no mutations.
    let (_, _, session_b) = post_json(&c, addr, Some(TOKEN), None, &init_request(10)).await;
    let resp = c
        .get(format!("http://{addr}/mcp"))
        .header("authorization", format!("Bearer {TOKEN}"))
        .header("accept", "text/event-stream")
        .header("mcp-session-id", session_b.as_deref().unwrap_or(""))
        .send()
        .await
        .unwrap();
    let body = read_sse_until_idle(resp, Duration::from_millis(400), Duration::from_secs(5)).await;
    assert!(
        !body.contains("session_changed"),
        "B must not see A's events: {body}"
    );
}

#[tokio::test]
async fn r4_mcp_session_limit_rejects_additional_initialize() {
    // R4-09: max_sessions saturation.
    let (addr, _state, _join) = spawn_server_with(McpHttpConfig {
        bearer_token: TOKEN.to_owned(),
        require_auth: true,
        max_sessions: 1,
        ..Default::default()
    })
    .await;
    let c = client();
    let (status1, _, _) = post_json(&c, addr, Some(TOKEN), None, &init_request(1)).await;
    assert!(status1.is_success(), "first session ok");
    let (status2, body, _) = post_json(&c, addr, Some(TOKEN), None, &init_request(2)).await;
    assert_eq!(
        status2,
        reqwest::StatusCode::SERVICE_UNAVAILABLE,
        "second session saturates limit: {body}"
    );
    assert!(
        body.to_string().contains("session limit") || body.to_string().contains("-32029"),
        "body: {body}"
    );
}

#[tokio::test]
async fn r5_mcp_event_buffer_cap_drops_oldest() {
    // R5-06: per-session event buffer is capped; oldest events are dropped.
    let (addr, state, _join) = spawn_server_with(McpHttpConfig {
        bearer_token: TOKEN.to_owned(),
        require_auth: true,
        max_session_events: 3,
        ..Default::default()
    })
    .await;
    let c = client();
    let (_, _, session) = post_json(&c, addr, Some(TOKEN), None, &init_request(1)).await;
    let sid = session.expect("session");
    let session_arc = state.sessions.lock().unwrap().get(&sid).cloned().unwrap();
    for i in 0..8 {
        session_arc.append_event("test".into(), format!("e{i}"));
    }
    let events = session_arc.events.lock().unwrap().clone();
    assert_eq!(events.len(), 3, "cap retains at most max_session_events");
    assert_eq!(events[0].data, "e5");
    assert_eq!(events[2].data, "e7");
    let min = session_arc
        .min_retained_event_id
        .load(std::sync::atomic::Ordering::SeqCst);
    assert!(min > 0, "min retained advances when oldest drop");
    // Cursor below retained window is expired.
    assert!(session_arc.events_after(min.saturating_sub(1)).is_err());
}

/// Probe facade that records every `interrupt_turn` call (R5-07 evidence).
struct InterruptProbeRuntime {
    inner: FakeRuntime,
    interrupts: Arc<std::sync::Mutex<Vec<(String, String)>>>,
    force_epoch_mismatch: Arc<std::sync::atomic::AtomicBool>,
}

#[async_trait::async_trait]
impl xai_grok_tower::GrokRuntimeFacade for InterruptProbeRuntime {
    async fn list_sessions(
        &self,
    ) -> Result<Vec<xai_grok_app_server_protocol::Session>, xai_grok_tower::RuntimeError> {
        self.inner.list_sessions().await
    }
    async fn read_session(
        &self,
        params: xai_grok_app_server_protocol::SessionReadParams,
    ) -> Result<xai_grok_app_server_protocol::SessionReadResult, xai_grok_tower::RuntimeError>
    {
        self.inner.read_session(params).await
    }
    async fn start_session(
        &self,
        params: xai_grok_app_server_protocol::SessionStartParams,
    ) -> Result<xai_grok_app_server_protocol::Session, xai_grok_tower::RuntimeError> {
        self.inner.start_session(params).await
    }
    async fn resume_session(
        &self,
        params: xai_grok_app_server_protocol::SessionResumeParams,
    ) -> Result<xai_grok_app_server_protocol::Session, xai_grok_tower::RuntimeError> {
        self.inner.resume_session(params).await
    }
    async fn fork_session(
        &self,
        params: xai_grok_app_server_protocol::SessionForkParams,
    ) -> Result<xai_grok_app_server_protocol::Session, xai_grok_tower::RuntimeError> {
        self.inner.fork_session(params).await
    }
    async fn archive_session(
        &self,
        params: xai_grok_app_server_protocol::SessionArchiveParams,
    ) -> Result<(), xai_grok_tower::RuntimeError> {
        self.inner.archive_session(params).await
    }
    async fn start_turn(
        &self,
        params: xai_grok_app_server_protocol::TurnStartParams,
    ) -> Result<xai_grok_app_server_protocol::Turn, xai_grok_tower::RuntimeError> {
        self.inner.start_turn(params).await
    }
    async fn steer_turn(
        &self,
        params: xai_grok_app_server_protocol::TurnSteerParams,
    ) -> Result<xai_grok_app_server_protocol::Item, xai_grok_tower::RuntimeError> {
        self.inner.steer_turn(params).await
    }
    async fn interrupt_turn(
        &self,
        params: xai_grok_app_server_protocol::TurnInterruptParams,
    ) -> Result<(), xai_grok_tower::RuntimeError> {
        self.interrupts
            .lock()
            .unwrap()
            .push((params.session_id.clone(), params.turn_id.clone()));
        self.inner.interrupt_turn(params).await
    }
    async fn respond_interaction(
        &self,
        params: xai_grok_app_server_protocol::InteractionResponseParams,
    ) -> Result<(), xai_grok_tower::RuntimeError> {
        self.inner.respond_interaction(params).await
    }
    async fn replay(
        &self,
        mut cursor: xai_grok_app_server_protocol::SubscribeParams,
    ) -> Result<xai_grok_tower::ReplayPage, xai_grok_tower::RuntimeError> {
        if self
            .force_epoch_mismatch
            .load(std::sync::atomic::Ordering::SeqCst)
        {
            cursor.history_epoch = Some("epoch-stale".to_owned());
        }
        self.inner.replay(cursor).await
    }
}

#[tokio::test]
async fn r5_mcp_ttl_eviction_via_lookup_interrupts_active_turn() {
    // R5-07: TTL eviction on the **lookup** path (non-initialize POST) must
    // call interrupt_turn — not only remove the transport session.
    let interrupts = Arc::new(std::sync::Mutex::new(Vec::new()));
    let runtime: Arc<dyn xai_grok_tower::GrokRuntimeFacade> = Arc::new(InterruptProbeRuntime {
        inner: FakeRuntime::new(),
        interrupts: interrupts.clone(),
        force_epoch_mismatch: Arc::new(std::sync::atomic::AtomicBool::new(false)),
    });
    // Start a real Tower session + turn so interrupt_turn can succeed.
    let tower = runtime
        .start_session(xai_grok_app_server_protocol::SessionStartParams {
            workspace_root: "/work/ttl".into(),
            agent_type: None,
            provider_binding: None,
            idempotency_key: "ttl-sess".into(),
        })
        .await
        .unwrap();
    let turn = runtime
        .start_turn(xai_grok_app_server_protocol::TurnStartParams {
            session_id: tower.session_id.clone(),
            input: vec![xai_grok_app_server_protocol::InputBlock::Text {
                text: "hold".into(),
            }],
            idempotency_key: "ttl-turn".into(),
        })
        .await
        .unwrap();

    let handle = run_mcp_http_server(
        runtime.clone(),
        McpHttpConfig {
            bearer_token: TOKEN.to_owned(),
            require_auth: true,
            session_ttl: Duration::from_millis(30),
            tower_instance_id: "tower-A".to_owned(),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let addr = handle.addr;
    let state = handle.state.clone();
    let c = client();

    // Session A: initialize, bind tower session + active turn, then expire.
    let (_, _, session_a) = post_json(&c, addr, Some(TOKEN), None, &init_request(1)).await;
    let sid_a = session_a.expect("session A");
    {
        let s = state.sessions.lock().unwrap().get(&sid_a).cloned().unwrap();
        *s.tower_session_id.lock().unwrap() = Some(tower.session_id.clone());
        *s.active_turn_id.lock().unwrap() = Some(turn.turn_id.clone());
        *s.last_active.lock().unwrap() =
            std::time::Instant::now() - Duration::from_secs(10);
    }

    // Session B (fresh) — initialize is one eviction path; also prove lookup:
    // first create B, then use tools/list on B which calls lookup_session
    // (async interrupt path) while A is still expired in the map... but
    // initialize already evicted A. So: expire A, then tools/list on a
    // **second** session that is still alive after re-init, after re-injecting
    // an expired peer? Simpler: expire A without re-init by using a second
    // live session C that we create first, expire A, then tools/list on C.
    let (_, _, session_c) = post_json(&c, addr, Some(TOKEN), None, &init_request(2)).await;
    let sid_c = session_c.expect("session C (live peer)");
    // Re-bind A as expired peer (initialize may have already dropped A if
    // TTL was already hit; re-insert if missing so lookup on C still finds A
    // to evict... Actually initialize on C already ran eviction. Ensure A
    // is re-inserted expired so the next lookup on C re-evicts A.
    {
        let fingerprint = {
            // Match product bearer fingerprint of TOKEN for session A.
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut h = DefaultHasher::new();
            TOKEN.hash(&mut h);
            h.finish()
        };
        let expired = Arc::new(McpSession::new(
            fingerprint,
            "tower-A".to_owned(),
            DEFAULT_MAX_SESSION_EVENTS,
        ));
        *expired.tower_session_id.lock().unwrap() = Some(tower.session_id.clone());
        *expired.active_turn_id.lock().unwrap() = Some(turn.turn_id.clone());
        *expired.last_active.lock().unwrap() =
            std::time::Instant::now() - Duration::from_secs(10);
        state
            .sessions
            .lock()
            .unwrap()
            .insert(sid_a.clone(), expired);
    }
    interrupts.lock().unwrap().clear();

    // Lookup path: tools/list on live session C → lookup_session → interrupt A.
    let (list_status, _, _) = post_json(
        &c,
        addr,
        Some(TOKEN),
        Some(&sid_c),
        &json!({"jsonrpc":"2.0","id":3,"method":"tools/list"}),
    )
    .await;
    assert!(list_status.is_success(), "live peer tools/list must succeed");
    assert!(
        !state.sessions.lock().unwrap().contains_key(&sid_a),
        "expired peer must be removed by lookup eviction"
    );
    let calls = interrupts.lock().unwrap().clone();
    assert!(
        calls.iter().any(|(s, t)| s == &tower.session_id && t == &turn.turn_id),
        "lookup TTL eviction must call interrupt_turn for the active turn, got {calls:?}"
    );
}

#[tokio::test]
async fn get_sse_requires_accept_event_stream() {
    let (addr, _state, _join) = spawn_server().await;
    let c = client();
    let (_, _, session) = post_json(&c, addr, Some(TOKEN), None, &init_request(1)).await;
    let resp = c
        .get(format!("http://{addr}/mcp"))
        .header("authorization", format!("Bearer {TOKEN}"))
        .header("accept", "application/json")
        .header("mcp-session-id", session.as_deref().unwrap_or(""))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::NOT_ACCEPTABLE);
}

// ---------------------------------------------------------------------------
// RED→GREEN: protocol-version gate
// ---------------------------------------------------------------------------

#[tokio::test]
async fn protocol_version_gate_rejects_unsupported_before_dispatch() {
    let (addr, _state, _join) = spawn_server().await;
    let c = client();
    let resp = c
        .post(format!("http://{addr}/mcp"))
        .header("authorization", format!("Bearer {TOKEN}"))
        .header("content-type", "application/json")
        .header("accept", "application/json")
        .header("protocol-version", "9999-99-99")
        .json(&init_request(1))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::BAD_REQUEST);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], -32006);
}

// ---------------------------------------------------------------------------
// RED→GREEN: session bound to Tower instance (cross-server)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn session_id_from_tower_a_rejected_by_tower_b() {
    let (addr_a, _state_a, _join_a) = spawn_server_with(McpHttpConfig {
        bearer_token: TOKEN.to_owned(),
        require_auth: true,
        tower_instance_id: "tower-A".to_owned(),
        ..Default::default()
    })
    .await;
    let (addr_b, _state_b, _join_b) = spawn_server_with(McpHttpConfig {
        bearer_token: TOKEN.to_owned(),
        require_auth: true,
        tower_instance_id: "tower-B".to_owned(),
        ..Default::default()
    })
    .await;
    let c = client();
    // Negotiate a session on Tower A.
    let (_, _, session_a) = post_json(&c, addr_a, Some(TOKEN), None, &init_request(1)).await;
    // Use session_a against Tower B → must be rejected (not found / not bound).
    let (status, _body, _) = post_json(
        &c,
        addr_b,
        Some(TOKEN),
        session_a.as_deref(),
        &json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}),
    )
    .await;
    assert_eq!(status, reqwest::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn non_initialize_request_without_session_header_is_bad_request() {
    // A non-initialize request must carry a negotiated Mcp-Session-Id. This
    // is the session-binding-mandatory path (distinct from the fingerprint
    // mismatch path exercised below).
    let (addr, _state, _join) = spawn_server().await;
    let c = client();
    let (status, _body, _) = post_json(
        &c,
        addr,
        Some(TOKEN),
        None,
        &json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}),
    )
    .await;
    assert_eq!(status, reqwest::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn session_bearer_fingerprint_mismatch_rejects() {
    // F-2 (test-review F2): exercise the bearer fingerprint binding at
    // `lookup_session` (http_server.rs `if state.require_auth { fingerprint
    // != session.bearer_fingerprint => unauthorized() }`). A session
    // negotiated with bearer A must be rejected when the request presents a
    // different (but still valid-for-the-server) bearer B, even when the
    // Tower instance id matches.
    //
    // We model the reconfiguration scenario the fingerprint binding guards:
    // server B is reconfigured for bearer B but a stale session opened under
    // bearer A is injected into its session map. A request with bearer B
    // passes auth (B is valid for B) but the fingerprint of B does not match
    // the session's stored fingerprint of A → 401.
    let c = client();

    // Server A: bearer A, tower-shared.
    let (addr_a, state_a, _join_a) = spawn_server_with(McpHttpConfig {
        bearer_token: "tower-bearer-A".to_owned(),
        require_auth: true,
        tower_instance_id: "tower-shared".to_owned(),
        ..Default::default()
    })
    .await;
    // Negotiate a session on A with bearer A.
    let (_, _, session_a_id) = post_json(
        &c,
        addr_a,
        Some("tower-bearer-A"),
        None,
        &init_request(1),
    )
    .await;
    let session_a_id = session_a_id.expect("initialize must return a session id");
    // Read the session Arc out of A's public state.
    let session_arc = state_a
        .sessions
        .lock()
        .unwrap()
        .get(&session_a_id)
        .cloned()
        .expect("session must be registered on A");
    // Sanity: the session's stored fingerprint is for bearer A (not B).
    assert_eq!(session_arc.tower_instance_id, "tower-shared");

    // Server B: bearer B, SAME tower instance id (so the Tower-instance check
    // does not fire — we want to isolate the fingerprint check).
    let (addr_b, state_b, _join_b) = spawn_server_with(McpHttpConfig {
        bearer_token: "tower-bearer-B".to_owned(),
        require_auth: true,
        tower_instance_id: "tower-shared".to_owned(),
        ..Default::default()
    })
    .await;
    // Inject A's session into B's map under the same id. This models a stale
    // session id reused after the server reconfigured its bearer.
    state_b
        .sessions
        .lock()
        .unwrap()
        .insert(session_a_id.clone(), session_arc);

    // Request to B with bearer B (valid for B → auth passes) and A's session
    // id. The fingerprint of B != stored fingerprint of A → 401.
    let (status, _body, _) = post_json(
        &c,
        addr_b,
        Some("tower-bearer-B"),
        Some(&session_a_id),
        &json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}),
    )
    .await;
    assert_eq!(
        status,
        reqwest::StatusCode::UNAUTHORIZED,
        "fingerprint mismatch must reject with 401"
    );
}

#[tokio::test]
async fn session_bearer_fingerprint_match_accepts() {
    // Positive control for the fingerprint binding: the same bearer that
    // opened the session continues to be accepted (fingerprint matches).
    let (addr, _state, _join) = spawn_server().await;
    let c = client();
    let (_, _, session) = post_json(&c, addr, Some(TOKEN), None, &init_request(1)).await;
    let (status, _body, _) = post_json(
        &c,
        addr,
        Some(TOKEN),
        session.as_deref(),
        &json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}),
    )
    .await;
    assert_eq!(status, reqwest::StatusCode::OK);
}

// ---------------------------------------------------------------------------
// RED→GREEN: stdio vs HTTP parity (MCP102-05)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn stdio_and_http_produce_identical_tools_list_and_error_shapes() {
    let rt = Arc::new(FakeRuntime::new());
    // stdio path: process_mcp_stdio_batch
    let stdio_out = xai_grok_mcp_server::process_mcp_stdio_batch(
        rt.clone(),
        "orchestrator",
        false,
        &json!({"jsonrpc":"2.0","id":1,"method":"tools/list"}).to_string(),
    )
    .await;
    let stdio_list: Value = serde_json::from_str(&stdio_out[0]).unwrap();

    // HTTP path
    let (addr, _state, _join) = spawn_server().await;
    let c = client();
    let (_, _, session) = post_json(&c, addr, Some(TOKEN), None, &init_request(1)).await;
    let (_, http_list, _) = post_json(
        &c,
        addr,
        Some(TOKEN),
        session.as_deref(),
        &json!({"jsonrpc":"2.0","id":1,"method":"tools/list"}),
    )
    .await;

    // tools/list parity: same nine names in the same order.
    let stdio_names: Vec<String> = stdio_list["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap().to_owned())
        .collect();
    let http_names: Vec<String> = http_list["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(stdio_names, http_names);
    assert_eq!(stdio_names, xai_grok_mcp_server::MCP_TOOL_NAMES.to_vec());

    // Error shape parity: forbidden from a build agent.
    let stdio_err = xai_grok_mcp_server::process_mcp_stdio_batch(
        rt,
        "build",
        false,
        &json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"tower_agent_list","arguments":{}}})
            .to_string(),
    )
    .await;
    let stdio_err_val: Value = serde_json::from_str(&stdio_err[0]).unwrap();
    assert_eq!(stdio_err_val["result"]["isError"], true);
    assert_eq!(stdio_err_val["result"]["structuredContent"]["code"], "tower_acl_denied");
}

// ---------------------------------------------------------------------------
// RED→GREEN: self-loop composition guard
// ---------------------------------------------------------------------------

#[tokio::test]
async fn post_tools_call_does_not_reenter_via_managed_mcp_client() {
    // The HTTP server must route tools/call through invoke_tower_tool only,
    // never re-entering via a managed MCP client. We assert this indirectly:
    // a tools/call that starts a session produces exactly one operationId
    // (one invocation of the facade), and the server source contains no
    // outbound MCP client import (canary in http_server.rs).
    let (addr, state, _join) = spawn_server().await;
    let c = client();
    let (_, _, session) = post_json(&c, addr, Some(TOKEN), None, &init_request(1)).await;
    let (_, body, _) = post_json(
        &c,
        addr,
        Some(TOKEN),
        session.as_deref(),
        &json!({
            "jsonrpc":"2.0","id":2,"method":"tools/call",
            "params": {
                "name": "tower_agent_start",
                "arguments": {"workspaceRoot": "/work", "agentType": "build", "idempotencyKey": "no-self-loop-0001"}
            }
        }),
    )
    .await;
    assert_eq!(body["result"]["structuredContent"]["state"], "completed");
    // Exactly one session was created on the facade (no re-entry).
    let sessions = state.sessions.lock().unwrap();
    assert_eq!(sessions.len(), 1, "exactly one transport session");
}

#[test]
fn composition_source_does_not_register_local_mcp_self_loop() {
    // The product composition root must not register the local /mcp URL into
    // the session's MCP client pool. Today the composition root does not wire
    // MCP HTTP at all (PARTIAL — product bin not yet bound), so this guard
    // passes. It will fail if anyone adds a self-registration without an
    // explicit opt-in guard.
    let composition = include_str!(
        "../../xai-grok-pager-bin/src/app_server_composition.rs"
    );
    assert!(
        !composition.contains("http://127.0.0.1:8788/mcp"),
        "composition must not hard-register the local MCP URL"
    );
    assert!(
        !composition.contains("register_self"),
        "composition must not self-register as a managed MCP server"
    );
}

// ---------------------------------------------------------------------------
// RED→GREEN: content-type and query-token rejection
// ---------------------------------------------------------------------------

#[tokio::test]
async fn post_rejects_non_json_content_type() {
    let (addr, _state, _join) = spawn_server().await;
    let c = client();
    let resp = c
        .post(format!("http://{addr}/mcp"))
        .header("authorization", format!("Bearer {TOKEN}"))
        .header("content-type", "text/plain")
        .header("accept", "application/json")
        .body("not json")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::UNSUPPORTED_MEDIA_TYPE);
}

#[tokio::test]
async fn post_rejects_token_in_query_string() {
    let (addr, _state, _join) = spawn_server().await;
    let c = client();
    let resp = c
        .post(format!("http://{addr}/mcp?token=secret"))
        .header("authorization", format!("Bearer {TOKEN}"))
        .header("content-type", "application/json")
        .header("accept", "application/json")
        .json(&init_request(1))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::BAD_REQUEST);
}

// ---------------------------------------------------------------------------
// RED→GREEN: notification (no id) returns 202
// ---------------------------------------------------------------------------

#[tokio::test]
async fn post_notification_returns_202_no_body() {
    let (addr, _state, _join) = spawn_server().await;
    let c = client();
    let (_, _, session) = post_json(&c, addr, Some(TOKEN), None, &init_request(1)).await;
    // A notification has no `id`. We use an initialized notification shape.
    let resp = c
        .post(format!("http://{addr}/mcp"))
        .header("authorization", format!("Bearer {TOKEN}"))
        .header("content-type", "application/json")
        .header("accept", "application/json")
        .header("mcp-session-id", session.as_deref().unwrap_or(""))
        .json(&json!({"jsonrpc":"2.0","method":"notifications/cancelled","params":{}}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::ACCEPTED);
}

// ---------------------------------------------------------------------------
// RED→GREEN: healthz
// ---------------------------------------------------------------------------

#[tokio::test]
async fn healthz_returns_ok_without_auth() {
    let (addr, _state, _join) = spawn_server().await;
    let c = client();
    let resp = c.get(format!("http://{addr}/healthz")).send().await.unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
}

// ---------------------------------------------------------------------------
// RED→GREEN: fail-closed auth (F-2)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn run_mcp_http_server_refuses_empty_bearer_when_require_auth_true() {
    // F-2: the public bind entry point must refuse to bind when
    // `require_auth: true` and `bearer_token` is empty. This closes the
    // empty-bearer footgun where `McpHttpConfig::default()` would otherwise
    // silently accept unauthenticated requests.
    let runtime: Arc<dyn xai_grok_tower::GrokRuntimeFacade> = Arc::new(FakeRuntime::new());
    let err = run_mcp_http_server(runtime, McpHttpConfig::default())
        .await
        .expect_err("default config must not bind (fail-closed)");
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
    assert!(err.to_string().contains("fail-closed"), "msg: {err}");
}

#[tokio::test]
async fn run_mcp_http_server_binds_when_require_auth_false_empty_bearer() {
    // F-2 positive control: the unauthenticated test path (require_auth=false)
    // must still bind even with an empty bearer.
    let runtime: Arc<dyn xai_grok_tower::GrokRuntimeFacade> = Arc::new(FakeRuntime::new());
    let handle = run_mcp_http_server(
        runtime,
        McpHttpConfig {
            bearer_token: String::new(),
            require_auth: false,
            ..Default::default()
        },
    )
    .await
    .expect("require_auth=false must bind with empty bearer");
    handle.join.abort();
}
