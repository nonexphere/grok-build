# Auditoria Exaustiva — grok-oss App Server, MCP Server e Tower

**Data:** 2026-07-19  
**Branch:** goblin-implement-epic-tree  
**Commit base:** d0ea385c9d0e7335da9f831a3031fca6e3f2bd67  
**Tipo:** auditoria read-only + testes black-box; nenhuma correção de código foi aplicada.

## 1. Veredito executivo

O control plane já possui uma superfície demonstrável: o binário sobe App Server WebSocket e MCP Streamable HTTP, autentica Bearer, registra nove tools, mantém sessões MCP e possui testes extensos de transporte, replay e segurança local.

Ele ainda não é um MCP funcional para orquestração real de agentes. A composição de produção usa ShellSessionActorRuntime::new(root) sem factory real para criar SessionActor; por isso start cria uma sessão storage-backed e send/interrupt falham com unsupported. O caminho de sucesso observado limita-se a registro, leitura, replay sintético e lifecycle administrativo.

**Conclusão:** não liberar como Tower/App Server operacional. O bloqueador principal é P0: wiring do actor real e dependências de produção. Há divergências P1 entre schema/documentação, adapter semântico e runtime observado.

| ID | Severity | Confidence | Resumo |
|---|---|---|---|
| F-01 | Critical | Confirmed | Sessões reais não executam turns: falta factory product-wired do actor. |
| F-02 | High | Confirmed | Adapter não preserva agent type, residency, paginação, filtros e campos do schema. |
| F-03 | High | Confirmed | tower_agent_wait retorna epoch hardcoded diferente do epoch real. |
| F-04 | High | Confirmed | Archive/resume produzem estados semanticamente inválidos. |
| F-05 | High | Confirmed | Erros reais não obedecem code/message/retryable. |
| F-06 | High | Likely | tools/list expõe $ref relativo sem schema resolvível para clientes genéricos. |
| F-07 | High | Confirmed | App Server, schema e SDK têm múltiplas fontes manuais de verdade. |
| F-08 | Medium | Confirmed | Suítes verdes cobrem principalmente FakeRuntime/adapters. |
| F-09 | Medium | Confirmed | TLS/remote threat model ainda é gate humano. |
| F-10 | Medium | Confirmed | Capabilities declaradas excedem a implementação product-wired. |

## 2. Escopo e limites

Auditados:

- xai-grok-tower e xai-grok-tower-tools;
- xai-grok-app-server-protocol, xai-grok-app-server, client e SDK;
- xai-grok-mcp-server, HTTP/stdio e composição do binário;
- ShellSessionActorRuntime e ShellRuntimeAdapter;
- schemas, planos .llms/grok-build, handoff e evidências existentes;
- MCP exposto nesta sessão mcp__grok_oss_tower__*;
- binário real target/debug/grok-oss via MCP HTTP local.

Não foram feitos: correções, commits, push, PR, migração, deploy, bind público, uso de credenciais reais ou comandos destrutivos. O worktree já estava dirty e foi preservado.

## 3. Mapa arquitetural atual

~~~text
MCP client / HTTP / stdio
          |
          v
xai-grok-mcp-server::dispatch_jsonrpc
          |
          v
xai-grok-tower-tools::invoke_tower_tool
          |
          v
xai-grok-tower::GrokRuntimeFacade
          |
          v
ShellSessionActorRuntime
  |- JSONL storage adapter: list/read/replay parcial
  |- SessionActor registry/residency
  |- ProductionSpawner sem real spawn no composition root
          |
          v
SessionActor / provider / tools / permissions / MCP client
              [não conectado no caminho product-wired]
~~~

O App Server WebSocket usa FacadeProcessor sobre o mesmo facade. A intenção de semantic core única é correta; adapters e fachada implementam apenas parte do contrato e não propagam todos os metadados do runtime.

