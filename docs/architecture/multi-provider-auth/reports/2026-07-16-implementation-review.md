---
title: "Multi-provider and native Codex authentication implementation review"
date: "2026-07-16"
slug: "implementation-review"
status: "relevant"
status_reason: "Current evidence-based review of the uncommitted Goblin implementation against task.md and GOBLIN.md."
last_reviewed_at: "2026-07-16"
source_path_legacy: "docs/architecture/multi-provider-auth/reports/2026-07-16-implementation-review.md"
---

# Multi-provider and native Codex authentication implementation review

> Type: `full-plan-review` / `final-readiness`
> Plan: `task.md` §1–§14, especially §12 and Appendix E
> Audited scope: all local changes relative to `b189869`, including auth contracts,
> provider implementations, store, token manager, CLI, model integration, tests,
> progress ledger, and fork contract
> Reviewer: Codex (direct review; no subagents)

## 1. Executive summary

**Verdict: `NOT READY`.** The implementation has a substantial and often useful
control-plane foundation, but the product path does not implement the architecture
that the plan makes central. Native Codex login may complete and persist a token,
but inference does not consume the request-scoped resolver or token manager. Instead,
the shell synchronously reads credentials while building the model catalog and copies
an OAuth access token into a static `api_key` field. This breaks automatic refresh,
generation-aware 401 recovery, account-immutable request binding, concurrent account
selection, and safe multi-provider extensibility.

The implementation therefore proves several isolated units, not the promised
end-to-end behavior. The empirical `done` claims in `PROGRESS.md` are materially
incorrect. Phase 4 is explicitly a stub, Phase 7 is not integrated into the sampler,
Phase 8 implements only a small subset of the CLI grammar, Phase 9 is deferred, and
Phases 10–11 lack their required gates.

There is also a release/security blocker: the live default provider configuration
uses the observed Codex OAuth client ID and an unapproved `originator`, even though
D10, OQ1–OQ4, the rollout plan, and Appendix E say approval is required before this
can be enabled as normal product behavior.

### What is genuinely implemented

- Typed provider/credential/model contracts and an object-safe provider registry.
- A multi-account file/ephemeral credential store with aliases, CAS generations,
  atomic per-file writes, Unix permission tests, and advisory locks.
- Codex PKCE/device/token/model/header protocol modules with useful unit/mock tests.
- A token-manager implementation with in-process per-key locking and stale-generation
  handling tests.
- Basic provider picker, Codex login routing, status JSON, and model listing helpers.
- A request-auth resolver that works in its own unit test but is not connected to
  production inference.

### What is not proven or not implemented

- Request-scoped authentication in the real inference client.
- Concurrent selection of two Codex accounts or two providers by parent/subagent.
- Refresh and one-retry 401 recovery in production requests.
- A real xAI provider/legacy-store adapter.
- Account-scoped model identities/cache and explicit account selection.
- Required CLI grammar (`grok login codex`, account/provider logout/status commands).
- TUI workflows, migration, keyring-first storage, runtime config gates, metrics,
  fuzz/fault-injection/cross-process tests, and an approved OAuth client.

## 2. Source-of-truth precedence and artifacts audited

Precedence used: current user request and repository governance; then `task.md` for
product behavior; then `GOBLIN.md` for fork layout/process; then `PROGRESS.md`; then
code/tests. This follows `GOBLIN.md` §Source of Truth. A status claim never overrides
contradictory code or a failed/missing acceptance test.

| Artifact | Role | Review result |
| --- | --- | --- |
| `task.md` | Product spec, architecture, phases, acceptance gates | Strong architecture, but too broad for one implementation wave and missing executable IDs/checkpoints |
| `GOBLIN.md` | Fork/process/module/feature contract | Clear, but module and feature declarations drift from code |
| `protocol-baseline.md` | Codex wire baseline | Detailed; still records unresolved authorization questions |
| `PROGRESS.md` | Phase ledger | Internally contradictory and overstates completion |
| `xai-grok-auth` | Public control-plane contracts | Meaningful implementation and tests |
| `xai-grok-multi-auth` | Provider/store/token/login logic | Meaningful isolated implementation; incomplete integration |
| pager/pager-bin/shell changes | Product wiring | Main source of end-to-end contract violations |
| Git history | Snapshot traceability | No implementation commits; all changes are uncommitted over `b189869` |

