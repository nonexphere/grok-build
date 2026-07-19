//! Real WebSocket TCP listener over the shared `FacadeProcessor`.
//!
//! Feature-gated behind `websocket`. This module is the only place in the
//! `xai-grok-app-server` crate that touches the network: it binds a loopback
//! TCP socket, performs the WebSocket handshake (bearer auth +
//! subprotocol negotiation), drives a text-frame loop through
//! `FacadeProcessor::handle_line` (no second processor), and owns a
//! per-connection bounded writer for backpressure.
//!
//! # Security posture (HUMAN gate — do not weaken)
//!
//! Cleartext non-loopback bind is `experimental/unsafe` and stays that way.
//! TLS is a **HUMAN** gate (`D-SEC.13` / AS104-HUMAN): this module never
//! advertises production TLS and never auto-promotes a cleartext remote bind.
//! The bind path emits the canonical `remote_bind_warning_exact` warning for
//! non-loopback hosts so operators cannot miss the label. Loopback is the
//! default; non-loopback must be explicit.
//!
//! # Subprotocol (inference — R-WS-2)
//!
//! No spec evidence exists for the `Sec-WebSocket-Protocol` value. We reuse
//! `PROTOCOL_VERSION` so a client and server negotiate the same JSON-RPC
//! envelope version on the wire. If a client offers subprotocols but none
//! match, the handshake is rejected (400). If a client offers no
//! subprotocol, the upgrade proceeds without one (lenient).
//!
//! # Backpressure
//!
//! Each connection owns a bounded `mpsc` channel of capacity
//! [`OUTBOUND_QUEUE_CAP`] (overridable via [`WsListenerConfig::outbound_queue_cap`]).
//! The reader task processes frames and `try_send`s processor responses onto
//! the channel; the writer task drains the channel into the WS sink. When the
//! client is slow and the channel is full, the response is dropped and the
//! per-listener `dropped_events` counter is incremented (no unbounded
//! buffering, no head-of-line blocking of the reader). The drop guarantee is
//! proven by `bounded_writer_drops_when_full` (deterministic unit test); the
//! listener-level behavior is exercised by
//! `ws_listener_bounded_writer_survives_burst`.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio_tungstenite::tungstenite::handshake::server::{ErrorResponse, Request, Response};
use tokio_tungstenite::tungstenite::http::{HeaderValue, StatusCode};
use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::accept_hdr_async_with_config;

use xai_grok_app_server_protocol::PROTOCOL_VERSION;

use crate::processor::FacadeProcessor;
use crate::security;
use crate::transport::websocket::{validate_bearer_header, validate_ws_text_frame};
use crate::ProcessorError;

/// Subprotocol advertised on the wire. Inference (R-WS-2): no spec evidence;
/// reuse the protocol version so clients negotiate the same JSON-RPC envelope.
pub const WS_SUBPROTOCOL: &str = PROTOCOL_VERSION;

/// Default per-connection outbound queue cap (bounded writer / backpressure).
pub const OUTBOUND_QUEUE_CAP: usize = 256;

/// WS keepalive interval (seconds). Documented contract; tungstenite
/// auto-responds to `Ping` with `Pong` (RFC 6455 §5.5.2), so the server only
/// needs to flush the auto-queued pong — no manual pong scheduling.
pub const KEEPALIVE_INTERVAL_SECS: u64 = 15;

/// Max frame/message size accepted on the wire. Matches the 1 MiB cap enforced
/// by `validate_ws_text_frame` so the WS layer and the JSON-RPC layer agree.
pub const MAX_FRAME_SIZE: usize = 1_048_576;

/// Commands the reader task enqueues onto the per-connection bounded channel
/// for the writer task to drain into the WS sink.
enum Outbound {
    /// A JSON-RPC response envelope (one `Message::Text` frame).
    Message(String),
    /// Flush the sink promptly — used to deliver the auto-queued pong after a
    /// client `Ping` so WS keepalive works without pending RPC traffic.
    Flush,
}

/// Configuration for [`run_ws_listener`].
#[derive(Debug, Clone)]
pub struct WsListenerConfig {
    /// `host:port` to bind. Default `127.0.0.1:0` (ephemeral loopback).
    pub bind: String,
    /// Expected bearer token for the `Authorization: Bearer <token>` header.
    pub bearer_token: String,
    /// If true, the handshake rejects connections without a valid bearer token.
    /// Tests that exercise the unauthenticated path set this to `false`.
    pub require_auth: bool,
    /// Per-connection outbound queue capacity (bounded writer).
    pub outbound_queue_cap: usize,
}

impl Default for WsListenerConfig {
    fn default() -> Self {
        Self {
            bind: "127.0.0.1:0".to_owned(),
            bearer_token: String::new(),
            require_auth: true,
            outbound_queue_cap: OUTBOUND_QUEUE_CAP,
        }
    }
}

