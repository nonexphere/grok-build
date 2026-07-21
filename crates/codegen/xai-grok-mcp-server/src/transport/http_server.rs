//! Real Streamable HTTP server for the MCP control plane.
//!
//! Feature-gated behind `streamable-http`. This module is the only place in
//! the `xai-grok-mcp-server` crate that touches the network: it binds a
//! loopback TCP socket, serves `POST/GET/DELETE /mcp` over axum, and routes
//! every `tools/call` through the shared semantic core
//! ([`invoke_tower_tool`]) — no second tool implementation, no local MCP
//! self-loop.
//!
//! # Security posture (HUMAN gate — do not weaken)
//!
//! Cleartext non-loopback bind is `experimental/unsafe` and stays that way.
//! TLS termination is a **HUMAN** gate (`D-SEC.13` / `MCP102-HUMAN`): this
//! module never advertises production TLS and never auto-promotes a cleartext
//! remote bind. The bind path emits the canonical `remote_bind_warning_exact`
//! warning for non-loopback hosts so operators cannot miss the label.
//! Loopback is the default; non-loopback must be explicit.
//!
//! # Self-loop guard
//!
//! Production composition must not register the local `/mcp` URL into the
//! session's MCP client pool (that would re-enter the nine tools through HTTP
//! and double-charge the facade). This module never imports the outbound MCP
//! client crate (`xai-grok-mcp`) and never auto-registers itself. The
//! composition-level guard is asserted by `no_self_loop_in_composition_source`.
//!
//! # SSE / event log
//!
//! Each negotiated MCP transport session owns a per-session event log fed by
//! polling [`GrokRuntimeFacade::replay`] for the bound Tower session after
//! mutating `tools/call` invocations. `GET /mcp` streams that log; `Last-Event-ID`
//! resumes within the same transport session. A foreign/expired id (greater
//! than the session's event count) returns a safe resumption error and never
//! switches Towers or replays another client's events — each transport session
//! has its own monotonic id space and its own event log.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::extract::{OriginalUri, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::sse::{Event as SseEvent, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use futures_util::StreamExt;
use futures_util::stream;
use serde_json::{Value, json};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use xai_grok_tower::GrokRuntimeFacade;
use xai_grok_tower_tools::{invoke_tower_tool, tool_error_json};

use crate::MCP_PROTOCOL_VERSION;
use crate::transport::http::{
    enforce_body_limit, presented_bearer, reject_token_query, validate_http_bearer,
};

/// Default inbound message size limit (matches the CLI matrix
/// `--max-message-bytes 1048576`).
pub const DEFAULT_MAX_MESSAGE_BYTES: usize = 1_048_576;

/// Header name carrying the negotiated MCP transport session id.
pub const MCP_SESSION_HEADER: &str = "mcp-session-id";

/// Header name carrying the MCP protocol version (gate before dispatch).
pub const MCP_PROTOCOL_VERSION_HEADER: &str = "protocol-version";

/// One event in a transport session's event log.
#[derive(Debug, Clone)]
pub struct McpSessionEvent {
    pub id: u64,
    pub event_type: String,
    pub data: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventCursorExpired;

/// Per-transport-session state.
#[derive(Debug)]
pub struct McpSession {
    /// Fingerprint of the bearer that opened this session. A request whose
    /// bearer does not match is rejected even if it is otherwise valid, so a
    /// session negotiated against Tower A's token cannot be reused against
    /// Tower B's token.
    pub bearer_fingerprint: u64,
    /// Tower instance id this transport session is bound to. A request whose
    /// server tower instance id differs is rejected.
    pub tower_instance_id: String,
    /// Monotonic event id counter for this transport session only.
    pub next_event_id: AtomicU64,
    /// Buffered events for `GET /mcp` SSE replay (capped circular buffer — R5-06).
    pub events: Mutex<Vec<McpSessionEvent>>,
    /// Max retained events per transport session (0 = unlimited; product default caps).
    pub max_events: usize,
    /// Lowest event id still retained; cursors strictly below this are expired.
    pub min_retained_event_id: AtomicU64,
    /// Bound Tower session id (the session whose facade events feed this
    /// transport session). Set when a `tools/call` returns a `sessionId`.
    pub tower_session_id: Mutex<Option<String>>,
    /// Last facade event seq already pulled into this transport session.
    pub last_replayed_seq: Mutex<u64>,
    /// History epoch learned from the Tower snapshot. Keeping it with the
    /// cursor makes a rewrite fail closed instead of silently mixing events
    /// from two histories.
    history_epoch: Mutex<Option<String>>,
    /// Monotonic binding generation used to invalidate SSE producers that
    /// were opened against a previous Tower-session identity.
    binding_generation: AtomicU64,
    /// Whether the synthetic sequence-0 snapshot has already been emitted to
    /// this transport stream. The Tower replay contract intentionally returns
    /// that snapshot whenever the cursor is 0; without this separate bit a
    /// quiet session would append the same snapshot on every SSE poll.
    snapshot_replayed: AtomicBool,
    /// Last snapshot payload used to distinguish a repeated synthetic
    /// snapshot from a legitimate later `SessionChanged` event.
    snapshot_session: Mutex<Option<xai_grok_app_server_protocol::Session>>,
    /// Serializes facade replay pulls for this MCP session. POST handlers and
    /// the long-lived SSE task may poll concurrently; without one guard both
    /// could read the same cursor and append the same non-snapshot events.
    replay_pull_lock: tokio::sync::Mutex<()>,
    /// Last known in-flight turn id for disconnect-cancels-turn (C4-F F-1).
    /// Set from mutating `tools/call` structured content when a `turnId` is
    /// present; cleared when the transport session is deleted after interrupt.
    pub active_turn_id: Mutex<Option<String>>,
    /// Notify waiters (long-lived SSE GET) that new events may be available.
    pub event_notify: tokio::sync::Notify,
    /// Last activity instant for TTL eviction (R4-09).
    pub last_active: Mutex<std::time::Instant>,
}

/// Default max buffered SSE events per MCP transport session (R5-06).
pub const DEFAULT_MAX_SESSION_EVENTS: usize = 1024;

impl McpSession {
    /// Construct a transport session (used by the server and integration tests
    /// that inject expired peers for TTL eviction coverage).
    pub fn new(bearer_fingerprint: u64, tower_instance_id: String, max_events: usize) -> Self {
        Self {
            bearer_fingerprint,
            tower_instance_id,
            next_event_id: AtomicU64::new(1),
            events: Mutex::new(Vec::new()),
            max_events,
            min_retained_event_id: AtomicU64::new(0),
            tower_session_id: Mutex::new(None),
            last_replayed_seq: Mutex::new(0),
            history_epoch: Mutex::new(None),
            binding_generation: AtomicU64::new(0),
            snapshot_replayed: AtomicBool::new(false),
            snapshot_session: Mutex::new(None),
            replay_pull_lock: tokio::sync::Mutex::new(()),
            active_turn_id: Mutex::new(None),
            event_notify: tokio::sync::Notify::new(),
            last_active: Mutex::new(std::time::Instant::now()),
        }
    }

