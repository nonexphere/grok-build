# Wave — cross-component regression gate (2026-07-19)

## Escopo

Revalidação após as waves de Shell full-lib, ACP prompt context e sampling API
drift.

## Evidência

```text
cargo test -p xai-grok-tower-tools
PASS: 23 unit + 24 integration

cargo test -p xai-grok-mcp-server
PASS: 13 unit, doc-tests verdes

cargo test -p xai-grok-app-server
PASS: 39 unit, doc-tests verdes

cargo test -p xai-grok-pager-bin --bin goblin composition_tests
PASS: 16 composition tests

git diff --check
PASS
```

## Resultado

Não houve regressão nos adapters, schemas, ACL, auth/limits, replay/cursor,
WS/stdio/HTTP, composição ACP ou capabilities de Turn. Readiness total segue
parcial por design: decisão de interactions, item lifecycle/deltas completos,
subscription push, provider black-box controlado, soak e release/TLS humano
continuam fora do caminho comprovado.
