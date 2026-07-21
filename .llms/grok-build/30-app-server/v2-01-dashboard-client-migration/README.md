# Epic v2-01 — Migração opcional do dashboard para App Server
Owner: App Server/protocol owners
Escopo: conforme a seção Escopo deste epic

Status: rascunho/backlog
Prioridade: pós-lançamento
Estimativa: 2–4 semanas
Depende de: `../v1-07-release-hardening/`
Habilita: futuro App Server-default TUI, se aprovado
Skills relacionadas: `@implementation-loop`, `@code-review`, `@delivery-report`
Proveniência: [provenance: user-input, workspace, doc-tree, code, inferred]

## Arquitetura

Somente após o MVP, avalia migrar a TUI em gates: shadow, adapter, native
tracker/controller/reconnect e possível default. No v1, dashboard permanece
ACP/leader/roster sem qualquer mutation. [provenance: user-input]

## Escopo

### ADICIONAR
- typed client integration, shadow comparator, controller/reconnect UX;
- native Session/Turn/Item tracker e parity/performance reports.

### REFACTORIZAR
- pager consome App Server sem perder rich Grok blocks.

### REMOVER
- ACP-default somente no gate final e após rollback proof.

### MANTÉM
- ACP fallback durante janela e TUI como richest supported client.

## Business rules

- mismatch shadow é finding, não silently normalized;
- nenhuma approval, delta, subagent, goal ou background state pode sumir;
- overhead shadow <3%; latency/render targets devem passar;
- feature flag reverte sem migration destrutiva.

## Contratos

- [identity/event ordering](../../_shared/identity-event-ordering.md)
- [runtime ownership](../../_shared/runtime-ownership.md)
- [security/authority](../../_shared/control-plane-security.md)

## Tasks

- [tasks.md](./tasks.md)

## Gate de saída

- Shadow report não contém mismatch material e overhead fica abaixo do target.
- Rich flows, approvals, reconnect, Goal/subagent/background e rollback passam.
- Parity humana e automatizada aprovam o switch de default.

## Riscos e incertezas

- **[HIGH][Confirmed] TUI regression:** staged shadow/parity; não bloquear v1 deixando este epic futuro.
- **[MEDIUM][Likely] duplicated rendering semantics:** native tracker só após comparison.
- **Human decision required:** parity acceptance thresholds e default switch.
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
