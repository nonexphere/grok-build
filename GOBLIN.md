# Goblin Fork Contract

## Purpose

Goblin is a fork of `xai-org/grok-build` that adds **native multi-provider
authentication** to the `grok` CLI/TUI. The first external provider is
**Codex / ChatGPT OAuth**, implemented entirely inside Grok-build with no
dependency on the Codex CLI binary or `~/.codex` directory.

The fork exists to evolve Grok-build from a predominantly xAI-scoped
authentication system into a provider-neutral authentication control plane
that supports multiple simultaneous providers, multiple accounts per
provider, and request-scoped token resolution.

## Branches

| Branch | Role |
|--------|------|
| **`main`** | **Upstream mirror** of `xai-org/grok-build`. Keep as a clean mirror. **Never land Goblin-specific commits on `main`.** Never open feature PRs into `main`. |
| **`goblin`** | **Principal / default development branch** of this fork. Features, CI, releases, and documentation land here. **All product PRs target `goblin`.** |
| **`goblin-*`** | Feature branches. PR **into `goblin` only**. |

### Remotes (this clone)

| Remote | Repository |
|--------|------------|
| `origin` | `xai-org/grok-build` (upstream) |
| `fork` | `nonexphere/grok-build` (push + PRs) |

### Sync workflow

1. Update local `main` from `origin/main` (prefer updating the ref without
   checkout if another agent owns the worktree: `git branch -f main origin/main`).
2. Push mirror: `git push fork main --force-with-lease` so `fork/main` matches upstream.
3. Rebase or merge `goblin` onto the updated `main` (after the fork has unique
   history — do not blind-reset `goblin` to `main` once it has product merges).
4. Resolve conflicts in fork-only files (this file, `docs/architecture/`,
   `task.md`) in favor of the fork, unless upstream changed the same line.
5. Never push Goblin product commits to `main`.
6. Open PRs with base **`goblin`** — skill `@create-pr`.

Also see root [`AGENTS.md`](AGENTS.md).

## Stable Base

- Goblin releases start from the latest stable upstream snapshot unless a
  maintainer explicitly chooses an alpha, beta, or development snapshot.
- The current open-source snapshot is the starting point for Wave 0. The
  exact upstream tag/commit will be recorded here when the first Goblin
  release is cut.
- The first Goblin release tag will be `goblin-v0.1.0` (see Release Tags
  below).

## Public Surface

- **Public fork CLI command: `goblin`** (installed onto `PATH` for users of this
  fork). Build with `cargo build -p xai-grok-pager-bin --bin goblin` or
  `./scripts/install-goblin.sh` (see **Local build & test loop** below).
- Upstream artifact name `xai-grok-pager` is still produced to minimize merge
  drift; `goblin` is an additional `[[bin]]` entry pointing at the same
  `main.rs`. Clap accepts `goblin` as `argv[0]` for help/usage.
- Internal Rust crate names remain `xai-grok-*` (no renames).
- No new npm package is introduced by this fork.

### Local build & test loop

Rust does not hot-reload the TUI: after code changes that affect the CLI, you
must **rebuild the binary** (or use a wrapper that always execs the latest
`target/` artifact). `cargo check` / `cargo test -p …` prove libraries; they
do **not** update the interactive `goblin` you run.

#### Canonical commands

```bash
# From repo root (this clone):
cd ~/github/grok-goblin   # or your checkout path

# 1) Fastest day-to-day binary (debug, incremental)
cargo build -p xai-grok-pager-bin --bin goblin

# 2) Run the just-built binary (always correct for this checkout)
./target/debug/goblin --version
./target/debug/goblin --model gpt-5.6-luna

# 3) Install / refresh PATH helper (wrapper prefers newest target/* build)
PROFILE=debug ./scripts/install-goblin.sh
goblin --version   # should match: grok … (<git-sha>) …

# Optional: slower optimized binary (only when you care about runtime speed)
cargo build -p xai-grok-pager-bin --bin goblin --release
./target/release/goblin --version
```

#### What each mode is for

| Command | Produces runnable TUI? | Typical use |
|---------|------------------------|-------------|
| `cargo check -p xai-grok-shell --lib` | No | Agent / CI typecheck |
| `cargo test -p xai-grok-sampler --lib` | No | Unit proof |
| `cargo build -p xai-grok-pager-bin --bin goblin` | **Yes** → `target/debug/goblin` | **Human test after code changes** |
| `… --release` | **Yes** → `target/release/goblin` | Perf / handoff install |
| `PROFILE=debug ./scripts/install-goblin.sh` | **Yes** + PATH wrapper | Daily PATH = latest debug/release in `target/` |

#### PATH wrapper behavior (`scripts/install-goblin.sh`)

The install script writes `~/.local/bin/goblin` as a **small launcher**, not a
stale copy of the binary. On each invocation it picks, in order of preference:

1. Newest among `target/debug/goblin` and `target/release/goblin` under the
   repo path baked into the wrapper (this clone).
