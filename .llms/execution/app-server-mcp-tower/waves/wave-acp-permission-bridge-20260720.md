# Wave — ACP permission bridge contract (2026-07-20)

## Implementado

- `HostClient::request_permission` publica o reverse request ACP com sessão,
  turno, item, `toolCallId` e payload bruto.
- O waiter é one-shot e fail-closed: decisão ausente, cancelada, duplicada ou
  tardia não autoriza a operação.
- `AcpHostHandle`/`AcpCommandHandle` expõem `respond_permission`.
- Residentes ACP carregam o command handle opcional; `respond_interaction`
  encaminha `Selected(optionId)` ou cancelamento ao ACP.

## Gate executado

O teste `permission_reverse_request_waits_for_selected_decision` exerce o
reverse request ACP, observa a solicitação publicada, valida a correlação
`sessionId`/`turnId`/`itemId`/`toolCallId`, entrega uma decisão e confirma o
`SelectedPermissionOutcome` retornado ao cliente ACP.

Comando:

```text
cargo test -p xai-grok-shell --lib app_server_runtime::acp_host::tests
```

Resultado: passou.

## Gate product ACP adicional

O teste `product_acp_host_round_trips_real_tool_permission` usa duas respostas
SSE scripted: uma chamada real ao tool `write` e uma resposta posterior. A
execução percorre `session/prompt → tool call → permission engine →
request_permission → respond_permission → tool outcome → inferência seguinte`.
Também valida a correlação de turno/item e exige pelo menos duas requisições de
inferência.

Comando adicional:

```text
cargo test -p xai-grok-shell --test product_acp_host -- --nocapture
```

Resultado: 6 testes passaram.

O limite restante não é mais a geração do tool call no ACP host. Ainda falta
provar reconnect/lease/timeout no mesmo caminho product ACP. O teste
`product_facade_respond_interaction_round_trips_real_acp_permission` já prova a
entrada pelo `ShellSessionActorRuntime`/facade: inicia o Turn, descobre o turno
ativo, envia `interaction/respond` e só observa o Turn completo depois da
inferência de follow-up.

Comando:

```text
cargo test -p xai-grok-shell --test product_acp_host product_facade_respond_interaction_round_trips_real_acp_permission -- --nocapture
```

Resultado: passou. As capabilities públicas de interactions permanecem
desabilitadas até os gates de reconnect/lease/timeout product-wired.

## Expiry e first-answer-wins

O waiter ACP agora recebe uma duração explícita: 300 segundos no host de
produção e duração injetável nos testes. Em expiry, o ID é removido do mapa de
decisões antes de retornar `Cancelled`; uma resposta posterior não encontra
waiter e não pode autorizar. IDs duplicados também não substituem o primeiro
waiter, preservando first-answer-wins.

Gate:

```text
cargo test -p xai-grok-shell --lib app_server_runtime::acp_host::tests
```

Resultado: passou, incluindo `permission_reverse_request_expires_and_removes_waiter`.
