//! MCP stdio transport: protocol JSON on stdout; diagnostics on stderr only.

use std::io::{BufRead, Write};
use std::sync::Arc;

use serde_json::Value;
use xai_grok_tower::GrokRuntimeFacade;

use crate::{handle_mcp_jsonrpc, process_mcp_stdio_batch};

pub async fn run_mcp_stdio<R, W, E>(
    runtime: Arc<dyn GrokRuntimeFacade>,
    agent_type: &str,
    explicit_opt_in: bool,
    mut reader: R,
    mut writer: W,
    mut stderr: E,
) -> std::io::Result<()>
where
    R: BufRead,
    W: Write,
    E: Write,
{
    let mut line = String::new();
    loop {
        line.clear();
        let n = reader.read_line(&mut line)?;
        if n == 0 {
            let _ = writeln!(stderr, "mcp stdio eof");
            break;
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        match serde_json::from_str::<Value>(trimmed) {
            Ok(req) => {
                let resp =
                    handle_mcp_jsonrpc(runtime.clone(), agent_type, explicit_opt_in, req).await;
                writeln!(writer, "{resp}")?;
                writer.flush()?;
            }
            Err(err) => {
                let _ = writeln!(stderr, "parse error: {err}");
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod stdio_tests {
    use super::*;
    use serde_json::json;
    use xai_grok_tower::FakeRuntime;

    #[tokio::test]
    async fn stdio_protocol_only_stdout_and_graceful_eof() {
        let rt = Arc::new(FakeRuntime::new());
        let input = format!(
            "{}\n",
            json!({"jsonrpc":"2.0","id":1,"method":"tools/list"})
        );
        let out = process_mcp_stdio_batch(rt, "orchestrator", false, &input).await;
        assert_eq!(out.len(), 1);
        let v: Value = serde_json::from_str(&out[0]).unwrap();
        assert_eq!(v["result"]["tools"].as_array().unwrap().len(), 9);
        // Empty input is graceful EOF with zero lines.
        let empty = process_mcp_stdio_batch(
            Arc::new(FakeRuntime::new()),
            "orchestrator",
            false,
            "",
        )
        .await;
        assert!(empty.is_empty());
    }
}
