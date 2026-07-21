# Wave: Tower resume/archive lifecycle evidence — 2026-07-20

## Evidence

```text
cargo test -p xai-grok-shell c7_conformance_ -- --nocapture
18 passed; 0 failed
```

The real Shell adapter and the conformance fixture cover:

- unknown-session resume → `session_not_found`;
- resume preserving the Session ID;
- persisted summary loading;
- concurrent resume coverage;
- reversible archive via `archived.flag` (data is retained);
- archive behavior and honest Fake/real divergence characterization;
- list/read/replay interactions around lifecycle state.

## Remaining scope

TA103-08/09 remain partial. Explicit dormant-only admission, epoch-preserving
concurrent convergence, busy archive policy, actor drain/detach, and complete
Tower-tool product transport coverage still require dedicated gates.
