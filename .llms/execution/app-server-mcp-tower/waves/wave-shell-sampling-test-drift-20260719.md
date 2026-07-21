# Wave — Shell sampling test API drift (2026-07-19)

## Mudança

Atualizados os destructurings de `conversation_stream_responses` em
`crates/codegen/xai-grok-shell/tests/test_sampling_client.rs` para a assinatura
atual de quatro retornos:

```text
(stream, metadata, doom_loop_collector, assistant_phase_map)
```

Os testes continuam ignorando `assistant_phase_map` e preservam as asserções do
collector quando aplicável.

## Validação

```text
CARGO_BUILD_JOBS=1 cargo test -p xai-grok-shell --test test_sampling_client
PASS: 28 testes

git diff --check
PASS
```

Isso removeu o bloqueio de compilação do integration test; não altera código de
produção nem promove capabilities de App Server/MCP/Tower.
