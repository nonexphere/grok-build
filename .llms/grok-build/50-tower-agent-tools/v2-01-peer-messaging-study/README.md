# Epic v2-01 — Estudo de peer messaging interno

Status: planejado
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

