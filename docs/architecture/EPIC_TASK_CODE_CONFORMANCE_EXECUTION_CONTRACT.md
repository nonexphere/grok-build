# Contrato de Conformidade Código–Epic–Task e Execução Completa

**Status:** ativo e obrigatório  
**Data:** 2026-07-21  
**Owner:** execução do programa App Server + MCP + Tower + Build/Release  
**Fonte de planejamento:** `.llms/grok-build/`  
**Fonte de código:** worktree atual e contratos Rust/JSON/TypeScript versionados  

## 1. Objetivo

Este documento define o contrato operacional para analisar e executar todos os
epics e tasks da árvore `.llms/grok-build`.

Nenhuma task pode ser executada apenas porque está escrita no plano. Antes da
implementação, o agente deve:

1. localizar o requisito e a task na árvore;
2. ler contratos, consumers, código e testes relevantes;
3. confirmar se a task se encaixa na arquitetura atual;
4. identificar conflito, duplicação, código ausente ou decisão humana;
5. definir o menor conjunto de arquivos e interfaces afetados;
6. escrever ou selecionar o teste correto;
7. executar a implementação;
8. validar o comportamento e os contratos downstream;
9. registrar evidência, status e limitações;
10. continuar para o próximo item desbloqueado.

O objetivo não é transformar todos os checkboxes em `[x]`. O objetivo é provar
que cada requisito está implementado, explicitamente bloqueado, deferred ou
superseded, sem status artificial.

## 2. Cobertura obrigatória

O contrato cobre todos os programas e epics atualmente presentes em:

| Programa | Área | Obrigatório nesta execução |
|---:|---|---|
| 05 | Build, release, cache, CI e dead code | sim |
| 10 | Providers e autenticação | sim quando consumido pelo runtime core; demais itens ficam explicitamente classificados |
| 20 | Tower core, actor, lifecycle, recovery e observabilidade | sim |
| 30 | App Server, protocol, transports e release | sim |
| 40 | MCP, transports, segurança, scopes e TLS | sim |
| 50 | Tower Agent Tools e ACL | sim |
| 60 | SDK TypeScript e geração | sim |
| 70 | Goal Runtime | análise obrigatória; execução somente quando não invadir a critical path core |
| 80 | Telegram | análise obrigatória; backlog sem execução até dependências serem liberadas |
| 90 | Realtime Voice | análise obrigatória; backlog sem execução até dependências serem liberadas |

Snapshot de inventário: 57 epics e 95 arquivos README/tasks foram encontrados
na árvore no início desta execução. O inventário deve ser regenerado antes do
gate final para capturar epics adicionados durante o trabalho.

## 3. Status normativo por item

Cada epic e task deve possuir exatamente um status:

| Status | Significado |
|---|---|
| `pending` | conhecido, ainda não analisado ou executado |
| `analyzing` | aderência ao código e contratos em investigação |
| `fit` | encaixa na arquitetura e possui caminho de implementação definido |
| `partial` | existe implementação, mas faltam comportamento ou evidência |
| `in_progress` | implementação da unidade está em andamento |
| `done` | implementação e aceite comprovados |
| `blocked` | impossível avançar sem input/credencial/estado externo específico |
| `deferred` | explicitamente fora da wave atual, com motivo e condição |
| `superseded` | substituído por outro epic/task com link de rastreabilidade |
| `mismatch` | task contradiz código/contrato; requer correção do plano antes da implementação |

`done` exige implementação real, teste proporcional e revisão do diff. Build
verde, código compilável, schema existente, handshake, teste fake ou teste
pulável não são suficientes isoladamente.

## 4. Registro obrigatório por epic

Cada epic deve manter ou apontar para um registro contendo:

```text
Epic ID
Path do epic
Programa
Owner
Consumers
Dependências upstream
Tasks contidas
Contratos referenciados
Código candidato
Testes existentes
Comandos previstos
Status do epic
Última análise
Última execução
Evidência
Riscos
Bloqueios
Decisões humanas
```

