# Wave — Tower wait bounded polling (2026-07-19)

## Mudança

`tower_agent_wait` agora honra `timeoutMs` quando o cursor não tem eventos:

- consulta o replay canônico;
- retorna imediatamente quando surgem eventos;
- faz polling cooperativo de até 20 ms;
- encerra no deadline solicitado, limitado a 300 s pelo contrato;
- preserva epoch, cursor e envelope de wake reason.

Foi adicionada dependência normal de Tokio com feature `time`, pois isso é
comportamento de produção e não apenas infraestrutura de teste.

## Validação

```text
cargo test -p xai-grok-tower-tools wait_polls_until_timeout_after_cursor_without_events
PASS

cargo clippy -p xai-grok-tower-tools --all-targets -- -D warnings
PASS

git diff --check
PASS
```

## Limitações

O polling é bounded e não bloqueia a thread, mas ainda não é uma subscription
push real; cancelamento externo da chamada e classificação terminal/Interaction
precisam de gates posteriores.
