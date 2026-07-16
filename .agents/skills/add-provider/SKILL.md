---
name: add-provider
description: >-
  Add or extend an authentication/inference provider in Goblin end to end. Use when
  implementing a new provider, completing a provider stub, adding a provider login,
  or wiring provider credentials, models, request auth, refresh, CLI/TUI, and tests.
---

# Add Provider

## When To Use

Use this skill when a Goblin change:

- adds a new `AuthProvider` implementation;
- turns an existing provider stub into working production behavior;
- adds a login transport, account type, endpoint, model catalog, or auth header for a provider;
- wires a provider through the store, token manager, session binding, sampler, CLI, or TUI;
- claims that a provider supports multi-account or concurrent parent/subagent use.

### Quando NÃO usar

- Do not use for a narrow bug in an already complete provider with no contract or
  integration change; use `@implementation-loop`.
- Do not use to review an implementation without changing it; use `@code-review` or
  `@create-report-from-code`.
- Do not use for generic API-key model configuration that does not own login,
  credentials, refresh, or provider lifecycle; follow the existing custom-model path.
- Do not use for designing the overall auth architecture from scratch; use
  `@architecture-spec-authoring` first.

## Prerequisites

Before editing:

1. Read the applicable `AGENTS.md`, `GOBLIN.md`, provider specification, protocol
   baseline, phase ledger, and this skill's
   [provider checklist](./references/provider-checklist.md).
2. Inspect `git status` and preserve unrelated/overlapping work.
3. Identify an authoritative protocol source and authorization to use it. An observed
   client ID or private endpoint is protocol evidence, not permission for production.
4. Identify the provider's login, account, token, endpoint, model, refresh, logout,
   error, configuration, storage, and rollout contracts.
5. Define one vertical-slice acceptance test: login/import → persist → bind → resolve
   request auth → inference → refresh/401 recovery.

If authorization, account identity rules, callback registration, tenancy/workspace
selection, or endpoint support is unknown, stop the affected production integration.

## Responsibility Boundary

**Do:**

- implement the provider from protocol/config through production request execution;
- reuse the provider-neutral control plane and extend generic seams where required;
- add provider-specific protocol code behind `AuthProvider`;
- prove multi-account, refresh, error, storage, CLI/TUI, and concurrency behavior;
- update canonical specs, phase status, changelog/review surfaces, and tests.

**Do not:**

- treat registry presence, a model-list response, or isolated unit tests as completion;
- copy OAuth access tokens into static `api_key` model fields;
- add process-global “current provider/account” state;
- hard-code a new provider into generic CLI/session/sampler code when registry
  descriptors/capabilities can drive it;
- enable unsupported OAuth identities/endpoints in stable/default product paths;
- claim a phase complete while acceptance criteria are absent or only mocked below the
  production composition root.

## Contract Precedence

Resolve conflicts in this order:

1. current user instruction and applicable `AGENTS.md`;
2. provider authorization/security requirements;
3. `task.md` product behavior and architectural invariants;
4. `GOBLIN.md` fork/module/feature conventions;
5. frozen provider protocol baseline;
6. phase ledger and implementation notes;
7. existing code patterns.

Code and ledger status never override a normative security or product contract. Record
intentional deviations before implementation and mark them as an **ENHANCEMENT** or
approved spec change; never silently redefine completion.

## Complementaridade

| Skill | Role | When |
| --- | --- | --- |
| `@architecture-spec-authoring` | Define/repair provider architecture and contracts | Product or cross-boundary decisions are still open |
| `@add-provider` | Execute the provider-specific end-to-end workflow | A provider is being added or completed |
| `@implementation-loop` | Implement one bounded item from the provider checklist | During each coherent code wave |
| `@code-review` | Independently verify the resulting diff | Before merge/readiness claim |
| `@create-report-from-code` | Produce phase/spec/code evidence report | At phase or final gate |
| `@issue-lifecycle` | Persist out-of-scope findings | A durable gap cannot be fixed in the current change |

Typical pipeline: architecture/spec → `@add-provider` → bounded implementation loops →
code review → evidence report → release gate.

## Workflow

### 1. Establish the provider contract

Create a provider implementation matrix from
[provider checklist](./references/provider-checklist.md). Classify every row as
`required`, `not applicable` with rationale, or `blocked`. Freeze exact HTTP method,
URL, encoding, headers, response schema, errors, retry rules, scopes, callback/device
behavior, account/workspace identity, refresh rotation, logout, and model discovery.

### 2. Decide the provider boundary

Keep generic lifecycle behavior in `xai-grok-auth` / `xai-grok-multi-auth` seams and
provider-specific wire behavior in `providers/<provider>/`. The provider must implement
the real capability it advertises. A compatibility adapter is acceptable; a stub
registered with production capabilities is not.

Use the skeleton in [provider template](./templates/provider-skeleton.md). Adapt names
and protocol details; do not copy Codex assumptions into other providers.

### 3. Implement one vertical slice first

Implement the smallest real slice in dependency order:

1. validated configuration and kill switches;
2. provider descriptor/registration with propagated registration errors;
3. one login/import transport and account identity extraction;
4. crash-consistent credential persistence;
5. immutable `ModelBinding` containing provider, credential, and model identity;
6. request-time token resolution and provider endpoint/header construction;
7. production sampler/client consumption;
8. refresh rotation and generation-stamped one-retry 401 recovery;
9. one end-to-end test through the production composition root.

Do not broaden to TUI, additional transports, or several account UX paths until this
slice passes.

### 4. Complete lifecycle and multi-account behavior

