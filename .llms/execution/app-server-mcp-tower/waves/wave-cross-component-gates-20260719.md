# Wave — cross-component gates (2026-07-19)

## Escopo

Revalidação após a composição default ACP, replay/history por cursor e wait
bounded polling. O objetivo foi confirmar que App Server, Shell ACP e a
composição do binário continuam alinhados.

## Evidência

```text
cargo test -p xai-grok-mcp-server
PASS: 13 testes

cargo test -p xai-grok-app-server
PASS: 39 testes

cargo test -p xai-grok-shell --test product_acp_host
PASS: 5 testes

cargo test -p xai-grok-pager-bin --bin goblin composition_tests
PASS: 16 testes

git diff --check
PASS
```

## Resultado

Os contratos de transporte, auth, replay, composição, persistência ACP,
cancelamento e capabilities de Turn continuam verdes. A prontidão total não
foi declarada: a matriz permanece parcial para actor factory canônico,
interactions produtivas, projeção completa de item/delta, subscription push,
provider black-box controlado, soak e release/TLS humano.
