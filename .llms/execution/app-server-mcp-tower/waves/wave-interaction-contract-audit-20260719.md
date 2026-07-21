# Wave — interaction contract audit (2026-07-19)

## Evidência atual

- `ShellSessionActorRuntime` já possui `PendingInteractions` e
  `InteractionDeliveryHub` para `respond_interaction`.
- O `HostClient` do bridge ACP recebe `request_permission`, mas responde
  sempre `RequestPermissionOutcome::Cancelled`.
- O ACP host só expõe `AcpNotificationSink`; não há canal público para
  publicar uma solicitação pendente nem para aguardar a decisão do App Server.
- O projector de `updates.jsonl` não persiste `InteractionRequested`, então
  replay/history não pode reconstruir uma interação pendente após reconnect.
- O contrato App Server exige `interactionId`, `sessionId`, `turnId`,
  `itemId`, kind, prompt, choices, expiry e first-answer-wins.
- O `RequestPermissionRequest` ACP observado fornece `SessionId`, `ToolCall`
  e opções, mas não carrega diretamente `turnId`/`itemId`; esses campos devem
  ser correlacionados pelo actor com o Turn/item ativo antes da publicação.
- No resident atual, `prompt_id` é a identidade operacional do Turn e a
  projeção persistida deriva `item_id` de `tool_call_id`; essa informação não
  é transportada para o `HostClient` reverse-RPC.

## Decisão de readiness

Interaction approvals/questions/MCP elicitation permanecem `false`. Alterar
esse valor agora criaria uma capability enganosa: o caminho atual cancela a
operação e não permite `interaction/respond` chegar ao ACP prompt suspenso.

## Próxima task executável

Criar um `InteractionBridge` dono do ciclo completo:

1. converter a reverse request ACP em `InteractionRequest` canônico com
   identidade estável;
2. inserir a solicitação no pending table antes de publicar o evento;
3. aguardar decisão ou expiry com cancelamento seguro;
4. mapear decisão para `SelectedPermissionOutcome`/cancelamento ACP;
5. remover a entrada exatamente uma vez e persistir/replay o estado necessário;
6. testar parked actor, resposta concorrente, timeout, reconnect e
   `tower_agent_wait`/subscription em todos os adapters.

O contrato ACP de `RequestPermissionOutcome` e os campos reais de cada tipo de
reverse request devem ser tratados como fonte de verdade; não usar uma segunda
identidade derivada do JSON-RPC request id.

## Dependência de implementação

O próximo desenho deve transportar um contexto imutável por prompt para o
`HostClient` (session, turn e correlação de tool call), ou fazer o ACP client
ser criado pelo próprio resident com acesso a esse estado. Um helper isolado
no adapter não é suficiente e seria código morto sem resolver a entrega da
decisão.

## Resultado desta wave

Não foi feita uma alteração de capability nem uma ponte parcial. A falta de
`turnId`/`itemId` no reverse request é uma dependência de desenho do actor,
não um detalhe que possa ser preenchido com valores sintéticos. O próximo
owner deve implementar a correlação dentro do resident actor e só então
conectar o canal público do App Server.

## Incremento implementado posteriormente

O ACP host agora transporta contexto de prompt para o comando product-wired e
publica `AcpPermissionRequest` em um broadcast separado, preservando:

- `sessionId`;
- `turnId`, quando o prompt foi iniciado pelo resident com contexto;
- `itemId` derivado deterministicamente como `tc_{toolCallId}`;
- `toolCallId` e payload ACP bruto.

O host continua respondendo `Cancelled` até existir o canal de decisão. Isso é
intencionalmente fail-closed: observar o request não equivale a anunciar
interaction capability.

## Incremento do canal de decisão

O `AcpHostHandle`/`AcpCommandHandle` agora expõe `respond_permission` por
`toolCallId`. O host registra um waiter one-shot, aguarda até 300 segundos e
converte `Selected(optionId)` ou `Cancelled` para o outcome ACP. Respostas
duplicadas ou IDs inexistentes falham sem efeito; ausência de decisão expira
em cancelamento.

Naquele ponto, esse canal ainda não estava conectado ao
`ShellSessionActorRuntime`/`respond_interaction`. Essa lacuna foi fechada no
incremento seguinte: o `ResidentHandle` ACP transporta o `AcpCommandHandle`
opcional e `respond_interaction` encaminha a decisão ao waiter ACP. A
capability continua `false` até o round-trip product-wired ser comprovado.

Validação adicional: `cargo test -p xai-grok-shell --lib
app_server_runtime::acp_host::tests` — 2 testes passaram; `cargo check -p
xai-grok-shell --lib` — passou; `git diff --check` — passou.

Validação: `cargo test -p xai-grok-shell --lib
app_server_runtime::acp_host::tests` — 2 testes passaram; `cargo check -p
xai-grok-shell --lib` — passou. O integration test amplo do Shell ainda tem
dívida preexistente de assinatura em `test_sampling_client.rs`.

## Integração no resident

O `ResidentHandle` agora carrega um `AcpCommandHandle` opcional para residentes
ACP. `respond_interaction` encaminha decisões para esse waiter quando o canal
está presente; fixtures locais continuam usando o delivery hub Shell. IDs de
opção `cancel`, `cancelled`, `deny` e `denied` mapeiam para cancelamento ACP;
outros valores são tratados como `optionId` selecionado.

Validação: `cargo check -p xai-grok-shell --tests` passou; o suite
`cargo test -p xai-grok-shell --test c6_respond_interaction` passou com 13
testes. Ainda falta provocar um reverse `request_permission` real no product
ACP host e comprovar o round-trip ACP → App Server → ACP; sem esse gate, a
capability permanece desabilitada.
