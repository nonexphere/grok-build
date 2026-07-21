# Epic v2-03 — Runtime e continuation determinística v2
Owner: goal runtime owners
Escopo: conforme a seção Escopo deste epic

Status: rascunho/backlog
Prioridade: pós-lançamento core
Estimativa: 2–4 semanas
Depende de: `../v2-02-persistence-leases-accounting/`
Habilita: `../v2-04-tools-verification/`, `../v2-05-task-graph-subagents/`
Skills relacionadas: `@implementation-loop`, `@code-review`
Proveniência: [provenance: user-input, workspace, doc-tree, code, inferred]

## Arquitetura

Extrai `GoalService`/`GoalRuntime`, cria `GoalSessionPort` sobre `SessionActor`
e concentra continuation numa policy pura seguida por protocolo race-safe.

## Escopo

### ADICIONAR
- service registry/handles, callbacks de turn e continuation decisions;
- global resource governor e recovery entrypoint.

### REFACTORIZAR
- lógica distribuída em `acp_session_impl/goal.rs` migra para runtime.

### REMOVER
- starts sintéticos fora da policy/lease/intent gate após compat rollout.

### MANTÉM
- prompt queue, inference, cancellation e compaction no SessionActor.

## Business rules

- user command vence synthetic continuation;
- decisão pura não produz side effects;
- start revalida revision, lease, session idle e queued input;
- budgets/no-progress/run caps são hard gates;
- um runtime não inicia trabalho enquanto recovery estiver inconclusivo.

## Contratos

- [runtime ownership](../../_shared/runtime-ownership.md)
- [leases/idempotency](../../_shared/leases-idempotency.md)

## Tasks

- [tasks.md](./tasks.md)

## Gate de saída

- E2E determinístico continua um goal sob limits sem verifier.
- Pause/edit/clear/user-input races nunca iniciam Turn indevido.
- Ordinary sessions e compaction permanecem compatíveis e lazy.

## Riscos e incertezas

- **[HIGH][Confirmed] races com user input/cancel:** double turn — intent+CAS+final idle check.
- **[HIGH][Likely] blocking entre actor/runtime:** deadlock — portas unidirecionais e testes determinísticos.
- **UNVERIFIED:** hook exato para compaction usage e scheduler governor.
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