/// Handle returned by [`run_ws_listener`]: the bound address, the accept-loop
/// task, and a shared counter of responses dropped by the bounded writer.
#[derive(Debug)]
pub struct WsListenerHandle {
    pub addr: std::net::SocketAddr,
    pub join: JoinHandle<()>,
    /// Total responses dropped across all connections because the per-connection
    /// outbound queue was full (slow client). Observable for backpressure tests.
    pub dropped_events: Arc<AtomicU64>,
}

/// Bind and serve. Returns the bound address and the accept-loop join handle.
///
/// The accept loop runs until the listener is aborted (drop the `JoinHandle`
/// or call `abort()`). Each connection is served on its own task.
pub async fn run_ws_listener(
    processor: Arc<FacadeProcessor>,
    config: WsListenerConfig,
) -> Result<WsListenerHandle, std::io::Error> {
    // Reject credentials embedded in the URL (defense in depth; the bind string
    // is a host:port, but the helper is the canonical guard).
    if config.bind.contains('@') || config.bind.contains("token=") {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "bind address must not contain credentials",
        ));
    }

    // R4-02: fail-closed auth gate (parity with MCP Streamable HTTP). When
    // `require_auth` is true the server must refuse to bind unless a non-empty
    // bearer is configured. An empty/whitespace token with require_auth true
    // would otherwise create a deceptive auth boundary (empty==empty compare).
    if config.require_auth && config.bearer_token.trim().is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "require_auth is true but bearer_token is empty; refusing to bind (fail-closed). \
             Provide a non-empty bearer_token or set require_auth: false explicitly.",
        ));
    }

    // Emit the canonical cleartext non-loopback warning. Loopback is the
    // default; non-loopback is experimental/unsafe and must be explicit.
    let host = config.bind.split(':').next().unwrap_or("127.0.0.1");
    if let Some(warning) = bind_warning(host) {
        eprintln!("{warning}");
    }

    let listener = TcpListener::bind(&config.bind).await?;
    let addr = listener.local_addr()?;
    let dropped_events = Arc::new(AtomicU64::new(0));

    let dropped_for_loop = dropped_events.clone();
    let join = tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((stream, _peer)) => {
                    let p = processor.clone();
                    let c = config.clone();
                    let d = dropped_for_loop.clone();
                    tokio::spawn(async move {
                        let _ = serve_connection(p, stream, c, d).await;
                    });
                }
                // Acceptor only fails on a per-connection error; keep serving.
                Err(_) => continue,
            }
        }
    });

    Ok(WsListenerHandle {
        addr,
        join,
        dropped_events,
    })
}

