# Epic v1-01 — Protocolo Session/Turn/Item

Status: rascunho
Prioridade: lançamento-bloqueante
Estimativa: 2–3 semanas
Depende de: `../../20-tower-core/v1-01-leader-characterization-promotion/`
Habilita: `../v1-02-runtime-facade-projection/`
Skills relacionadas: `@repository-exploration`, `@architecture-spec-authoring`, `@implementation-loop`
Proveniência: [provenance: user-input, workspace, doc-tree, code, inferred]

## Arquitetura

Trava ownership/identity/protocol/security ADRs e planeja
`xai-grok-app-server-protocol` como fonte Rust única para schema, TypeScript,
fixtures e SDK skeleton. O bundle em `changes/` é seed, não source gerada.

## Escopo

### ADICIONAR
- envelopes JSON-RPC, IDs, **Session**/Turn/Item, capabilities, errors/interactions;
- codegen/snapshots/examples/fuzz deserialization.

### REFACTORIZAR
- vocabulário leader/ACP é mapeado, não substituído ainda.

### REMOVER
- nenhuma runtime behavior; inconsistências no bundle são corrigidas na fonte.

### MANTÉM
- core inspirado no Codex; `thread/*` somente em adapter de compatibilidade.

## Business rules

- native wire exige `jsonrpc:"2.0"`; compat omission somente por decisão;
- request ID não é Interaction ID;
- additive v1, unknown fields/enums conforme explicit policy;
- generated artifacts devem ser reproducíveis e drift-checked.

## Contratos

- [Session protocol v1](./contracts/session-protocol-v1.md)
- [identity/event ordering](../../_shared/identity-event-ordering.md)
- [runtime ownership](../../_shared/runtime-ownership.md)
- [security/authority](../../_shared/control-plane-security.md)

## Tasks

- [tasks.md](./tasks.md)

## Gate de saída

- ADRs públicos estão aceitos e vertical ownership spike valida one registry.
- Rust gera schema/TS/examples reproducivelmente sem drift.
- Serde/snapshot/fuzz e protocol security review passam.

## Riscos e incertezas

- **[HIGH][Confirmed] protocol freeze prematuro:** debt pública — core mínimo e experimental capabilities.
- **[HIGH][Likely] generated artifact drift:** CI regeneration check.
- **[MEDIUM][Likely] mapping Codex:** adapter pode contaminar nomes nativos — contract test sem `thread`.
- **UNVERIFIED:** strictness de unknown fields até teste com clients seed.
