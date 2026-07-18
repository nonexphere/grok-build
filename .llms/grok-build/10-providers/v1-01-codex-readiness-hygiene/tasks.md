# Tasks — v1-01 Codex readiness hygiene

- [x] `PR101-01` Reconcile requirement matrix from `TO_RELEASE.md` into [REQUIREMENT_MATRIX.md](./REQUIREMENT_MATRIX.md); live PC8 remains PARTIAL.
- [x] `PR101-02` [D-TD] Offline package gate: `cargo test -p xai-grok-auth -p xai-grok-multi-auth --no-fail-fast` (record under execution ledger).
- [x] `PR101-03` Prove production `ProviderBinding` identifiers only (protocol crate tests + matrix note).
- [x] `PR101-04` Policy: live tests never PASS without credentials; PC8 stays PARTIAL in matrix and TO_RELEASE.
- [ ] [D-PR] `(HUMAN, external, non-blocking)` full PC8 live pack + dual OS-process flock when credentials/host allow.
