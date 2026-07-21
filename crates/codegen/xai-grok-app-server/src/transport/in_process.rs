//! In-process typed client handle over the shared processor.

use std::sync::Arc;

use serde_json::{Value, json};
use xai_grok_app_server_protocol::PROTOCOL_VERSION;

use crate::ProcessorError;
use crate::processor::FacadeProcessor;

pub struct InProcessClient {
    processor: Arc<FacadeProcessor>,
    next_id: u64,
}

impl InProcessClient {
    pub fn new(processor: Arc<FacadeProcessor>) -> Self {
        Self {
            processor,
            next_id: 1,
        }
    }

    fn next_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub async fn request(&mut self, method: &str, params: Value) -> Result<Value, ProcessorError> {
        let id = self.next_id();
        let line = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        })
        .to_string();
        let response = self
            .processor
            .handle_line(&line)
            .await?
            .ok_or_else(|| ProcessorError {
                code: -32603,
                message: "expected response".into(),
            })?;
        let value: Value = serde_json::from_str(&response).map_err(|e| ProcessorError {
            code: -32700,
            message: e.to_string(),
        })?;
        if let Some(err) = value.get("error") {
            return Err(ProcessorError {
                code: err["code"].as_i64().unwrap_or(-32603),
                message: err["message"].as_str().unwrap_or("error").into(),
            });
        }
        Ok(value["result"].clone())
    }

    pub async fn initialize(&mut self) -> Result<Value, ProcessorError> {
        self.request(
            "initialize",
            json!({
                "protocolVersion": PROTOCOL_VERSION,
                "clientInfo": {"name": "in-process", "version": "0"},
                "capabilities": {"reconnect": true}
            }),
        )
        .await
    }
}

#[cfg(test)]
mod in_process_tests {
    use super::*;
    use xai_grok_tower::FakeRuntime;

    #[tokio::test]
    async fn in_process_initialize_session_turn_item_stream() {
        let processor = Arc::new(FacadeProcessor::new(Arc::new(FakeRuntime::new())));
        let mut client = InProcessClient::new(processor);
        let init = client.initialize().await.unwrap();
        assert_eq!(init["protocolVersion"], PROTOCOL_VERSION);
        let session = client
            .request(
                "session/start",
                json!({"workspaceRoot":"/work","idempotencyKey":"ip-1"}),
            )
            .await
            .unwrap();
        let session_id = session["session"]["sessionId"].as_str().unwrap().to_owned();
        let turn = client
            .request(
                "turn/start",
                json!({
                    "sessionId": session_id,
                    "input":[{"type":"text","text":"hi"}],
                    "idempotencyKey":"ip-t1"
                }),
            )
            .await
            .unwrap();
        assert!(turn["turn"]["turnId"].is_string());
        let replay = client
            .request(
                "session/subscribe",
                json!({"sessionId": session_id, "afterEventSeq":"0"}),
            )
            .await
            .unwrap();
        assert!(!replay["replay"]["events"].as_array().unwrap().is_empty());
    }
}