2. Else a last-built copy under `~/.local/lib/goblin/goblin-pager` if present.

So the loop is: **edit → `cargo build -p xai-grok-pager-bin --bin goblin` →
`goblin …`** without reinstalling every time. First-time setup still needs
`PROFILE=debug ./scripts/install-goblin.sh` once so the launcher is on PATH.

Confirm you are on the right binary:

```bash
goblin --version
# expect: grok 0.2.xxx (<short-sha of HEAD>) [stable]
```

If the SHA does not match `git rev-parse --short HEAD`, rebuild (or you are
running an old install outside this wrapper).

#### Speed notes (monorepo reality)

- **Debug + incremental** is the default (`[profile.dev]` in workspace
  `Cargo.toml`: `incremental = true`, high `codegen-units`). Do **not** use
  `--release` for the daily test loop.
- First / cold build of the full CLI is slow (many crates + large final link).
  Warm incremental builds only recompile what changed.
- Do not delete `target/` casually; it is the incremental cache.
- Optional accelerators (not required): `mold` as linker, `sccache` for
  compiler cache — install at the OS user level and point
  `~/.cargo/config.toml` at them if desired.
- Agents validating product behavior for a human tester should end the slice
  with a **binary** build (`cargo build -p xai-grok-pager-bin --bin goblin`),
  not only `cargo check`.

### Install

```bash
PROFILE=debug ./scripts/install-goblin.sh   # recommended for dev (wrapper + debug build)
# or release install (slower build, faster runtime):
./scripts/install-goblin.sh                 # PROFILE=release by default

goblin login --provider codex
goblin auth status
goblin logout
```

## Architecture Contract

The following decisions (D1–D10) are binding for this fork. They are sourced
from `task.md` §1.2 and are the authoritative product-behavior specification.

| ID  | Decision                                                                                                                      |
| --- | ----------------------------------------------------------------------------------------------------------------------------- |
| D1  | All login flows execute inside Grok-build. No shelling out to `codex login`.                                                  |
| D2  | The Codex CLI binary and `~/.codex` directory are never required.                                                             |
| D3  | Provider and account selection is immutable for the lifetime of an in-flight request.                                         |
| D4  | Refresh synchronization is per credential, not process-global.                                                                |
| D5  | Existing xAI credentials remain usable without mandatory migration.                                                           |
| D6  | Built-in providers are registered at compile time in version 1.                                                               |
| D7  | The Codex device flow is implemented as a provider-specific protocol, not incorrectly forced into generic RFC 8628 semantics. |
| D8  | Keyring storage is preferred; secure file storage remains available for headless systems.                                     |
| D9  | Codex model availability comes from its authenticated `/models` endpoint with a bundled/cache fallback.                       |
| D10 | The observed Codex OAuth client ID is documented but must not be treated as authorization for third-party production use.     |

## Module Layout

New code is added primarily under the existing `xai-grok-auth` crate and a
new `auth/` module tree inside `xai-grok-shell`. Prefer new files over
editing upstream files to reduce merge conflicts.

```text
crates/codegen/xai-grok-auth/
└── src/
    ├── lib.rs
    ├── auth_provider.rs              # Existing HTTP seam (unchanged)
    ├── provider.rs                   # New control-plane AuthProvider trait
    ├── types.rs                      # ProviderId, CredentialId, CredentialKey, ModelBinding
    ├── login.rs                     # LoginFlow, LoginStart, LoginInput, LoginCompletion
    ├── credential.rs                 # CredentialMetadata, CredentialSecret
    ├── request_auth.rs              # RequestAuthResolver trait
    └── errors.rs                     # ProviderError, StoreError, AuthError

crates/codegen/xai-grok-shell/src/auth/
├── mod.rs
├── registry.rs                      # ProviderRegistry
├── command_service.rs                # AuthCommandService
├── login_coordinator.rs             # LoginCoordinator
├── token_manager.rs                  # TokenManager
├── model_binding.rs                  # ModelResolver
├── migration.rs
├── compatibility/
│   ├── mod.rs
│   └── legacy_xai.rs                 # LegacyXaiCredentialStore adapter
├── store/
│   ├── mod.rs                        # CredentialStore trait
│   ├── metadata.rs
│   ├── file.rs
│   ├── keyring.rs
│   ├── encrypted_file.rs
│   ├── lock.rs
│   └── composite.rs                  # CompositeCredentialStore
└── providers/
    ├── mod.rs
    ├── xai.rs                        # XaiAuthProvider
    └── codex/
        ├── mod.rs                    # CodexAuthProvider
        ├── config.rs
        ├── browser.rs
        ├── callback.rs
        ├── device.rs
        ├── token.rs
        ├── claims.rs
        ├── models.rs
        ├── request_auth.rs
        ├── errors.rs
        └── fixtures/                 # Frozen wire fixtures
```

After stabilization, provider implementations may be extracted into
`xai-grok-auth-xai` and `xai-grok-auth-codex` crates.

