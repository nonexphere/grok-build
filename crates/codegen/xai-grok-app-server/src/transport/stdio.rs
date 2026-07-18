//! NDJSON stdio transport: one UTF-8 JSON object per line on stdout; stderr diagnostics only.

use std::io::{BufRead, Write};
use std::sync::Arc;

use crate::processor::FacadeProcessor;
use crate::ProcessorError;

/// Run a single-threaded NDJSON request/response loop until stdin EOF.
pub async fn run_stdio_loop<R, W, E>(
    processor: Arc<FacadeProcessor>,
    mut reader: R,
    mut writer: W,
    mut stderr: E,
) -> Result<(), ProcessorError>
where
    R: BufRead,
    W: Write,
    E: Write,
{
    let mut line = String::new();
    loop {
        line.clear();
        let n = reader.read_line(&mut line).map_err(|e| ProcessorError {
            code: -32603,
            message: format!("stdin read failed: {e}"),
        })?;
        if n == 0 {
            // EOF — begin drain (nothing buffered in this simple loop).
            let _ = writeln!(stderr, "stdio eof; draining");
            break;
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        match processor.handle_line(trimmed).await {
            Ok(Some(response)) => {
                // Exactly one JSON object per line on stdout.
                writeln!(writer, "{response}").map_err(|e| ProcessorError {
                    code: -32603,
                    message: format!("stdout write failed: {e}"),
                })?;
                writer.flush().ok();
            }
            Ok(None) => {}
            Err(err) => {
                let _ = writeln!(stderr, "processor error: {} {}", err.code, err.message);
                // If parse failed without id, still emit a failure envelope when possible.
                if err.code == -32700 {
                    let failure = serde_json::json!({
                        "jsonrpc":"2.0",
                        "id": null,
                        "error":{"code": err.code, "message": err.message, "data":{"code":"parse_error","retryable":false}}
                    });
                    writeln!(writer, "{failure}").ok();
                }
            }
        }
    }
    Ok(())
}

/// Process a batch of NDJSON request lines into response lines (test helper / embedded stdio).
pub async fn process_ndjson_batch(
    processor: Arc<FacadeProcessor>,
    input: &str,
) -> Result<Vec<String>, ProcessorError> {
    let mut outputs = Vec::new();
    for line in input.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(response) = processor.handle_line(trimmed).await? {
            outputs.push(response);
        }
    }
    Ok(outputs)
}

#[cfg(test)]
mod stdio_tests {
    use super::*;
    use serde_json::{json, Value};
    use xai_grok_app_server_protocol::PROTOCOL_VERSION;
    use xai_grok_tower::FakeRuntime;

    #[tokio::test]
    async fn stdio_ndjson_one_json_object_per_line_vertical_slice() {
        let processor = Arc::new(FacadeProcessor::new(Arc::new(FakeRuntime::new())));
        let input = format!(
            "{}\n{}\n{}\n",
            json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":PROTOCOL_VERSION,"clientInfo":{"name":"stdio","version":"0"},"capabilities":{}}}),
            json!({"jsonrpc":"2.0","id":2,"method":"session/start","params":{"workspaceRoot":"/work","idempotencyKey":"stdio-s"}}),
            json!({"jsonrpc":"2.0","id":3,"method":"session/list","params":{}}),
        );
        let outputs = process_ndjson_batch(processor.clone(), &input)
            .await
            .unwrap();
        assert_eq!(outputs.len(), 3);
        for line in &outputs {
            assert_eq!(line.lines().count(), 1, "stdout must be one object per line");
            let v: Value = serde_json::from_str(line).unwrap();
            assert_eq!(v["jsonrpc"], "2.0");
            assert!(v.get("result").is_some());
        }
        let start: Value = serde_json::from_str(&outputs[1]).unwrap();
        let session_id = start["result"]["session"]["sessionId"].as_str().unwrap();
        let turn_input = json!({"jsonrpc":"2.0","id":4,"method":"turn/start","params":{
            "sessionId": session_id,
            "input":[{"type":"text","text":"stdio hi"}],
            "idempotencyKey":"stdio-t"
        }})
        .to_string();
        let turn_out = process_ndjson_batch(processor, &turn_input).await.unwrap();
        assert_eq!(turn_out.len(), 1);
        let turn_v: Value = serde_json::from_str(&turn_out[0]).unwrap();
        assert!(turn_v["result"]["turn"]["turnId"].is_string());
    }
}