The report skill normally expects `SPECS.md`, `CODE_SPECS.md`,
`STATE_MACHINE_DIAGRAM.md`, `TODO.md`, and `changelog/`. This repository instead has
one 4,293-line `task.md`, `GOBLIN.md`, a protocol document, and a phase ledger. There
is no changelog or per-item commit traceability. The matrices below map those actual
sources rather than inventing missing IDs.

## 3. Requirements-to-code synchronization matrix

| Requirement | Planned component | Current evidence | Real status |
| --- | --- | --- | --- |
| D1/D2 native login, no Codex dependency | Codex provider + coordinator | `providers/codex/*`; pager-bin login branch | **Partial**: native flow exists; live authorization/support is unresolved |
| D3 immutable request binding | `ModelBinding` + `RequestAuthResolver` + sampler | resolver exists; shell catalog copies token to `api_key` | **Contradicted** |
| D4 per-credential refresh synchronization | `TokenManager` + cross-process credential lock | in-process mutex exists; production request path bypasses manager | **Partial/unused** |
| D5 xAI compatibility | `XaiAuthProvider`, legacy adapter | provider file says “Minimal ... stub”; methods return errors | **Not implemented** |
| D6 compile-time registry | registry builder | xAI/Codex registered in code | **Implemented**, but registration errors are discarded |
| D7 provider-specific device protocol | Codex device module | dedicated request/poll/exchange code + mock tests | **Implemented in isolation** |
| D8 keyring preferred | keyring/composite backends | composite only forwards to one backend; CLI constructs file store | **Not implemented** |
| D9 authenticated models + fallback/cache | provider model catalog | live `/models` fetch exists | **Partial**: no bundled/cache fallback or account-scoped cache |
| D10 client approval boundary | configuration/rollout gate | observed client ID is the live default | **Contradicted / release blocker** |
| G1 provider selection | CLI/TUI | stdin picker in CLI | **Partial**: no noninteractive TTY guard; TUI absent |
| G2 native Codex login grammar | CLI | `--provider codex`; `--device-code` alias | **Partial**: positional `login codex` and `--browser` absent |
| G3 simultaneous credentials | store + account UX | store holds many records | **Partial**: storage yes, usable account selection no |
| G4 mixed parent/subagent binding | session/subagent runtime | no production references to resolver/model binding | **Not implemented** |
| G5 transparent refresh | token manager + client retry | unit implementation only | **Not implemented end-to-end** |
| G6 add provider cohesively | provider plugin + generic CLI/runtime | CLI enum/parser hard-code xAI/Codex | **Contradicted** |
| G7 backward compatibility | adapters + regression suite | legacy xAI route remains | **Weak evidence**: broad regression suite not completed |

## 4. Phase/TODO execution matrix

| Phase | Ledger claim | Code/test evidence | Correct status |
| --- | --- | --- | --- |
| 0 protocol/authorization | both `done` and `in_progress` | baseline exists; OQ1–OQ4 unresolved; no live verification evidence | `in_progress` |
| 1 contracts/registry | `done` | types/trait/registry tests exist | `mostly done` |
| 2 credential store | `done` | file/ephemeral/CAS tests; no keyring, fault injection, stale-lock recovery | `partial` |
| 3 token manager | note says `done` | two concurrency/401 tests; no production consumer, cache/failure/subscriber/cross-process coverage missing | `partial` |
| 4 xAI compatibility | note says `done` | `providers/xai.rs:1-6` calls itself a stub; core methods error | `not done` |
| 5 browser PKCE | note says `done` | protocol/unit/mock implementation exists | `partial`, pending approved live validation/workspace policy |
| 6 device/refresh/logout | note says `done` | isolated provider methods exist | `partial`, production refresh/logout semantics unproven |
| 7 inference/models | note says `done` | resolver unit test exists; sampler has no integration; catalog uses static token | `not done` |
| 8 CLI | note says `done` | picker, `--provider`, status only; grammar/account ops incomplete | `partial` |
| 9 TUI | `partial` | intentionally deferred | `not done` against plan |
| 10 migration/hardening | `partial` | no migration/keyring/fuzz/fault injection/security gate | `not done` |
| 11 rollout | `partial` | docs/kill switch only; no runtime config/metrics/cohort/release validation | `not done` |

## 5. Findings

### BLOCKER

#### B1 — Production inference bypasses the architecture's central request-scoped auth path

