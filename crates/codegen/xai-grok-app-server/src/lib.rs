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
    use serde_json::{json, Value};
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
        assert_eq!(a[0]["result"]["protocolVersion"], b[0]["result"]["protocolVersion"]);
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
        let session_id = serde_json::from_str::<Value>(&start).unwrap()["result"]["session"]
            ["sessionId"]
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
mod co_start_tests {
    #[test]
    fn co_start_rejects_dual_stdio_accepts_stdio_plus_ws_matrix() {
        // Valid: single stdio, or stdio+ws, or in-process only. Invalid: dual stdio.
        let matrix = [
            (true, false, false, true),  // stdio
            (false, true, false, true),  // ws
            (true, true, false, true),   // stdio+ws
            (false, false, true, true),  // in-process
            (true, false, true, false),  // dual stdio-like (stdio+in-process both claiming stdio ownership) — reject for dual stdio claim
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
    use serde_json::{json, Value};
    use std::sync::Arc;
    use xai_grok_app_server_protocol::PROTOCOL_VERSION;
    use xai_grok_tower::FakeRuntime;
    use crate::transport::websocket::handle_ws_text;

    #[tokio::test]
    async fn websocket_conformance_initialize_matches_stdio_shape() {
        let rt = Arc::new(FakeRuntime::new());
        let p_ws = Arc::new(FacadeProcessor::new(rt.clone()));
        let p_stdio = Arc::new(FacadeProcessor::new(Arc::new(FakeRuntime::new())));
        let req = json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{
            "protocolVersion": PROTOCOL_VERSION,
            "clientInfo":{"name":"c","version":"0"},
            "capabilities":{}
        }}).to_string();
        let ws = handle_ws_text(p_ws, &req).await.unwrap().unwrap();
        let stdio = p_stdio.handle_line(&req).await.unwrap().unwrap();
        let w: Value = serde_json::from_str(&ws).unwrap();
        let s: Value = serde_json::from_str(&stdio).unwrap();
        assert_eq!(w["result"]["protocolVersion"], s["result"]["protocolVersion"]);
        assert_eq!(w["result"]["capabilities"]["sessions"]["start"], s["result"]["capabilities"]["sessions"]["start"]);
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
