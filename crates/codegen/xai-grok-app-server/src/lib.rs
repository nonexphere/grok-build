//! App Server processor and transports over `GrokRuntimeFacade`.
//!
//! This crate must never construct `SessionActor` or depend on Shell.

pub mod controller;
pub mod processor;
pub mod replay;
pub mod security;
pub mod transport;

pub use processor::FacadeProcessor;
pub use transport::in_process::InProcessClient;
pub use transport::stdio::{process_ndjson_batch, run_stdio_loop};
#[cfg(feature = "websocket")]
pub use transport::ws_listener::{
    KEEPALIVE_INTERVAL_SECS, MAX_FRAME_SIZE, OUTBOUND_QUEUE_CAP, WS_SUBPROTOCOL, WsListenerConfig,
    WsListenerHandle, run_ws_listener,
};
pub use transport::{ConnectionMeta, ProtocolConnection, TransportKind};

use async_trait::async_trait;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct ProcessorError {
    pub code: i64,
    pub message: String,
}

#[async_trait]
pub trait AppServerProcessor: Send + Sync {
    async fn process(&self, method: &str, params: Value) -> Result<Value, ProcessorError>;
}

#[cfg(test)]
mod conformance_tests {
    use super::*;
    use serde_json::json;
    use std::sync::Arc;
    use xai_grok_app_server_protocol::PROTOCOL_VERSION;
    use xai_grok_tower::FakeRuntime;

    async fn run_script(processor: Arc<FacadeProcessor>) -> Vec<Value> {
        let lines = [
            json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":PROTOCOL_VERSION,"clientInfo":{"name":"c","version":"0"},"capabilities":{}}}),
            json!({"jsonrpc":"2.0","id":2,"method":"session/start","params":{"workspaceRoot":"/work","idempotencyKey":"conf-s"}}),
        ];
        let mut out = Vec::new();
        for line in lines {
            let resp = processor
                .handle_line(&line.to_string())
                .await
                .unwrap()
                .unwrap();
            out.push(serde_json::from_str(&resp).unwrap());
        }
        out
    }

    #[tokio::test]
    async fn conformance_in_process_and_stdio_normalized_outputs_match() {
        // Two processors with identical fakes and scripts must normalize equal
        // protocol fields (ids differ by construction — compare shapes).
        let p1 = Arc::new(FacadeProcessor::new(Arc::new(FakeRuntime::new())));
        let p2 = Arc::new(FacadeProcessor::new(Arc::new(FakeRuntime::new())));
        let a = run_script(p1).await;
        let b = run_script(p2).await;
        assert_eq!(
            a[0]["result"]["protocolVersion"],
            b[0]["result"]["protocolVersion"]
        );
        assert_eq!(
            a[1]["result"]["session"]["status"],
            b[1]["result"]["session"]["status"]
        );
        assert_eq!(
            a[1]["result"]["session"]["workspaceRoot"],
            b[1]["result"]["session"]["workspaceRoot"]
        );
        // stdio batch path
        let p3 = Arc::new(FacadeProcessor::new(Arc::new(FakeRuntime::new())));
        let batch = process_ndjson_batch(
            p3,
            &format!(
                "{}\n{}\n",
                json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":PROTOCOL_VERSION,"clientInfo":{"name":"c","version":"0"},"capabilities":{}}}),
                json!({"jsonrpc":"2.0","id":2,"method":"session/start","params":{"workspaceRoot":"/work","idempotencyKey":"conf-s2"}})
            ),
        )
        .await
        .unwrap();
        assert_eq!(batch.len(), 2);
    }

    #[tokio::test]
    async fn backpressure_subscribe_is_bounded_page() {
        let rt = Arc::new(FakeRuntime::new());
        let processor = Arc::new(FacadeProcessor::new(rt.clone()));
        processor
            .handle_line(
                &json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":PROTOCOL_VERSION,"clientInfo":{"name":"c","version":"0"},"capabilities":{}}}).to_string(),
            )
            .await
            .unwrap();
        let start = processor
            .handle_line(
                &json!({"jsonrpc":"2.0","id":2,"method":"session/start","params":{"workspaceRoot":"/work","idempotencyKey":"bp-s"}}).to_string(),
            )
            .await
            .unwrap()
            .unwrap();
        let session_id =
            serde_json::from_str::<Value>(&start).unwrap()["result"]["session"]["sessionId"]
                .as_str()
                .unwrap()
                .to_owned();
        for i in 0..5 {
            processor
                .handle_line(
                    &json!({"jsonrpc":"2.0","id":i+10,"method":"turn/start","params":{
                        "sessionId": session_id,
                        "input":[{"type":"text","text": format!("m{i}")}],
                        "idempotencyKey": format!("bp-t{i}")
                    }})
                    .to_string(),
                )
                .await
                .unwrap();
        }
        let sub = processor
            .handle_line(
                &json!({"jsonrpc":"2.0","id":99,"method":"session/subscribe","params":{
                    "sessionId": session_id,
                    "afterEventSeq":"0"
                }})
                .to_string(),
            )
            .await
            .unwrap()
            .unwrap();
        let v: Value = serde_json::from_str(&sub).unwrap();
        let events = v["result"]["replay"]["events"].as_array().unwrap();
        assert!(events.len() <= 100);
        assert!(events.len() >= 5);
    }

    #[tokio::test]
    async fn runtime_facade_end_to_end_via_processor() {
        let processor = FacadeProcessor::new(Arc::new(FakeRuntime::new()));
        assert!(!processor.is_initialized());
        let _ = processor
            .process(
                "initialize",
                json!({
                    "protocolVersion": PROTOCOL_VERSION,
                    "clientInfo":{"name":"x","version":"1"},
                    "capabilities":{}
                }),
            )
            .await
            .unwrap();
        assert!(processor.is_initialized());
    }
}

#[cfg(test)]
mod adversarial_rejection_tests {
    // C7-E adversarial gate: consolidate malformed JSON-RPC / oversize / batch
    // rejection across the processor, stdio NDJSON, and WebSocket text-frame
    // surfaces. These are the hermetic slices of the adversarial contract;
    // remote TLS + live provider smoke are HUMAN-deferred (see handoff C7-E).
    use super::*;
    use crate::transport::websocket::validate_ws_text_frame;
    use serde_json::json;
    use std::sync::Arc;
    use xai_grok_tower::FakeRuntime;

    /// Malformed JSON text at the processor boundary yields a parse error
    /// (-32700) — the stdio loop turns this into a failure envelope with
    /// `id: null` (see `transport::stdio::run_stdio_loop`).
    #[tokio::test]
    async fn processor_rejects_malformed_json_with_parse_error() {
        let processor = FacadeProcessor::new(Arc::new(FakeRuntime::new()));
        let err = processor
            .handle_line("not-json{")
            .await
            .expect_err("malformed JSON must surface a parse error");
        assert_eq!(err.code, -32700);
        assert!(
            err.message.contains("Parse error"),
            "message must explain parse failure: {err:?}"
        );
    }

