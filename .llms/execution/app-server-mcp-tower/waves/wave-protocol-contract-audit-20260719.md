# Wave — protocol contract audit (2026-07-19)

## Scope

Verify the canonical App Server protocol after clippy cleanup and audit the
declared Tower tool start overrides against the actual `SessionStartParams`
contract.

## Findings

- `SessionStartParams` supports `workspaceRoot`, `agentType`,
  `providerBinding`, and `idempotencyKey`.
- `model` and `sandboxMode` exist in `tower_agent_start_input` but have no
  corresponding protocol/facade fields or runtime propagation path.
- The gap is retained as an explicit cross-contract task; no silent discard is
  relabeled as implementation.

## Evidence

```text
cargo test -p xai-grok-app-server-protocol --no-fail-fast
22 tests passed
git diff --check passed
```

Protocol schema generation, JSON Schema compilation, Tower tool examples,
goldens, and round-trip wire-shape tests all passed.
