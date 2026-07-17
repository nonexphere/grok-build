# Epic v1-tools-verification — Intents e completion audit

Status: rascunho
Prioridade: lançamento-bloqueante
Depende de: `../v1-runtime-continuation/`
Habilita: `v1-task-graph-subagents`, `v1-clients-projections`
Skills relacionadas: `@implementation-loop`, `@session-evidence-gate`, `@code-review`

## Arquitetura

Entrega tools v2, prompt protocol e registry extensível de verifiers. Completion
request inicia audit; somente report conclusivo atual autoriza `Complete`.

## Escopo

### ADICIONAR
- `get_goal`, action-based `update_goal`, schemas e acknowledgements;
- verifier registry/composites/evidence providers e completion/block audit.

### REFACTORIZAR
- skeptic panel vira verifier plugin fail-closed.

### REMOVER
- model-driven terminal mutation e fail-open infra behavior.

### MANTÉM
- legacy tool schema via telemetry-backed compatibility adapter.

## Business rules

- cada required requirement/deliverable precisa evidence conclusiva atual;
- verifier é read-only e stale report não completa;
- infrastructure error causa `InfraPaused`;
- MCP/skill output é advisory salvo configuração authoritative explícita;
- blocker exige mesma condição repetida e fingerprint runtime-owned.

## Contratos

- [security/authority](../../_shared/security-authority-boundaries.md)
- [identity/event ordering](../../_shared/identity-event-ordering.md)

## Tasks

- [tasks.md](./tasks.md)

## Gate de saída

- False completion é rejeitada em unit, integration e adversarial suites.
- True completion exige requirement matrix e report conclusivo current-revision.
- Infra failure pausa; tool/model nunca administra lifecycle.

## Riscos e incertezas

- **[HIGH][Confirmed] false completion:** dano central — deterministic verifiers + audit matrix + fail-closed.
- **[MEDIUM][Likely] verifier caro/falso negativo:** loops — bounded retries e actionable gaps.
- **Human decision required:** trust de MCP e verificação visual defaults.
