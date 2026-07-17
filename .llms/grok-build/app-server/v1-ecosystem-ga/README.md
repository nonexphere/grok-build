# Epic v1-ecosystem-ga — Adapters, SDK e produção

Status: rascunho  
Prioridade: lançamento-bloqueante  
Depende de: `../v1-tui-migration/`, `../../goal-runtime/v1-clients-projections/`  
Habilita: App Server v1 GA  
Skills relacionadas: `@code-audit`, `@release-checklist`, `@code-review`, `@delivery-report`

## Arquitetura

Fecha ACP/Codex adapters, SDK/examples, Goal Item integration, remote reference
flow, observability, load/fuzz/security, compatibility policy e runbooks.

## Escopo

### ADICIONAR
- ACP shared-runtime adapter, separate Codex compatibility adapter;
- TypeScript SDK e Electron/VS Code/remote examples;
- Goal lifecycle/task/verifier projections e commands;
- GA operations/recovery/stability evidence.

### REFACTORIZAR
- ACP path usa shared registry/facade; não cria duplicate actor.

### REMOVER
- experimental flags somente após individually proven gates.

### MANTÉM
- ACP support, provider-neutral core e remote disabled default.

## Business rules

- Goal Item é projeção; app-server nunca completa/continua goal;
- adapters traduzem IDs/capabilities sem mudar core semantics;
- SDK generated surfaces não divergem do Rust protocol;
- GA exige threat findings closed/accepted e supported projection rebuild.

## Contratos

- [runtime ownership](../../_shared/runtime-ownership.md)
- [identity/event ordering](../../_shared/identity-event-ordering.md)
- [leases/idempotency](../../_shared/leases-idempotency.md)
- [security/authority](../../_shared/security-authority-boundaries.md)
- [Goal domain v1](../../goal-runtime/v1-characterization-domain/contracts/goal-domain-v1.md)
- [App Server protocol v1](../v1-architecture-protocol/contracts/protocol-v1.md)

## Tasks

- [tasks.md](./tasks.md)

## Gate de saída

- ACP/native/Codex-adapter/SDK conformance e drift checks passam.
- Multi-client reconnect durante Turn/approval/goal não perde nem duplica efeito.
- Todos os 15 itens da Definition of Done têm evidence e production sign-off.

## Riscos e incertezas

- **[HIGH][Confirmed] adapter vira segundo semantic core:** conformance against native processor.
- **[HIGH][Likely] Goal/App schema drift:** cross-group fixtures and contract tests.
- **Human decision required:** stable extensions, remote GA scope e security acceptance.
