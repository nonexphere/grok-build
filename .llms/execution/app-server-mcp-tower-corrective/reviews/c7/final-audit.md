# C7-A — Independent final audit (GLM `glm-5.2`, read-only)

| Field | Value |
|---|---|
| Wave | C7-A (final independent audit) |
| Review mode | `final-goal` (read-only) |
| Reviewer | GLM `glm-5.2` |
| Date | 2026-07-19 |
| Handoff | `handoffs/HANDOFF-C7-A-final-audit.md` |
| Branch inspected | `goblin-implement-epic-tree` (working tree, uncommitted) |
| Authority contracts | `.llms/tasks/20260718-correct-app-server-mcp-tower-execution.md` (acceptance gates §5, stop conditions §6), `.llms/tasks/20260718-execute-app-server-mcp-tower-plan.md` (stop conditions) |
| Artifacts read | `STATUS.md`, `BLOCKERS.md`, `CHANGES.md`, all `waves/*.md`, all `reviews/{c0,c1,c3,c4}/*.md`, `SCRATCH/waves/c3-g.md`, `tests/{c1,c2,c3,c4,c5,c6}/*` log names, sampled source (`app_server_composition.rs`, `shell_session_actor_runtime.rs`, `ws_listener.rs`, `http_server.rs`) |

## Re-run disclosure (honest)

This reviewer has **no shell/command-execution tool**. No `cargo`/`npm`/gate
command was re-executed. The audit is a static, independent re-check of the
working-tree source against the contracts and the implementer-captured
GREEN/RED logs. All GREEN/RED claims below are the implementer's captured
logs, cross-checked against current source where the reviewer could verify
behavior statically (composition root, FakeRuntime absence, Tower≠Shell, real
WS/MCP bind). Commands not re-run by this reviewer:

- `cargo test -p xai-grok-shell --test c1_shell_port`
- `cargo test -p xai-grok-shell --test c1_turn_lifecycle`
- `cargo test -p xai-grok-shell --test c1_production_spawn`
- `cargo test -p xai-grok-shell --test c3_history_projection`
- `cargo test -p xai-grok-app-server --features websocket`
- `cargo test -p xai-grok-mcp-server --features streamable-http`
- `cargo test -p xai-grok-multi-auth --test byok_providers`
- `cargo test -p xai-grok-tower --test tower_instance_isolation`
- `cargo test -p xai-grok-tower-tools --test c6_tools_acl`
- `cargo test -p xai-grok-pager-bin --features app-server-ws app_server_ws`
- `cargo build -p xai-grok-pager-bin --bin grok-oss`
- `npm --prefix packages/grok-oss-app-server run typecheck/test/check:drift`
- Any C7 adversarial suite (concurrency, malformed, secret canary,
  path/symlink, replay/backpressure, cancellation, restart, multi-instance)
- Any workspace-wide `cargo test` / `cargo clippy` / schema drift gate

## Independent static verification (re-checked, not trusted)

1. **Composition root does not instantiate `FakeRuntime`.**
   `crates/codegen/xai-grok-pager-bin/src/app_server_composition.rs:31-33`
   builds `Arc::new(ShellSessionActorRuntime::new(root))` wrapped in
   `ShellRuntimeAdapter`. `FakeRuntime` appears only in doc comments
   (`:5`, `:12`) and a test name (`:90`); no `FakeRuntime::new` in product
   code. **PROVEN.**
2. **Tower ≠ Shell.** `crates/codegen/xai-grok-tower` has no `xai-grok-shell`
   dependency; the only match is the negative guard assertion at
   `xai-grok-tower/src/lib.rs:123` (`!cargo.contains("xai-grok-shell")`).
   **PROVEN.**
3. **No second `SessionActor` / no hybrid Fake+JSONL authority.**
   `shell_session_actor_runtime.rs` production section contains no
   `struct SessionActor`/`enum SessionActor`/`FakeRuntime::new`/`use ...FakeRuntime`/`: FakeRuntime`
   (matches only in doc comments and the `#[cfg(test)]` static guard at
   `:1311-1326`). **PROVEN.**
4. **Real WebSocket listener.** `xai-grok-app-server/src/transport/ws_listener.rs`
   uses `TcpListener::bind` (`:146`), `accept_hdr_async_with_config` (`:231`),
   and dispatches text frames through `FacadeProcessor::handle_line` (`:332`);
   no `xai_grok_shell` import. **PROVEN REAL** (not helper-only).
