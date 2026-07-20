# Codex ↔ Grok OSS Tower MCP

Date: 2026-07-19  
Scope: local project-scoped Codex MCP integration  
Subagents: none

## Configuration

- Project config: `.codex/config.toml`
- Server name: `grok_oss_tower`
- URL: `http://127.0.0.1:8788/mcp`
- Authentication: `bearer_token_env_var = GROK_OSS_TOWER_SECRET`
- Project trust: `/home/guilherme/github/grok-goblin` is trusted in `~/.codex/config.toml`.

## Runtime

- `grok-oss tower` is running with App Server on `127.0.0.1:2419`.
- MCP Streamable HTTP is running on `127.0.0.1:8788`.
- Codex app-server is running on `ws://127.0.0.1:1455`.
- The secret value is intentionally omitted from this evidence.

## Codex session result

- Thread: `019f7b7f-cf5f-7502-9be7-3e8ab2bdde5d`
- Turn: `019f7b7f-cfcd-7453-8db1-c9f7ea3bcd0a`
- Prompt requested a real `tools/list` call against `grok_oss_tower`.
- Result: `MCP_OK 9`
- Exit code: `0`

Verdict: PASS. The Codex app-server discovered the configured MCP, authenticated
to the tower, completed the Streamable HTTP handshake, and returned the tool
catalog through a real Codex session.
