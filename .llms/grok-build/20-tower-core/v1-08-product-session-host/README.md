# Epic v1-08 — ProductSessionHost ACP boundary

Status: parcial — boundary ACP experimental comprovado; wiring product e actor canônico ainda pendentes
Escopo: ADICIONAR + REFACTORIZAR
Owner: `xai-grok-shell`
Consumidores: `xai-grok-pager-bin`, `xai-grok-tower`, `xai-grok-app-server`, `xai-grok-mcp-server`
Depende de: `v1-06-canonical-session-actor-runtime`

## Objetivo

Criar a fronteira única entre o agente ACP real do Shell (`MvpAgent`,
`AgentSideConnection`, `GatewayReceiver`) e a facade Tower, que é `Send + Sync`.
O ACP e o actor permanecem em um thread Tokio current-thread com `LocalSet`; a
facade recebe apenas handles e mensagens `Send`.

## Contrato obrigatório

1. `ProductSessionHost` possui ownership do thread/`LocalSet`, connection ACP,
   gateway de notificações, encerramento e join.
2. A API pública expõe somente comandos `Send`: start/new session, prompt,
   interject, cancel, shutdown e leitura de estado/erro.
3. Cada sessão tem um único command channel e um único sink de eventos; não há
   segundo actor, `FakeRuntime`, echo ou estado mutável paralelo.
4. Falhas de bootstrap, auth, modelo, ACP ou thread não criam resident token,
   capability ou readiness falso.
5. `SessionNotification` é convertido para a autoridade durável existente;
   notificações não podem ser apenas consumidas e descartadas.
6. Drop/shutdown encerra gateway, actor, LocalSet e thread com timeout observável.
7. A implementação não depende de credenciais humanas nos testes: o teste usa
   o mock de inferência existente, mas executa `MvpAgent`/ACP reais.

## Tasks

- [ ] PSH-01 Definir `ProductSessionDependencies` sem placeholders opcionais e
  documentar ownership de cada dependência.
- [ ] PSH-02 Extrair/reutilizar o bootstrap ACP existente sem duplicar
  `MvpAgent` ou o gateway do agent server.
- [~] PSH-03 Implementar thread current-thread + `LocalSet` e command bridge
  `Send` com lifecycle/join determinístico. O host e o join explícito já
  existem; o bridge experimental para resident actor passa o vertical, mas
  faltam concurrency e wiring de produto.
- [~] PSH-04 Implementar client ACP interno: permission fail-closed,
  notification sink durável e capacidades explícitas. Fail-closed e sink
  thread-safe existem; o consumidor `persist_notifications` grava JSONL e
  falha em caso de lag, e agora é lifecycle-owned pelo `AcpHostHandle`; ainda
  precisa ser conectado ao resident actor.
- [~] PSH-05 Adaptar comandos Tower para ACP/actor real e retornar erros
  normalizados, incluindo cancelamento e rollback de spawn. A factory
  experimental já roteia Prompt/Interject/Cancel e valida identidade; steer,
  interação e rollback/concurrency ainda não têm gate.
- [~] PSH-06 Adicionar teste RED/GREEN com mock de inferência:
  initialize → new session → prompt → notification/history → cancel/shutdown.
  A suíte agora prova initialize/session/prompt/notification/shutdown,
  cancelamento observável (`Cancelled`), updates duráveis e replay através da
  facade `ShellSessionActorRuntime`.
- [~] PSH-07 Integrar a factory no composition root e só então habilitar
  capabilities de turns/items/interactions. O composition root de produção
  agora injeta a factory ACP e promove somente `turn/start`, `turn/steer` e
  `turn/interrupt`; o construtor explícito de root permanece storage-only para
  testes herméticos. Interaction e item lifecycle continuam fail-closed.
- [ ] PSH-08 Adicionar soak/concurrency/cleanup e gate black-box dos três
  binários; atualizar a matriz de readiness.

## Não é evidência suficiente

As suítes FakeRuntime, `experimental_local_turn_spawn`, `tools/list`,
`initialize`, listagem JSONL ou um handshake ACP isolado não satisfazem este
epic. A prova precisa atravessar o agente real, o mock de inferência, a
persistência e a facade product composition.
## Revisão de implementação

Este epic só pode ser executado quando cada task tiver owner, arquivos ou
contrato afetado, pré-condição, comando de validação e evidência esperada.
Alterações de comportamento exigem Red-Green-Refactor; alterações de contrato
exigem contract test e atualização da matriz de rastreabilidade.

### Gate mínimo

- [ ] dependências e links deste epic foram verificados;
- [ ] interfaces, schemas, estados, erros e compatibilidade estão definidos;
- [ ] caminho fake/conformance está separado do caminho product-backed;
- [ ] testes unitários, integração, black-box e segurança foram classificados;
- [ ] timeout, cancelamento, retry, restart e falhas parciais foram tratados;
- [ ] observabilidade, limites de recurso e redaction foram especificados;
- [ ] comando reproduzível e artefato de evidência foram registrados;
- [ ] bloqueios humanos/externos possuem owner e condição de desbloqueio;
- [ ] status do epic foi reconciliado com `TRACEABILITY.md` e `COMPLETION_COVERAGE.md`.