5. **Real MCP Streamable HTTP server.** `xai-grok-mcp-server/src/transport/http_server.rs`
   uses `TcpListener::bind` (`:271`) + `axum::serve` (`:276`), routes
   `tools/call` through `invoke_tower_tool` (`:523`); no `xai_grok_shell`,
   no `McpClient`/`register_self` in production (only in the negative canary
   at `:754-755`). **PROVEN REAL** (not helper-only).
6. **MCP HTTP not wired into product composition.**
   `app_server_composition.rs` wires WS (`run_app_server_ws`, C3-G) but
   contains no `run_mcp_http_server`/`McpHttpConfig` reference. The C4-F
   handoff exists but has **no wave note, no review, no STATUS entry** —
   MCP HTTP product wiring is **NOT done**. Confirmed against STATUS.md
   "C4 residual: product bin `--mcp` wiring".

## 1. Acceptance gate table (corrective contract §5)

| # | Mandatory acceptance gate | Status | Evidence / gap |
|---|---|---|---|
| G1 | Product composition does not instantiate `FakeRuntime` | **PROVEN** | `app_server_composition.rs:31-33` injects `ShellSessionActorRuntime::new(root)`; static guard `shell_session_actor_runtime_does_not_use_fake_runtime`; C1-E review PASS. |
| G2 | Every facade method reaches existing Shell leader/`SessionActor` command path | **INCOMPLETE** | Storage methods (list/read/start/resume/fork/replay) REAL via `JsonlStorageAdapter` (C1-D). Turn methods (start/steer/interrupt) route real `SessionCommand::{Prompt,Interject,Cancel}` via `cmd_tx` (C1-G) **only when a resident exists**; production `spawn_session_on_thread` assembly is PARTIAL (C1-J: `ProductionSpawner::new()` returns `unsupported` — needs HUMAN creds + ~80 factory args wired at composition root). `respond_interaction` returns `unsupported` (R10 deferred). `archive_session` returns `unsupported` (R6 product decision). |
| G3 | Real adapter and fake pass a shared conformance suite | **BLOCKED (local)** | No shared conformance suite run comparing normalized results across `FakeRuntime` and `ShellSessionActorRuntime` was produced. C1-D ran real-adapter tests (`c1_shell_port.rs`, 18 tests) and Fake conformance separately, but no shared normalized comparison matrix exists in the ledger. Contract item 13 ("Run the same conformance suite against FakeRuntime and the real adapter and compare normalized results") is **not evidenced**. |
| G4 | Real WebSocket and MCP HTTP servers pass black-box tests | **PARTIAL** | WS: C3-B 16 tests GREEN over real `TcpListener` (item 20/21/24). MCP HTTP: C4-B 23 integration + 12 lib GREEN over real `axum::serve`; C4-E added fail-closed auth + fingerprint test (27 int + 15 lib). **However**, neither server is wired into the product composition root for MCP HTTP (C4-F not executed); WS is wired (C3-G). Black-box test surface is REAL; product wiring is PARTIAL for MCP. |
| G5 | Canonical persisted sessions drive history/replay recovery tests | **PROVEN (with PARTIAL depth)** | C3-F `c3_history_projection.rs` 16 RED→GREEN tests over real `updates.jsonl` via `UpdatesIterator`; one shared projector feeds `read_session` (R2) + `replay` (R11). Honest PARTIAL: `TurnChanged` not emitted (Shell writes no turn lifecycle), `InteractionRequested` not projected (in-memory only), `created_at_ms=0`, no item grouping, `provider_binding=None`. AS105-01..07 remain OPEN/PARTIAL in the matrix for full epoch/revision/crash-rebuild semantics. |
| G6 | OpenRouter, Groq, Cloudflare registered functional verticals; missing live creds SKIP only | **PARTIAL** | C5-B registers `ByokAuthProvider` for all three with `API_KEY_LOGIN`, hardens `run_api_key_login` (registry + capability + xAI fallback guard), 17 offline contract tests GREEN. **PARTIAL**: composition-root Turn binding NOT proven (`provider_binding` still `None` on projected Session — C1-G residual); File/Keyring backend policy undecided; interactive TTY prompt not added; live smoke correctly SKIP (no creds). Item 37 (composition root end-to-end) NOT done. |
| G7 | Task files, ledger, tests and reviews agree | **INCOMPLETE** | STATUS/CHANGES/BLOCKERS kept current. **Missing ledger files**: `DECISIONS.md` (required by contract §3), `FINAL_REPORT.md` (required by contract §3 + §5 item 50). **Missing reviews**: no `reviews/c2/`, `reviews/c5/`, `reviews/c6/` directories — C2-A, C5-B, C6-A have NO independent code/test review. C0 has architecture review only (no code/test review pair). C1 has code+test review for C1-D and turn-lifecycle reviews (C1-E/F, C1-H/I) but C1-J has no review. Reviews agree with STATUS where they exist. |
| G8 | Every behavior change has non-vacuous RED/GREEN evidence | **PARTIAL** | C1-D/C1-G/C1-J/C3-B/C3-F/C3-G/C5-B/C6-A: RED+GREEN captured. C4-B initially missing RED log (C4-D F1, Medium) — C4-E captured `c4e_fail_closed_auth_RED.log` but the original 23-test RED baseline is still inferential (map-documented, not a captured log). C1-J F-4 RED is race-dependent (not captured). C4-E fingerprint tests have no RED (coverage additions, accepted). C2-A RED captured. |
| G9 | Every wave has independent code and test review with findings triaged | **INCOMPLETE** | C1: code+test reviews exist (C1-E/F for C1-D; C1-H/I for C1-G). C3: code+test reviews exist (C3-C/D for C3-B); **C3-F (history) and C3-G (composition) have NO review**. C4: code+test reviews exist (C4-C/D for C4-B; C4-E fixes reviewed within C4-C/D scope). **C0, C2, C5, C6: NO independent code/test review pair** (C0 has architecture review only). Contract §2 item 14 / §5 require independent review after every bounded epic/slice. |
| G10 | All local automated gates green and no Critical/High finding remains | **INCOMPLETE** | Package-scoped gates GREEN per wave (C1 18+9+7, C2 9+4+22, C3 16+3, C4 23+15, C5 17+89, C6 24). **No Critical/High finding open** in existing reviews (C1-E F-1..F-6 all Low/Medium non-blocking; C3-C F-1..F-6 Low; C4-C F-2 Medium closed by C4-E; C4-D F1/F2 Medium — F1 RED log partially closed, F2 fingerprint test closed by C4-E). **However**: no workspace-wide gate run; pre-existing `is_managed_install_matches_only_the_bin_grok_target` failure (grok-oss identity cutover) unaddressed; C7 adversarial gate (item 45-46) NOT run. |
| G11 | No secret, public contract, downstream consumer or production-readiness claim left ambiguous | **INCOMPLETE** | HUMAN gates documented (TLS/threat AS104/AS107/MCP102, npm TS101, live creds PR101/OR-02/GQ-02/CF-02, jsonrpc SP101, auto-deny AS106, leader fixture TW101). `archive_session` (R6) product decision still open. `steer_turn` `Item` shape (R8) product decision pending. File/Keyring backend policy for BYOK undecided. `PublicProviderBinding` vs protocol `ProviderBinding` reconciliation open. Production-readiness correctly NOT claimed (TLS HUMAN). |

