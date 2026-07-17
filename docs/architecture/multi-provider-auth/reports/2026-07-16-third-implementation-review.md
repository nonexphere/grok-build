---
title: "Third multi-provider implementation review: late progress and regressions"
date: "2026-07-16"
slug: "third-implementation-review"
status: "relevant"
status_reason: "Current evidence-based recheck after additional Codex payload, header, and reasoning-effort changes."
last_reviewed_at: "2026-07-16"
source_path_legacy: "docs/architecture/multi-provider-auth/reports/2026-07-16-third-implementation-review.md"
---

# Third multi-provider implementation review: late progress and regressions

> Type: `full-plan-review` / third incremental readiness review
> Baseline: `2026-07-16-second-implementation-review.md`
> Scope: current uncommitted worktree over `b189869`
> Reviewer: Codex, directly; no subagents

## 1. Verdict

**NOT READY.** The other agent advanced the Codex happy path, but did not close the
four architectural blockers that were explicitly prioritized in the second review.
One of those four, the internal credential-ID wire leak, is now contained at the HTTP
boundary. The other three remain unchanged. The planned xAI adapter also remains an
explicit stub.

The recent work added:

- filtering of `x-goblin-credential-id` before HTTP headers are emitted;
- Codex-specific conversion of system/developer input into Responses `instructions`;
- merge-order and reasoning-effort regression tests;
- a test override allowing a synthetic Codex report to enter the model catalog.

This is useful runtime progress, but it is not release-readiness progress in the
highest-risk areas. The worktree now has **1,309 tracked-line additions across 20
modified tracked files**, plus the new auth crates/files and documentation, still with
no implementation commit boundary. The agent is spending substantial time compiling
a shell integration test while the request identity, refresh serialization, journal
recovery, and xAI scope decisions remain unresolved.

## 2. Delta from the second review

| Second-review item | Current state | Verdict |
| --- | --- | --- |
| NB1 exact request owns 401 stamp | global `LAST_SENT_STAMPS` still keyed only by credential | **BLOCKER open** |
| NB2 shared manager + cross-process refresh lock | store/registry/manager still recreated per resolve/recovery | **BLOCKER open** |
| NB3 safe journal recovery | constructor still recovers unlocked and discards error | **BLOCKER open** |
| NB4 internal credential UUID sent upstream | sampler now filters the internal header | **Wire leak fixed; architecture partial** |
| NB5 real xAI adapter or explicit scope decision | provider still labels itself a minimal stub | **BLOCKER open** |
| Typed immutable model binding | credential ID still travels through generic `extra_headers` internally | **MAJOR open** |
| Provider-sourced effort capabilities | all Codex entries still receive a hard-coded effort menu | **MAJOR open** |

The readiness count should therefore not be presented as green: **one blocker was
fixed at the wire boundary; four blocker-level release conditions remain open.**

## 3. Confirmed blocker findings

### T3-B1 — Late 401 recovery still uses another request's credential generation

`xai-grok-multi-auth/src/token_resolve.rs:26-28` keeps one last stamp per credential.
Every request overwrites it at `:88-94`; recovery reads the current map value at
`:116-131`. Concurrent requests cannot associate their own sent generation with their
own 401 response. This is the same stale-401 race documented in the second review.

Required proof remains absent: two overlapping requests using different generations,
with the older request returning 401 after the newer request has sent, must not refresh
or invalidate the newer credential.

### T3-B2 — Refresh single-flight remains instance-local and ineffective across calls/processes

`make_store_and_manager` at `token_resolve.rs:30-37` creates a fresh file store,
registry, and `TokenManager` on every resolution and recovery. Consequently the
manager's in-memory per-credential lock is not shared. The production refresh path
still does not acquire the store's cross-process credential lock before calling the
provider.

CAS after the upstream call is insufficient for rotating refresh tokens: two callers
may already have submitted the same one-use refresh token. Required proof remains
absent: two independent managers/processes must produce exactly one upstream refresh.

### T3-B3 — Store recovery is still unlocked, repeated, and error-suppressing

`xai-grok-multi-auth/src/store/file.rs:39-44` still invokes
`recover_pending_txn(&paths)` in every constructor and assigns the result to `_`.
Because production repeatedly constructs stores, recovery can race an active commit.
No transaction generation/base-state validation prevents stale journal replay.

Required proof remains absent: fault injection after each write/rename/fsync/remove,
plus a concurrent constructor/writer test and propagated recovery failures.

### T3-B4 — The planned xAI provider implementation remains an explicit registry stub

`providers/xai.rs:1-6` calls the provider a minimal stub. Login, refresh, token
resolution, logout, endpoint resolution, and request-auth construction all return
`InvalidConfig` (`:72-154`). Empty advertised capabilities are honest, but do not
deliver Phase 4. This must be either implemented or explicitly removed/deferred by a
human-approved plan change; documentation cannot call it complete as a “legacy
boundary”.

## 4. Newly introduced or newly confirmed major findings

### T3-M1 — Codex message hoisting silently discards non-text system/developer content

`hoist_system_messages_to_instructions` removes every system/developer item from the
input. Its extraction retains only `InputText`; images, files, or any other input
content are ignored. If no text is found, the original message is still removed. This
is confirmed data loss, not just a missing test.

Required fix: reject unsupported content explicitly or preserve a lossless supported
representation. Never remove a message merely because extraction produced no text.
Add mixed-content and non-text-only tests for both EasyMessage and typed Message forms.

### T3-M2 — Backend detection is an unsafe substring heuristic

