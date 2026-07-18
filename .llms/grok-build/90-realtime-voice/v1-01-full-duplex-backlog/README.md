# Epic v1-01 — Realtime voice full duplex (backlog)

Status: planejado
Prioridade: pós-lançamento
Estimativa: 3–4 semanas após research
Depende de: `../../30-app-server/v1-07-release-hardening/`
Habilita: voz agent full duplex
Skills relacionadas: `@architecture-spec-authoring`, `@implementation-loop`
Proveniência: [provenance: user-input, workspace, doc-tree, code, inferred]

## Escopo

### ADICIONAR
- futuro VAD/STT partial/Turn input/TTS stream/barge-in client.

### REFACTORIZAR
- evoluir `xai-grok-voice` sem confundir ditado com full duplex.

### REMOVER
- nada no core.

### MANTÉM
- Session stream/interrupt contracts e dictation baseline.

## Contratos

- [Session identity](../../_shared/session-turn-item-identity.md)
- [Tower lifecycle](../../_shared/tower-instance-lifecycle.md)

## TODO checklist

- [ ] (HUMAN) Escolher local/cloud STT/TTS e privacy — type: product-decision — blocking: design
- [ ] Caracterizar latency/capture pipeline atual
- [ ] Especificar audio/session state machine e barge-in
- [ ] Prototipar após stream/interrupt core stable
- [ ] Testar hardware loss, echo, cancellation e privacy
- [ ] Não adicionar código ao core wave

## Riscos e incertezas

- **[HIGH][Confirmed] áudio sensível:** explicit consent/retention policy.
- **[HIGH][Likely] latency/echo complexity:** dedicated program.

