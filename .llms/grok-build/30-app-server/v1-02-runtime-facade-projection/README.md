# Epic v1-02 — Facade única e eventos normalizados

Status: rascunho
Prioridade: lançamento-bloqueante
Estimativa: 2–4 semanas
Depende de: `../v1-01-session-protocol/`, `../../20-tower-core/v1-02-multi-session-workspace-registry/`
Habilita: `../v1-03-core-in-process-stdio/`
Skills relacionadas: `@repository-exploration`, `@implementation-loop`, `@code-review`
Proveniência: [provenance: user-input, workspace, doc-tree, code, inferred]

## Arquitetura

Cria `GrokRuntime`/fake adapter e projector determinístico sobre
SessionActor/ACP/xAI events. Define allocator de IDs e source offsets sem
alterar a autoridade do runtime.

## Escopo

### ADICIONAR
- facade methods/events, fake runtime, shell adapter, ID allocator;
- normalization fixtures para todos os flows relevantes.

### REFACTORIZAR
- leader/ACP consumers passam a poder compartilhar a facade gradualmente.

### REMOVER
- nenhuma ACP/TUI path nesta fase.

### MANTÉM
- runtime behaviors, tools, provider e session files.

## Business rules

- uma loaded session corresponde a um runtime handle;
- projector é deterministic/pure sempre que possível;
- hidden reasoning/secrets nunca viram Item;
- source event duplicado não duplica Item/lifecycle.

## Contratos

- [runtime ownership](../../_shared/runtime-ownership.md)
- [identity/event ordering](../../_shared/identity-event-ordering.md)
- [security/authority](../../_shared/control-plane-security.md)

## Tasks

- [tasks.md](./tasks.md)

## Gate de saída

- Fake e shell adapter passam o mesmo facade contract suite.
- Golden fixtures cobrem todos os flows do tracker oracle com IDs estáveis.
- Projector não bloqueia actor nem expõe secrets/hidden reasoning.

## Riscos e incertezas

- **[HIGH][Confirmed] falta de stable source IDs:** rebuild muda IDs — deterministic allocator + golden fixtures.
- **[HIGH][Likely] event coverage incompleta:** transcript loss — enumerate tracker oracle flows.
- **UNVERIFIED:** pontos mínimos de instrumentação no SessionActor e hot paths Goal v1.
