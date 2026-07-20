# Wave — planning consistency audit (2026-07-19)

## Resultado

O índice `.llms/grok-build/30-app-server/README.md` estava obsoleto: declarava
que processor, facade e transports ainda não existiam. A implementação atual
tem esses componentes e testes de nível de componente, mas não tem ainda a
integração produtiva completa com actor canônico residente, Turn/steer/
Interaction/replay real e consumer TS de release.

O índice foi corrigido para distinguir infraestrutura implementada de runtime
produtivo incompleto.

## Claims conferidos

- `v1-08-product-session-host`: permanece parcial; ACP host experimental foi
  comprovado, wiring de produto e actor canônico continuam pendentes.
- `v1-03-tower-product-runtime` e `v1-08-tower-product-runtime`: permanecem
  parciais; transports/supervisor têm evidência, mas o runtime de turnos real
  ainda não está ligado ao composition root.
- `v1-04-mcp-contract-transport-completion`: permanece rascunho/P0; o gate
  descrito é um objetivo de saída, não uma declaração de conclusão.
- `TA103-03/04/10/11` e `MCP104-03`: permanecem `partial`, com os gaps
  explícitos nas próprias tasks.

## Gaps que não foram promovidos indevidamente

Ainda não há evidência suficiente para marcar como concluídos: factory
canônica do actor, steer semântico ACP, Interaction produtiva, blocking wait,
projeção cursor→evento, identidade de erro entre App Server/MCP, black-box
cross-transport completo, limites SSE/queue e release/human smoke produtivo.