## 2. Stop-condition check (contract §6)

| Stop condition | Tripped? |
|---|---|
| Would require a second actor/state machine | No — single `SessionActor` preserved. |
| Fake-backed production composition | No — real port injected. |
| Helper presented as a server | No — real `TcpListener`+`axum::serve`/`accept_hdr_async`. |
| Skipped live test presented as PASS | No — SKIP discipline maintained. |
| Undocumented contract break | No. |
| Unverified external ownership | No. |
| Insecure secret handling | No — fail-closed auth (C4-E); constant-time bearer; no token in URL. |
| Bypassing unresolved HUMAN security/product decision | No — TLS/archive/auto-deny kept HUMAN. |

No stop condition is tripped. The program is not forced into BLOCKED by §6;
it is **incomplete** because multiple §5 acceptance gates are unmet.

## 3. Critical / High findings

None of the completed wave reviews record an open Critical/High finding.
This audit raises the following **High** program-level findings (not wave
defects; completeness/contract gaps):

- **F-A1 [High] — C7 adversarial release gate NOT executed.** Contract §4
  item 45-50 (Wave C7) is entirely unperformed: no formatting/lint/schema
  drift/full Rust test run, no TypeScript typecheck/test/build, no adversarial
  concurrency/malformed/secret canary/path/symlink/replay/backpressure/
  cancellation/restart/multi-instance suites, no independent diff review, no
  checkbox reconciliation, no `FINAL_REPORT.md`. The program cannot be
  COMPLETE without C7.
