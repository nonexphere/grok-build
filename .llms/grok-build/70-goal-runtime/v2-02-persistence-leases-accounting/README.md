# Epic v2-02 — Persistência, leases e accounting v2
Owner: goal runtime owners
Escopo: conforme a seção Escopo deste epic

Status: rascunho/backlog
Prioridade: pós-lançamento core
Estimativa: 2–4 semanas
Depende de: `../v2-01-domain-foundation/`
Habilita: `../v2-03-runtime-continuation/`
Skills relacionadas: `@architecture-spec-authoring`, `@implementation-loop`, `@code-review`
Proveniência: [provenance: user-input, workspace, doc-tree, code, inferred]

## Arquitetura

Adiciona SQLite/WAL, migrations, CAS, append-only ledgers, execution leases e
import legacy. O store é process-agnostic e não conhece UI/model loop.

## Escopo

### ADICIONAR
- materialized state, event/usage/evidence/verifier/subagent ledgers;
- lease manager, intents e recovery metadata;
- legacy importer e dual projection.

### REFACTORIZAR
- persistence snapshot vira projection; usage deixa mutable baseline.

### REMOVER
- escrita não-versionada como lifecycle truth após rollout.

### MANTÉM
- session JSONL/GoalUpdated legíveis e rollback feature flag.

## Business rules

- mutation e event commit na mesma transação;
- CAS por record revision; usage idempotente por provider-call ID;
- lease epoch faz fencing; ambiguous recovery não dirige;
- budget arithmetic é overflow-safe e custo incompleto é explícito.

## Contratos

- [leases/idempotency](../../_shared/leases-idempotency.md)
- [identity/event ordering](../../_shared/identity-event-ordering.md)

## Tasks

- [tasks.md](./tasks.md)

## Gate de saída

- Duas conexões/processos não criam a mesma continuação/effect intent.
- Crash em cada boundary converge sem perda, duplicação ou driving ambíguo.
- Legacy import, CAS, ledgers e budget accounting passam fixtures/race tests.

## Riscos e incertezas

- **[HIGH][Confirmed] duas fontes durante migração:** divergência — SQLite é truth e JSONL projection com comparison telemetry.
- **[HIGH][Likely] crash boundary incompleta:** duplicação de side effect — intent/resolution e kill tests.
- **UNVERIFIED:** disponibilidade/padrão SQLite já usado no workspace e cross-platform locking.
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