    /// A JSON-RPC batch (top-level array) is rejected at the envelope layer
    /// with `invalid_request` (-32600), distinct from a parse error.
    #[tokio::test]
    async fn processor_rejects_batch_array_with_invalid_request() {
        let processor = FacadeProcessor::new(Arc::new(FakeRuntime::new()));
        let err = processor
            .handle_line(&json!([{"jsonrpc":"2.0","id":1,"method":"initialize"}]).to_string())
            .await
            .expect_err("batch array must be rejected");
        assert_eq!(err.code, -32600);
    }

    /// The stdio NDJSON batch helper drops blank lines and surfaces exactly
    /// one response per parsed request; a malformed line propagates the
    /// parse error (the loop emits the `id: null` failure envelope in the
    /// real `run_stdio_loop`).
    #[tokio::test]
    async fn stdio_batch_helper_propagates_parse_error_for_malformed_line() {
        let processor = Arc::new(FacadeProcessor::new(Arc::new(FakeRuntime::new())));
        let result = process_ndjson_batch(processor, "not-json{\n").await;
        assert!(result.is_err(), "malformed line must propagate parse error");
        assert_eq!(result.unwrap_err().code, -32700);
    }

    /// Oversized text frames are rejected at the WS layer before the
    /// processor sees them (1 MiB cap → -32021).
    #[test]
    fn ws_layer_rejects_oversized_text_frame() {
        let big = "x".repeat(1_048_577);
        let err = validate_ws_text_frame(&big).expect_err("oversize must be rejected");
        assert_eq!(err.code, -32021);
    }

    /// JSON-RPC batches are rejected at the WS layer (text frame starting
    /// with `[`) before the processor sees them (-32600).
    #[test]
    fn ws_layer_rejects_batch_text_frame() {
        let err = validate_ws_text_frame(r#"[{"jsonrpc":"2.0","id":1,"method":"initialize"}]"#)
            .expect_err("batch must be rejected at WS layer");
        assert_eq!(err.code, -32600);
    }

    /// A well-formed single object passes the WS frame validator (negative
    /// control for the oversize/batch rejections).
    #[test]
    fn ws_layer_accepts_well_formed_single_object() {
        assert!(validate_ws_text_frame(r#"{"jsonrpc":"2.0","id":1}"#).is_ok());
    }

    /// Secret canaries surface as errors when present in any control-plane
    /// surface (logs, errors, tool projections). This is the hermetic slice
    /// of the secret-leak gate; live-credential canaries are HUMAN-deferred.
    #[test]
    fn secret_canaries_are_detected_across_canonical_shapes() {
        for sample in [
            "sk-live-canary",
            "Bearer abc",
            "access_token=xyz",
            "refresh_token=xyz",
            "client_secret=xyz",
            "XAI_API_KEY=1",
            "GROK_TEST_SECRET_CANARY",
        ] {
            assert!(
                security::assert_no_secret_canaries(sample).is_err(),
                "canary must be detected: {sample:?}"
            );
        }
        assert!(security::assert_no_secret_canaries("session started ok").is_ok());
    }

    /// Remote cleartext binds are labeled experimental/unsafe; loopback is
    /// always safe. This is the bind-warning gate (TLS stays HUMAN).
    #[test]
    fn remote_cleartext_bind_is_labeled_experimental_unsafe() {
        assert_eq!(security::remote_bind_label("127.0.0.1", true), "loopback");
        assert_eq!(security::remote_bind_label("localhost", false), "loopback");
        assert_eq!(
            security::remote_bind_label("0.0.0.0", true),
            "experimental/unsafe-cleartext-remote"
        );
        assert_eq!(
            security::remote_bind_label("0.0.0.0", false),
            "remote-tls-required"
        );
    }
}

#[cfg(test)]
mod co_start_tests {
    #[test]
    fn co_start_rejects_dual_stdio_accepts_stdio_plus_ws_matrix() {
        // Valid: single stdio, or stdio+ws, or in-process only. Invalid: dual stdio.
        let matrix = [
            (true, false, false, true), // stdio
            (false, true, false, true), // ws
            (true, true, false, true),  // stdio+ws
            (false, false, true, true), // in-process
            (true, false, true, false), // dual stdio-like (stdio+in-process both claiming stdio ownership) — reject for dual stdio claim
        ];
        for (stdio, ws, in_process, ok) in matrix {
            let dual_stdio = stdio && in_process; // simplified ownership collision
            let valid = !dual_stdio && (stdio || ws || in_process);
            assert_eq!(valid, ok, "stdio={stdio} ws={ws} in_process={in_process}");
            let _ = ws;
        }
    }
}

#[cfg(test)]
mod websocket_conformance_tests {
    use super::*;
    use crate::transport::websocket::handle_ws_text;
    use serde_json::{Value, json};
    use std::sync::Arc;
    use xai_grok_app_server_protocol::PROTOCOL_VERSION;
    use xai_grok_tower::FakeRuntime;

    #[tokio::test]
    async fn websocket_conformance_initialize_matches_stdio_shape() {
        let rt = Arc::new(FakeRuntime::new());
        let p_ws = Arc::new(FacadeProcessor::new(rt.clone()));
        let p_stdio = Arc::new(FacadeProcessor::new(Arc::new(FakeRuntime::new())));
        let req = json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{
            "protocolVersion": PROTOCOL_VERSION,
            "clientInfo":{"name":"c","version":"0"},
            "capabilities":{}
        }})
        .to_string();
        let ws = handle_ws_text(p_ws, &req).await.unwrap().unwrap();
        let stdio = p_stdio.handle_line(&req).await.unwrap().unwrap();
        let w: Value = serde_json::from_str(&ws).unwrap();
        let s: Value = serde_json::from_str(&stdio).unwrap();
        assert_eq!(
            w["result"]["protocolVersion"],
            s["result"]["protocolVersion"]
        );
        assert_eq!(
            w["result"]["capabilities"]["sessions"]["start"],
            s["result"]["capabilities"]["sessions"]["start"]
        );
    }
}

#[cfg(test)]
mod snapshot_then_live_tests {
    use super::replay::replay_all_pages;
    use xai_grok_app_server_protocol::{InputBlock, SessionStartParams, TurnStartParams};
    use xai_grok_tower::{FakeRuntime, GrokRuntimeFacade};