Evidence: `task.md:53-66`, `task.md:3030-3051`, and `task.md:4265-4283` require
immutable `(provider, account, model)` bindings, token refresh, account headers, and
401 recovery. The only production wiring is `agent/models.rs:1896-2001`, which loads
the secret while resolving the catalog and stores the raw access token as
`ModelEntry.api_key`. Repository search finds no `RequestAuthResolver` or
`TokenManager` consumer in `xai-grok-sampler` or production shell code.

Impact: a token snapshot expires; 401 cannot refresh/retry; generation stamps are
discarded; the request is not bound to a credential; and the tested resolver is dead
code from the application's perspective.

Corrective action: integrate a provider-aware auth resolver into the sampler request
boundary. Carry `ModelBinding` through session/agent/subagent state, resolve a token
immediately before each request, stamp the sent generation, and route a single 401
retry through `TokenManager::recover_unauthorized`. Remove OAuth tokens from model
configuration.

#### B2 — Multiple Codex accounts cannot be selected correctly and collide by model ID

Evidence: `agent/models.rs:1929-2001` loops through accounts but keys every entry as
`codex/<model-id>` and skips later collisions with `catalog.contains_key`. It then
looks the credential up by alias and embeds that account's token/header. The model ID
contains neither credential ID nor alias, and there is no account picker/binding.

Impact: when two accounts expose the same model, the first account silently wins;
the other is unusable. Catalog order becomes credential selection. This contradicts
G3/G4, D3, Phase 7 concurrent-account tests, and Appendix E.

Corrective action: model identities must resolve to an explicit `ModelBinding` with a
credential ID (or provider default resolved once at session binding). Display grouping
may use aliases, but runtime identity must never depend on iteration order.

#### B3 — Automatic refresh and 401 recovery are not connected to actual requests

Evidence: `token_manager.rs` implements isolated methods and two tests, while
`agent/models.rs:1939-1995` directly reads `access_token` and constructs a static API
key. No production caller invokes `recover_unauthorized`; no sampler integration is
present (`rg RequestAuthResolver|TokenManager crates/codegen/xai-grok-sampler` returns
no matches).

Impact: login may appear successful and models may appear, but inference stops when
the initial access token expires. This matches the reported “login exists but it does
not work” symptom.

Corrective action: same integration as B1, plus wire tests that send an expired token,
receive 401, refresh once, persist rotation, and retry exactly once.

#### B4 — The xAI compatibility provider is a declared stub, not Phase 4

Evidence: `providers/xai.rs:1-6` describes a “Minimal xAI auth provider stub”;
`start_login`, `complete_login`, `refresh`, `get_valid_token`, `logout`, endpoint, and
request-auth methods return `InvalidConfig` (`:73-155`). The ledger nevertheless says
the xAI adapter is done because it is “registered”.

Impact: the registry is not a uniform control plane. Future generic CLI/runtime code
cannot operate over xAI, and simultaneous multi-provider behavior is not proved.

Corrective action: implement the legacy xAI credential-store and AuthManager adapters
specified in Phase 4, or accurately mark the phase pending and keep all generic paths
from claiming xAI capability until adapters exist.

#### B5 — Unapproved observed OAuth client identity is enabled as the live default

Evidence: `config.rs:5-28` sets the observed client ID and a pending-approval
`originator`; `CodexOAuthConfig::default` uses both at `:64-72`; normal CLI login builds
the default registry. This contradicts D10, OQ1–OQ4 (`task.md:3256-3264`), Stage 0/1
gating (`task.md:3133-3148`), and the approval acceptance gate (`task.md:4290`).

Impact: unsupported third-party client use may fail unpredictably and creates product,
terms, and release risk.

Corrective action: fail closed unless an explicitly approved/configured client is
provided in development mode. Do not show Codex in the normal picker by default until
authorization and callback/originator values are approved.

#### B6 — Store refresh updates are not transactionally atomic across metadata and secrets

Evidence: `store/file.rs:231-283` writes the new secret file first and metadata second.
Each file write is atomic, but the pair is not. A crash between `save_secrets` and
`save_accounts` exposes a rotated secret under an old generation/expiration. Create
has the inverse orphan risk at `:191-228`; delete writes metadata before secrets at
`:286-306`. The plan explicitly calls out crash, rotated-token reuse, and persistence
failure cases (`task.md:2928-2938`, `:3243-3250`).

