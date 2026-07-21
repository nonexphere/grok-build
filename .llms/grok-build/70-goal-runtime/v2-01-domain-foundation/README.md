# Epic v2-01 — Domínio puro e state machine v2
Owner: goal runtime owners
Escopo: conforme a seção Escopo deste epic

Status: rascunho/backlog
Prioridade: pós-lançamento core
Estimativa: 2–4 semanas
Depende de: `../v1-01-legacy-characterization/`
Habilita: `../v2-02-persistence-leases-accounting/`
Skills relacionadas: `@repository-exploration`, `@architecture-spec-authoring`, `@implementation-loop`
Proveniência: [provenance: user-input, workspace, doc-tree, code, inferred]

## Arquitetura

Usa o baseline v1 já caracterizado para criar IDs, records, status/phase,
commands e transition engine **v2** puros, sem substituir o v1.

## Escopo

### ADICIONAR
- domain types v2, transition table, revisions e property tests;
- mappings v1→v2 explícitos e decision log.

### REFACTORIZAR
- criar implementation v2 atrás da port; não refatorar v1 in-place.

### REMOVER
- nenhuma behavior v1 ou default flag.

### MANTÉM
- implementação v1 intacta e selecionável.

## Business rules

- lifecycle status e execution phase são campos distintos;
- model-origin nunca pausa, resume, edita, limpa ou completa diretamente;
- estado desconhecido restaura não-driving;
- edit incrementa revision e invalida resultados stale.

## Contratos

- [goal domain v2](./contracts/goal-domain-v2.md)
- [runtime ownership](../../_shared/runtime-ownership.md)
- [identity/event ordering](../../_shared/identity-event-ordering.md)
- [security/authority](../../_shared/control-plane-security.md)

## Plano de execução

1. Mapear entrypoints/state/tests atuais.
2. Aprovar ADRs e matrizes de compatibilidade.
3. Escrever characterization/golden tests.
4. Extrair domínio puro atrás de adapter.
5. Executar property/serde/transition tests.

## Tasks

- [tasks.md](./tasks.md)

## Gate de saída

- Characterization cobre todos os entrypoints atuais e está verde.
- Transition properties provam authority, completion e stale-result invariants.
- ADRs de storage/restart estão aceitos e domínio v2 não exige I/O.

## Riscos e incertezas

- **[HIGH][Confirmed] refactor de comportamento maduro:** regressões silenciosas — mitigar com characterization antes de mudança.
- **[HIGH][Likely] status legacy ambíguo:** import pode dirigir execução indevida — unknown/ambiguous restaura paused.
- **UNVERIFIED:** comandos exatos e feature flags de teste do baseline.
- **Human decision required:** localização do SQLite e política de auto-resume interativo. [provenance: doc-tree]
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