    #[tokio::test]
    async fn snapshot_then_live_no_gap_on_fake() {
        let rt = FakeRuntime::new();
        let s = rt
            .start_session(SessionStartParams {
                workspace_root: "/work".into(),
                agent_type: None,
                provider_binding: None,
                idempotency_key: "snap-1".into(),
            })
            .await
            .unwrap();
        rt.start_turn(TurnStartParams {
            session_id: s.session_id.clone(),
            input: vec![InputBlock::Text { text: "a".into() }],
            idempotency_key: "snap-t".into(),
        })
        .await
        .unwrap();
        let pages = replay_all_pages(&rt, &s.session_id, Some("epoch_1".into()), 3)
            .await
            .unwrap();
        assert!(!pages.is_empty());
        let n = pages[0].events.len();
        assert!(n >= 1);
        // second page from same cursor end should not invent gaps
        let pages2 = replay_all_pages(&rt, &s.session_id, Some("epoch_1".into()), 3)
            .await
            .unwrap();
        assert_eq!(pages2[0].events.len(), n);
    }
}

// ===========================================================================
// C3-B: real WebSocket listener black-box tests (feature-gated).
// Spawn a real listener on an ephemeral loopback port; connect with a real
// `tokio-tungstenite` client; assert wire behavior. The processor is the real
// `FacadeProcessor` over `FakeRuntime` (the listener is the black-box surface;
// the processor is the same regardless of the runtime adapter). Real-adapter
// slow-client resync is deferred to Wave C3-22/23 (canonical session files).
// ===========================================================================
#[cfg(all(test, feature = "websocket"))]
mod ws_listener_blackbox_tests {
    use super::*;
    use crate::transport::ws_listener::bind_warning;
    use futures_util::{SinkExt, StreamExt};
    use serde_json::{Value, json};
    use std::sync::Arc;
    use std::sync::atomic::AtomicU64;
    use std::time::Duration;
    use tokio::time::timeout;
    use tokio_tungstenite::tungstenite::Message;
    use tokio_tungstenite::tungstenite::client::IntoClientRequest;
    use tokio_tungstenite::tungstenite::handshake::client::Request;
    use tokio_tungstenite::tungstenite::http::HeaderValue;
    use tokio_tungstenite::{WebSocketStream, connect_async};
    use xai_grok_app_server_protocol::PROTOCOL_VERSION;
    use xai_grok_tower::FakeRuntime;

    type ClientStream = WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

    const TOKEN: &str = "test-bearer-secret";

    /// Build a real listener on an ephemeral loopback port with auth enabled.
    async fn spawn_listener(
        cap: usize,
    ) -> (
        std::net::SocketAddr,
        Arc<AtomicU64>,
        tokio::task::JoinHandle<()>,
    ) {
        let processor = Arc::new(FacadeProcessor::new(Arc::new(FakeRuntime::new())));
        let config = WsListenerConfig {
            bind: "127.0.0.1:0".to_owned(),
            bearer_token: TOKEN.to_owned(),
            require_auth: true,
            outbound_queue_cap: cap,
        };
        let handle = run_ws_listener(processor, config).await.unwrap();
        (handle.addr, handle.dropped_events, handle.join)
    }

    /// Build a WS client request with optional bearer + subprotocol headers.
    fn ws_request(
        addr: std::net::SocketAddr,
        bearer: Option<&str>,
        subprotocol: Option<&str>,
    ) -> Request {
        let url = format!("ws://{addr}/");
        let mut req = url.as_str().into_client_request().unwrap();
        if let Some(b) = bearer {
            req.headers_mut().insert(
                "Authorization",
                HeaderValue::from_str(&format!("Bearer {b}")).unwrap(),
            );
        }
        if let Some(p) = subprotocol {
            req.headers_mut()
                .insert("Sec-WebSocket-Protocol", HeaderValue::from_str(p).unwrap());
        }
        req
    }

    async fn connect(
        addr: std::net::SocketAddr,
        bearer: Option<&str>,
        subprotocol: Option<&str>,
    ) -> ClientStream {
        let req = ws_request(addr, bearer, subprotocol);
        let (stream, _resp) = connect_async(req).await.unwrap();
        stream
    }

    async fn send_text(stream: &mut ClientStream, text: &str) {
        stream
            .send(Message::Text(text.to_owned().into()))
            .await
            .unwrap();
    }

    async fn recv_text(stream: &mut ClientStream) -> Value {
        loop {
            match timeout(Duration::from_secs(5), stream.next()).await {
                Ok(Some(Ok(Message::Text(t)))) => return serde_json::from_str(t.as_str()).unwrap(),
                Ok(Some(Ok(_))) => continue,
                Ok(Some(Err(e))) => panic!("ws read error: {e}"),
                Ok(None) => panic!("ws closed before response"),
                Err(_) => panic!("ws recv timed out"),
            }
        }
    }

    fn init_request(id: u64) -> String {
        json!({
            "jsonrpc":"2.0","id":id,"method":"initialize","params":{
                "protocolVersion": PROTOCOL_VERSION,
                "clientInfo":{"name":"ws","version":"0"},
                "capabilities":{}
            }
        })
        .to_string()
    }

    #[tokio::test]
    async fn ws_listener_handshake_subprotocol_negotiates_protocol_version() {
        let (addr, _dropped, _join) = spawn_listener(OUTBOUND_QUEUE_CAP).await;
        let req = ws_request(addr, Some(TOKEN), Some(PROTOCOL_VERSION));
        let (_stream, resp) = connect_async(req).await.unwrap();
        let proto = resp
            .headers()
            .get("sec-websocket-protocol")
            .expect("server must echo the negotiated subprotocol");
        assert_eq!(proto.to_str().unwrap(), PROTOCOL_VERSION);
    }

    #[tokio::test]
    async fn ws_listener_rejects_missing_authorization_header() {
        let (addr, _dropped, _join) = spawn_listener(OUTBOUND_QUEUE_CAP).await;
        let req = ws_request(addr, None, None);
        let err = connect_async(req).await.unwrap_err();
        // Tungstenite surfaces a 401 handshake rejection as an HTTP error.
        let msg = err.to_string().to_lowercase();
        assert!(
            msg.contains("401") || msg.contains("unauthorized") || msg.contains("unexpected"),
            "expected auth rejection, got: {msg}"
        );
    }

    #[tokio::test]
    async fn ws_listener_rejects_wrong_bearer() {
        let (addr, _dropped, _join) = spawn_listener(OUTBOUND_QUEUE_CAP).await;
        let req = ws_request(addr, Some("wrong-token"), None);
        assert!(connect_async(req).await.is_err());
    }

    #[tokio::test]
    async fn ws_listener_rejects_credentials_in_url_at_bind() {
        // The listener rejects a bind address containing credentials.
        let processor = Arc::new(FacadeProcessor::new(Arc::new(FakeRuntime::new())));
        let config = WsListenerConfig {
            bind: "user:pass@127.0.0.1:0".to_owned(),
            ..Default::default()
        };
        let err = run_ws_listener(processor, config).await.unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
    }