Impact: after refresh-token rotation, cross-file inconsistency can make recovery
unsafe or lose the only valid token state.

Corrective action: introduce a recoverable transaction/journal or single atomic
encrypted record boundary, with fault-injection tests at every persistence edge.

### MAJOR

#### M1 — Provider extensibility is undercut by hard-coded CLI routing

`cli.rs:16-39` defines an enum with only xAI/Codex and parses literal names; the pager
matches those variants. A third provider therefore requires editing CLI and binary
composition, contradicting G6 and `task.md` §4.3. Use provider IDs/descriptors and
capabilities from the registry, with only explicit legacy-xAI adaptation at a boundary.

#### M2 — CLI contract is incomplete and inconsistent

`app/cli.rs:25-50` supports only `login --provider`; the specified positional
`login codex`, `--browser`, `--account`, `--force`, provider/account logout, account
list/use/remove, and full `auth` grammar are absent. `AuthCommand::Status` accepts
`--json`, but `main.rs:1842-1859` ignores the boolean and always prints JSON.

#### M3 — The interactive picker can block noninteractive callers

`cli.rs:47-89` claims non-TTY compatibility but performs a blocking stdin read whenever
two providers exist and contains no `IsTerminal` check. The spec requires noninteractive
safety in Phase 8. Preserve legacy xAI default or require `--provider` when stdin is not
a terminal.

#### M4 — Logout deletes local secrets without provider revocation and lacks scoping

`cli.rs:94-139` deletes every native credential and then clears legacy xAI; it never
calls `AuthProvider::logout`, despite Codex revocation support. The top-level command
has no provider/account arguments. This contradicts §7.1 and best-effort revocation.

#### M5 — Runtime configuration and most feature gates are documentation-only

Only `native-multi-provider-auth` and a shell alias feature are present. Searches find
no runtime `[features]` config plumbing and no `auth-keyring`/
`auth-encrypted-file` features. Environment switches exist, but the documented staged
rollout cannot be enforced.

#### M6 — Keyring-first and encrypted storage are absent

`store/composite.rs:1-75` is only a forwarding wrapper, while every CLI/model path
constructs `FileCredentialStore`. D8 and Phase 2 require keyring preference and
headless fallback; the security/release gate requires keyring support.

#### M7 — Model discovery has no account-scoped cache/fallback and performs blocking network I/O during catalog resolution

`cli.rs:164-260` fetches models for every account; `agent/models.rs:1921` calls the
blocking wrapper from synchronous catalog construction. There is no bundled/cache/ETag
integration and no account-scoped cache path. Startup/model resolution can block on
network and repeat calls.

#### M8 — RequestAuthResolver is Codex-hard-coded despite a provider-neutral API

`request_auth.rs:68` hard-codes the OpenAI issuer and `:121-123` hard-codes the Codex
Responses endpoint/method instead of using the provider's endpoint resolver and the
actual request. Even if wired, it would not be a generic multi-provider resolver.

#### M9 — TokenManager falls short of its claimed Phase 3 contract

The plan requires memory cache, permanent-failure cache, subscribers, cross-process
single-flight, account mismatch, logout wakeups, and seven acceptance scenarios.
Current tests cover 50-task in-process refresh and stale generation only. The ledger's
“done” status is unsupported.

#### M10 — Provider registration errors are silently discarded

`registry.rs:14-45` calls `.ok()` on every registration. Duplicate/invalid provider
configuration can disappear silently at composition time. A compile-time registry
should fail construction loudly and be tested as the actual default registry.

#### M11 — Login loses requested alias and account policy

The CLI offers no alias; provider completion creates records with
`requested_alias: None` in both browser and device paths (`codex/mod.rs:307-315` and
`:361-369`). Multiple logins become generated `default`, `default-2`, etc., not the
specified personal/work workflow. `LoginCoordinator` does not preserve an alias from
the start request through provider completion.

#### M12 — TUI, session, agent, and subagent integration are absent

No provider login modal, badges, grouped account picker, session `ModelBinding`, agent
profile binding, or subagent binding is implemented. The only TUI-adjacent behavior is
injecting static models into the shared catalog. Phase 9 and G4 are not partial in the
functional sense; they are pending.

#### M13 — Test strategy is much narrower than the risk surface

There are useful unit/mock tests, but no CLI snapshots, TUI PTY tests, simultaneous
provider/account inference, process-level refresh, session restore, migration,
fault-injection, fuzzing, or protected OAuth evidence. The implemented production
catalog bridge itself has no focused tests.