- **F-A2 [High] — `FINAL_REPORT.md` missing.** Required by contract §3 and
  §5 item 50. No `FINAL_REPORT.md` exists in
  `.llms/execution/app-server-mcp-tower-corrective/`.
- **F-A3 [High] — `DECISIONS.md` missing.** Required by contract §3. Absent.
- **F-A4 [High] — Independent reviews missing for C0, C2, C5, C6, C3-F, C3-G, C1-J.**
  Contract §2 item 14 and §5 require independent code + test review after
  every bounded slice. Only C1-D, C1-G, C3-B, C4-B have review pairs. C0 has
  architecture review only. C2-A, C5-B, C6-A, C3-F, C3-G, C1-J have NO
  independent review. These waves' claims (Tower isolation, BYOK providers,
  tools ACL, history projection, WS composition, production spawn seam) are
  unverified by an independent reviewer.
- **F-A5 [High] — Shared conformance suite (real vs fake) not produced.**
  Contract §4 item 13 and §5 G3 require running the same conformance suite
  against `FakeRuntime` and the real adapter and comparing normalized
  results. No such shared matrix is in the ledger.
- **F-A6 [High] — MCP HTTP product composition wiring NOT done (C4-F).**
  C4-F handoff exists but was not executed: no wave note, no review, no
  STATUS entry. `app_server_composition.rs` has no `run_mcp_http_server`
  reference. The real MCP HTTP server is proven only via tests, not via the
  product path. Contract §5 G4 ("real WebSocket and MCP HTTP servers pass
  black-box tests") is met for tests but the product wiring is PARTIAL.
- **F-A7 [High] — Production spawn assembly PARTIAL blocks G2.**
  `ProductionSpawner::new()` returns `unsupported` enumerating missing deps
  (creds, `AgentDefinition`, `ToolContext`, ...). Without composition-root
  wiring of `spawn_session_on_thread` (C1-J residual, depends on C2-A
  composition + HUMAN creds), `start_turn`/`steer_turn`/`interrupt_turn`
  cannot run a real actor in production. G2 ("every facade method reaches
  Shell leader/`SessionActor` command path") is INCOMPLETE for the
  production path.

Medium findings (non-blocking for verdict, recorded):

- **F-A8 [Medium] — `respond_interaction` (R10) and `archive_session` (R6)
  remain `unsupported`.** Documented product decisions pending; not blocking
  local progress but G2 is not fully met.
- **F-A9 [Medium] — Provider composition-root Turn binding PARTIAL.**
  `provider_binding` is `None` on projected Session; the end-to-end
  login→persist→bind→resolve→inference composition test (item 37) is not
  proven. G6 is PARTIAL.
- **F-A10 [Medium] — Dual-OS-process flock isolation PARTIAL (C2-A).**
  Single-winner `instance.lock`, endpoint-in-use, stale-PID reconciliation
  not implemented; only in-process dir/registry isolation proven. TW103-03
  remains PARTIAL.
- **F-A11 [Medium] — Pre-existing `is_managed_install_*` failure
  unaddressed.** `cargo test -p xai-grok-pager-bin` reports 1 failure
  (grok-oss identity cutover). Not caused by the corrective program but
  leaves a non-green package gate; C7 item 45 ("treat warnings introduced by
  this program as failures") cannot be cleanly assessed.
- **F-A12 [Medium] — C4-B original RED log still inferential.** C4-E closed
  the fail-closed-auth RED, but the 23-test RED baseline for the server's
  initial absence is map-documented, not a captured log (C4-D F1).

## 4. Recommended program verdict

### **BLOCKED**

The corrective program is **not COMPLETE**. It has made genuine, well-evidenced
progress on the core invariants (no `FakeRuntime` in product composition,
Tower≠Shell, real WS + MCP HTTP servers, real history projection, BYOK
registry, fail-closed auth), but multiple mandatory acceptance gates (§5)
are unmet and the terminal Wave C7 has not been executed.

### Exact unmet requirements (must close before COMPLETE)

1. **G2 — production command path:** wire `spawn_session_on_thread` into the
   composition root via `with_production_spawn`/`RealSpawnFn` and prove a
   live-actor turn with real credentials (C1-J residual + C2-A composition).
   Decide `respond_interaction` (R10) delivery channel and `archive_session`
   (R6) product semantics, or record them as HUMAN with the exact decision
   needed.
2. **G3 — shared conformance suite:** run one normalized conformance matrix
   across `FakeRuntime` and `ShellSessionActorRuntime` and record the
   comparison.
3. **G4 — MCP HTTP product wiring:** execute C4-F (wire `run_mcp_http_server`
   into pager-bin with fail-closed bearer + self-loop guard) and capture
   RED/GREEN evidence.
4. **G6 — provider composition-root Turn binding:** add the end-to-end
   login→persist→bind→resolve→inference composition test for at least one
   BYOK vertical; reconcile `PublicProviderBinding` to protocol
   `ProviderBinding` or delete it.
5. **G7 — ledger completeness:** create `DECISIONS.md` and `FINAL_REPORT.md`.
6. **G9 — independent reviews:** commission independent code + test reviews
   for C0, C2-A, C5-B, C6-A, C3-F, C3-G, C1-J; triage findings; rerun gates.
7. **C7 adversarial release gate (§4 items 45-50):** run formatting,
   lint/check, schema/golden drift, all relevant Rust tests, TypeScript
   typecheck/tests/builds; run adversarial concurrency/malformed/secret
   canary/path/symlink/replay/backpressure/cancellation/restart/multi-instance
   suites; commission one independent subagent to review the complete diff
   against the epics/contracts and another to audit test adequacy and
   vacuous filters; reconcile every checkbox with final executable evidence;
   record HUMAN/external gates separately; produce `FINAL_REPORT.md` with
   exact commands, counts, skips, blockers, commits, downstream impact, and
   a strict COMPLETE/BLOCKED verdict.
8. **G10 — workspace gate:** run the full workspace `cargo test`/`clippy` and
   resolve or explicitly attribute the pre-existing
   `is_managed_install_matches_only_the_bin_grok_target` failure.
9. **G8 — RED evidence parity:** capture the C4-B original RED log (or
   formally accept the C4-A map as the RED baseline with a written
   rationale in `DECISIONS.md`).

### Residual risk

- The core architecture invariants are sound and independently verified
  (FakeRuntime absent, Tower≠Shell, real servers). The risk is
  **completeness/contract closure**, not a correctness regression.
- HUMAN gates (TLS/threat, npm, live creds, jsonrpc, auto-deny, archive
  semantics) are correctly NOT claimed and remain external; they do not block
  local work but block any future COMPLETE that depends on them.
- Uncommitted working tree (orchestrator does not auto-commit): all evidence
  is against the working tree on `goblin-implement-epic-tree`; no commit/PR
  ledger exists in `CHANGES.md` beyond prose change descriptions.

## 5. Verdicts (separated)

- `IMPLEMENTATION_OR_ARTIFACT: FAIL` — real, well-evidenced slices exist, but
  the program-as-a-whole does not meet §5 acceptance gates G2/G3/G4/G6/G7/G9/G10/G11
  and Wave C7 is unperformed.
- `AGENT_BEHAVIOR: PASS` — the executing agent followed Red-Green-Refactor,
  preserved invariants, documented PARTIALs honestly, did not mark false DONE,
  did not weaken security, kept HUMAN gates explicit.
- `HANDOFF_QUALITY: PASS` — wave notes, STATUS, CHANGES, BLOCKERS, and
  reviews are detailed with file:line evidence and honest PARTIAL framing;
  gaps are ledger-missing (DECISIONS/FINAL_REPORT) not handoff-vague.
- `GOAL_GATE: FAIL` — the final-goal gate cannot PASS: multiple in-scope ACs
  are unproven (G2 production path, G3 shared conformance, G4 MCP wiring,
  G6 provider composition, G7/G9 ledger+reviews, C7 adversarial gate,
  FINAL_REPORT). No mutation has a later independent PASS for the unreviewed
  waves (C2/C5/C6/C3-F/C3-G/C1-J).