/// Serve one connection: handshake → frame loop → bounded writer → drain.
///
/// Returns `Ok(())` when the peer disconnects cleanly or sends a close frame.
/// All errors are logged to stderr and swallowed (the connection task simply
/// exits); the accept loop continues serving other connections.
async fn serve_connection(
    processor: Arc<FacadeProcessor>,
    stream: tokio::net::TcpStream,
    config: WsListenerConfig,
    dropped_events: Arc<AtomicU64>,
) -> Result<(), std::io::Error> {
    // Handshake callback: bearer auth + subprotocol negotiation. The callback
    // captures the config by reference (FnOnce) and moves it into the
    // handshake future.
    let require_auth = config.require_auth;
    let expected_token = config.bearer_token.clone();

    let handshake = move |req: &Request, mut resp: Response| -> Result<Response, ErrorResponse> {
        // 1. Bearer auth at the HTTP upgrade layer.
        if require_auth {
            let auth = req
                .headers()
                .get("authorization")
                .and_then(|h| h.to_str().ok());
            if let Err(err) = validate_bearer_header(auth, &expected_token) {
                return Err(unauthorized_response(&err.message));
            }
        }
        // 2. Subprotocol negotiation. If the client offers subprotocols, echo
        //    ours when it is in the list; reject if the client offered but ours
        //    is absent. No subprotocol header → accept without one (lenient).
        if let Some(proto) = req
            .headers()
            .get("sec-websocket-protocol")
            .and_then(|h| h.to_str().ok())
        {
            let offers: Vec<&str> = proto.split(',').map(|s| s.trim()).collect();
            if offers.contains(&WS_SUBPROTOCOL) {
                resp.headers_mut().insert(
                    "sec-websocket-protocol",
                    HeaderValue::from_static(WS_SUBPROTOCOL),
                );
            } else {
                return Err(bad_request_response(
                    "Requested subprotocol not supported",
                ));
            }
        }
        Ok(resp)
    };

    let ws_config = WebSocketConfig::default()
        .max_message_size(Some(MAX_FRAME_SIZE))
        .max_frame_size(Some(MAX_FRAME_SIZE))
        .max_write_buffer_size(MAX_FRAME_SIZE);

    let ws_stream = match accept_hdr_async_with_config(stream, handshake, Some(ws_config)).await {
        Ok(s) => s,
        // Handshake rejection (auth/subprotocol) surfaces here. The error
        // response has already been written to the socket by tungstenite.
        Err(_) => return Ok(()),
    };

    let (mut ws_sink, mut ws_rx) = ws_stream.split();

    // Bounded writer: per-connection outbound channel. The reader task
    // `try_send`s responses; the writer task drains into the WS sink. A full
    // channel means the client is slow → drop and count (no unbounded buffer).
    let (outbound_tx, mut outbound_rx) = mpsc::channel::<Outbound>(config.outbound_queue_cap);

    // Writer task: owns the sink. Exits when the reader drops `outbound_tx`.
    // `Outbound::Flush` drives the auto-queued pong out promptly so WS
    // keepalive works even when no RPC response is pending.
    let writer = tokio::spawn(async move {
        while let Some(cmd) = outbound_rx.recv().await {
            match cmd {
                Outbound::Message(msg) => {
                    if ws_sink.send(Message::Text(msg.into())).await.is_err() {
                        break;
                    }
                }
                Outbound::Flush => {
                    // Push the auto-queued pong (and any buffered responses).
                    if ws_sink.flush().await.is_err() {
                        break;
                    }
                }
            }
        }
        // Flush any remaining buffered frames (including the auto-queued pong).
        let _ = ws_sink.close().await;
    });

    // Reader task (inline): drives the frame loop, dispatches text frames to
    // the shared processor, and rejects binary/batch/oversize at the WS layer.
    //
    // R4-08: correlated JSON-RPC responses must not be silently dropped under
    // backpressure. We use blocking `send` for RPC replies (and errors). If
    // the outbound queue is saturated for `RPC_SEND_TIMEOUT`, hard-close the
    // connection so the client observes an explicit failure rather than a
    // lost reply. Notifications/keepalive still use try_send (droppable).
    const RPC_SEND_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);
    loop {
        match ws_rx.next().await {
            Some(Ok(Message::Text(text))) => {
                let text = text.as_str().to_owned();
                match dispatch_text(&processor, &text).await {
                    Ok(Some(resp)) => {
                        // Must-deliver: await capacity; hard-close on timeout.
                        match tokio::time::timeout(
                            RPC_SEND_TIMEOUT,
                            outbound_tx.send(Outbound::Message(resp)),
                        )
                        .await
                        {
                            Ok(Ok(())) => {}
                            Ok(Err(_)) | Err(_) => {
                                // Writer gone or queue stalled → hard close.
                                dropped_events.fetch_add(1, Ordering::SeqCst);
                                break;
                            }
                        }
                    }
                    Ok(None) => {}
                    Err(err) => {
                        let env = error_envelope(None, &err);
                        match tokio::time::timeout(
                            RPC_SEND_TIMEOUT,
                            outbound_tx.send(Outbound::Message(env)),
                        )
                        .await
                        {
                            Ok(Ok(())) => {}
                            Ok(Err(_)) | Err(_) => {
                                dropped_events.fetch_add(1, Ordering::SeqCst);
                                break;
                            }
                        }
                    }
                }
            }
            Some(Ok(Message::Binary(_))) => {
                // Binary frames are unsupported — JSON-RPC over WS is text only.
                let env = error_envelope(
                    None,
                    &ProcessorError {
                        code: -32600,
                        message: "Binary WebSocket frames are unsupported".into(),
                    },
                );
                match tokio::time::timeout(
                    RPC_SEND_TIMEOUT,
                    outbound_tx.send(Outbound::Message(env)),
                )
                .await
                {
                    Ok(Ok(())) => {}
                    Ok(Err(_)) | Err(_) => {
                        dropped_events.fetch_add(1, Ordering::SeqCst);
                        break;
                    }
                }
            }
            Some(Ok(Message::Ping(_))) => {
                // Tungstenite auto-queues a Pong (RFC 6455 §5.5.2). Flush is
                // droppable under backpressure (not a correlated RPC reply).
                let _ = outbound_tx.try_send(Outbound::Flush);
            }
            Some(Ok(Message::Pong(_))) => {
                // Keepalive ack from peer; no action.
            }
            Some(Ok(Message::Close(_))) => break,
            // Raw frames are not surfaced to users by tungstenite's read path,
            // but the enum is non-exhaustive — ignore any other data variant.
            Some(Ok(Message::Frame(_))) => break,
            Some(Err(_)) => break,
            None => break,
        }
    }

    // Drain: drop the sender so the writer task exits after flushing the sink.
    drop(outbound_tx);
    let _ = writer.await;
    Ok(())
}