#### M14 — Documentation/ledger is internally contradictory and not auditable by commit

`PROGRESS.md` marks the same waves differently in their sections, summary, and appended
“empirical” table. No code changes have commits; `git log` contains only base commit
`b189869`. There is no changelog recording the major architectural deviation to a
new crate or the static-token bridge.

### MINOR

#### N1 — `auth status` silently suppresses store failures

`cli.rs:335-338` uses `unwrap_or_default`/`unwrap_or(None)`, converting corruption or
permission failures into an empty status. Diagnostics should surface per-provider
errors without exposing secrets.

#### N2 — `localhost` is used where the security contract says loopback-only

`config.rs:108-110` constructs `http://localhost:<port>`. The listener may still bind
to loopback, but the frozen redirect and validation contract should consistently use
the approved literal/host and test DNS/IPv4/IPv6 behavior.

#### N3 — The same source file is declared as two bin targets

Cargo warns that `main.rs` is present in both `goblin` and `xai-grok-pager`. This is
not itself a correctness failure, but it adds noise and doubles build-target concerns.

## 6. Plan-quality review

### What the plan got right

The plan's central architecture is strong. D3/D4 are exactly the right invariants for
multi-agent concurrency. Separating `AuthProvider`, `CredentialStore`, `TokenManager`,
`ModelResolver`, and request authentication is materially better than a global current
provider. The protocol, security, migration, UX, storage, and testing sections expose
the right risk categories. In particular, it correctly predicted refresh-token races,
workspace cross-contamination, static credential leakage, provider mutation, model
feature mismatch, and unsupported OAuth client use.

### Where the plan was weak as an execution contract

1. It is a 4,293-line architecture document, not an executable change plan. There are
   no stable requirement/task IDs beyond D/G/phase labels, no checkboxes per deliverable,
   no owner/commit/test trace per item, and no explicit dependency gate preventing Phase
   7/8 from being called done before Phase 3/4 are integrated.
2. It mixes stable-release architecture with early-spike implementation. That allowed
   a shortcut bridge to be presented as completion instead of an explicitly temporary,
   nonconforming spike.
3. Phase exit criteria are lists, not mandatory gates. The ledger can label a phase
   done even when most listed tests do not exist.
4. The plan recommends extending existing seams before crate extraction, while the
   implementation immediately creates `xai-grok-multi-auth`. The deviation may be
   reasonable because the shell test graph is problematic, but it was not approved or
   documented as a spec change before implementation.
5. The plan lacks an explicit “vertical slice” milestone. The first milestone should
   have been one provider/account/model request flowing login → store → binding → token
   manager → sampler → 401 refresh, before broadening CLI/model catalogs.

### Recommended plan correction

Replace optimistic wave completion with dependency-gated vertical slices:

1. **Foundation gate:** contracts + store durability, including crash transaction and
   keyring decision.
2. **Codex vertical slice:** approved test config; one account; login → binding → real
   sampler request → one refresh/retry; no TUI and no multi-account claim yet.
3. **Compatibility slice:** real xAI adapter through the same binding/resolver contract;
   legacy regression suite.
4. **Concurrency slice:** multiple Codex accounts and xAI/Codex parent/subagent tests;
   session persistence.
5. **Product UX slice:** generic CLI account management, then TUI.
6. **Release gate:** keyring/fallback, migrations, runtime gates, metrics, fuzz/fault
   injection, approved OAuth identity, and real protected smoke evidence.

No slice is done until its production composition root uses the tested abstraction.

## 7. Test and command audit

| Command/check | Result | Notes |
| --- | --- | --- |
| `git status --short --branch` | dirty, all implementation uncommitted | user/other-agent work preserved |
| `git diff --check` | pass at review time | no whitespace errors detected |
| `CARGO_TARGET_DIR=/tmp/grok-goblin-review-target CARGO_BUILD_JOBS=2 cargo test -p xai-grok-auth -p xai-grok-multi-auth --no-fail-fast` | pass: 40 tests | 12 auth unit + 16 multi-auth unit + 1 guarded live-model helper + 4 login integration + 7 multi-auth integration; emitted unused-import/variable warnings and duplicate-bin-source warning |
| repository search for production resolver/manager consumers | fail | no sampler integration; only store re-export and catalog bridge |
| plan acceptance criteria audit | fail | numerous required artifacts/tests absent |
| real OAuth/browser/device smoke | not run | requires approved client/account and external interaction; D10/OQ gates unresolved |
| full shell/pager workspace tests | not run to completion | another agent was actively compiling/editing the same worktree; this review avoids racing their implementation |

