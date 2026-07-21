# Wave — Tower history replay cursor (2026-07-19)

## Mudança

`tower_agent_history` deixou de rejeitar todo `afterEventSeq` diferente de
zero. Para cursores não-zero, agora delega ao `GrokRuntimeFacade::replay`,
preserva `historyEpoch`, filtra eventos projetáveis em Items, aplica os mesmos
limites `mode`/`lastItems`/`maxBytes` e retorna `nextEventSeq` baseado no cursor
canônico.

O caminho `afterEventSeq == "0"` continua usando `read_session` para obter o
snapshot completo de itens.

## Validação

```text
cargo test -p xai-grok-tower-tools
PASS: 21 unitários + 24 integração

cargo test -p xai-grok-tower-tools history_parity_epoch_and_redaction_flag
PASS: inclui cursor não-zero

git diff --check
PASS
```

## Limitações honestas

Eventos `ItemDelta`, lifecycle de Interaction e parity black-box entre MCP,
stdio e App Server ainda não têm agregação/conformance completa. O cursor
agora é funcional sobre eventos projetáveis, mas não é declarado como replay
completo de todos os tipos de Item.
