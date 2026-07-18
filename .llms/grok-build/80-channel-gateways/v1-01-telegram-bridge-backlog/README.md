# Epic v1-01 — Telegram bridge (backlog)

Status: planejado
Prioridade: pós-lançamento
Estimativa: 2–4 semanas após design
Depende de: `../../30-app-server/v1-07-release-hardening/`, `../../60-sdk-typescript/v1-01-generated-sdk-client-examples/`
Habilita: primeiro channel gateway
Skills relacionadas: `@architecture-spec-authoring`, `@implementation-loop`
Proveniência: [provenance: user-input, workspace, doc-tree, code, inferred]

## Escopo

### ADICIONAR
- futuro bridge client, chat→Session mapping, stream, interrupt e bot secret handling.

### REFACTORIZAR
- nada no core.

### REMOVER
- nada.

### MANTÉM
- Tower/App Server como authority; sem plugin system core.

## Contratos

- [Session identity](../../_shared/session-turn-item-identity.md)
- [Tower lifecycle](../../_shared/tower-instance-lifecycle.md)

## TODO checklist

- [ ] (HUMAN) Escolher Bot API vs MTProto e hosting — type: product-decision — blocking: design
- [ ] Especificar identity/isolation/workspace/approval policy
- [ ] Prototipar como SDK client somente após core stable
- [ ] Testar multi-chat/reconnect/redaction/webhook auth
- [ ] Não adicionar código ao core wave

## Riscos e incertezas

- **[HIGH][Confirmed] external chat auth/privacy:** programa isolado.
- **[MEDIUM][Likely] approval UX:** decisão futura.