O README do epic é a especificação. `tasks.md` é a unidade executável quando
existir. `TRACEABILITY.md` liga requisito → contrato → epic → task → teste. A
`COMPLETION_COVERAGE.md` é o gate global de completude.

## 5. Registro obrigatório por task

Antes de executar uma task, adicionar ou confirmar os seguintes campos:

### Identidade e escopo

- ID estável e único;
- descrição comportamental, não apenas nome de arquivo;
- owner técnico;
- prioridade;
- dependências;
- arquivos/packages prováveis;
- consumidores upstream/downstream;
- escopo incluído e excluído.

### Análise de aderência ao código

Registrar um dos veredictos:

| Veredicto | Uso |
|---|---|
| `FIT` | a task corresponde à arquitetura e há ponto de extensão claro |
| `PARTIAL-FIT` | parte existe, parte requer refactor ou contrato adicional |
| `MISMATCH` | task contradiz código, contrato ou ownership atual |
| `MISSING-PRIMITIVE` | falta tipo, factory, persistence, listener ou capability base |
| `EXTERNAL-BLOCKED` | depende de credencial, API, infraestrutura ou decisão humana |
| `OBSOLETE` | comportamento já foi substituído e o plano deve ser atualizado |

A análise deve citar símbolos, módulos, schemas, testes e comandos que
justificam o veredicto. Nenhuma implementação começa em `MISMATCH`,
`MISSING-PRIMITIVE` ou `EXTERNAL-BLOCKED` sem registrar a correção necessária.

### Implementação e validação

Cada task deve declarar:

- teste RED ou reprodução do defeito;
- contrato/interface que será alterado;
- implementação mínima;
- teste GREEN;
- validação ampla proporcional;
- revisão de segurança;
- impacto em generated files/migrations;
- evidência esperada;
- critério objetivo de aceite.

## 6. Fluxo de execução por item

### Fase A — Descoberta

Ler, nesta ordem:

1. `AGENTS.md` global e local;
2. epic README e `tasks.md`;
3. contratos `_shared/`;
4. manifests e features;
5. entrypoints e composition roots;
6. consumers e testes;
7. histórico/relatórios apenas como evidência secundária.

Comandos mínimos:

```bash
git status --short
rg -n "<task-id>|<symbol>|<method>|<schema>" .llms crates packages
cargo metadata --no-deps --format-version 1
cargo tree -p <package> --depth 1
```

### Fase B — Aderência

Responder explicitamente:

- o requisito já existe parcialmente?
- qual módulo possui a autoridade?
- o epic aponta para o owner correto?
- há segundo runtime, fake ou estado duplicado?
- o contrato público está definido?
- o teste atual prova produto ou somente fake?
- a task conflita com outra task/epic?
- existe impacto cross-repository?
- há decisão humana pendente?

### Fase C — Teste

Para comportamento novo ou bug:

1. escrever o menor teste real;
2. confirmar RED pelo motivo esperado;
3. implementar o mínimo;
4. confirmar GREEN;
5. executar teste de contrato e integração;
6. executar gate de produto se a task alterar capability real.

Para documentação/configuração, executar validação estrutural, links, parser,
`git diff --check` e comandos específicos do artefato.

### Fase D — Implementação

Aplicar somente a unidade selecionada. Não misturar:

- refactor amplo com feature;
- otimização com mudança de contrato sem baseline;
- remoção de código sem prova de obsolescência;
- fake em caminho product-facing;
- mudança pública sem consumer/migration analysis.

### Fase E — Revisão

Revisar o diff contra:

- contrato do epic;
- interfaces públicas;
- schemas e generated files;
- erros, retry, timeout e cancelamento;
- segurança e redaction;
- concorrência e persistência;
- consumers downstream;
- worktree alheio.

### Fase F — Checkpoint

Registrar:

- comandos executados;
- resultado completo;
- testes aprovados/falhos;
- arquivos alterados;
- evidência;
- limitações;
- próximo item desbloqueado.

## 7. Waves de execução