    fn touch(&self) {
        *self.last_active.lock().unwrap() = std::time::Instant::now();
    }

    /// Bind this transport session to a Tower session and reset replay state
    /// when the identity changes. Cursor and history epoch are scoped to the
    /// Tower session; carrying either across a rebind would create a false
    /// epoch mismatch or mix events from two sessions.
    async fn bind_tower_session(&self, tower_session_id: String) {
        let _pull_guard = self.replay_pull_lock.lock().await;
        let mut bound = self.tower_session_id.lock().unwrap();
        if bound.as_deref() == Some(tower_session_id.as_str()) {
            return;
        }
        *bound = Some(tower_session_id);
        *self.last_replayed_seq.lock().unwrap() = 0;
        *self.history_epoch.lock().unwrap() = None;
        self.snapshot_replayed.store(false, Ordering::SeqCst);
        *self.snapshot_session.lock().unwrap() = None;
        self.binding_generation.fetch_add(1, Ordering::SeqCst);
        // The active turn identifier is scoped to the old Tower session too;
        // retaining it would make a later DELETE/TTL interrupt the wrong
        // session after this rebind.
        *self.active_turn_id.lock().unwrap() = None;
        self.events.lock().unwrap().clear();
        self.min_retained_event_id
            .store(self.next_event_id.load(Ordering::SeqCst), Ordering::SeqCst);
        self.event_notify.notify_waiters();
    }

    pub fn append_event(&self, event_type: String, data: String) -> u64 {
        let id = self.next_event_id.fetch_add(1, Ordering::SeqCst);
        let mut events = self.events.lock().unwrap();
        events.push(McpSessionEvent {
            id,
            event_type,
            data,
        });
        // R5-06: circular cap — drop oldest when over limit.
        if self.max_events > 0 {
            while events.len() > self.max_events {
                if let Some(dropped) = events.first() {
                    self.min_retained_event_id
                        .store(dropped.id.saturating_add(1), Ordering::SeqCst);
                }
                events.remove(0);
            }
        }
        self.event_notify.notify_waiters();
        id
    }

    /// Events strictly after `after`. Returns `Err(())` when the cursor is
    /// expired (below the retained window) — caller should treat as resync.
    pub fn events_after(&self, after: u64) -> Result<Vec<McpSessionEvent>, EventCursorExpired> {
        let min = self.min_retained_event_id.load(Ordering::SeqCst);
        if after > 0 && after < min {
            return Err(EventCursorExpired);
        }
        Ok(self
            .events
            .lock()
            .unwrap()
            .iter()
            .filter(|e| e.id > after)
            .cloned()
            .collect())
    }

    fn last_event_id(&self) -> u64 {
        self.next_event_id.load(Ordering::SeqCst).saturating_sub(1)
    }
}

/// Shared server state.
pub struct McpHttpState {
    pub runtime: Arc<dyn GrokRuntimeFacade>,
    pub agent_type: String,
    pub explicit_opt_in: bool,
    pub bearer_token: String,
    pub require_auth: bool,
    pub max_message_bytes: usize,
    pub tower_instance_id: String,
    pub sessions: Mutex<HashMap<String, Arc<McpSession>>>,
    /// R4-09: max concurrent transport sessions (0 = unlimited).
    pub max_sessions: usize,
    /// R4-09: idle TTL for transport sessions.
    pub session_ttl: Duration,
    /// R5-06: max buffered SSE events per transport session (0 = unlimited).
    pub max_session_events: usize,
}

impl std::fmt::Debug for McpHttpState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("McpHttpState")
            .field("agent_type", &self.agent_type)
            .field("require_auth", &self.require_auth)
            .field("max_message_bytes", &self.max_message_bytes)
            .field("tower_instance_id", &self.tower_instance_id)
            .field(
                "sessions",
                &self.sessions.lock().unwrap().keys().collect::<Vec<_>>(),
            )
            .finish_non_exhaustive()
    }
}

/// Configuration for [`run_mcp_http_server`].
///
/// # Fail-closed auth (F-2)
///
/// [`McpHttpConfig::default()`] sets `require_auth: true` with an empty
/// `bearer_token`. [`run_mcp_http_server`] **refuses to bind** in that state:
/// a non-empty `bearer_token` is required whenever `require_auth` is true.
/// This closes the empty-bearer footgun where two empty strings compare equal
/// in the constant-time bearer check. Tests that exercise the
/// unauthenticated path must set `require_auth: false` explicitly.
#[derive(Debug, Clone)]
pub struct McpHttpConfig {
    /// `host:port` to bind. Default `127.0.0.1:0` (ephemeral loopback).
    pub bind: String,
    /// Expected bearer token for `Authorization: Bearer <token>`. Must be
    /// non-empty when `require_auth` is true, otherwise the server refuses
    /// to bind (fail-closed).
    pub bearer_token: String,
    /// If true, requests without a valid bearer are rejected. When true,
    /// `bearer_token` must be non-empty or [`run_mcp_http_server`] returns an
    /// error. Tests that exercise the unauthenticated path set this to
    /// `false`.
    pub require_auth: bool,
    /// Inbound message size limit in bytes.
    pub max_message_bytes: usize,
    /// Tower instance id this server is bound to. Sessions carry this and a
    /// request whose server id differs is rejected.
    pub tower_instance_id: String,
    /// Agent type used for the ACL (`is_authorized`). Default `orchestrator`.
    pub agent_type: String,
    /// Explicit opt-in for non-orchestrator agent types.
    pub explicit_opt_in: bool,
    /// R4-09: max concurrent MCP transport sessions (0 = unlimited).
    pub max_sessions: usize,
    /// R4-09: idle TTL for transport sessions.
    pub session_ttl: Duration,
    /// R5-06: max buffered SSE events per transport session (0 = unlimited).
    pub max_session_events: usize,
}