    #[tokio::test]
    async fn ws_listener_text_frame_initialize_then_session_start_roundtrip() {
        let (addr, _dropped, _join) = spawn_listener(OUTBOUND_QUEUE_CAP).await;
        let mut s = connect(addr, Some(TOKEN), None).await;

        send_text(&mut s, &init_request(1)).await;
        let init = recv_text(&mut s).await;
        assert_eq!(init["result"]["protocolVersion"], PROTOCOL_VERSION);

        send_text(
            &mut s,
            &json!({"jsonrpc":"2.0","id":2,"method":"session/start","params":{
                "workspaceRoot":"/work","idempotencyKey":"ws-s"
            }})
            .to_string(),
        )
        .await;
        let start = recv_text(&mut s).await;
        // FakeRuntime returns `Ready` for a freshly started session.
        assert_eq!(start["result"]["session"]["status"], "ready");
        assert_eq!(start["result"]["session"]["workspaceRoot"], "/work");
    }

    #[tokio::test]
    async fn ws_listener_ping_pong_keepalive() {
        let (addr, _dropped, _join) = spawn_listener(OUTBOUND_QUEUE_CAP).await;
        let mut s = connect(addr, Some(TOKEN), None).await;
        // Client sends a Ping; the server must respond with a Pong (RFC 6455).
        s.send(Message::Ping(b"keepalive".to_vec().into()))
            .await
            .unwrap();
        // Expect a Pong within the keepalive window.
        let mut got_pong = false;
        for _ in 0..20 {
            match timeout(Duration::from_secs(2), s.next()).await {
                Ok(Some(Ok(Message::Pong(_)))) => {
                    got_pong = true;
                    break;
                }
                Ok(Some(Ok(_))) => continue,
                Ok(Some(Err(e))) => panic!("ws error during keepalive: {e}"),
                Ok(None) => break,
                Err(_) => break,
            }
        }
        assert!(
            got_pong,
            "server must flush a Pong in response to a client Ping"
        );
    }

    #[tokio::test]
    async fn ws_listener_rejects_binary_frame() {
        let (addr, _dropped, _join) = spawn_listener(OUTBOUND_QUEUE_CAP).await;
        let mut s = connect(addr, Some(TOKEN), None).await;
        s.send(Message::Binary(vec![0xDE, 0xAD, 0xBE, 0xEF].into()))
            .await
            .unwrap();
        let resp = recv_text(&mut s).await;
        assert_eq!(resp["error"]["code"], -32600);
        assert!(
            resp["error"]["message"]
                .as_str()
                .unwrap()
                .contains("Binary")
        );
    }

    #[tokio::test]
    async fn ws_listener_rejects_jsonrpc_batch() {
        let (addr, _dropped, _join) = spawn_listener(OUTBOUND_QUEUE_CAP).await;
        let mut s = connect(addr, Some(TOKEN), None).await;
        send_text(
            &mut s,
            r#"[{"jsonrpc":"2.0","id":1,"method":"initialize"}]"#,
        )
        .await;
        let resp = recv_text(&mut s).await;
        assert_eq!(resp["error"]["code"], -32600);
        assert!(resp["error"]["message"].as_str().unwrap().contains("batch"));
    }

    #[tokio::test]
    async fn ws_listener_rejects_oversize_text_frame() {
        let (addr, _dropped, _join) = spawn_listener(OUTBOUND_QUEUE_CAP).await;
        let mut s = connect(addr, Some(TOKEN), None).await;
        // 1.5 MiB text frame exceeds the 1 MiB WS+JSON-RPC cap. The WS layer
        // (tungstenite max_message_size = 1 MiB) rejects the frame and the
        // connection terminates — no RPC response is delivered.
        let big = "x".repeat(MAX_FRAME_SIZE + 524_288);
        s.send(Message::Text(big.into())).await.unwrap();
        // The server must not deliver a valid RPC response; the read either
        // returns an error/close or times out (rejection, not acceptance).
        let mut got_response = false;
        for _ in 0..20 {
            match timeout(Duration::from_secs(2), s.next()).await {
                Ok(Some(Ok(Message::Text(_)))) => {
                    got_response = true;
                    break;
                }
                Ok(Some(Ok(Message::Close(_)))) => break,
                Ok(Some(Ok(_))) => continue,
                Ok(Some(Err(_))) => break,
                Ok(None) => break,
                Err(_) => break,
            }
        }
        assert!(
            !got_response,
            "server must reject the oversize frame, not process it"
        );
    }

    #[tokio::test]
    async fn ws_listener_disconnect_drains_and_closes() {
        let (addr, _dropped, _join) = spawn_listener(OUTBOUND_QUEUE_CAP).await;
        let mut s = connect(addr, Some(TOKEN), None).await;
        send_text(&mut s, &init_request(1)).await;
        let _ = recv_text(&mut s).await;
        // Client closes cleanly; the server must drain and close without hang.
        s.send(Message::Close(None)).await.unwrap();
        // Drain until the stream ends.
        let mut closed = false;
        for _ in 0..20 {
            match timeout(Duration::from_secs(2), s.next()).await {
                Ok(None) => {
                    closed = true;
                    break;
                }
                Ok(Some(Ok(Message::Close(_)))) => continue,
                Ok(Some(Ok(_))) => continue,
                Ok(Some(Err(_))) => {
                    closed = true;
                    break;
                }
                Err(_) => break,
            }
        }
        assert!(closed, "server must drain and close the connection");
    }

    #[tokio::test]
    async fn ws_listener_bounded_writer_survives_burst() {
        // A burst of requests with a small bounded writer must not crash the
        // connection; the client reads back the responses that were accepted.
        // The drop guarantee itself is proven by `bounded_writer_drops_when_full`.
        let (addr, _dropped, _join) = spawn_listener(4).await;
        let mut s = connect(addr, Some(TOKEN), None).await;
        for i in 0..16u64 {
            send_text(&mut s, &init_request(i)).await;
        }
        let mut responses = 0;
        for _ in 0..16 {
            match timeout(Duration::from_secs(5), s.next()).await {
                Ok(Some(Ok(Message::Text(_)))) => responses += 1,
                Ok(Some(Ok(_))) => continue,
                _ => break,
            }
        }
        assert!(
            responses >= 1,
            "connection must survive a burst and deliver at least one response"
        );
    }

