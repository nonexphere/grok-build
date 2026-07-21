# Wave — MCP NDJSON packet fixture (2026-07-20)

## Objetivo

Materializar um pacote stdio reproduzível e verificar que o transport mantém
ordem, IDs e envelopes ao processar NDJSON.

## Alterações

- Adicionado `tests/fixtures/stdio_tools_packet.ndjson` com `tools/list` e
  `tools/call`.
- Adicionado teste que processa o fixture pelo `process_mcp_stdio_batch` e
  valida duas respostas, IDs preservados, nove ferramentas e conteúdo
  estruturado.

## Evidência

```text
cargo test -p xai-grok-mcp-server \
  stdio_packet_fixture_preserves_ndjson_request_response_order -- --nocapture
1 passed
```

## Limites

O fixture ainda não cobre reconnect, resumption, epoch mismatch ou erro de
autorização; esses cenários continuam cobertos por testes HTTP separados e
precisam de fixtures de pacote dedicados para fechar MCP104-10.
