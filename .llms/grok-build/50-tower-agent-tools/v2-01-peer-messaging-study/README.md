# Epic v2-01 — Estudo de peer messaging interno
Owner: Tower tools owners
Escopo: conforme a seção Escopo deste epic

Status: rascunho/backlog
Prioridade: pós-lançamento
Estimativa: 1–2 semanas (spike/spec, sem implementação v1)
Depende de: `../v1-02-in-process-acl-mcp-parity/`
Habilita: futuro agent↔agent direto
Skills relacionadas: `@architecture-spec-authoring`, `@repository-exploration`
Proveniência: [provenance: user-input, workspace, doc-tree, code, inferred]

## Arquitetura

Analisa mailbox/delivery/identity/ACL para peers top-level sem transformar
subagents depth=1 ou duplicar Tower. Este epic não bloqueia o MVP.

## Escopo

### ADICIONAR
- ADR, threat model, delivery semantics e proposta de contratos.

### REFACTORIZAR
- nada em v1.

### REMOVER
- nada.

### MANTÉM
- toda comunicação v1 passa pela Tower facade.

## Contratos

- [Tower tools](../../_shared/tower-agent-tools.md)
- [runtime ownership](../../_shared/runtime-ownership.md)
- [control-plane security](../../_shared/control-plane-security.md)

## TODO checklist

- [ ] Caracterizar Codex bus como inspiração, não nomenclatura copiada
- [ ] Comparar mailbox vs send-to-Session Turn
- [ ] Definir delivery/ack/order/replay/retention candidates
- [ ] Reusar ACL e instance identity
- [ ] Analisar deadlock/cycle/spam/impersonation
- [ ] Produzir recommendation e rejected alternatives
- [ ] (HUMAN) Aprovar produto peer messaging — type: product-decision — blocking: qualquer implementação v2

## Riscos e incertezas

- **[HIGH][Likely] novo distributed messaging system:** manter spike/spec isolado.
- **[MEDIUM][Possible] confusão com send:** exigir use cases distintos.
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
