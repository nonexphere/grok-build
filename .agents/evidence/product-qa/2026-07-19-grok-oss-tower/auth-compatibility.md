# Tower authentication compatibility validation

Date: 2026-07-19  
Subagents: none

## Implemented

- `grok-oss tower --insecure-no-auth` disables auth on both listeners.
- Secure MCP accepts `Authorization: Bearer <token>` or `?bearer=<token>`.
- Header authentication remains the preferred path when both are supplied.
- Invalid query bearer remains rejected with HTTP 401.

## Evidence

| Check | Result |
| --- | --- |
| CLI flag parser | PASS — `tower_insecure_no_auth_flag_is_opt_in` |
| No-auth MCP initialize without header | PASS — HTTP 200 / MCP 2024-11-05 |
| No-auth App Server initialize without header | PASS — WS protocol 2026-07-18.experimental-v2 |
| Query bearer initialize | PASS — HTTP 200 / MCP 2024-11-05 |
| Wrong query bearer | PASS — HTTP 401 |
| Codex MCP session after rebuild | PASS — real `tower_agent_list`, `MCP_OK 9`, exit 0 |

The active runtime was restarted from the rebuilt binary and is listening on
`127.0.0.1:2419` (App Server), `127.0.0.1:8788` (MCP), and the Codex
app-server remains on `127.0.0.1:1455`. No secret value is recorded here.