Add remaining transports, refresh classifications, revocation/logout, account metadata,
models/cache/fallback, and provider-specific errors. Then prove two accounts with the
same model IDs remain separately selectable. Account selection must be explicit in a
binding or resolved once from a provider default at session creation; never let catalog
order choose credentials.

### 5. Integrate generic product surfaces

Drive provider discovery and available actions from registry descriptors and
capabilities. Implement provider/account CLI grammar, non-TTY behavior, machine output,
TUI flows, session restore, agent profiles, and subagent overrides. Preserve legacy xAI
and custom/API-key models through adapters and regression tests.

### 6. Harden storage, security, and operations

Complete keyring/fallback policy, permissions/ACLs, crash consistency, cross-process
locking, redaction, callback safety, configuration gates, metrics without secrets,
diagnostics, migrations, and rollout switches. Exercise every sensitive persistence
edge with fault injection. Never log tokens, codes, verifiers, device IDs, cookies, or
raw sensitive upstream bodies.

### 7. Validate by evidence, not module presence

Run the repository commands plus the full checklist. Required proof includes unit,
wire, integration, concurrency, cross-process, CLI/TUI (when implemented), migration,
and protected real-protocol validation where authorized. Search the production code to
prove that the composition root consumes `ModelBinding`, request resolver, and token
manager; isolated tests of unused abstractions do not count.

### 8. Reconcile documentation and hand off

Update specs, protocol baseline, feature/config docs, phase ledger, changelog, and
release gates. A phase has one canonical status. Map requirement → file/symbol → test →
command result → commit. Report missing external authorization separately from code
completion. Never convert `blocked` or `partial` to `done` for narrative convenience.

## Stop Conditions

Stop and request direction when:

- provider/client/endpoint authorization is absent or explicitly unresolved;
- callback URI, tenant/workspace identity, account-switch semantics, or secret-storage
  policy requires a human product/security decision;
- changing a shared API/schema/session contract has unverified downstream consumers;
- crash-consistent refresh rotation cannot be guaranteed with the selected store;
- existing user changes overlap the same files and cannot be safely preserved;
- a real protocol test would require secrets, a human account, browser scraping, or an
  external mutation beyond the user's authorization;
- the same failure survives two evidence-based fix attempts.

Do not stop for an independently blocked transport or UI surface: finish other safe,
in-scope checklist items and report the remaining blocker precisely.

## Conventions

- Provider IDs are validated, stable lowercase identifiers; display names are UI only.
- Credential identity is `(ProviderId, CredentialId)`; aliases are not runtime keys.
- Request identity is immutable `(provider, credential, model)` for the in-flight request.
- Resolve tokens immediately before a request; never cache them in model config/session
  serialization or public status output.
- Refresh locking is per credential and must work across tasks and processes.
- A 401 retry occurs at most once and only if the sent generation is still current or a
  controlled refresh succeeds.
- Provider model cache keys include provider and credential/account identity.
- Reserved authentication headers cannot be overridden by model/user configuration.
- Capabilities describe working behavior, not roadmap intent.
- Findings use severity and confidence separately, e.g. `[BLOCKER][Confirmed]`.

## Common Mistakes

These mistakes are confirmed by the Goblin implementation review at
`docs/architecture/multi-provider-auth/reports/2026-07-16-implementation-review.md`
(findings B1–B6 and M1–M14); they are not hypothetical style preferences.

- **Static-token bridge:** copying an OAuth token into `ModelEntry.api_key` makes login
  appear integrated while bypassing refresh and request binding.
- **Model-ID account collision:** using `provider/model` for several accounts silently
  selects the first account.
- **Registered stub:** listing a provider whose lifecycle methods return “handled by
  legacy path” is not a provider adapter.
- **Dead resolver:** unit-testing a request resolver that the sampler never calls.
- **Atomic-file illusion:** atomically writing metadata and secrets separately is not an
  atomic credential transaction during refresh-token rotation.
- **Observed-client default:** treating a discovered OAuth client ID as permission to
  enable production login.
- **Hard-coded generic UI:** adding enum variants and match arms for each provider makes
  every future provider cross-cut the CLI/runtime.
- **Checkbox completion:** calling a wave done because modules/tests exist while its
  acceptance scenarios and production composition are absent.

## Verification

Complete every applicable item in
[provider checklist](./references/provider-checklist.md). At minimum, verify:

- `git diff --check` and repository formatting/lint/type/build commands pass;
- provider contract/unit/wire tests pass with exact request assertions;
- the production client uses request-time resolution (repository search + integration test);
- expired-token and 401 flows refresh, persist rotation, and retry at most once;
- two accounts with the same model remain independently selectable;
- parent/subagent mixed-provider and mixed-account requests are concurrent and isolated;
- storage survives injected failures without token-generation inconsistency;
- status/errors/logs/telemetry contain none of the seeded test secrets;
- legacy xAI/custom model/auth files remain compatible;
- kill switches and non-TTY CLI behavior are tested;
- OAuth authorization and real smoke evidence are explicitly proven or marked blocked;
- documentation, phase status, test evidence, and commits agree.

The provider is complete only when the production composition root—not just isolated
provider modules—passes the end-to-end gate.

## Provenance

Type: project-scoped `engineering-pattern` with an end-to-end workflow. Reference:
the multi-provider/Codex implementation reviewed on 2026-07-16, plus the normative
architecture in `task.md` and fork contract in `GOBLIN.md`. Rules that directly enforce
those contracts are **REPRODUCTION**. The vertical-slice ordering, single canonical
evidence matrix, and explicit production-composition search are **ENHANCEMENTS** added
to prevent the implementation and ledger failures documented in that review.