impl Default for McpHttpConfig {
    fn default() -> Self {
        Self {
            bind: "127.0.0.1:0".to_owned(),
            bearer_token: String::new(),
            require_auth: true,
            max_message_bytes: DEFAULT_MAX_MESSAGE_BYTES,
            tower_instance_id: "default".to_owned(),
            agent_type: "orchestrator".to_owned(),
            explicit_opt_in: false,
            max_sessions: 1024,
            session_ttl: Duration::from_secs(3600),
            max_session_events: DEFAULT_MAX_SESSION_EVENTS,
        }
    }
}

/// Handle returned by [`run_mcp_http_server`].
#[derive(Debug)]
pub struct McpHttpHandle {
    pub addr: SocketAddr,
    pub join: JoinHandle<()>,
    /// Shared state; exposed so tests can inspect negotiated sessions.
    pub state: Arc<McpHttpState>,
}

/// Bind and serve the Streamable HTTP MCP surface. Returns the bound address,
/// the accept-loop join handle, and the shared state.
pub async fn run_mcp_http_server(
    runtime: Arc<dyn GrokRuntimeFacade>,
    config: McpHttpConfig,
) -> std::io::Result<McpHttpHandle> {
    // Fail-closed auth gate (F-2): when `require_auth` is true the server must
    // refuse to bind unless a non-empty bearer is configured. An empty
    // `bearer_token` with `require_auth: true` would otherwise silently accept
    // unauthenticated requests (the constant-time compare sees two empty
    // strings as equal). `McpHttpConfig::default()` sets `require_auth: true`
    // with an empty token, so the default config cannot bind — operators must
    // explicitly provide a bearer or set `require_auth: false` for the
    // unauthenticated test path.
    if config.require_auth && config.bearer_token.trim().is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "require_auth is true but bearer_token is empty; refusing to bind (fail-closed). \
             Provide a non-empty bearer_token or set require_auth: false explicitly.",
        ));
    }

    // Reject credentials embedded in the bind string (defense in depth).
    if config.bind.contains('@') || config.bind.contains("token=") {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "bind address must not contain credentials",
        ));
    }

    // Emit the canonical cleartext non-loopback warning. Loopback is the
    // default; non-loopback is experimental/unsafe and must be explicit.
    let host = config.bind.split(':').next().unwrap_or("127.0.0.1");
    if let Some(warning) = bind_warning(host) {
        eprintln!("{warning}");
    }

    let state = Arc::new(McpHttpState {
        runtime,
        agent_type: config.agent_type,
        explicit_opt_in: config.explicit_opt_in,
        bearer_token: config.bearer_token,
        require_auth: config.require_auth,
        max_message_bytes: config.max_message_bytes,
        tower_instance_id: config.tower_instance_id,
        sessions: Mutex::new(HashMap::new()),
        max_sessions: config.max_sessions,
        max_session_events: config.max_session_events,
        session_ttl: config.session_ttl,
    });

    let app = Router::new()
        .route("/mcp", post(post_mcp).get(get_mcp).delete(delete_mcp))
        .route("/healthz", get(healthz))
        .with_state(state.clone());

    let listener = TcpListener::bind(&config.bind).await?;
    let addr = listener.local_addr()?;
    let join = tokio::spawn(async move {
        // `axum::serve` runs until the listener is closed (abort the join
        // handle to stop the server).
        let _ = axum::serve(listener, app).await;
    });

    Ok(McpHttpHandle { addr, join, state })
}

/// Cleartext non-loopback bind warning, exposed for testability. Returns the
/// canonical `experimental/unsafe` warning string for non-loopback hosts and
/// `None` for loopback. TLS stays a HUMAN gate (D-SEC.13).
pub fn bind_warning(host: &str) -> Option<&'static str> {
    if is_loopback_host(host) {
        None
    } else {
        Some(
            "experimental/unsafe: binding MCP Streamable HTTP on a non-loopback address without TLS is unsafe; TLS termination is a HUMAN gate (D-SEC.13)",
        )
    }
}

