# Wave — MCP default-feature gate (2026-07-20)

## Evidência

```text
cargo test -p xai-grok-mcp-server --all-targets
14 library tests passed; HTTP integration target has 0 tests without feature

cargo check -p xai-grok-app-server -p xai-grok-app-server-protocol -p xai-grok-tower
Finished successfully
```

O caminho stdio, schemas, ACL, redaction e parity básica permanecem válidos
sem habilitar Streamable HTTP. O transporte HTTP continua coberto
separadamente com a suíte feature-gated de 41 testes.
