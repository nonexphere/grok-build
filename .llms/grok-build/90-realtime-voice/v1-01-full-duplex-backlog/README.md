# Epic v1-01 — Realtime voice full duplex (backlog)
Owner: voice/runtime owners
Escopo: conforme a seção Escopo deste epic

Status: rascunho/backlog
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
