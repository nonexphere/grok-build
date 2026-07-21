# Epic v1-01 — Telegram bridge (backlog)
Owner: channel gateway owners
Escopo: conforme a seção Escopo deste epic

Status: rascunho/backlog
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