The focused suites pass, which proves useful isolated contracts and login/store behavior.
It does not change the verdict because the highest-severity findings concern missing
production composition, unsupported completion claims, and end-to-end request behavior.

## 8. Commit and changelog audit

| Commit | Scope | Coherence | Result |
| --- | --- | --- | --- |
| `b189869` | upstream open-source base | clean base only | no feature evidence |
| uncommitted worktree | all multi-provider/Codex work | mixes docs, contracts, store, providers, CLI, model bridge, packaging | not reviewable as coherent snapshots |

No implementation commit maps to any phase or acceptance criterion. There is no
change changelog. Before merge, split verified work by dependency/vertical slice; do
not commit ledger `done` states until the corresponding gate is proven.

## 9. Corrective action backlog

- [ ] **ACTION-001:** Remove the static OAuth-token-as-`api_key` model bridge.
- [ ] **ACTION-002:** Carry `ModelBinding` through session, agent, subagent, and sampler.
- [ ] **ACTION-003:** Integrate request-time token resolution and generation-stamped 401 retry.
- [ ] **ACTION-004:** Add simultaneous xAI/Codex and two-Codex-account inference tests.
- [ ] **ACTION-005:** Implement real xAI/legacy-store adapters or mark Phase 4 pending.
- [ ] **ACTION-006:** Fail closed on unapproved/default Codex OAuth client configuration.
- [ ] **ACTION-007:** Make credential refresh persistence crash-consistent as one transaction.
- [ ] **ACTION-008:** Implement generic provider/account CLI grammar from registry capabilities.
- [ ] **ACTION-009:** Add non-TTY safety and provider/account-scoped logout with revocation.
- [ ] **ACTION-010:** Implement account-scoped model cache/fallback/ETag behavior.
- [ ] **ACTION-011:** Implement keyring-first auto store and explicit secure fallback policy.
- [ ] **ACTION-012:** Add runtime config gates and align compile-time features with docs.
- [ ] **ACTION-013:** Add TUI/session/subagent workflows only after the vertical slice passes.
- [ ] **ACTION-014:** Add fault injection, fuzzing, cross-process refresh, CLI/TUI snapshots,
  restore/migration, and protected OAuth validation.
- [ ] **ACTION-015:** Rewrite `PROGRESS.md` statuses from empirical evidence and maintain one
  canonical table.
- [ ] **ACTION-016:** Record architectural deviations and produce coherent commits per gate.

## 10. Final verdict

`NOT READY`. The implementation should not be merged or represented as working
multi-provider authentication. Preserve the useful contracts/provider/store tests,
but treat the current app wiring as a spike to replace. The minimum next proof is a
single Codex account making a real request through `ModelBinding` → request resolver →
`TokenManager` → sampler, including refresh and one 401 retry, under an explicitly
approved/test-only client configuration.

## 11. References

- `task.md:53-66`, `task.md:2870-3250`, `task.md:3256-3264`, `task.md:4263-4292`
- `GOBLIN.md` §§Architecture Contract, Module Layout, Feature Flags, Security Rules,
  Source of Truth
- `docs/architecture/multi-provider-auth/PROGRESS.md` (all wave declarations and
  Canonical Implementation Note)
- `crates/codegen/xai-grok-multi-auth/src/providers/xai.rs:1-164`
- `crates/codegen/xai-grok-multi-auth/src/providers/codex/config.rs:5-72`
- `crates/codegen/xai-grok-multi-auth/src/registry.rs:10-48`
- `crates/codegen/xai-grok-multi-auth/src/cli.rs:16-374`
- `crates/codegen/xai-grok-multi-auth/src/request_auth.rs:43-141`
- `crates/codegen/xai-grok-multi-auth/src/store/file.rs:191-306`
- `crates/codegen/xai-grok-multi-auth/src/store/composite.rs:1-75`
- `crates/codegen/xai-grok-shell/src/agent/models.rs:1896-2001`
- `crates/codegen/xai-grok-pager/src/app/cli.rs:8-157`
- `crates/codegen/xai-grok-pager-bin/src/main.rs:1753-1898`