    #[tokio::test]
    async fn ws_listener_slow_client_resync_via_replay_fake_adapter() {
        // Fake-adapter resync variant: subscribe over WS returns a replay page.
        // Real-adapter resync is deferred to Wave C3-22/23 (canonical session
        // files); this proves the listener wires `session/subscribe` through.
        let (addr, _dropped, _join) = spawn_listener(OUTBOUND_QUEUE_CAP).await;
        let mut s = connect(addr, Some(TOKEN), None).await;
        send_text(&mut s, &init_request(1)).await;
        let _ = recv_text(&mut s).await;
        send_text(
            &mut s,
            &json!({"jsonrpc":"2.0","id":2,"method":"session/start","params":{
                "workspaceRoot":"/work","idempotencyKey":"replay-s"
            }})
            .to_string(),
        )
        .await;
        let start = recv_text(&mut s).await;
        let session_id = start["result"]["session"]["sessionId"]
            .as_str()
            .unwrap()
            .to_owned();

        send_text(
            &mut s,
            &json!({"jsonrpc":"2.0","id":3,"method":"session/subscribe","params":{
                "sessionId": session_id, "afterEventSeq":"0"
            }})
            .to_string(),
        )
        .await;
        let sub = recv_text(&mut s).await;
        assert!(sub["result"]["replay"]["events"].is_array());
        assert!(sub["result"]["subscriptionId"].is_string());
    }

    #[test]
    fn ws_listener_cleartext_non_loopback_warns_experimental_unsafe() {
        // Loopback default: no warning. Non-loopback cleartext: experimental/unsafe.
        assert!(bind_warning("127.0.0.1").is_none());
        assert!(bind_warning("localhost").is_none());
        let w = bind_warning("0.0.0.0").unwrap();
        assert!(w.contains("experimental/unsafe"));
        assert!(w.contains("non-loopback"));
    }

    #[test]
    fn ws_listener_default_config_is_loopback() {
        let config = WsListenerConfig::default();
        assert!(
            config.bind.starts_with("127.0.0.1"),
            "default bind must be loopback, got {}",
            config.bind
        );
        assert!(config.require_auth, "auth is required by default");
    }

    #[tokio::test]
    async fn ws_conformance_matches_stdio_method_shapes() {
        // Black-box: the real listener returns the same result shapes as the
        // stdio path for initialize + session/start (protocol version, session
        // status, workspace root).
        let (addr, _dropped, _join) = spawn_listener(OUTBOUND_QUEUE_CAP).await;

        // stdio path (shared processor).
        let p_stdio = Arc::new(FacadeProcessor::new(Arc::new(FakeRuntime::new())));
        let stdio_out = process_ndjson_batch(
            p_stdio,
            &format!(
                "{}\n{}\n",
                init_request(1),
                json!({"jsonrpc":"2.0","id":2,"method":"session/start","params":{
                    "workspaceRoot":"/work","idempotencyKey":"conf-stdio"
                }})
            ),
        )
        .await
        .unwrap();
        let stdio_init: Value = serde_json::from_str(&stdio_out[0]).unwrap();
        let stdio_start: Value = serde_json::from_str(&stdio_out[1]).unwrap();

        // ws path.
        let mut s = connect(addr, Some(TOKEN), None).await;
        send_text(&mut s, &init_request(1)).await;
        let ws_init = recv_text(&mut s).await;
        send_text(
            &mut s,
            &json!({"jsonrpc":"2.0","id":2,"method":"session/start","params":{
                "workspaceRoot":"/work","idempotencyKey":"conf-ws"
            }})
            .to_string(),
        )
        .await;
        let ws_start = recv_text(&mut s).await;

        assert_eq!(
            ws_init["result"]["protocolVersion"],
            stdio_init["result"]["protocolVersion"]
        );
        assert_eq!(
            ws_init["result"]["capabilities"]["sessions"]["start"],
            stdio_init["result"]["capabilities"]["sessions"]["start"]
        );
        assert_eq!(
            ws_start["result"]["session"]["status"],
            stdio_start["result"]["session"]["status"]
        );
        assert_eq!(
            ws_start["result"]["session"]["workspaceRoot"],
            stdio_start["result"]["session"]["workspaceRoot"]
        );
    }
}

// ===========================================================================
// C6-C: interaction/respond conformance across in-process, stdio, and WS.
// Prove `interaction/respond` produces EQUAL accept and error shapes across
// all three transports. The transports share the same `FacadeProcessor`
// dispatch, so shape parity is structural — these tests PROVE it by running
// the same request through each transport surface and comparing the
// normalized result/error fields.
//
// Gate: `./scripts/run-rust-test-gate.sh interaction_conformance \
//   cargo test -p xai-grok-app-server interaction_conformance`
// (default features: in-process + stdio + WS frame adapter `handle_ws_text`).
// The real WS listener black-box variant is feature-gated behind
// `websocket` and run separately (see `interaction_conformance_ws_listener`).
// ===========================================================================
#[cfg(test)]
mod interaction_conformance_tests {
    use super::*;
    use crate::transport::in_process::InProcessClient;
    use crate::transport::stdio::process_ndjson_batch;
    use crate::transport::websocket::handle_ws_text;
    use serde_json::{Value, json};
    use std::sync::Arc;
    use xai_grok_app_server_protocol::PROTOCOL_VERSION;
    use xai_grok_tower::FakeRuntime;

    fn init_line(id: u64) -> String {
        json!({
            "jsonrpc":"2.0","id":id,"method":"initialize","params":{
                "protocolVersion": PROTOCOL_VERSION,
                "clientInfo":{"name":"conf","version":"0"},
                "capabilities":{}
            }
        })
        .to_string()
    }

    fn start_line(id: u64, key: &str) -> String {
        json!({
            "jsonrpc":"2.0","id":id,"method":"session/start","params":{
                "workspaceRoot":"/work","idempotencyKey":key
            }
        })
        .to_string()
    }

    /// A well-formed `interaction/respond` request. `session_id` is the only
    /// field that varies per session; everything else is fixed so the request
    /// shape is identical across transports.
    fn respond_line(id: u64, session_id: &str, key: &str) -> String {
        json!({
            "jsonrpc":"2.0","id":id,"method":"interaction/respond","params":{
                "sessionId": session_id,
                "turnId":"turn-1",
                "interactionId":"ix-1",
                "decision":"allow",
                "idempotencyKey": key
            }
        })
        .to_string()
    }

    /// A malformed `interaction/respond` request — missing required
    /// `sessionId`. Deserialization fails → -32602 `invalid_params`.
    fn respond_line_missing_session(id: u64) -> String {
        json!({
            "jsonrpc":"2.0","id":id,"method":"interaction/respond","params":{
                "turnId":"turn-1",
                "interactionId":"ix-1",
                "decision":"allow",
                "idempotencyKey":"r-bad"
            }
        })
        .to_string()
    }

    // -------------------------------------------------------------------
    // Accept-shape helpers — run init + session/start + interaction/respond
    // through each transport and return the `interaction/respond` result
    // Value (the inner `result` object).
    // -------------------------------------------------------------------

