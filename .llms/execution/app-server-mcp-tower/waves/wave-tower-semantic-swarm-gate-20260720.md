# Wave — Tower semantic swarm gate (2026-07-20)

## Objetivo

Revalidar a superfície dos nove tools e os limites de múltiplas Sessions antes
de avançar para o swarm product-backed.

## Evidência

```text
cargo test -p xai-grok-tower-tools --all-targets
24 unit tests passed
24 C6 integration tests passed
```

Os testes cobrem ACL fail-closed, nove invokes, idempotência, inputs
estruturados, cursores/history/wait, wake reason de interaction, schemas
independentes e limites de N Sessions sem hub.

## Limite

Isso é conformance semântica sobre o runtime fake/in-memory. Não prova N
Sessions com atores reais, providers, epochs independentes e recursos
limitados; TA103-13 permanece parcial.
