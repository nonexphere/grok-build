# Wave — consolidated lightweight gate (2026-07-20)

## Comando

```text
cargo test -p xai-grok-app-server-protocol \
  -p xai-grok-app-server \
  -p xai-grok-mcp-server \
  --features xai-grok-mcp-server/streamable-http \
  -p xai-grok-tower --all-targets
```

## Resultado

```text
App Server protocol: 22 passed
App Server: 41 passed
MCP server library: 21 passed
MCP Streamable HTTP: 41 passed
Tower core: 29 passed
Tower instance integration: 10 passed
Total: 164 passed, 0 failed
```

Este gate cobre os componentes leves em conjunto, mas não inclui o
`xai-grok-shell`/pager product actor por causa da compilação pesada e não deve
ser confundido com prontidão de runtime real.
