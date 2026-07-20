# `grok-oss tower` product validation

Date: 2026-07-19  
Binary: `grok 0.2.102 (d0ea385)`  
Install: `/home/guilherme/.local/bin/grok-oss`  
Subagents: none

## Implementation

- `tower` starts App Server WS and MCP Streamable HTTP concurrently.
- Defaults: `127.0.0.1:2419` and `127.0.0.1:8788`.
- `--no-mcp` and `--no-app-server` select either listener; both flags fail.
- Secret is optional; `GROK_AGENT_SECRET` is honored, otherwise a random
  32-character token is generated in memory. Explicit whitespace is rejected.
- Startup rollback and coordinated signal shutdown are implemented.

## Evidence

| Check | Result | Evidence |
| --- | --- | --- |
| CLI help | PASS | `grok-oss tower --help` exposes both binds, secret and no-flags |
| No listener guard | PASS | both `--no-*` exits 1 before bind |
| CLI unit tests | PASS | `cargo test -p xai-grok-pager --lib tower_` → 2/2 |
| Default build | PASS | `cargo build -p xai-grok-pager-bin --bin grok-oss` |
| Feature-off compatibility | PASS | `cargo check -p xai-grok-pager-bin --bin grok-oss --no-default-features` |
| App-only | PASS | `tower --no-mcp`, bind and clean exit 0 |
| MCP-only | PASS | `tower --no-app-server`, bind and clean exit 0 |
| Combined runtime | PASS | both listeners started in one process |
| App initialize | PASS | real WS returned `2026-07-18.experimental-v2` |
| MCP initialize | PASS | real HTTP returned `2024-11-05` + `Mcp-Session-Id` |
| Auth rejection | PASS | App wrong bearer rejected; MCP wrong bearer returned `401` |
| Bind rollback | PASS | App port collision exited 1 and MCP port was not left listening |
| Shutdown | PASS | SIGINT closed combined supervisor with exit 0 |
| Installed launcher | PASS | `PROFILE=debug ./scripts/install-grok-oss.sh`, installed binary repeated endpoint smoke |

Warnings are pre-existing duplicate-bin and unused-import/dead-code warnings;
no new warning was treated as a failure.
