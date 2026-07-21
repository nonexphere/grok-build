# Wave — production ACP default composition (2026-07-19)

## Mudança

Os construtores default de produção passaram a usar a factory ACP real:

- `experimental_app_server_processor()` injeta
  `experimental_acp_resident_spawn` no `ShellSessionActorRuntime`;
- `experimental_mcp_http_runtime()` usa a mesma factory e a mesma autoridade
  Shell/JSONL;
- `*_with_root` continua storage-only para testes herméticos, sem credenciais
  ou rede.

O runtime continua fail-closed em auth/bootstrap e não usa echo offline.

## Capability contract

O default de produção anuncia `turn/start`, `turn/steer` e `turn/interrupt`.
Interaction approvals/questions/MCP elicitation e item lifecycle/deltas seguem
`false`, pois ainda não têm gates produtivos equivalentes.

## Validação

```text
cargo test -p xai-grok-pager-bin --bin goblin composition_tests
PASS: 15 testes

cargo test -p xai-grok-pager-bin --bin goblin production_default_composes_acp_turn_capabilities
PASS

git diff --check
PASS
```

## Riscos restantes

O caminho default agora pode alcançar auth/model/network em `session/start` e
Turn; ainda faltam black-box com credenciais controladas, Interaction
produtiva, item projection completa e soak/cleanup dos binários reais.
