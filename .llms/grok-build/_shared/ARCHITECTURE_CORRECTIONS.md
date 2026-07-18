# Architecture correction record — 2026-07-18

[provenance: post-deepening architecture review and user-approved correction cycle]

This record closes the blocking findings from the second architecture review.

| Finding | Resolution | Evidence |
|---|---|---|
| Tower/Shell ownership ambiguity | `xai-grok-pager-bin` is composition root; Shell implements and is injected into Tower; Tower never imports Shell | `crate-map.md`, `runtime-facade.md` |
| opaque provider binding | structured identifier-only `ProviderBinding`; unknown fields rejected | Rust type, operational/generated schema, TS type, golden |
| JavaScript precision loss | revisions/event sequences/cursors use canonical decimal strings on wire | `WireCounter`, schemas, goldens, SDK |
| multiple schema authorities | Rust-generated schema is byte snapshot checked; operational schema and TS receive structural drift gates | generator example and `check-schema-drift.mjs` |
| vacuous filtered tests | named Rust tasks use a wrapper that requires a matching passing test | `scripts/run-rust-test-gate.sh`, `TDD.md` |
| unbounded replay | facade returns bounded `ReplayPage` | Tower scaffold and runtime facade contract |
| same-Session subscription collision | SDK routes queues by `subscriptionId` | client regression test |
| unlimited resource interpretation | no arbitrary product cap, but measured safety budgets gate residency/work | Tower lifecycle/tasks |
| cleartext remote release posture | remains explicitly runnable only as experimental/unsafe; production remote requires TLS gate | security contract/tasks |
| overly broad first milestone | local in-process/stdio vertical slice precedes remote WS/MCP/SDK | root wave table |

These changes are protocol-breaking only within the explicitly experimental-v2
scaffold. No production consumer or legacy ACP/leader byte contract is changed.