| Camada | Responsabilidade | Estado observado |
|---|---|---|
| Protocol/schema | wire types, errors, state/event contracts | Existe e compila; parte é seed/manual. |
| Tower core | registry, lifecycle, replay, facade | Existe; FakeRuntime é mais completo que production runtime. |
| Tower tools | nove names, ACL, semantic dispatch | Existe, com defaults/omissões que violam schema. |
| App Server | JSON-RPC, sessions, turns, subscriptions, WS | Transport/protocol testados; turns incompletos. |
| MCP Server | initialize/list/call, HTTP/stdio, bearer, SSE | Transport funcional; semantics incompletas. |
| Composition root | injetar runtime real | Injeta storage facade sem factory real de actor. |
| SDK TS | cliente tipado/gerado | Cliente existe; types.ts é mirror hand-authored. |

## 4. Bateria de testes

### 4.1 MCP conectado diretamente nesta sessão

| Tool | Casos executados | Resultado |
|---|---|---|
| tower_agent_list | vazio, após sessões, listagem final | Transporte PASS; metadata incorreta. |
| tower_agent_start | ausente, válido, retry idempotente, conflito | PASS; agentType não persiste. |
| tower_agent_status | ausente, criada, pós-archive | PASS de chamada; residency incorreta. |
| tower_agent_history | ausente, full, last, criada | PASS; epoch real, cursor fixo. |
| tower_agent_wait | cursor zero, timeout mínimo | PASS; epoch epoch_1, snapshot cursor 0. |
| tower_agent_send | válido, modo inválido, turnId em new_turn | Validações PASS; turn real unsupported. |
| tower_agent_resume | ausente, sessão starting | Sucesso fora do estado dormant. |
| tower_agent_interrupt | alvo ausente e turn ausente | unsupported, não erro estável esperado. |
| tower_agent_archive | ausente, criada, status posterior | Sucesso; projection inconsistente. |

Também foi validado start sem workspaceRoot; a API rejeitou com invalid_params.

### 4.2 MCP HTTP do binário real

Comando:

~~~bash
GROK_OSS_HOME=/tmp/grok-oss-audit-home \
target/debug/grok-oss tower \
  --bind 127.0.0.1:24919 \
  --mcp-bind 127.0.0.1:24988 \
  --secret audit-secret
~~~

Resultados:

- /healthz: 200 sem auth;
- initialize sem Bearer: 401;
- initialize com Bearer: 200 e Mcp-Session-Id;
- initialize com ?bearer=: 200;
- tools/list: exatamente nove tools;
- tower_agent_list: resposta estruturada válida;
- tower_agent_start: cria sessão;
- status/history/wait/send/resume/archive reproduzem as divergências live;
- send falha unsupported e enumera dependências ausentes;
- Ctrl-C encerra o supervisor com código 0.

A sessão criada foi arquivada.

### 4.3 Testes Rust e TypeScript

Executados com sucesso:

~~~text
cargo test -p xai-grok-app-server-protocol -p xai-grok-tower -p xai-grok-tower-tools --no-fail-fast
  22 protocol + 39 tower/isolamento + 35 tower-tools = 96 pass

cargo test -p xai-grok-mcp-server --features streamable-http --test streamable_http --no-fail-fast
  38 pass

cargo check -p xai-grok-app-server -p xai-grok-tower -p xai-grok-tower-tools \
  -p xai-grok-mcp-server -p xai-grok-pager-bin \
  --features xai-grok-mcp-server/streamable-http,xai-grok-app-server/websocket \
  --no-default-features
  exit 0

packages/grok-oss-app-server: npm test -- --runInBand
  5 pass

cargo test -p xai-grok-shell --test c6_respond_interaction \
  --test r5_runtime_correctness --no-fail-fast
  13 + 4 = 17 pass

node packages/grok-oss-app-server/scripts/check-schema-drift.mjs
  pass
~~~

O build do xai-grok-shell foi demorado (9m43s), mas terminou com sucesso. O schema drift gate também passou após o build terminar.

Warnings: import não usado em xai-grok-mcp-server/src/transport/stdio.rs, imports não usados em multi-auth e funções dead em xai-grok-sampling-types.

## 5. Achados detalhados

### F-01 — actor real não está montado

