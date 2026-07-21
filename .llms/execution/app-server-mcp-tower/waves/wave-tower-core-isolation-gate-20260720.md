# Wave — Tower core isolation/lifecycle gate (2026-07-20)

## Objetivo

Revalidar os invariantes fundamentais do Tower antes de depender dele para o
actor product-backed.

## Evidência

```text
cargo test -p xai-grok-tower --all-targets
29 unit tests passed
10 integration tests passed
```

O gate cobre registry one-actor, concorrência get-or-insert, isolamento entre
instâncias, locks/flock, drain/restart/epoch, leases, budgets, lifecycle
telemetry, projeção/redaction e symlink escape.

## Limite

O Tower core está verde, mas esses testes não montam o `SessionActor` real nem
validam provider/gateway. O gap TW106 de composição product-backed permanece.