fn is_loopback_host(host: &str) -> bool {
    host == "127.0.0.1"
        || host == "localhost"
        || host == "::1"
        || host == "[::1]"
        || host.starts_with("127.")
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

async fn healthz() -> impl IntoResponse {
    StatusCode::OK
}

/// `POST /mcp` — JSON-RPC request or notification.
///
/// Negotiation: if the client `Accept` only `text/event-stream` (no JSON),
/// the response is a single SSE event containing the JSON-RPC envelope;
/// otherwise the response is `application/json`. Notifications (no `id`)
/// return `202 Accepted` with no body.
async fn post_mcp(
    State(state): State<Arc<McpHttpState>>,
    OriginalUri(uri): OriginalUri,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> Response {
    // 1. Auth (indistinguishable 401 for missing/wrong/malformed bearer).
    if let Err(resp) = require_auth(&state, &headers, uri.query()) {
        return resp;
    }
    // 2. Reject credentials in the query string (defense in depth).
    if let Some(query) = uri.query()
        && let Err(resp) = check_query(&state, query)
    {
        return resp;
    }
    // 3. Content-Type must be JSON.
    if !is_json_content_type(&headers) {
        return StatusCode::UNSUPPORTED_MEDIA_TYPE.into_response();
    }
    // 4. Body limit (before tool dispatch).
    if let Err(resp) = check_body_limit(&state, body.len()) {
        return resp;
    }
    // 5. Protocol-version gate (before dispatch).
    if let Err(resp) = check_protocol_version(&headers) {
        return resp;
    }
    // 6. Parse JSON-RPC request.
    let request: Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(_) => return jsonrpc_error(StatusCode::BAD_REQUEST, None, -32700, "Parse error"),
    };
    let id = request.get("id").cloned();
    let method = request.get("method").and_then(Value::as_str).unwrap_or("");

    // 7. Session binding. `initialize` opens a new transport session; every
    //    other method requires a negotiated `Mcp-Session-Id` header bound to
    //    this server's Tower instance and bearer fingerprint.
    let session = if method == "initialize" {
        // R4-09 / R5-07: evict idle sessions (and cancel their turns), then
        // enforce max_sessions quota.
        evict_expired_sessions_and_interrupt(&state).await;
        let fingerprint = bearer_fingerprint(&headers, uri.query(), &state.bearer_token);
        let session = Arc::new(McpSession::new(
            fingerprint,
            state.tower_instance_id.clone(),
            state.max_session_events,
        ));
        let sid = new_session_id();
        {
            let mut map = state.sessions.lock().unwrap();
            if state.max_sessions > 0 && map.len() >= state.max_sessions {
                return jsonrpc_error(
                    StatusCode::SERVICE_UNAVAILABLE,
                    id,
                    -32029,
                    "MCP transport session limit reached",
                );
            }
            map.insert(sid.clone(), session.clone());
        }
        Some((sid, session))
    } else {
        match lookup_session(&state, &headers, uri.query()).await {
            Ok(s) => {
                s.1.touch();
                Some(s)
            }
            Err(resp) => return resp,
        }
    };

    // 8. Dispatch.
    let response = dispatch_jsonrpc(&state, &request, session.as_ref()).await;

    // 9. After a mutating tools/call, pull new facade events into the
    //    transport session's event log (feeds GET /mcp SSE).
    if let Some((_, session)) = &session
        && method == "tools/call"
        && let Err(err) = pull_facade_events(&state, session).await
    {
        record_pull_error(session, err);
    }

    // 10. Notification (no id) → 202 Accepted, no body.
    if id.is_none() {
        return StatusCode::ACCEPTED.into_response();
    }

    // 11. Negotiate JSON vs SSE response.
    let accept = headers
        .get(header::ACCEPT)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("application/json");
    let wants_sse_only =
        accept.contains("text/event-stream") && !accept.contains("application/json");

    let (sid, _) = session.expect("session is present for requests with id");
    if wants_sse_only {
        let event = SseEvent::default().data(response.to_string());
        let sse = Sse::new(stream::once(async {
            Ok::<SseEvent, std::convert::Infallible>(event)
        }))
        .keep_alive(KeepAlive::default());
        let mut resp = sse.into_response();
        resp.headers_mut().insert(
            MCP_SESSION_HEADER,
            sid.parse().expect("session id is a valid header value"),
        );
        return resp;
    }

    // JSON response, always echoing the negotiated session id.
    let mut resp = Json(response).into_response();
    resp.headers_mut().insert(
        MCP_SESSION_HEADER,
        sid.parse().expect("session id is a valid header value"),
    );
    resp
}

/// `GET /mcp` — long-lived SSE event stream / resume (C4-F F-1 live push).
///
/// Streams events already buffered for the transport session, then keeps the
/// connection open and polls the facade for new events (via
/// [`pull_facade_events`]) until the client disconnects or the session is
/// deleted. Foreign/expired `Last-Event-ID` still yields a one-shot
/// `resumption_error` and ends.
async fn get_mcp(
    State(state): State<Arc<McpHttpState>>,
    OriginalUri(uri): OriginalUri,
    headers: HeaderMap,
) -> Response {
    if let Err(resp) = require_auth(&state, &headers, uri.query()) {
        return resp;
    }
    // Require Accept: text/event-stream.
    let accept = headers
        .get(header::ACCEPT)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");
    if !accept.contains("text/event-stream") {
        return StatusCode::NOT_ACCEPTABLE.into_response();
    }
    let session = match lookup_session(&state, &headers, uri.query()).await {
        Ok(s) => s,
        Err(resp) => return resp,
    };
    let last_event_id = headers
        .get("last-event-id")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok());

    let (sid, session) = session;
    let last = session.last_event_id();
    let cursor = last_event_id.unwrap_or(0);
    let foreign = last_event_id.map(|id| id > last).unwrap_or(false);
    // R5-06: cursor below retained window is expired (cap dropped old events).
    let expired_cursor =
        cursor > 0 && cursor < session.min_retained_event_id.load(Ordering::SeqCst);

    let sse_stream: stream::BoxStream<'static, Result<SseEvent, std::convert::Infallible>> =
        if foreign || expired_cursor {
            // Foreign/expired id: emit a safe resumption error and end. Never
            // replay another client's events.
            let payload = json!({
                "error": "resumption_error",
                "lastEventId": last_event_id,
                "sessionLastEventId": last,
                "minRetainedEventId": session.min_retained_event_id.load(Ordering::SeqCst),
            });
            let event = SseEvent::default()
                .event("resumption_error")
                .data(payload.to_string());
            stream::once(async { Ok(event) }).boxed()
        } else {
            let state_c = state.clone();
            let session_c = session.clone();
            let sid_c = sid.clone();
            let binding_generation = session.binding_generation.load(Ordering::SeqCst);
            // Long-lived push via channel: producer pulls facade events and exits
            // when the transport session is deleted or the SSE consumer drops
            // (send fails → disconnect-cancels-turn).
            let (tx, rx) = tokio::sync::mpsc::channel::<McpSessionEvent>(64);
            tokio::spawn(async move {
                let mut after = cursor;
                match session_c.events_after(after) {
                    Ok(initial) => {
                        for e in initial {
                            after = e.id;
                            if tx.send(e).await.is_err() {
                                interrupt_active_turn(&state_c, &session_c, &sid_c).await;
                                return;
                            }
                        }
                    }
                    Err(_) => {
                        // The cursor can expire between the preflight check above
                        // and this first buffer read. Do not turn that race into a
                        // silent clean disconnect: the client must receive an
                        // explicit resumption error and reconnect from a snapshot.
                        let last = session_c.last_event_id();
                        let min = session_c.min_retained_event_id.load(Ordering::SeqCst);
                        let error = replay_resumption_error(after, last, min);
                        if tx.send(error).await.is_err() {
                            interrupt_active_turn(&state_c, &session_c, &sid_c).await;
                        }
                        return;
                    }
                }
                loop {
                    if session_c.binding_generation.load(Ordering::SeqCst) != binding_generation {
                        let last = session_c.last_event_id();
                        let error = McpSessionEvent {
                        id: last,
                        event_type: "resumption_error".to_owned(),
                        data: json!({
                            "error": "resumption_error",
                            "code": "session_rebound",
                            "message": "Tower session binding changed; reconnect from a new snapshot.",
                            "sessionLastEventId": last,
                        })
                        .to_string(),
                    };
                        let _ = tx.send(error).await;
                        return;
                    }
                    if !state_c.sessions.lock().unwrap().contains_key(&sid_c) {
                        break;
                    }
                    if let Err(err) = pull_facade_events(&state_c, &session_c).await {
                        let last = session_c.last_event_id();
                        let message = replay_error_message(err.code);
                        let error = McpSessionEvent {
                            id: last,
                            event_type: "resumption_error".to_owned(),
                            data: json!({
                                "error": "resumption_error",
                                "code": err.code,
                                "message": message,
                                "sessionLastEventId": last,
                            })
                            .to_string(),
                        };
                        let _ = tx.send(error).await;
                        return;
                    }
                    let pending = match session_c.events_after(after) {
                        Ok(p) => p,
                        Err(_) => {
                            // The buffer may expire while this stream is already
                            // open. Tell the client to resync instead of ending
                            // silently, which would otherwise look like a clean
                            // disconnect and lose the gap signal.
                            let last = session_c.last_event_id();
                            let min = session_c.min_retained_event_id.load(Ordering::SeqCst);
                            let error = replay_resumption_error(after, last, min);
                            if tx.send(error).await.is_err() {
                                interrupt_active_turn(&state_c, &session_c, &sid_c).await;
                            }
                            return;
                        }
                    };
                    if pending.is_empty() {
                        tokio::select! {
                            _ = session_c.event_notify.notified() => {}
                            _ = tokio::time::sleep(Duration::from_millis(250)) => {}
                        }
                        continue;
                    }
                    for e in pending {
                        after = e.id;
                        if tx.send(e).await.is_err() {
                            // Client disconnected mid-stream → cancel in-flight turn.
                            interrupt_active_turn(&state_c, &session_c, &sid_c).await;
                            return;
                        }
                    }
                }
            });
            stream::unfold(rx, |mut rx| async move {
                match rx.recv().await {
                    Some(e) => {
                        let event = SseEvent::default()
                            .id(e.id.to_string())
                            .event(e.event_type)
                            .data(e.data);
                        Some((Ok(event), rx))
                    }
                    None => None,
                }
            })
            .boxed()
        };

    let sse = Sse::new(sse_stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(15)));
    let _ = &sid;
    sse.into_response()
}