- **Severity:** Critical; **Confidence:** Confirmed.
- **Evidence:** app_server_composition.rs:16-39; shell_session_actor_runtime.rs:146-202,286-336; resposta live de tower_agent_send.
- **Failure:** start cria sessão JSONL, mas produção não injeta RealSpawnFn. O primeiro turn não encontra actor residente.
- **Impact:** MCP não orquestra agentes, provider, tools ou lifecycle real.
- **Fix direction:** montar uma única factory baseada em spawn_session_on_thread com credentials/auth, AgentDefinition, ToolContext, GatewaySender, ModelsManager, persistência, MCP servers, WorkspaceOps, plugins, sampling, thread e LocalSet.
- **Acceptance:** start → send → wait/history produz turn/item reais; interrupt termina o turn; restart/resume preserva identidade.

### F-02 — adapter ignora/inventa campos

- **Severity:** High; **Confidence:** Confirmed.
- **Evidence:** xai-grok-tower-tools/src/lib.rs:179-216,287-303,397-415; tower-tools.schema.json.
- **Observed:** agentType=unknown, residency=resident; list ignora filtros, cursor/page size; start ignora model/provider/sandbox e aceita defaults para required.
- **Impact:** ACL, provider, pagination, dormant/archived lifecycle e sandbox não são confiáveis.
- **Fix direction:** input structs e schema validation centralizados; Session/facade deve fornecer metadados canônicos; campos não podem ser ignorados silenciosamente.

### F-03 — epoch de replay inconsistente

- **Severity:** High; **Confidence:** Confirmed.
- **Evidence:** xai-grok-tower-tools/src/lib.rs:323-357 fixa epoch_1; history usa epoch da sessão em 287-303; live mostrou epochs diferentes.
- **Impact:** clientes não validam continuidade; resync/epoch mismatch perde significado.
- **Fix direction:** retornar epoch canônico do ReplayPage; nunca usar literal. Testar history→wait, restart e rebind.

### F-04 — lifecycle incoerente

- **Severity:** High; **Confidence:** Confirmed.
- **Evidence:** live archive resultou status=archived,residency=resident; resume em starting retornou completed; adapter 397-415 hardcodes residency.
- **Impact:** cliente pode acreditar que archive deixou actor residente ou que resume montou actor.
- **Fix direction:** state machine formal status × residency × activeTurn, transições atômicas e erros de transição.

### F-05 — erros fora do contrato

- **Severity:** High; **Confidence:** Confirmed.
- **Evidence:** schema exige code/message/retryable; ToolError em xai-grok-tower-tools/src/lib.rs:149-161 só tem code/message; HTTP serializa em http_server.rs:877-884; live retornou unsupported sem retryable.
- **Impact:** cliente não sabe se repete, ressincroniza ou falha definitivamente.
- **Fix direction:** tipo comum de erro público, catálogo canônico, retryability, operationId opcional e safe details.

### F-06 — schema MCP possivelmente não resolvível

- **Severity:** High; **Confidence:** Likely.
- **Evidence:** http_server.rs:814-822 envia inputSchema com $ref relativo; resposta live contém apenas esse ref.
- **Impact:** cliente MCP genérico não valida required fields nem constrói tool call com segurança.
- **Fix direction:** schema inline, URI resolvível ou registry; testar cliente MCP independente.

### F-07 — fontes de verdade desalinhadas

- **Severity:** High; **Confidence:** Confirmed.
- **Evidence:** packages/grok-oss-app-server/src/types.ts:1 diz Interim hand-authored mirror; schema Tower separado; tower-tools constrói JSON manualmente; check-schema-drift é gate separado.
- **Impact:** Rust/schema/SDK podem divergir sem quebrar consumidores.
- **Fix direction:** schema Rust/protocol como source of truth; gerar TS e descriptors; drift check obrigatório no CI.

### F-08 — cobertura verde não prova produto

- **Severity:** Medium; **Confidence:** Confirmed.
- **Evidence:** 96 testes core e 38 HTTP; App Server/Tower tools usam FakeRuntime; composição real testa bind/shape mais que turn/provider.
- **Impact:** regressões de actor, provider, permissions, projection e lifecycle podem passar.
- **Fix direction:** vertical slice real com provider test double; marcar fake-only; adicionar black-box real.