`is_codex_responses_backend` returns true whenever a lowercased URL contains
`chatgpt.com` or `/codex`. It therefore matches unrelated hosts such as
`evilchatgpt.com`, query strings, and arbitrary custom proxy paths. Provider-specific
payload semantics must follow an explicit provider/protocol binding, or at minimum a
parsed and allowlisted origin/path contract.

### T3-M3 — System and developer roles are flattened without role boundaries

The helper concatenates both roles into one `instructions` string separated only by
blank lines. This loses their distinct role/precedence information and can alter prompt
semantics. Tests cover only a simple system-text case and no-system no-op; they do not
cover developer precedence, interleaving, existing instructions, tools, or multiple
messages.

### T3-M4 — Provider protocol logic is leaking into generic sampling code

Both the internal-header exception and URL-driven Codex payload rewrite live in the
generic sampler/sampling-types layer. This hard-codes Goblin/Codex knowledge into the
transport instead of using `ProviderRequestAuth` and an explicit provider payload
adapter. It makes the next provider harder to add and contradicts the purpose of the
new provider abstraction.

### T3-M5 — A test-only mutable report override ships in the production library

`agent/models.rs:2190-2215` adds thread-local override state and public
`*_for_test` functions without `#[cfg(test)]`, because the integration test imports
them from the built library. This expands production API/state solely to facilitate a
test. Test the pure merge/order functions directly, or inject the report through a
real dependency boundary available in all builds.

### T3-M6 — Reasoning-effort support remains asserted rather than discovered

`agent/models.rs:2066` and `:2164` assign the same low/medium/high/xhigh capability
menu and Medium default to every Codex model. The new tests only prove that this
self-authored default is merged and overridden in the intended order; they do not
prove that each returned provider model accepts those values. Capability evidence must
come from the authenticated catalog or a versioned, provider-confirmed compatibility
table.

### T3-M7 — Internal binding is still encoded as a generic header

Filtering `x-goblin-credential-id` at `sampler/client.rs:436-449` fixes the immediate
privacy/wire issue. However, `agent/models.rs:2155` still stores credential identity in
`extra_headers`, and resolution still reads it back heuristically. The correct
end-state remains a typed, immutable, non-wire `ModelBinding` carried through session,
agent/subagent, auth resolution, and retry.

## 5. Test and evidence assessment

| Evidence | Result | Interpretation |
| --- | --- | --- |
| `git diff --check` | **PASS** | no whitespace errors in current diff |
| focused `hoist_system` cargo test | **still compiling after >8 minutes** | no pass/fail evidence yet; compiling a very large dependency graph for two narrow tests |
| new `codex_effort_after_merge` tests | source reviewed; result not independently established in this pass | tests merge order, not provider capability truth |
| stale-401 concurrent request test | **missing** | B1 unproven |
| two-manager/two-process refresh test | **missing** | B2 unproven |
| crash-boundary/concurrent journal tests | **missing** | B3 unproven |
| HTTP wire assertion for internal header | **missing** | implementation inspection supports fix, but regression coverage is absent |
| mixed/non-text Codex system content tests | **missing** | confirmed data-loss path uncovered by review |
| exact-origin backend routing tests | **missing** | false-positive routing remains possible |
| real xAI adapter contract tests | **missing/not applicable to stub** | B4 remains open |

Repeated green unit tests cannot substitute for the missing concurrency, process,
fault-injection, and wire-boundary evidence. The current long compilation should not be
used as a reason to postpone the invariant-level work.

## 6. Plan and execution-quality assessment

The original plan was directionally good: provider abstraction, credential-scoped
storage, request-scoped auth, generation-aware recovery, cross-process serialization,
and explicit provider adapters are the correct decomposition. The implementation has
followed the visible surfaces—types, CLI, catalog, login, and a production retry
branch—but repeatedly replaced the plan's invariants with local bridges:

- request state became a global last-value map;
- a shared coordinator became a newly constructed manager per call;
- atomic multi-file persistence became an unlocked constructor replay;
- typed binding became a private header filtered at the last moment;
- provider protocol adaptation became URL substring checks in a generic sampler;
- a required xAI adapter became a registered empty-capability stub.

The immediate problem is therefore not lack of activity. It is prioritization and
closure discipline: convenience/runtime symptoms receive new code and tests while the
explicit blocker invariants remain unchanged.

## 7. Required next sequence

1. Carry a request-owned `SentCredentialStamp` from auth resolution through the exact
   in-flight request and pass it explicitly into 401 recovery; remove global stamp
   semantics.
2. Install one composition-root-owned manager/store coordinator and take the
   cross-process credential lock before reloading/calling refresh; prove one upstream
   refresh across independent instances.
3. Redesign journal recovery under the same transaction locks, propagate failures,
   add durability/base-state validation, and execute crash/race tests.
4. Replace header/URL inference with typed `ModelBinding` and an explicit provider
   request/payload adapter. Keep the new wire-header filter as defense in depth.
5. Make Codex system-message conversion lossless or explicitly rejecting, preserve
   role semantics, and add wire fixtures for official backend routing.
6. Obtain the human scope decision for xAI: implement the adapter or mark the phase
   deferred with specs/TODO/readiness synchronized.
7. Remove production test hooks and source effort capabilities from an authoritative
   provider contract.
8. Only then run the focused auth/concurrency/fault/wire suites, followed by workspace
   checks, and create reviewable commits.

## 8. Release gate

Do not merge or release the multi-provider/Codex functionality while T3-B1 through
T3-B4 remain open. The Codex happy path may appear closer to working, but concurrent
401s, rotating refresh tokens, crash recovery, and the unimplemented xAI contract are
still correctness boundaries. The newly added message conversion additionally needs a
losslessness fix before it is safe for general use.
