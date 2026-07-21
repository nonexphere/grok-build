//! Smoke stdio server (C7-D).
//!
//! A tiny black-box entry point that wires the real [`FacadeProcessor`] to the
//! real NDJSON stdio transport ([`run_stdio_loop`]) over the process's actual
//! stdin/stdout/stderr. It is built by `scripts/smoke/stdio-vertical.sh` and
//! driven as a real subprocess: the smoke script feeds NDJSON request lines on
//! stdin and reads one JSON-RPC response object per line on stdout.
//!
//! Runtime: [`xai_grok_tower::FakeRuntime`] (documented in the smoke log). The
//! processor + transport code is the shipped production path; only the runtime
//! is an in-memory contract fake, so this exercises the real stdio framing,
//! the initialize gate, and JSON-RPC dispatch end-to-end.
//!
//! Run by hand:
//!   cargo build --example stdio_smoke_server -p xai-grok-app-server
//!   ./target/debug/examples/stdio_smoke_server < requests.ndjson

use std::io::{stderr, stdin, stdout};
use std::sync::Arc;

use xai_grok_app_server::{FacadeProcessor, run_stdio_loop};
use xai_grok_tower::FakeRuntime;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), String> {
    let processor = Arc::new(FacadeProcessor::new(Arc::new(FakeRuntime::new())));
    eprintln!(
        "stdio_smoke_server: ready | processor=FacadeProcessor runtime=FakeRuntime transport=stdio-ndjson"
    );
    let stderr_lock = stderr().lock();
    run_stdio_loop(
        processor,
        std::io::BufReader::new(stdin()),
        stdout(),
        stderr_lock,
    )
    .await
    .map_err(|e| format!("stdio loop failed: {} {}", e.code, e.message))
}