### F-09 — segurança remota incompleta

- **Severity:** Medium; **Confidence:** Confirmed.
- **Evidence:** composition comments indicam TLS como HUMAN gate; startup real imprime TLS: not provided.
- **Impact:** cleartext remote pode vazar controle da Tower.
- **Fix direction:** decidir TLS ownership, bloquear remote cleartext no release e aceitar threat model.

### F-10 — capabilities excedem implementação

- **Severity:** Medium; **Confidence:** Confirmed.
- **Evidence:** comentários PARTIAL em shell_session_actor_runtime.rs e planos App Server/MCP.
- **Gaps:** interactions/approvals product-wired, TurnChanged do Shell, crash recovery, status fidelity, provider binding real, TLS, rate limits, metrics, generated SDK e operation durability.
- **Fix direction:** cada capability deve ter acceptance test ou ser removida do anúncio.

## 6. Matriz das nove tools

| Tool | Estado live | Falta |
|---|---|---|
| list | lista, ignora filtros e inventa metadata | query canônica, pagination, projection |
| start | storage session e idempotência básica | trust/profile/provider/sandbox/actor spawn |
| send | valida modo; execução unsupported | input, queue, provider, interaction, lifecycle |
| history | leitura parcial; cursor fixo | paging, bytes, redaction, epoch |
| resume | sucesso em starting | somente dormant e actor resurrection |
| wait | snapshot sintético; epoch errado; cursor 0 | wait real, timeout, terminal/interaction |
| interrupt | sem actor resulta unsupported | cancellation e race tests |
| archive | status muda, residency fica resident | drain/detach e projection coerente |
| status | leitura, unknown/resident fixos | metadata canônica e freshness |

## 7. App Server e MCP: gaps adicionais

