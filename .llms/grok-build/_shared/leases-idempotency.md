# Leases, CAS and Idempotency

**Fonte de verdade.** App Server possui controller leases; futuro Goal Runtime
v2 possui execution leases. Tower e tools respeitam fencing, sem duplicar o
mechanism.

| Lease | Recurso | Uso |
|---|---|---|
| session controller | `SessionId` | responder approval/input reverse request |
| goal execution v2 | `GoalId` | iniciar continuation/verifier/subtask |
| instance startup | `TowerInstanceId` | impedir dois owners do mesmo endpoint/state |

Cada aquisição produz epoch monotônico. Mutation valida owner, epoch, expiry e
record revision na mesma transação do efeito. Heartbeat não revive lease
substituído; disconnect não implica aprovação; recovery ambígua é non-driving.

Operation externa usa intent durável antes do efeito e resolution depois.
Idempotency key nunca é confundida com JSON-RPC request id, MCP request id ou
connection id.
