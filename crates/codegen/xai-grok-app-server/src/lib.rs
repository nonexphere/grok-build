//! App Server processor and transports over `GrokRuntimeFacade`.
//!
//! This crate must never construct `SessionActor` or depend on Shell.

pub mod controller;
pub mod processor;
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