/// Build the transport-level signal for a replay cursor that can no longer be
/// resumed. Keeping this shape shared by the initial-read race and the
/// long-lived polling path prevents either path from silently dropping a gap.
fn replay_resumption_error(after: u64, last: u64, min_retained: u64) -> McpSessionEvent {
    McpSessionEvent {
        id: last,
        event_type: "resumption_error".to_owned(),
        data: json!({
            "error": "resumption_error",
            "lastEventId": after,
            "sessionLastEventId": last,
            "minRetainedEventId": min_retained,
        })
        .to_string(),
    }
}

/// `DELETE /mcp` — terminate the transport session; interrupt any active turn
/// (C4-F F-1 disconnect-cancels-turn).
async fn delete_mcp(
    State(state): State<Arc<McpHttpState>>,
    OriginalUri(uri): OriginalUri,
    headers: HeaderMap,
) -> Response {
    if let Err(resp) = require_auth(&state, &headers, uri.query()) {
        return resp;
    }
    // DELETE has the same session-bound authorization contract as GET/POST:
    // validate the negotiated bearer fingerprint and Tower instance before
    // removing anything. Checking only the current global bearer would let a
    // stale or foreign session id be deleted after a reconfiguration.
    let (sid, _) = match lookup_session(&state, &headers, uri.query()).await {
        Ok(session) => session,
        Err(resp) => return resp,
    };
    let removed = state.sessions.lock().unwrap().remove(&sid);
    let Some(session) = removed else {
        return StatusCode::NOT_FOUND.into_response();
    };
    // Wake any long-lived SSE GET so it exits promptly.
    session.event_notify.notify_waiters();
    interrupt_active_turn(&state, &session, &sid).await;
    StatusCode::OK.into_response()
}

/// Best-effort interrupt of the session's active turn (disconnect-cancels-turn).
async fn interrupt_active_turn(
    state: &Arc<McpHttpState>,
    session: &Arc<McpSession>,
    transport_sid: &str,
) {
    let tower_sid = session.tower_session_id.lock().unwrap().clone();
    let turn_id = session.active_turn_id.lock().unwrap().take();
    if let (Some(session_id), Some(turn_id)) = (tower_sid, turn_id) {
        let _ = state
            .runtime
            .interrupt_turn(xai_grok_app_server_protocol::TurnInterruptParams {
                session_id,
                turn_id,
                idempotency_key: format!("mcp-disconnect-{transport_sid}"),
            })
            .await;
    }
}

// ---------------------------------------------------------------------------
// Dispatch — routes tools/list and tools/call through the shared semantic core.
// ---------------------------------------------------------------------------

