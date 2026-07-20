# Epic v1-03 — Semântica completa das nove tools

Status: rascunho
Prioridade: P0 lançamento-bloqueante
Estimativa: 2–4 semanas
Depende de: ../../20-tower-core/v1-07-lifecycle-metadata-recovery/, ../../30-app-server/v1-05-history-replay/
Habilita: 40/v1-04, 30/v1-07
Skills relacionadas: @implementation-loop, @code-review, @human-product-test
Proveniência: [provenance: user-input, skill-output, code, doc-tree]

## Objetivo

Substituir a implementação scaffold/ad hoc por nove operações integralmente conformes ao schema, lifecycle e runtime real.

## Escopo

### ADICIONAR

- parser typed e validation compartilhada;
- list filters/cursor, start overrides, structured send, history paging/bytes, wait real;
- error/retryability e output validation;
- product E2E de swarm.

### REFACTORIZAR

- invoke_tower_tool chama facade typed sem hardcodes/defaults;
- metadata, epoch, residency e sequence vêm da autoridade.

### REMOVER

- unknown/resident/epoch_1 e default idempotency keys;
- first-text-only input reduction;
- sucesso de resume/archive fora da state machine.

### MANTÉM

- nove nomes públicos;
- ACL orchestrator default e self-MCP-loop proibido.

## Contratos

- [Tower tools](../../_shared/tower-agent-tools.md)
- [conformance](../../_shared/contract-conformance-capability-truth.md)
- [lifecycle](../../_shared/tower-instance-lifecycle.md)

## Gate de saída

Cada tool passa happy, malformed, ACL, not-found, conflict, runtime unavailable, oversize, redaction e concurrency em in-process e runtime real.

