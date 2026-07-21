# Wave — capability truth with ACP factory (2026-07-19)

## Mudança

`ShellSessionActorRuntime` passou a carregar explicitamente se uma resident
factory foi injetada:

- `new`/`with_storage`: storage-only, sem capabilities de mutação;
- `with_spawner`/`with_production_spawn`: Turn mutation disponível;
- `interaction_respond`, `item_lifecycle` e `item_deltas`: sempre false até
  seus gates específicos.

Isso evita tanto o falso negativo da seam ACP quanto o falso positivo do
runtime storage-only.

## Gates

```text
cargo test -p xai-grok-shell shell_runtime_capabilities --lib
PASS: storage-only permanece fail-closed

cargo test -p xai-grok-pager-bin --bin goblin acp_composition_seam_builds_with_only_verified_turn_capabilities
PASS: seam ACP promove apenas Turn capabilities
```

## Estado

A promoção ainda é experimental: o construtor default do produto não injeta a
factory, e a capability matrix completa depende de Interaction/item lifecycle,
black-box cross-transport e actor canônico de produção.