async fn dispatch_jsonrpc(
    state: &Arc<McpHttpState>,
    request: &Value,
    session: Option<&(String, Arc<McpSession>)>,
) -> Value {
    let id = request.get("id").cloned().unwrap_or(Value::Null);
    let method = request.get("method").and_then(Value::as_str).unwrap_or("");
    match method {
        "initialize" => json!({
            "jsonrpc":"2.0","id":id,
            "result":{
                "protocolVersion": MCP_PROTOCOL_VERSION,
                "capabilities": {"tools": {}},
                "serverInfo": {"name":"grok-oss-mcp-server","version":"0.0.0-experimental"}
            }
        }),
        "tools/list" => json!({
            "jsonrpc":"2.0","id":id,
            "result": {
                "tools": xai_grok_tower_tools::TOWER_TOOL_DESCRIPTORS.iter().map(|d| json!({
                    "name": d.name,
                    "description": d.description,
                    "inputSchema": xai_grok_tower_tools::tool_schema(d.name, false).unwrap_or_else(|| json!({"type":"object"})),
                })).collect::<Vec<_>>()
            }
        }),
        "tools/call" => {
            let name = request["params"]["name"].as_str().unwrap_or("");
            let args = request["params"]["arguments"].clone();
            match invoke_tower_tool(
                state.runtime.clone(),
                &state.agent_type,
                state.explicit_opt_in,
                name,
                args,
            )
            .await
            {
                Ok(result) => {
                    // Bind the transport session to the Tower session id
                    // returned by mutating tools so the SSE feed can pull
                    // facade events for the right session.
                    if let Some((_, s)) = session {
                        if let Some(tsid) = result.get("sessionId").and_then(Value::as_str) {
                            s.bind_tower_session(tsid.to_owned()).await;
                        }
                        // Track only non-terminal turns for disconnect/TTL
                        // cancellation. Tool adapters may include the
                        // historical `turnId` in a completed result; treating
                        // that identifier as active would interrupt a turn
                        // that has already finished when the MCP session is
                        // later deleted or evicted.
                        let is_completed =
                            result.get("state").and_then(Value::as_str) == Some("completed");
                        if is_completed {
                            *s.active_turn_id.lock().unwrap() = None;
                        } else if let Some(tid) = result
                            .get("turnId")
                            .or_else(|| result.get("turn").and_then(|t| t.get("turnId")))
                            .and_then(Value::as_str)
                        {
                            *s.active_turn_id.lock().unwrap() = Some(tid.to_owned());
                        }
                        s.event_notify.notify_waiters();
                    }
                    // Pull fresh facade events into the SSE buffer after mutations.
                    if let Some((_, s)) = session
                        && let Err(err) = pull_facade_events(state, s).await
                    {
                        record_pull_error(s, err);
                    }
                    json!({
                        "jsonrpc":"2.0","id":id,
                        "result": {
                            "content": [{"type":"text","text": result.to_string()}],
                            "structuredContent": result
                        }
                    })
                }
                Err(err) => json!({
                    "jsonrpc":"2.0","id":id,
                    "result": {
                        "content": [{"type":"text","text": format!("{}: {}", err.code, err.message)}],
                        "structuredContent": tool_error_json(&err),
                        "isError": true
                    }
                }),
            }
        }
        other => json!({
            "jsonrpc":"2.0","id":id,
            "error": {"code": -32601, "message": format!("Method not found: {other}")}
        }),
    }
}

/// Pull new facade events for the bound Tower session into the transport
/// session's event log. Each runtime event becomes one SSE event with a
/// monotonic transport-session-scoped id.
async fn pull_facade_events(
    state: &Arc<McpHttpState>,
    session: &Arc<McpSession>,
) -> Result<(), xai_grok_tower::RuntimeError> {
    // Keep cursor read, replay, append, and cursor advance as one per-session
    // critical section. The HTTP POST and SSE paths intentionally share this
    // helper and may otherwise race on the same replay page.
    let _pull_guard = session.replay_pull_lock.lock().await;
    let tower_session_id = match session.tower_session_id.lock().unwrap().clone() {
        Some(s) => s,
        None => return Ok(()),
    };
    let after = *session.last_replayed_seq.lock().unwrap();
    let history_epoch = session.history_epoch.lock().unwrap().clone();
    let cursor = xai_grok_app_server_protocol::SubscribeParams {
        session_id: tower_session_id.clone(),
        after_event_seq: xai_grok_app_server_protocol::WireCounter::new(after),
        history_epoch,
    };
    let page = state.runtime.replay(cursor).await?;
    let snapshot_was_seen = session.snapshot_replayed.swap(true, Ordering::SeqCst);
    for event in page.events {
        let repeated_snapshot =
            if let xai_grok_tower::RuntimeEvent::SessionChanged(ref session_snapshot) = event {
                *session.history_epoch.lock().unwrap() =
                    Some(session_snapshot.history_epoch.clone());
                let mut known_snapshot = session.snapshot_session.lock().unwrap();
                let repeated = snapshot_was_seen
                    && after == 0
                    && known_snapshot.as_ref() == Some(session_snapshot);
                // Keep the first snapshot identity stable. Later real
                // SessionChanged events must not replace it, otherwise a repeated
                // synthetic snapshot could be mistaken for a new event.
                if known_snapshot.is_none() {
                    *known_snapshot = Some(session_snapshot.clone());
                }
                repeated
            } else {
                false
            };
        if repeated_snapshot {
            continue;
        }
        let (event_type, data) = runtime_event_to_json(event);
        session.append_event(event_type, data);
    }
    *session.last_replayed_seq.lock().unwrap() = page.replayed_through.as_u64();
    Ok(())
}

/// Keep replay diagnostics useful without forwarding storage paths or other
/// runtime details through the public SSE stream.
fn replay_error_message(code: &str) -> &'static str {
    match code {
        "epoch_mismatch" | "resync_required" | "cursor_too_old" => {
            "Replay cursor is no longer valid; reconnect from a new snapshot."
        }
        _ => "Replay is unavailable; reconnect from a new snapshot.",
    }
}

