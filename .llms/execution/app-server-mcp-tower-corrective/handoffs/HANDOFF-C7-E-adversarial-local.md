# Handoff C7-E — Hermetic adversarial local suite (C7 items 45–46 partial)

| Field | Value |
|---|---|
| Agent role | **build** |
| Model | `glm-5.2` |

## Goal

Add/run hermetic adversarial tests already possible without HUMAN TLS/creds:

1. Malformed JSON-RPC / oversize / batch rejection (WS + MCP + stdio already partially covered — consolidate gate)
2. Secret canaries (existing app-server security tests)
3. Path/symlink fail-closed (tower workspace tests)
4. Cancellation + concurrent session starts (shell c1 already)
5. Multi-instance lock contention (C2-B)
6. Capture one master log under `tests/c7/adversarial_GREEN.log` and SCRATCH

Also run:
- `cargo test -p xai-grok-app-server --features websocket` 
- `cargo test -p xai-grok-mcp-server --features streamable-http`
- `cargo test -p xai-grok-tower --test tower_instance_isolation`
- security canaries
- `cargo build -p xai-grok-pager-bin --bin grok-oss`

Document HUMAN-deferred: live remote TLS, live provider smoke.

## Report

Commands + counts + residual HUMAN.
