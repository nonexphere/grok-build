# Epic v1-07 — Release hardening do App Server

Status: rascunho
Prioridade: lançamento-bloqueante
Estimativa: 2–4 semanas
Depende de: `../v1-04-websocket-remote-auth/`, `../v1-05-history-replay/`, `../v1-06-approvals-control/`, `../../40-mcp-control-plane/v1-02-remote-security-conformance/`, `../../50-tower-agent-tools/v1-02-in-process-acl-mcp-parity/`, `../../60-sdk-typescript/v1-01-generated-sdk-client-examples/`
Habilita: App Server v1 GA
Skills relacionadas: `@code-audit`, `@release-checklist`, `@code-review`, `@delivery-report`
Proveniência: [provenance: user-input, workspace, doc-tree, code, inferred]

## Arquitetura

Fecha Codex mapping adapter, conformance cross-program, observability,
load/fuzz/security, compatibility policy e runbooks. SDK e MCP são dependências,
não subtasks duplicadas. Dashboard/ACP permanece como está.

## Escopo

### ADICIONAR
- separate Codex Thread↔Session compatibility adapter;
- cross-program release/conformance evidence;
- inventário documentado de hot paths Goal v1 para dual-version futuro;
- GA operations/recovery/stability evidence.

### REFACTORIZAR
- nenhuma migração ACP/dashboard no v1; apenas regressão/compatibility tests.

### REMOVER
- experimental flags somente após individually proven gates.

### MANTÉM
- ACP/dashboard support atual e remote contract permissivo explícito.

## Business rules

- Goal Item é projeção; app-server nunca completa/continua goal;
- adapters traduzem IDs/capabilities sem mudar core semantics;
- SDK generated surfaces não divergem do Rust protocol;
- GA exige threat findings closed/accepted e supported projection rebuild.

## Contratos

- [runtime ownership](../../_shared/runtime-ownership.md)
- [identity/event ordering](../../_shared/identity-event-ordering.md)
- [leases/idempotency](../../_shared/leases-idempotency.md)
- [security/authority](../../_shared/control-plane-security.md)
- [App Server protocol v1](../v1-01-session-protocol/contracts/session-protocol-v1.md)

## Tasks

- [tasks.md](./tasks.md)

## Gate de saída

- Native/Codex-adapter/SDK/MCP conformance e drift checks passam.
- Multi-client reconnect durante Turn/approval/goal não perde nem duplica efeito.
- Todos os 15 itens da Definition of Done têm evidence e production sign-off.

## Riscos e incertezas

- **[HIGH][Confirmed] adapter vira segundo semantic core:** conformance against native processor.
- **[MEDIUM][Likely] Goal hot-path coupling:** inventory e versioned port, sem Goal v2 dependency.
- **Human decision required:** stable extensions e remote threat acceptance.