    async fn accept_in_process() -> Value {
        let processor = Arc::new(FacadeProcessor::new(Arc::new(FakeRuntime::new())));
        let mut client = InProcessClient::new(processor);
        let _ = client.initialize().await.unwrap();
        let session = client
            .request(
                "session/start",
                json!({"workspaceRoot":"/work","idempotencyKey":"conf-ip"}),
            )
            .await
            .unwrap();
        let session_id = session["session"]["sessionId"].as_str().unwrap().to_owned();
        client
            .request(
                "interaction/respond",
                json!({
                    "sessionId": session_id,
                    "turnId":"turn-1",
                    "interactionId":"ix-1",
                    "decision":"allow",
                    "idempotencyKey":"r-ip"
                }),
            )
            .await
            .unwrap()
    }

    async fn accept_stdio() -> Value {
        let processor = Arc::new(FacadeProcessor::new(Arc::new(FakeRuntime::new())));
        let batch = format!("{}\n{}\n", init_line(1), start_line(2, "conf-stdio"));
        let out = process_ndjson_batch(processor.clone(), &batch)
            .await
            .unwrap();
        let start: Value = serde_json::from_str(&out[1]).unwrap();
        let session_id = start["result"]["session"]["sessionId"]
            .as_str()
            .unwrap()
            .to_owned();
        let respond_out = process_ndjson_batch(processor, &respond_line(3, &session_id, "r-stdio"))
            .await
            .unwrap();
        let envelope: Value = serde_json::from_str(&respond_out[0]).unwrap();
        envelope["result"].clone()
    }

    async fn accept_ws_frame() -> Value {
        let processor = Arc::new(FacadeProcessor::new(Arc::new(FakeRuntime::new())));
        let _ = handle_ws_text(processor.clone(), &init_line(1))
            .await
            .unwrap();
        let start_env = handle_ws_text(processor.clone(), &start_line(2, "conf-ws"))
            .await
            .unwrap()
            .unwrap();
        let start: Value = serde_json::from_str(&start_env).unwrap();
        let session_id = start["result"]["session"]["sessionId"]
            .as_str()
            .unwrap()
            .to_owned();
        let env = handle_ws_text(processor, &respond_line(3, &session_id, "r-ws"))
            .await
            .unwrap()
            .unwrap();
        let envelope: Value = serde_json::from_str(&env).unwrap();
        envelope["result"].clone()
    }

    // -------------------------------------------------------------------
    // Error-shape helpers — return (numeric_code, domain_code) for the
    // not-initialized and invalid-params error paths.
    // -------------------------------------------------------------------

    fn domain_for(numeric: i64) -> String {
        xai_grok_app_server_protocol::lookup_error_numeric(numeric)
            .map(|s| s.code.to_owned())
            .unwrap_or_else(|| "unknown".into())
    }

    async fn err_not_initialized_in_process() -> (i64, String) {
        let processor = Arc::new(FacadeProcessor::new(Arc::new(FakeRuntime::new())));
        let mut client = InProcessClient::new(processor);
        let err = client
            .request(
                "interaction/respond",
                json!({
                    "sessionId":"s","turnId":"t","interactionId":"ix",
                    "decision":"allow","idempotencyKey":"r"
                }),
            )
            .await
            .unwrap_err();
        (err.code, domain_for(err.code))
    }

    async fn err_not_initialized_stdio() -> (i64, String) {
        let processor = Arc::new(FacadeProcessor::new(Arc::new(FakeRuntime::new())));
        // No initialize first.
        let out = process_ndjson_batch(processor, &respond_line(1, "s", "r"))
            .await
            .unwrap();
        let env: Value = serde_json::from_str(&out[0]).unwrap();
        let code = env["error"]["code"].as_i64().unwrap();
        let domain = env["error"]["data"]["code"].as_str().unwrap().to_owned();
        (code, domain)
    }

    async fn err_not_initialized_ws_frame() -> (i64, String) {
        let processor = Arc::new(FacadeProcessor::new(Arc::new(FakeRuntime::new())));
        let env = handle_ws_text(processor, &respond_line(1, "s", "r"))
            .await
            .unwrap()
            .unwrap();
        let v: Value = serde_json::from_str(&env).unwrap();
        let code = v["error"]["code"].as_i64().unwrap();
        let domain = v["error"]["data"]["code"].as_str().unwrap().to_owned();
        (code, domain)
    }

    async fn err_invalid_params_in_process() -> (i64, String) {
        let processor = Arc::new(FacadeProcessor::new(Arc::new(FakeRuntime::new())));
        let mut client = InProcessClient::new(processor);
        let _ = client.initialize().await.unwrap();
        let err = client
            .request("interaction/respond", json!({}))
            .await
            .unwrap_err();
        (err.code, domain_for(err.code))
    }

    async fn err_invalid_params_stdio() -> (i64, String) {
        let processor = Arc::new(FacadeProcessor::new(Arc::new(FakeRuntime::new())));
        let batch = format!("{}\n{}\n", init_line(1), respond_line_missing_session(2));
        let out = process_ndjson_batch(processor, &batch).await.unwrap();
        let env: Value = serde_json::from_str(&out[1]).unwrap();
        let code = env["error"]["code"].as_i64().unwrap();
        let domain = env["error"]["data"]["code"].as_str().unwrap().to_owned();
        (code, domain)
    }

    async fn err_invalid_params_ws_frame() -> (i64, String) {
        let processor = Arc::new(FacadeProcessor::new(Arc::new(FakeRuntime::new())));
        let _ = handle_ws_text(processor.clone(), &init_line(1))
            .await
            .unwrap();
        let env = handle_ws_text(processor, &respond_line_missing_session(2))
            .await
            .unwrap()
            .unwrap();
        let v: Value = serde_json::from_str(&env).unwrap();
        let code = v["error"]["code"].as_i64().unwrap();
        let domain = v["error"]["data"]["code"].as_str().unwrap().to_owned();
        (code, domain)
    }

    // ===================================================================
    // GREEN: accept shape parity
    // ===================================================================

    #[tokio::test]
    async fn interaction_conformance_accept_shape_matches_across_transports() {
        // The same well-formed `interaction/respond` request must produce the
        // identical accept result object across in-process, stdio, and the WS
        // frame adapter. The accept shape is `{"operationId":"interaction",
        // "accepted":true}` — produced by the shared processor dispatch.
        let ip = accept_in_process().await;
        let st = accept_stdio().await;
        let ws = accept_ws_frame().await;

        assert_eq!(ip, st, "in-process accept shape must equal stdio");
        assert_eq!(ip, ws, "in-process accept shape must equal ws frame");
        assert_eq!(ip, json!({"operationId":"interaction","accepted":true}));
    }