/// Convert a replay failure observed during POST dispatch into a transport
/// event. The JSON-RPC response may still describe a successful tool call, but
/// the SSE consumer must not be left believing its cursor is complete. Keep
/// the runtime message private; clients only need the stable error code and
/// cursor coordinates needed to resynchronize.
fn record_pull_error(session: &McpSession, err: xai_grok_tower::RuntimeError) {
    let after = *session.last_replayed_seq.lock().unwrap();
    let last = session.last_event_id();
    let min_retained = session.min_retained_event_id.load(Ordering::SeqCst);
    session.append_event(
        "resumption_error".to_owned(),
        json!({
            "error": "resumption_error",
            "code": err.code,
            "lastEventId": after,
            "sessionLastEventId": last,
            "minRetainedEventId": min_retained,
        })
        .to_string(),
    );
}

fn runtime_event_to_json(event: xai_grok_tower::RuntimeEvent) -> (String, String) {
    use xai_grok_tower::RuntimeEvent;
    let (ty, value) = match event {
        RuntimeEvent::SessionChanged(s) => ("session_changed", json!({"session": s})),
        RuntimeEvent::TurnChanged(t) => ("turn_changed", json!({"turn": t})),
        RuntimeEvent::ItemStarted(i) => ("item_started", json!({"item": i})),
        RuntimeEvent::ItemCompleted(i) => ("item_completed", json!({"item": i})),
        RuntimeEvent::ItemDelta {
            session_id,
            turn_id,
            item_id,
            revision,
            delta,
        } => (
            "item_delta",
            json!({"sessionId": session_id, "turnId": turn_id, "itemId": item_id, "revision": revision, "delta": delta}),
        ),
        RuntimeEvent::InteractionRequested(r) => ("interaction_requested", json!({"request": r})),
    };
    (ty.to_owned(), value.to_string())
}

// ---------------------------------------------------------------------------
// Auth / validation helpers.
// ---------------------------------------------------------------------------

fn require_auth(
    state: &McpHttpState,
    headers: &HeaderMap,
    query: Option<&str>,
) -> Result<(), Response> {
    if !state.require_auth {
        return Ok(());
    }
    let auth = headers
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());
    let presented = presented_bearer(auth, query);
    let valid = presented.as_deref().is_some_and(|token| {
        let auth = format!("Bearer {token}");
        validate_http_bearer(Some(&auth), &state.bearer_token).is_ok()
    });
    if !valid {
        return Err(unauthorized());
    }
    Ok(())
}

fn check_query(state: &McpHttpState, query: &str) -> Result<(), Response> {
    let _ = state;
    if reject_token_query(query).is_err() {
        return Err(StatusCode::BAD_REQUEST.into_response());
    }
    Ok(())
}

fn is_json_content_type(headers: &HeaderMap) -> bool {
    headers
        .get(header::CONTENT_TYPE)
        .and_then(|h| h.to_str().ok())
        .map(|ct| ct.to_ascii_lowercase().contains("application/json"))
        .unwrap_or(false)
}

fn check_body_limit(state: &McpHttpState, len: usize) -> Result<(), Response> {
    match enforce_body_limit(len, state.max_message_bytes) {
        Ok(()) => Ok(()),
        Err(_) => Err(StatusCode::PAYLOAD_TOO_LARGE.into_response()),
    }
}

fn check_protocol_version(headers: &HeaderMap) -> Result<(), Response> {
    if let Some(v) = headers
        .get(MCP_PROTOCOL_VERSION_HEADER)
        .and_then(|h| h.to_str().ok())
        && v != MCP_PROTOCOL_VERSION
    {
        return Err(jsonrpc_error(
            StatusCode::BAD_REQUEST,
            None,
            -32006,
            "Unsupported protocol version",
        ));
    }
    Ok(())
}

/// Resolve a negotiated transport session after R5-07 TTL eviction that
/// **always** interrupts active turns on expired peers (same path as
/// initialize / DELETE). Async because interrupt goes through the facade.
async fn lookup_session(
    state: &Arc<McpHttpState>,
    headers: &HeaderMap,
    query: Option<&str>,
) -> Result<(String, Arc<McpSession>), Response> {
    // R5-07: every lookup path must cancel in-flight Tower turns on TTL
    // expiry — not only initialize. Sync-only retain left orphans.
    evict_expired_sessions_and_interrupt(state).await;
    let sid = match headers
        .get(MCP_SESSION_HEADER)
        .and_then(|h| h.to_str().ok())
    {
        Some(s) => s.to_owned(),
        None => return Err(StatusCode::BAD_REQUEST.into_response()),
    };
    let session = match state.sessions.lock().unwrap().get(&sid) {
        Some(s) => s.clone(),
        None => return Err(StatusCode::NOT_FOUND.into_response()),
    };
    // Bearer fingerprint binding: the request bearer must match the session's
    // opening bearer. A session negotiated with Tower A's token cannot be
    // reused once the server is reconfigured for Tower B's token.
    if state.require_auth {
        let fingerprint = bearer_fingerprint(headers, query, &state.bearer_token);
        if fingerprint != session.bearer_fingerprint {
            return Err(unauthorized());
        }
    }
    // Tower instance binding.
    if session.tower_instance_id != state.tower_instance_id {
        return Err(StatusCode::NOT_FOUND.into_response());
    }
    Ok((sid, session))
}

/// R4-09 / R5-07: drop transport sessions idle longer than `session_ttl`.
/// Expired sessions are removed first; callers that need async interrupt of
/// active turns should use [`evict_expired_sessions_and_interrupt`].
fn evict_expired_sessions(state: &McpHttpState) -> Vec<(String, Arc<McpSession>)> {
    if state.session_ttl.is_zero() {
        return Vec::new();
    }
    let now = std::time::Instant::now();
    let mut map = state.sessions.lock().unwrap();
    let mut expired = Vec::new();
    map.retain(|sid, s| {
        let alive = now.duration_since(*s.last_active.lock().unwrap()) < state.session_ttl;
        if !alive {
            expired.push((sid.clone(), s.clone()));
        }
        alive
    });
    for (_, s) in &expired {
        s.event_notify.notify_waiters();
    }
    expired
}

