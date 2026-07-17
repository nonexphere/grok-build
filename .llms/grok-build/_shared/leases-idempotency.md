# Leases, CAS and Idempotency

**Fonte de verdade.** Goal Runtime possui execution leases de goals; App Server
possui controller leases de clientes. Ambos seguem este contrato de fencing.

## Tipos de lease

| Lease | Recurso | Permite | Não permite |
|---|---|---|---|
| goal execution | `GoalId` | iniciar continuação/subagent/verifier | controlar UI/clientes |
| thread controller | `ThreadId` | responder reverse requests | mutar lifecycle sem comando autorizado |
| store transaction | recurso persistido | commit atômico | chamadas externas fora do fencing |

## Regras

1. Cada aquisição produz `lease_epoch` monotônico.
2. Toda mutation valida owner, epoch, expiry e record revision na mesma
   transação do efeito.
3. Heartbeat não revive lease já substituído.
4. Takeover só ocorre após expiry/reconciliation ou comando autorizado.
5. Operação externa usa intent durável antes do efeito e resolution depois.
6. Repetição do mesmo intent é deduplicada; payload divergente é conflito.
7. Pausa/clear/interrupt revogam novos starts antes de cancelar trabalho ativo.

## Recovery

```text
load → reconcile intents/leases → classify external work → CAS state
     → only then permit new execution
```

Estado desconhecido, lease ambíguo ou infraestrutura indisponível restaura em
modo não-driving. Nunca se presume sucesso.

## Provas mínimas

- dois processos não iniciam a mesma continuação;
- controller desconectado não permite duas respostas válidas;
- crash em cada fronteira intent/effect/resolution converge sem duplicação;
- stale epoch e stale revision são rejeitados;
- cancellation concorrente tem precedência documentada.
