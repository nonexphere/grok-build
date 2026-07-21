# Epic v1-08 — App Server no supervisor Tower
Owner: App Server/protocol owners
Escopo: conforme a seção Escopo deste epic

Status: parcial — transportes/supervisor concluídos; runtime de turnos ainda pendente
Escopo: REFACTORIZAR + ADICIONAR
Depende de: `../../20-tower-core/v1-05-tower-supervisor/`
Contrato: [supervisor compartilhado](../../_shared/tower-command-runtime.md)

## Objetivo

Tornar o WebSocket App Server uma unidade controlável pelo `grok-oss tower`,
com bind independente, token compartilhado, rollback de startup e shutdown
coordenado.

## Tasks

> Gate de prontidão: os itens de transporte abaixo não provam um App Server
> product-ready. A composição ainda usa `ShellSessionActorRuntime` sem uma
> factory real de `SessionActor`/ACP; portanto turn/item/interaction continuam
> corretamente não anunciados. A conclusão deste epic exige a entrega de
> `TW106-02`/`TW106-03` e um smoke start→prompt→history real.

- [x] Expor factory de listener que aceite bind/secret já validados.
- [x] Retornar handle/join observável ao supervisor sem bloquear o runtime.
- [x] Preservar `initialize`, auth bearer e `session/start` no caminho real.
- [x] Testar app-only e combined com `--no-mcp`.
- [x] Testar falha de bind e limpeza de socket/tarefa.
- [x] Testar SIGINT/SIGTERM sem orphan listener.
- [x] Registrar evidência de handshake WebSocket real.
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