/// R5-07: evict idle sessions and interrupt any in-flight Tower turns so
/// TTL expiry does not leave orphan execution without a controller.
async fn evict_expired_sessions_and_interrupt(state: &Arc<McpHttpState>) {
    let expired = evict_expired_sessions(state);
    for (sid, session) in expired {
        interrupt_active_turn(state, &session, &sid).await;
    }
}

fn bearer_fingerprint(headers: &HeaderMap, query: Option<&str>, expected: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let header = headers
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());
    let presented = presented_bearer(header, query);
    let token = presented.as_deref().unwrap_or("");
    // Fingerprint the *expected* token against the presented token: if they
    // match (auth already passed), the fingerprint is stable for this bearer.
    // If they differ, the fingerprint is of the presented (wrong) token, which
    // will not match the session's stored fingerprint of the valid bearer.
    let mut hasher = DefaultHasher::new();
    if token == expected {
        expected.hash(&mut hasher);
    } else {
        token.hash(&mut hasher);
    }
    hasher.finish()
}

fn unauthorized() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        [(header::WWW_AUTHENTICATE, "Bearer")],
    )
        .into_response()
}

fn jsonrpc_error(status: StatusCode, id: Option<Value>, code: i64, message: &str) -> Response {
    let body = Json(json!({
        "jsonrpc":"2.0","id": id,
        "error": {"code": code, "message": message}
    }));
    (status, body).into_response()
}

fn new_session_id() -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::sync::atomic::AtomicU64;
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let mut h = DefaultHasher::new();
    (std::time::SystemTime::now(), n).hash(&mut h);
    format!("mcp-{:x}", h.finish())
}

// ---------------------------------------------------------------------------
// Canaries — production source must not introduce a local MCP self-loop.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod self_loop_canary {
    /// The HTTP server module must not import the outbound MCP client crate
    /// (`xai-grok-mcp`) or reference a `McpClient` symbol. A self-loop would
    /// re-enter the nine tools through HTTP and double-charge the facade.
    #[test]
    fn http_server_does_not_import_outbound_mcp_client() {
        let production = include_str!("http_server.rs")
            .split("#[cfg(test)]")
            .next()
            .unwrap();
        assert!(!production.contains("xai_grok_mcp::"));
        assert!(!production.contains("McpClient"));
        assert!(!production.contains("register_self"));
    }
}

#[cfg(test)]
mod replay_error_contract {
    use super::{McpSession, record_pull_error, replay_error_message, replay_resumption_error};
    use serde_json::Value;

    #[test]
    fn expired_cursor_error_is_explicit_and_safe() {
        let event = replay_resumption_error(7, 12, 10);
        assert_eq!(event.event_type, "resumption_error");
        let data: Value = serde_json::from_str(&event.data).unwrap();
        assert_eq!(data["error"], "resumption_error");
        assert_eq!(data["lastEventId"], 7);
        assert_eq!(data["sessionLastEventId"], 12);
        assert_eq!(data["minRetainedEventId"], 10);
        assert!(!event.data.contains("token"));
    }

    #[test]
    fn post_pull_failure_becomes_safe_transport_event() {
        let session = McpSession::new(0, "default".into(), 8);
        record_pull_error(
            &session,
            xai_grok_tower::RuntimeError {
                code: "epoch_mismatch",
                message: "private storage path".into(),
            },
        );
        let events = session.events.lock().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "resumption_error");
        assert!(!events[0].data.contains("private storage path"));
        let data: Value = serde_json::from_str(&events[0].data).unwrap();
        assert_eq!(data["code"], "epoch_mismatch");
    }

    #[test]
    fn replay_error_message_never_forwards_runtime_detail() {
        assert_eq!(
            replay_error_message("epoch_mismatch"),
            "Replay cursor is no longer valid; reconnect from a new snapshot."
        );
        assert_eq!(
            replay_error_message("storage_error"),
            "Replay is unavailable; reconnect from a new snapshot."
        );
    }
}

#[cfg(test)]
mod fail_closed_auth_canary {
    use super::*;
    use xai_grok_tower::FakeRuntime;

    /// F-2: `McpHttpConfig::default()` (require_auth=true, empty bearer) must
    /// refuse to bind — fail-closed. The empty-bearer footgun is closed at the
    /// bind boundary.
    #[tokio::test]
    async fn default_config_refuses_to_bind_with_empty_bearer() {
        let runtime: Arc<dyn GrokRuntimeFacade> = Arc::new(FakeRuntime::new());
        let err = run_mcp_http_server(runtime, McpHttpConfig::default())
            .await
            .expect_err("default config must not bind (fail-closed)");
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
        assert!(
            err.to_string().contains("fail-closed"),
            "error must explain fail-closed: {err}"
        );
    }

    /// F-2: an explicit empty bearer with `require_auth: true` is also
    /// rejected, even when other fields are set.
    #[tokio::test]
    async fn empty_bearer_with_require_auth_refuses_to_bind() {
        let runtime: Arc<dyn GrokRuntimeFacade> = Arc::new(FakeRuntime::new());
        let err = run_mcp_http_server(
            runtime,
            McpHttpConfig {
                bearer_token: "   ".to_owned(), // whitespace-only is empty
                require_auth: true,
                ..Default::default()
            },
        )
        .await
        .expect_err("whitespace-only bearer must not bind");
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
    }

    /// F-2: `require_auth: false` with an empty bearer is permitted (the
    /// unauthenticated test path must still bind).
    #[tokio::test]
    async fn require_auth_false_with_empty_bearer_binds() {
        let runtime: Arc<dyn GrokRuntimeFacade> = Arc::new(FakeRuntime::new());
        let handle = run_mcp_http_server(
            runtime,
            McpHttpConfig {
                bearer_token: String::new(),
                require_auth: false,
                ..Default::default()
            },
        )
        .await
        .expect("require_auth=false with empty bearer must bind");
        handle.join.abort();
    }
}