### Wave 0 — Inventário e contrato

- reconciliar os 57 epics;
- identificar tasks sem ID, owner, contrato, teste ou gate;
- classificar duplicações e supersession;
- atualizar a matriz global;
- não marcar comportamento como concluído.

### Wave 1 — Build baseline

- medir check/test/build debug/release;
- registrar cargo timings, link, memória e artefato;
- separar falha de código, timeout e ambiente;
- não alterar profile antes da medição.

### Wave 2 — Runtime Tower

- actor product-backed;
- composition root única;
- lifecycle, persistence, identity e recovery;
- readiness e capability truth;
- vertical `grok-oss`.

### Wave 3 — App Server

- contrato de métodos;
- capability registry;
- erro/retryability/operationId;
- parity in-process/stdio/WebSocket;
- replay, interaction, cancel e reconnect.

### Wave 4 — MCP e Tools

- parity stdio/HTTP;
- schemas dos nove tools;
- multi-session;
- provider-backed calls;
- independent `rmcp` clients;
- ACL e lifecycle.

### Wave 5 — Segurança, SDK e operação

- scopes, tokens, revocation e TLS;
- SDK clean regeneration;
- observabilidade, limits, fault injection e secret canary;
- load e recovery.

### Wave 6 — Otimização e release

- dependency slicing;
- features;
- profiles;
- linker/cache/CI;
- dead code terminal;
- release matrix e human smoke.

## 8. Contratos de comportamento obrigatórios

### Runtime

- uma única autoridade para criar/resumir/archive/fork/restart;
- nenhum actor paralelo criado por Tower, MCP ou App Server;
- failure de bootstrap não publica readiness;
- turn possui identity, binding e terminal state únicos;
- cursor/epoch preservam ordenação e resync.

### App Server

- capability só é `true` se o caminho executável passar product gate;
- erros têm catálogo, retryability e operation identity estáveis;
- `operationId` ausente é explicitamente `null` no wire;
- transportes compartilham projeção canônica.

### MCP

- `tools/list` publica schemas resolvíveis;
- `tools/call` valida antes do efeito;
- stdio e HTTP não divergem em resultado, erro ou identity;
- stdout stdio contém apenas JSON-RPC;
- auth/scopes/TLS são fail-closed no modo seguro.

### Build/release

- `--locked` e cache correto;
- artefatos identificados por SHA/toolchain/target/profile/features;
- fake-conformance não substitui product-integration;
- otimização só é aceita com benchmark antes/depois;
- release não depende de download implícito não verificável.

## 9. Gate global de conclusão

O programa só pode ser declarado completo quando:

- toda task está `done`, `blocked`, `deferred`, `superseded` ou `obsolete`;
- nenhum item permanece `pending`, `analyzing`, `partial` ou `mismatch` sem plano;
- cada requisito possui owner, contrato, task, teste, comando e evidência;
- os epics e trackers não se contradizem;
- o caminho product-backed passa a vertical completa;
- App Server, MCP e Tower têm parity comprovada;
- security, observability, build e release gates estão evidenciados;
- worktree não contém alterações acidentais, secrets ou arquivos gerados
  indevidos;
- bloqueios externos têm owner, evidência e condição objetiva de desbloqueio.

## 10. Estado inicial da execução

| Item | Estado inicial | Evidência |
|---|---|---|
| Árvore de epics | `pending` para auditoria operacional | `.llms/grok-build/` |
| Documentação estrutural | `partial` | README/tasks revisados, mas sem ledger por task |
| App Server/MCP/Tower core | `partial` | gates leves verdes; actor product-backed vertical ainda aberta |
| Product Shell/Pager | `partial` | composição experimental e build longo/instável |
| Build optimization | `pending` | perfis existem; baseline comparável ainda necessário |
| Release readiness | `blocked` parcial | depende de actor real, segurança remota e gates humanos |

Este documento deve ser atualizado a cada wave. Qualquer mudança de escopo,
owner, contrato ou status precisa conter motivo e evidência.
