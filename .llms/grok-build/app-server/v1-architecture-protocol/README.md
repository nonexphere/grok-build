# Epic v1-architecture-protocol — ADRs e contrato v1

Status: rascunho  
Prioridade: lançamento-bloqueante  
Depende de: nenhuma  
Habilita: `v1-runtime-facade-projection`  
Skills relacionadas: `@repository-exploration`, `@architecture-spec-authoring`, `@implementation-loop`

## Arquitetura

Trava ownership/identity/protocol/security ADRs e cria
`xai-grok-app-server-protocol` como fonte Rust única para schema, TypeScript,
fixtures e SDK skeleton. O bundle em `changes/` é seed, não source gerada.

## Escopo

### ADICIONAR
- envelopes JSON-RPC, IDs, Thread/Turn/Item, capabilities, errors/interactions;
- codegen/snapshots/examples/fuzz deserialization.

### REFACTORIZAR
- vocabulário leader/ACP é mapeado, não substituído ainda.

### REMOVER
- nenhuma runtime behavior; inconsistências no bundle são corrigidas na fonte.

### MANTÉM
- core próximo ao Codex; Grok-only sob `grok/*`.

## Business rules

- native wire exige `jsonrpc:"2.0"`; compat omission somente por decisão;
- request ID não é Interaction ID;
- additive v1, unknown fields/enums conforme explicit policy;
- generated artifacts devem ser reproducíveis e drift-checked.

## Contratos

- [protocol v1](./contracts/protocol-v1.md)
- [identity/event ordering](../../_shared/identity-event-ordering.md)
- [runtime ownership](../../_shared/runtime-ownership.md)
- [security/authority](../../_shared/security-authority-boundaries.md)

## Tasks

- [tasks.md](./tasks.md)

## Gate de saída

- ADRs públicos estão aceitos e vertical ownership spike valida one registry.
- Rust gera schema/TS/examples reproducivelmente sem drift.
- Serde/snapshot/fuzz e protocol security review passam.

## Riscos e incertezas

- **[HIGH][Confirmed] protocol freeze prematuro:** debt pública — core mínimo e experimental capabilities.
- **[HIGH][Likely] generated artifact drift:** CI regeneration check.
- **Human decision required:** strictness, UUID exposure e stable/experimental inventory.