- Native protocol usa session/*; mapping Codex usa thread/*. A fronteira deve ser única e testada.
- SDK TS usa OperationResult.accepted, enquanto Tower schema usa state=accepted|completed|rejected.
- Capabilities anunciam fork, approvals, questions e MCP elicitation; parte não está no composition root.
- Replay/reconnect têm cobertura FakeRuntime, mas falta prova em JSONL + actor real, restart e epoch rotation.
- Shell não produz todos os lifecycle events; TurnChanged, agrupamento de chunks e crash-mid-turn são parciais.
- Interactions dependem de hub/parked futures e precisam wiring ao actor canônico, TTL e disconnect.
- dispatch_jsonrpc clona arguments sem JSON Schema validation; semantic core faz parsing ad hoc e defaults silenciosos.
- Output schema não é enviado nem validado em runtime.
- HTTP aceita query bearer; política deve distinguir compatibilidade local de exposição remota.
- stdio tem adapter/teste, mas falta prova de launcher product-wired equivalente ao HTTP.
- SSE/replay tem testes de corrida/rebind/resumption, mas não evento real de turn/item.
- Scopes, token lifecycle e revocation descritos no handoff não aparecem completos no MCP live.

## 8. Código morto, placeholder ou desalinhado

1. experimental_local_turn_spawn: fixture/test seam; deve ser impossível no product path.
2. FakeRuntime: necessário para conformance, mas não prova readiness.
3. build_responses_input e easy_input_content_text: compiler reportou dead code.
4. Imports AuthProvider não usados em multi-auth.
5. Import process_mcp_stdio_batch não usado em stdio MCP.
6. Literais unknown, resident, epoch_1, tool-start e tool-send mascaram dados ausentes.
7. MCP serverInfo.version=0.0.0-experimental é placeholder sem política de versão.
8. Comentários C1-J PARTIAL/C2-A/HUMAN gate são honestos, mas não substituem readiness consumível.

## 9. Plano priorizado

### P0 — vertical slice real

1. Montar RealSpawnFn/factory product-owned usando spawn_session_on_thread, sem segundo SessionActor.
2. Injetar credentials/auth, AgentDefinition, ToolContext, GatewaySender, ModelsManager, persistência, MCP servers, WorkspaceOps, plugins, sampling, thread e LocalSet.
3. Criar provider/gateway fake controlado como teste de integração, sem credenciais reais.
4. Provar start → send → wait → history com user/agent items, statuses, event seq e terminal state.
5. Provar interrupt durante turn e corrida interrupt/complete.
6. Startup deve falhar ou declarar degraded se actor não estiver disponível.

### P1 — contrato e parity

1. Gerar descriptors/input/output a partir de schema canônico.
2. Validar JSON Schema antes do dispatch e rejeitar unknown/missing/oversized.
3. Remover hardcodes de agent type, residency, epoch e cursors.
4. Implementar filtros/page size/cursor com ordering estável.
5. Normalizar errors com retryability e operationId.
6. Formalizar state machine archive/resume/dormant/starting/ready/failed.
7. Uma suíte de fixtures deve rodar em facade, App Server, MCP HTTP, MCP stdio e SDK.
8. Publicar ou inline schema resolvível em tools/list.
9. Testar epoch, replay gap, resync, rebind, restart e cliente externo.

### P1/P2 — segurança/operação

1. Decidir TLS ownership e bloquear remote cleartext no release.
2. Scopes, token lifecycle, revocation e audit logs sem secrets.
3. Rate limits, backpressure e limites de starts/sends/SSE/body.
4. Metrics/tracing de sessions, actors, turns, spawn failures, replay gaps e auth failures.
5. Shutdown/drain com policy para novos work e turns ativos.
6. Gerar TS de Rust/schema e isolar/remover dead code.

## 10. Acceptance criteria de MCP funcional

- [ ] start preserva agentType, workspace, provider/sandbox e residency.
- [ ] send executa input real em actor real.
- [ ] wait retorna epoch canônico, eventos e cursor monotônico.
- [ ] history pagina por cursor/bytes e coincide com eventos.
- [ ] status reflete starting → ready/running → terminal/dormant/archived.
- [ ] interrupt só afeta o turn identificado e é idempotente.
- [ ] resume só aceita dormant e preserva identidade.
- [ ] archive preserva transcript e deixa status/residency coerentes.
- [ ] Nove tools validam input/output; nenhum campo é ignorado.
- [ ] Erros têm código público e retryability corretos.
- [ ] ACL ocorre antes de lookup e não vaza existência.
- [ ] HTTP, stdio, in-process, App Server e SDK são semanticamente equivalentes.
- [ ] Reconnect/resync funciona após cursor expirado, epoch rotation e restart.
- [ ] Nenhum secret aparece em logs, summaries, history, SSE ou errors.
- [ ] TLS/threat model está aceito ou remote bind está bloqueado.
- [ ] CI separa fake conformance de product integration.

## 11. Fontes primárias

- crates/codegen/xai-grok-tower/src/{lib,registry,lifecycle,fake}.rs
- crates/codegen/xai-grok-tower-tools/src/lib.rs
- crates/codegen/xai-grok-app-server-protocol/schemas/tower-tools.schema.json
- crates/codegen/xai-grok-app-server/src/{processor,transport}.rs
- crates/codegen/xai-grok-mcp-server/src/transport/{http_server,http,stdio}.rs
- crates/codegen/xai-grok-pager-bin/src/{app_server_composition,main}.rs
- crates/codegen/xai-grok-shell/src/app_server_runtime/shell_session_actor_runtime.rs
- packages/grok-oss-app-server/src/{types,client}.ts
- packages/grok-oss-app-server/scripts/check-schema-drift.mjs
- .llms/grok-build/_shared/{tower-agent-tools,runtime-ownership}.md
- .llms/grok-build/{20-tower-core,30-app-server,40-mcp-control-plane,50-tower-agent-tools}/**
- docs/architecture/APP_SERVER_MCP_TOWER_HANDOFF.md
- .agents/evidence/product-qa/2026-07-19-grok-oss-tower/REPORT.md

## 12. Estado final

**Audit status:** complete for the currently accessible code and MCP surface.  
**Product readiness:** blocked / not complete.  
**Primary dependency:** production actor composition, provider/auth context and TLS/remote threat decision.  
**Code changes:** none.  
**Documentation created:** this file.