## Feature Flags

### Compile-time

```text
native-multi-provider-auth
native-codex-auth
auth-keyring
auth-encrypted-file
```

### Runtime (config)

```toml
[features]
multi_provider_auth = true
codex_provider = true
codex_browser_login = true
codex_device_login = true
```

### Environment kill switches

```text
GROK_DISABLE_CODEX_AUTH=1
GROK_DISABLE_CODEX_BROWSER_LOGIN=1
GROK_DISABLE_CODEX_DEVICE_LOGIN=1
```

When Codex auth is fully disabled, the provider disappears from the login
UI. Existing sessions show a clear provider-disabled state.

## Credential Layout

The new multi-provider store is **additive** to the existing `auth.json`.
Existing xAI credentials continue to live in `~/.grok/auth.json`; new
providers use the structured layout below.

```text
~/.grok/
├── config.toml
├── auth.json                         # Existing xAI legacy store (unchanged)
├── auth/
│   ├── accounts.json                 # Non-secret metadata (all providers)
│   ├── accounts.json.lock
│   ├── file-secrets.json             # Only file/encrypted-file backend
│   ├── file-secrets.json.lock
│   ├── migration.json
│   └── locks/
│       ├── xai/
│       │   └── <credential-id>.lock
│       └── codex/
│           └── <credential-id>.lock
└── cache/
    └── models/
        ├── xai/
        └── codex/
            └── <credential-id>.json
```

## Sync Playbook and Conflict Resolution

1. **Fetch:** `git fetch origin && git fetch fork`
2. **Update main mirror (safe without dirty-checkout):**
   `git branch -f main origin/main && git push fork main --force-with-lease`
   (`fork/main` is allowed to force-update **only** as an upstream mirror.)
3. **Advance goblin:** on a clean tree or isolated worktree:
   `git checkout goblin && git rebase main` (or `git merge main`).
4. **Conflict resolution rules:**
   - Files that exist only in the fork (`GOBLIN.md`,
     `docs/architecture/multi-provider-auth/*`, `task.md`): keep the fork
     version unless the upstream change is a direct improvement to a shared
     area.
   - Shared source files: prefer upstream changes; re-apply fork-specific
     additions on top.
   - `README.md`: keep the upstream body intact; preserve the "Fork (Goblin)"
     section and branch policy table.
5. **Force-push:** allowed for `fork/main` only as mirror sync. Force-push
   `goblin` only during deliberate rebase cleanup before a release, and only
   if no collaborators are mid-rebase. Feature force-push: only that feature
   branch, preferably from an isolated worktree.
6. **PRs:** always `--base goblin` (never `--base main`). See `@create-pr`.

## Security Rules

- **No tokens in logs.** Access tokens, refresh tokens, ID tokens,
  authorization codes, device auth IDs, and PKCE verifiers must never appear
  in log output, telemetry, or error messages.
- Use `secrecy::SecretString` or equivalent for all secret values.
- Avoid `Debug` implementations that expose secrets.
- Redact known sensitive JSON fields before logging structured errors.
- JWT payload parsing is not signature validation. Use unverified claims
  only for display, routing, and expiration hints — never for local
  authorization.
- Callback servers must bind only to loopback (`127.0.0.1`), never
  `0.0.0.0`.
- File-backed secrets must use `0600` permissions on Unix and owner-only
  ACLs on Windows.

## Release Tags

- Goblin release tags use the `goblin-v*` format, e.g. `goblin-v0.1.0`.
- Tags are created on the `goblin` branch.
- The upstream `rust-v*` / version scheme is not reused; Goblin versions
  are independent to avoid confusion with upstream releases.

## Source of Truth

- **`GOBLIN.md`** (this file) is the source of truth for branch policy,
  sync workflow, module layout, feature flags, and release tagging
  **while this interim fork remains the product tree**
  (`nonexphere/grok-build`).
- **`task.md`** is the source of truth for product behavior, the full
  architecture specification (D1–D10, component specs, protocol baseline,
  CLI/TUI flows, configuration format, security model), and the
  implementation plan.
- **Deferred identity / dual-fork (Grok OSS → Goblin):** locked decisions and
  post-Codex cutover plan live in
  [`docs/architecture/GROK_OSS_IDENTITY_AND_DISTRIBUTION_PLAN.md`](docs/architecture/GROK_OSS_IDENTITY_AND_DISTRIBUTION_PLAN.md).
  **Do not implement** that plan until multi-provider/Codex is 100% and a
  maintainer opens the gate. Summary of locked targets: binary `grok-oss`,
  home `~/.grok-oss`, npm `@brasalabs/grok-oss`, repo
  `https://github.com/brasalabs6/grok-oss`; Goblin later gets a new repo and
  inherits Grok OSS.
- Where this file and `task.md` disagree on product behavior, `task.md`
  wins. Where they disagree on fork process (branches, tags, sync),
  this file wins until the Grok OSS cutover.