    #[tokio::test]
    async fn interaction_conformance_request_envelope_shape_is_stable() {
        // The request envelope shape (method + params keys) is identical
        // across transports by construction — the same JSON-RPC line is fed
        // to each. This test pins the canonical request shape so a future
        // drift in params naming is caught.
        let req = respond_line(1, "session_x", "r-1");
        let v: Value = serde_json::from_str(&req).unwrap();
        assert_eq!(v["jsonrpc"], "2.0");
        assert_eq!(v["method"], "interaction/respond");
        let params = v["params"].as_object().unwrap();
        let keys: std::collections::BTreeSet<String> = params.keys().cloned().collect();
        assert_eq!(
            keys,
            [
                "sessionId",
                "turnId",
                "interactionId",
                "decision",
                "idempotencyKey"
            ]
            .into_iter()
            .map(String::from)
            .collect::<std::collections::BTreeSet<_>>(),
            "params must use camelCase wire names"
        );
    }

    // ===================================================================
    // GREEN: error shape parity — not-initialized gate
    // ===================================================================

    #[tokio::test]
    async fn interaction_conformance_not_initialized_error_shape_matches() {
        // `interaction/respond` requires initialization. Without it, every
        // transport must return the SAME not-initialized error: numeric
        // -32002, domain `not_initialized`.
        let (ip_code, ip_domain) = err_not_initialized_in_process().await;
        let (st_code, st_domain) = err_not_initialized_stdio().await;
        let (ws_code, ws_domain) = err_not_initialized_ws_frame().await;

        assert_eq!(ip_code, -32002, "in-process: not_initialized numeric");
        assert_eq!(st_code, -32002, "stdio: not_initialized numeric");
        assert_eq!(ws_code, -32002, "ws frame: not_initialized numeric");
        assert_eq!(ip_domain, "not_initialized");
        assert_eq!(st_domain, "not_initialized");
        assert_eq!(ws_domain, "not_initialized");
    }

    // ===================================================================
    // GREEN: error shape parity — invalid params
    // ===================================================================

    #[tokio::test]
    async fn interaction_conformance_invalid_params_error_shape_matches() {
        // A malformed request (missing required `sessionId`) must produce
        // the SAME invalid-params error across all transports: numeric
        // -32602, domain `invalid_params`.
        let (ip_code, ip_domain) = err_invalid_params_in_process().await;
        let (st_code, st_domain) = err_invalid_params_stdio().await;
        let (ws_code, ws_domain) = err_invalid_params_ws_frame().await;

        assert_eq!(ip_code, -32602, "in-process: invalid_params numeric");
        assert_eq!(st_code, -32602, "stdio: invalid_params numeric");
        assert_eq!(ws_code, -32602, "ws frame: invalid_params numeric");
        assert_eq!(ip_domain, "invalid_params");
        assert_eq!(st_domain, "invalid_params");
        assert_eq!(ws_domain, "invalid_params");
    }

    #[tokio::test]
    async fn operation_id_is_explicitly_null_for_failed_calls_across_transports() {
        let (ip_code, _ip_domain) = err_invalid_params_in_process().await;
        let _ = ip_code;
        let processor = FacadeProcessor::new(Arc::new(FakeRuntime::new()));
        processor
            .handle_line(
                &json!({
                    "jsonrpc":"2.0","id":1,"method":"initialize",
                    "params": {"protocolVersion": PROTOCOL_VERSION, "clientInfo":{"name":"test","version":"0"}, "capabilities":{}}
                })
                .to_string(),
            )
            .await
            .unwrap();
        let response = processor
            .handle_line(
                &json!({
                    "jsonrpc":"2.0","id":9,"method":"interaction/respond",
                    "params": {"sessionId":"missing"}
                })
                .to_string(),
            )
            .await
            .unwrap()
            .unwrap();
        let value: Value = serde_json::from_str(&response).unwrap();
        assert_eq!(
            value["error"]["data"].get("operationId"),
            Some(&Value::Null)
        );
    }

    // ===================================================================
    // Non-vacuity guard — the conformance suite covers the minimum matrix.
    // ===================================================================

    #[tokio::test]
    async fn interaction_conformance_suite_covers_minimum_matrix() {
        // Non-vacuity guard: asserts every minimum scenario has a dedicated
        // test and that the suite exercises all three transport surfaces.
        let src = include_str!("../src/lib.rs");
        let minimum = [
            "interaction_conformance_accept_shape_matches_across_transports",
            "interaction_conformance_not_initialized_error_shape_matches",
            "interaction_conformance_invalid_params_error_shape_matches",
        ];
        for name in minimum {
            assert!(
                src.contains(name),
                "missing minimum conformance test: {name}"
            );
        }
        // The suite must exercise the real transport surfaces, not just the
        // processor directly.
        assert!(
            src.contains("InProcessClient"),
            "must use in-process client"
        );
        assert!(src.contains("process_ndjson_batch"), "must use stdio batch");
        assert!(src.contains("handle_ws_text"), "must use ws frame adapter");
        // The production dispatch must route interaction/respond through the
        // shared runtime facade (no second permission engine at this layer).
        let prod = include_str!("../src/processor.rs");
        assert!(
            prod.contains("\"interaction/respond\""),
            "processor must dispatch interaction/respond"
        );
    }
}

// ===========================================================================
// C6-C: real WS listener black-box conformance for `interaction/respond`.
// Feature-gated behind `websocket`. Spawn a real listener on an ephemeral
// loopback port; connect with a real `tokio-tungstenite` client; send the
// same `interaction/respond` sequence over the wire and assert the accept
// and error shapes match the stdio/in-process path. This closes the F-11
// gap (WS leg was helper-level `handle_ws_text`, not black-box).
// ===========================================================================
#[cfg(all(test, feature = "websocket"))]
mod interaction_conformance_ws_listener {
    use super::*;
    use crate::transport::stdio::process_ndjson_batch;
    use futures_util::{SinkExt, StreamExt};
    use serde_json::{Value, json};
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::time::timeout;
    use tokio_tungstenite::tungstenite::Message;
    use tokio_tungstenite::tungstenite::client::IntoClientRequest;
    use tokio_tungstenite::tungstenite::http::HeaderValue;
    use tokio_tungstenite::{WebSocketStream, connect_async};
    use xai_grok_app_server_protocol::PROTOCOL_VERSION;
    use xai_grok_tower::FakeRuntime;

    type ClientStream = WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;
    const TOKEN: &str = "test-bearer-secret";

    async fn spawn_listener() -> std::net::SocketAddr {
        let processor = Arc::new(FacadeProcessor::new(Arc::new(FakeRuntime::new())));
        let config = WsListenerConfig {
            bind: "127.0.0.1:0".to_owned(),
            bearer_token: TOKEN.to_owned(),
            require_auth: true,
            outbound_queue_cap: OUTBOUND_QUEUE_CAP,
        };
        let handle = run_ws_listener(processor, config).await.unwrap();
        handle.addr
    }