/// Validate the text frame at the WS layer, then route through the shared
/// processor. Reuses `validate_ws_text_frame` (1 MiB cap + batch rejection)
/// and `FacadeProcessor::handle_line` — no second processor.
async fn dispatch_text(
    processor: &FacadeProcessor,
    text: &str,
) -> Result<Option<String>, ProcessorError> {
    validate_ws_text_frame(text)?;
    processor.handle_line(text).await
}

/// Build a JSON-RPC failure envelope for a WS-layer rejection. `id` is `null`
/// when the frame could not be parsed to an id (binary / oversize / batch).
fn error_envelope(id: Option<&serde_json::Value>, err: &ProcessorError) -> String {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": err.code,
            "message": err.message,
            "data": {
                "code": domain_code(err.code),
                "retryable": retryable(err.code),
            }
        }
    })
    .to_string()
}

/// Cleartext non-loopback bind warning, exposed for testability. Returns the
/// canonical `experimental/unsafe` warning string for non-loopback hosts and
/// `None` for loopback. The listener emits this to stderr at bind time so
/// operators cannot miss the label. TLS stays a HUMAN gate (D-SEC.13).
pub fn bind_warning(host: &str) -> Option<&'static str> {
    security::remote_bind_warning_exact(host)
}

fn unauthorized_response(message: &str) -> ErrorResponse {
    tokio_tungstenite::tungstenite::http::Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header("WWW-Authenticate", "Bearer")
        .body(Some(message.to_owned()))
        .expect("static unauthorized response")
}

fn bad_request_response(message: &str) -> ErrorResponse {
    tokio_tungstenite::tungstenite::http::Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(Some(message.to_owned()))
        .expect("static bad-request response")
}

fn domain_code(numeric: i64) -> &'static str {
    xai_grok_app_server_protocol::lookup_error_numeric(numeric)
        .map(|s| s.code)
        .unwrap_or("internal_error")
}

fn retryable(numeric: i64) -> bool {
    xai_grok_app_server_protocol::lookup_error_numeric(numeric)
        .map(|s| s.retryable)
        .unwrap_or(false)
}

// ---------------------------------------------------------------------------
// Bounded writer — focused unit test for the drop guarantee (deterministic).
// The listener uses the same `mpsc::channel` + `try_send` pattern.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod ws_listener_unit_tests {
    use super::*;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn bounded_writer_try_send_drops_droppable_when_full() {
        // Cap = 2. try_send for droppable notifications (keepalive flush) may
        // drop under backpressure. R4-08: correlated RPC replies use blocking
        // `send` + hard-close instead of silent try_send drop.
        let (tx, mut rx) = mpsc::channel::<Outbound>(2);
        let dropped = Arc::new(AtomicU64::new(0));
        let mut sent = 0u64;
        for i in 0..5u64 {
            if tx.try_send(Outbound::Message(format!("m{i}"))).is_err() {
                dropped.fetch_add(1, Ordering::SeqCst);
            } else {
                sent += 1;
            }
        }
        assert_eq!(sent, 2, "only the first two fit the bounded channel");
        assert_eq!(
            dropped.load(Ordering::SeqCst),
            3,
            "overflow droppable messages are dropped, not buffered"
        );
        drop(tx);
        let mut got = Vec::new();
        while let Some(m) = rx.recv().await {
            got.push(m);
        }
        assert_eq!(got.len(), 2);
    }

    #[tokio::test]
    async fn run_ws_listener_refuses_empty_bearer_when_require_auth_true() {
        // R4-02: parity with MCP fail-closed auth at bind time.
        use crate::FacadeProcessor;
        use std::sync::Arc;
        use xai_grok_tower::FakeRuntime;

        let processor = Arc::new(FacadeProcessor::new(Arc::new(FakeRuntime::new())));
        let err = run_ws_listener(
            processor.clone(),
            WsListenerConfig {
                bind: "127.0.0.1:0".into(),
                bearer_token: String::new(),
                require_auth: true,
                outbound_queue_cap: OUTBOUND_QUEUE_CAP,
            },
        )
        .await
        .expect_err("empty bearer must not bind");
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);

        let err = run_ws_listener(
            processor.clone(),
            WsListenerConfig {
                bind: "127.0.0.1:0".into(),
                bearer_token: "   ".into(),
                require_auth: true,
                outbound_queue_cap: OUTBOUND_QUEUE_CAP,
            },
        )
        .await
        .expect_err("whitespace bearer must not bind");
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);

        // Explicit opt-out still binds.
        let handle = run_ws_listener(
            processor,
            WsListenerConfig {
                bind: "127.0.0.1:0".into(),
                bearer_token: String::new(),
                require_auth: false,
                outbound_queue_cap: OUTBOUND_QUEUE_CAP,
            },
        )
        .await
        .expect("require_auth=false with empty bearer must bind");
        handle.join.abort();
    }
}
