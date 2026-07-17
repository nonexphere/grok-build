# Epic v1-tui-migration — Cliente de referência e parity gate

Status: rascunho  
Prioridade: lançamento-bloqueante  
Depende de: `../v1-daemon-transports-security/`  
Habilita: `v1-ecosystem-ga`  
Skills relacionadas: `@implementation-loop`, `@code-review`, `@delivery-report`

## Arquitetura

Migra a TUI em quatro gates: shadow projection, adapter in-process, native
tracker/controller/reconnect, daemon default. `AcpUpdateTracker` é oracle até
parity comprovada; rollback permanece disponível.

## Escopo

### ADICIONAR
- typed client integration, shadow comparator, controller/reconnect UX;
- native Thread/Turn/Item tracker e parity/performance reports.

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
- [security/authority](../../_shared/security-authority-boundaries.md)

## Tasks

- [tasks.md](./tasks.md)

## Gate de saída

- Shadow report não contém mismatch material e overhead fica abaixo do target.
- Rich flows, approvals, reconnect, Goal/subagent/background e rollback passam.
- Parity humana e automatizada aprovam o switch de default.

## Riscos e incertezas

- **[HIGH][Confirmed] TUI regression é release blocker:** staged shadow/parity.
- **[MEDIUM][Likely] duplicated rendering semantics:** native tracker só após comparison.
- **Human decision required:** parity acceptance thresholds e default switch.
