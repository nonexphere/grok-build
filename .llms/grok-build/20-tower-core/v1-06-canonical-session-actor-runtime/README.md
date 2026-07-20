# Epic v1-06 — Runtime canônico do SessionActor

Status: rascunho
Prioridade: P0 lançamento-bloqueante
Estimativa: 2–4 semanas
Depende de: ../v1-05-tower-supervisor/, ../../30-app-server/v1-02-runtime-facade-projection/
Habilita: 20/v1-07, 30/v1-09, 50/v1-03
Skills relacionadas: @implementation-loop, @code-review, @human-product-test
Proveniência: [provenance: user-input, skill-output, code, doc-tree, inferred]

## Objetivo

Montar o único runtime de produção capaz de criar e operar o SessionActor real para App Server, MCP e Tower, eliminando o estado atual em que Session start cria storage mas Turn start retorna unsupported.

## Escopo

### ADICIONAR

- factory product-wired baseada em spawn_session_on_thread;
- dependency bundle tipado para auth, agent, tools, provider, persistence e workspace;
- readiness/degraded-state explícito;
- provider/gateway double fiel somente no boundary externo;
- vertical black-box do binário real.

### REFACTORIZAR

- xai-grok-pager-bin compartilha uma instância Shell-backed entre listeners;
- ShellSessionActorRuntime deixa de usar ProductionSpawner vazio no modo normal;
- startup/resume/turn usam o mesmo registry e actor handle.

### REMOVER

- caminho normal que confirma start e posterga wiring failure para send;
- qualquer seleção de experimental_local_turn_spawn/FakeRuntime no produto;
- diagnostics C1-J/C2-A como erro público permanente após completion.

### MANTÉM

- SessionActor e canonical session files como autoridade;
- FakeRuntime em conformance isolada;
- dashboard/ACP sem migração.

## Contratos

- [product runtime readiness](../../_shared/product-runtime-readiness.md)
- [runtime ownership](../../_shared/runtime-ownership.md)
- [runtime facade](../../_shared/runtime-facade.md)
- [provider contract](../../_shared/provider-contract.md)

## Gate de saída

O binário real executa initialize→start→send→wait→history→interrupt/archive com actor real, provider double fiel e sem caminho fake; readiness nunca anuncia execução indisponível.

## Riscos

- **[CRITICAL][Confirmed] wiring incompleto:** composition test e fail-fast.
- **[HIGH][Likely] segundo runtime acidental:** DAG/source assertions.
- **[HIGH][Likely] credential/sandbox drift:** dependency bundle typed e fixture fiel.
- **[MEDIUM][Possible] thread/LocalSet leak:** cancellation/load tests.

