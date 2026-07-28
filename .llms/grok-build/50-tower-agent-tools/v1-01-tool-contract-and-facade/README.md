# Epic v1-01 — Contrato e facade `tower_agent_*`

Status: rascunho
Prioridade: lançamento-bloqueante
Estimativa: 2–3 semanas
Depende de: `../../20-tower-core/v1-02-multi-session-workspace-registry/`, `../../30-app-server/v1-03-core-in-process-stdio/`
Habilita: MCP server e `v1-02`
Skills relacionadas: `@architecture-spec-authoring`, `@implementation-loop`, `@code-review`
Proveniência: [provenance: user-input, workspace, doc-tree, code, inferred]

## Arquitetura

Cria Rust tool facade e schemas a partir do contrato compartilhado, com
Session/Turn semantics, errors, pagination e idempotência únicos.

## Escopo

### ADICIONAR
- list/start/send/history/interrupt/resume/archive/status/wait typed operations.

### REFACTORIZAR
- roster/session handlers existentes como primitives atrás da facade.

### REMOVER
- conceito/tool `hub` separado.

### MANTÉM
- subagent tools atuais e runtime permissions.

## Contratos

- [Tower tools](../../_shared/tower-agent-tools.md)
- [runtime ownership](../../_shared/runtime-ownership.md)
- [control-plane security](../../_shared/control-plane-security.md)

## TODO checklist

- [ ] RED contract test para cada MUST operation
- [ ] Definir params/results/errors/idempotency/cursors
- [ ] Implementar facade sobre registry/App Server primitives
- [ ] History full/last + cursor/max bytes/redaction
- [ ] Wait subscription/timeout/cancel sem lock leak
- [ ] Start workspace/agent type e resume dormant
- [ ] Interrupt/archive truth/idempotency
- [ ] Schema snapshots e malformed/oversize tests
- [ ] Concurrency duplicate send/start/wait tests
- [ ] Docs/examples e drift gate

## Riscos e incertezas

- **[HIGH][Confirmed] duplicate effects:** idempotency + actor state gates.
- **[HIGH][Likely] history secret/size leak:** projection redaction + byte limits.
- **[MEDIUM][Possible] wait task leak:** cancellation-safe subscriptions.

