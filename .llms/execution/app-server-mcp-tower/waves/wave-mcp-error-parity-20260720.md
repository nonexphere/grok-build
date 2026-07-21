# Wave — MCP stdio/HTTP error parity (2026-07-20)

## Objetivo

Provar que o mesmo erro de ACL produzido por `tower_agent_list` tem a mesma
projeção semântica em stdio e Streamable HTTP.

## Alteração

O teste `stdio_and_http_produce_identical_tools_list_and_error_shapes` agora
executa a chamada negada nos dois adapters reais e compara `code`, `retryable`
e `operationId`, além de verificar `isError` no caminho HTTP.

## Evidência

```text
cargo test -p xai-grok-mcp-server --features streamable-http \
  --test streamable_http \
  stdio_and_http_produce_identical_tools_list_and_error_shapes -- --nocapture
1 passed
```

O teste completo da suíte HTTP/produto também passou nesta sessão com 41
testes, junto com 22 testes do protocolo e 41 do App Server.

## Limites

Ainda falta provar identidade de operação para falhas associadas a uma
operação real e convergência total do catálogo MCP com o App Server.