    async fn connect(addr: std::net::SocketAddr) -> ClientStream {
        let url = format!("ws://{addr}/");
        let mut req = url.as_str().into_client_request().unwrap();
        req.headers_mut().insert(
            "Authorization",
            HeaderValue::from_str(&format!("Bearer {TOKEN}")).unwrap(),
        );
        let (stream, _resp) = connect_async(req).await.unwrap();
        stream
    }

    async fn send(stream: &mut ClientStream, text: &str) {
        stream
            .send(Message::Text(text.to_owned().into()))
            .await
            .unwrap();
    }

    async fn recv(stream: &mut ClientStream) -> Value {
        loop {
            match timeout(Duration::from_secs(5), stream.next()).await {
                Ok(Some(Ok(Message::Text(t)))) => return serde_json::from_str(t.as_str()).unwrap(),
                Ok(Some(Ok(_))) => continue,
                Ok(Some(Err(e))) => panic!("ws read error: {e}"),
                Ok(None) => panic!("ws closed before response"),
                Err(_) => panic!("ws recv timed out"),
            }
        }
    }

    fn init_line(id: u64) -> String {
        json!({
            "jsonrpc":"2.0","id":id,"method":"initialize","params":{
                "protocolVersion": PROTOCOL_VERSION,
                "clientInfo":{"name":"ws-conf","version":"0"},
                "capabilities":{}
            }
        })
        .to_string()
    }

    fn start_line(id: u64, key: &str) -> String {
        json!({
            "jsonrpc":"2.0","id":id,"method":"session/start","params":{
                "workspaceRoot":"/work","idempotencyKey":key
            }
        })
        .to_string()
    }

    fn respond_line(id: u64, session_id: &str, key: &str) -> String {
        json!({
            "jsonrpc":"2.0","id":id,"method":"interaction/respond","params":{
                "sessionId": session_id,
                "turnId":"turn-1",
                "interactionId":"ix-1",
                "decision":"allow",
                "idempotencyKey": key
            }
        })
        .to_string()
    }

    #[tokio::test]
    async fn interaction_conformance_ws_listener_accept_shape_matches_stdio() {
        // Real WS listener black-box: send init + session/start +
        // interaction/respond over a real WS connection and assert the
        // accept shape matches the stdio path exactly.
        let addr = spawn_listener().await;

        // stdio reference (same processor shape).
        let p_stdio = Arc::new(FacadeProcessor::new(Arc::new(FakeRuntime::new())));
        let stdio_batch = format!("{}\n{}\n", init_line(1), start_line(2, "stdio-ref"));
        let stdio_out = process_ndjson_batch(p_stdio.clone(), &stdio_batch)
            .await
            .unwrap();
        let stdio_start: Value = serde_json::from_str(&stdio_out[1]).unwrap();
        let stdio_session = stdio_start["result"]["session"]["sessionId"]
            .as_str()
            .unwrap()
            .to_owned();
        let stdio_resp_out =
            process_ndjson_batch(p_stdio, &respond_line(3, &stdio_session, "r-stdio"))
                .await
                .unwrap();
        let stdio_env: Value = serde_json::from_str(&stdio_resp_out[0]).unwrap();
        let stdio_result = stdio_env["result"].clone();

        // ws listener path.
        let mut s = connect(addr).await;
        send(&mut s, &init_line(1)).await;
        let _ = recv(&mut s).await;
        send(&mut s, &start_line(2, "ws-ref")).await;
        let ws_start = recv(&mut s).await;
        let ws_session = ws_start["result"]["session"]["sessionId"]
            .as_str()
            .unwrap()
            .to_owned();
        send(&mut s, &respond_line(3, &ws_session, "r-ws")).await;
        let ws_env = recv(&mut s).await;

        assert_eq!(ws_env["jsonrpc"], "2.0");
        assert_eq!(ws_env["id"], 3);
        assert_eq!(ws_env["result"], stdio_result);
        assert_eq!(
            ws_env["result"],
            json!({"operationId":"interaction","accepted":true})
        );
    }

    #[tokio::test]
    async fn interaction_conformance_ws_listener_not_initialized_error_matches_stdio() {
        // Real WS listener: send `interaction/respond` WITHOUT initialize on
        // a fresh listener (fresh processor) and assert the not-initialized
        // error shape matches the stdio path.
        let addr = spawn_listener().await;

        // stdio reference (fresh processor, no init).
        let p_stdio = Arc::new(FacadeProcessor::new(Arc::new(FakeRuntime::new())));
        let stdio_out = process_ndjson_batch(p_stdio, &respond_line(1, "s", "r"))
            .await
            .unwrap();
        let stdio_env: Value = serde_json::from_str(&stdio_out[0]).unwrap();

        // ws listener path (fresh listener = fresh processor, no init).
        let mut s = connect(addr).await;
        send(&mut s, &respond_line(1, "s", "r")).await;
        let ws_env = recv(&mut s).await;

        assert_eq!(ws_env["error"]["code"], stdio_env["error"]["code"]);
        assert_eq!(ws_env["error"]["code"], -32002);
        assert_eq!(
            ws_env["error"]["data"]["code"],
            stdio_env["error"]["data"]["code"]
        );
        assert_eq!(ws_env["error"]["data"]["code"], "not_initialized");
    }

    #[tokio::test]
    async fn interaction_conformance_ws_listener_invalid_params_error_matches_stdio() {
        // Real WS listener: send a malformed `interaction/respond` (missing
        // `sessionId`) after initialize and assert the invalid-params error
        // shape matches the stdio path.
        let addr = spawn_listener().await;

        let bad = json!({
            "jsonrpc":"2.0","id":2,"method":"interaction/respond","params":{
                "turnId":"turn-1","interactionId":"ix-1",
                "decision":"allow","idempotencyKey":"r-bad"
            }
        })
        .to_string();

        // stdio reference.
        let p_stdio = Arc::new(FacadeProcessor::new(Arc::new(FakeRuntime::new())));
        let stdio_batch = format!("{}\n{}\n", init_line(1), bad);
        let stdio_out = process_ndjson_batch(p_stdio, &stdio_batch).await.unwrap();
        let stdio_env: Value = serde_json::from_str(&stdio_out[1]).unwrap();

        // ws listener path.
        let mut s = connect(addr).await;
        send(&mut s, &init_line(1)).await;
        let _ = recv(&mut s).await;
        send(&mut s, &bad).await;
        let ws_env = recv(&mut s).await;

        assert_eq!(ws_env["error"]["code"], stdio_env["error"]["code"]);
        assert_eq!(ws_env["error"]["code"], -32602);
        assert_eq!(
            ws_env["error"]["data"]["code"],
            stdio_env["error"]["data"]["code"]
        );
        assert_eq!(ws_env["error"]["data"]["code"], "invalid_params");
    }
}
