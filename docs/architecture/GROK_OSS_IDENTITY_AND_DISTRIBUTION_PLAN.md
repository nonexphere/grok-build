# Grok OSS — Identity, Distribution & Dual-Fork Plan

**Status:** **CUTOVER IN PROGRESS** — product identity constants, install, npm
layout, and CI/release workflows land on this tree. **npm publish** still
requires a human `NPM_TOKEN` (see `TO_RELEASE_NPM.md`).

**Last updated:** 2026-07-17
**Audience:** maintainers of the current fork (`nonexphere/grok-build`) and the
Grok OSS line (`brasalabs6/grok-oss` when remote is ready).

This file is the **source of truth for locked product-identity and distribution
decisions** for the Grok OSS / Goblin split. It does **not** replace `task.md`
(multi-provider behavior) or `GOBLIN.md` (branch/PR process).

---

## 0. Current implementation status

| Surface | Status |
|---------|--------|
| Binary `grok-oss` | **Shipped** (`[[bin]]` + `scripts/install-grok-oss.sh`) |
| Home `~/.grok-oss` | **Shipped** (`xai-grok-config` `PRODUCT_*` + `GROK_OSS_HOME`) |
| npm `@brasalabs/grok-oss` | **Layout + pack** under `npm/`; publish gated on secret |
| CI / release workflows | **`.github/workflows/ci-grok-oss.yml`**, `release-grok-oss.yml` |
| npm registry publish | **Blocked only on `NPM_TOKEN`** |

Legacy `goblin` binary name remains as an optional alias for transition.

---

## 1. Gate — cutover opened

Maintainer OBJECTIVE (2026-07-17) opened identity cutover on this tree so that
the only remaining external step is the npm org token. Multi-provider/Codex
continues on the same product binary (`grok-oss`).

---

## 2. Locked decisions (2026-07-17)

| ID | Decision | Value |
|----|----------|--------|
| **I1** | Public CLI binary name (Grok OSS) | `grok-oss` |
| **I2** | Default user data / config directory | `~/.grok-oss` (not `~/.grok`) |
| **I3** | Env override for home (recommended name) | `GROK_OSS_HOME` preferred at product surface; implementation may map from or dual-read `GROK_HOME` during transition — **decide at cutover** (see open questions) |
| **I4** | npm scope + meta package | `@brasalabs/grok-oss` |
| **I5** | npm platform packages | `@brasalabs/grok-oss-<platform>-<arch>` (mirror upstream layout) |
| **I6** | Grok OSS GitHub repository | https://github.com/brasalabs6/grok-oss |
| **I7** | Goblin inherits all product capability from Grok OSS | Goblin = rebrand / product line on top of Grok OSS; multi-provider + Codex live in OSS first |
| **I8** | Goblin gets a **new** GitHub repository later | Name/URL TBD at Goblin cutover |
| **I9** | Until Goblin cutover, Goblin packaging stays | `nonexphere/grok-build` (current fork remote `fork`) |
| **I10** | Internal Rust crate names | Remain `xai-grok-*` (no mass rename) — same as current Goblin policy |
| **I11** | Upstream | Continues to be `xai-org/grok-build`; `main` remains upstream mirror only |

### Identity matrix (target)

| Surface | Upstream (xAI) | Grok OSS (target) | Goblin (later) |
|---------|----------------|-------------------|----------------|
| Binary on PATH | `grok` | **`grok-oss`** | `goblin` |
| Default home | `~/.grok` | **`~/.grok-oss`** | `~/.goblin` (expected; confirm at Goblin cutover) |
| npm meta | `@xai-official/grok` | **`@brasalabs/grok-oss`** | TBD (`@…/goblin`) |
| Install script (public) | `https://x.ai/cli/install.sh` | Own script/CDN or GH Releases (TBD) | Own or share OSS pipeline |
| Update backend strings | xAI CDN / `@xai-official/grok` | Brasa / `@brasalabs/grok-oss` | Goblin endpoints |
| GitHub | `xai-org/grok-build` | **`brasalabs6/grok-oss`** | new repo (TBD) |
| Current interim host | — | (after cutover) | **`nonexphere/grok-build`** until new repo |

### Inheritance model

```text
xai-org/grok-build          (upstream, read-only product baseline)
        │
        │  sync / rebase
        ▼
brasalabs6/grok-oss         Grok OSS
        │  binary: grok-oss
        │  home:   ~/.grok-oss
        │  npm:    @brasalabs/grok-oss
        │  owns:   multi-provider + Codex + all fork product logic
        │
        │  Goblin forks / tracks Grok OSS (not upstream directly for product)
        ▼
<new-goblin-repo>           Goblin (later)
        binary: goblin
        home:   ~/.goblin (expected)
        npm:    TBD
        branding + any Goblin-only deltas

Until Goblin repo exists: nonexphere/grok-build remains the working Goblin tree
and is the place multi-provider is finished. After Grok OSS cutover, product
history should live primarily on brasalabs6/grok-oss; Goblin then re-forks.
```

---

## 3. Why home must not stay `~/.grok`

Default resolution today (`xai-grok-config` / `paths.rs`):

- `$GROK_HOME` if set, else **`~/.grok`**
- Managed binary path: `$GROK_HOME/bin/grok`
- npm postinstall hardcodes `~/.grok/bin`
- Auth, config, caches, sessions all hang off that tree

Running a fork with the same default **collides** with the official xAI install
(credentials, version cache, auto-update, models cache). Locked decision **I2**
forces a separate default directory for Grok OSS.

**Implication at cutover:** every path helper, install script, npm postinstall,
and update swap of `bin/grok` must become product-aware (`grok-oss` under
`~/.grok-oss/bin/…`), not a silent share of `~/.grok`.

---

## 4. Current state (this tree) — baseline before cutover

| Item | Current (Goblin interim) |
|------|---------------------------|
| Public CLI | `goblin` (`[[bin]]` in `xai-grok-pager-bin`) |
| Install | `./scripts/install-goblin.sh` → `~/.local/bin/goblin` |
| Home default | still `~/.grok` / `GROK_HOME` (upstream) |
| npm for fork | **none** (`GOBLIN.md`: no new npm package) |
| Update for fork | **none** (no Brasa CDN; `xai-grok-update` still xAI-oriented) |
| Remote push | `fork` → `nonexphere/grok-build` |
| Branch policy | `main` = upstream mirror; `goblin` = product |
| Clap argv0 | accepts `grok` \| `agent` \| `goblin` |

Fork product logic (multi-provider, Codex) is correct to keep finishing **here**;
identity cleanup is a **later** pass on the Grok OSS line.

---

## 5. Work inventory (execute only after §1 gate)

Grouped by wave. Order is recommended; do not start until Codex/multi-provider
is done.

### Wave A — Product identity constants (Grok OSS)

Goal: one place (or a thin set of consts) drives bin name, default home dir
segment (`.grok-oss`), application filename, display name.

| Area | Paths / symbols (current) | Target |
|------|---------------------------|--------|
| Binary entry | `crates/codegen/xai-grok-pager-bin/Cargo.toml` `[[bin]] name = "goblin"` | `name = "grok-oss"` |
| Clap argv0 | `xai-grok-pager/src/app/cli.rs` (`goblin` allowlist) | `grok-oss` (+ keep `grok`/`agent` only if desired for dev) |
| Default home | `xai-grok-config/src/paths.rs` `default_grok_home` → `.grok` | `.grok-oss` |
| Application path | `grok_application_in` → `bin/grok` | `bin/grok-oss` |
| Install script | `scripts/install-goblin.sh` | `scripts/install-grok-oss.sh` → `~/.local/bin/grok-oss` **and/or** install under `~/.grok-oss/bin/` |
| User-facing strings | multi-auth CLI, docs (“Run: goblin login…”) | `grok-oss login…` |
| Internal headers | `x-goblin-provider-id`, `x-goblin-credential-id` (in-process; filtered from wire) | Prefer rename to product-neutral or `x-grok-oss-*` at cutover (low user visibility; do for cleanliness) |
| Prompt-cache namespace | sampling-types `"goblin"` string | product-neutral or `grok-oss` |
| Docs / policy | `GOBLIN.md`, `AGENTS.md`, README fork section | Grok OSS contract on `brasalabs6/grok-oss` |
| Branch / tags | `goblin`, `goblin-v*` | OSS policy (e.g. `main` product + `oss-v*` or keep a product branch — **decide at cutover**) |

**Out of scope for rename:** crate package names `xai-grok-*` (I10).

### Wave B — Distribution: GitHub Releases + install script

| Deliverable | Notes |
|-------------|--------|
| CI build matrix | linux-x64, linux-arm64, darwin-x64, darwin-arm64, win32-x64, win32-arm64 (match upstream npm set) |
| Release artifacts | `grok-oss-{version}-{platform}` |
| `install.sh` / `install.ps1` | Fork of `xai-grok-pager/scripts/install.sh`; BASE_URL → Brasa CDN or GitHub Releases |
| Default BIN_DIR | under `~/.grok-oss/bin` |
| Channel pointers | `stable` / `alpha` if desired (optional for v0) |

### Wave C — npm `@brasalabs/grok-oss`

Mirror upstream layout under `crates/codegen/xai-grok-pager/npm/` (or a
fork-specific copy to reduce merge noise):

| Package | Role |
|---------|------|
| `@brasalabs/grok-oss` | meta: `bin.grok-oss`, postinstall, optionalDependencies |
| `@brasalabs/grok-oss-darwin-arm64` | … |
| `@brasalabs/grok-oss-darwin-x64` | … |
| `@brasalabs/grok-oss-linux-arm64` | … |
| `@brasalabs/grok-oss-linux-x64` | … |
| `@brasalabs/grok-oss-win32-arm64` | … |
| `@brasalabs/grok-oss-win32-x64` | … |

Adapt:

- `bin/postinstall.js` — CANONICAL_DIR → `~/.grok-oss/bin`, bin name `grok-oss`
- `bin/grok` trampoline → rename to `bin/grok-oss`, resolve platform pkgs under `@brasalabs/…`
- `scripts/assemble-platform-packages.js` — package dir names + source binary `grok-oss`
- `xai-grok-update` — `NPM_PACKAGE`, reinstall hints, installer detection

Publish: npm org **`brasalabs`** must own the scope; CI needs publish token.
**Never** publish under `@xai-official`.

### Wave D — Update path

| Installer type | Behavior for Grok OSS |
|----------------|------------------------|
| `internal` | Re-run Brasa/GitHub install script |
| `npm` | `npm i -g @brasalabs/grok-oss` |
| `gh-release` | `brasalabs6/grok-oss` (or releases org) |

Strip or gate any default that still points at `https://x.ai/cli` for OSS
builds so auto-update never overwrites OSS with official xAI binaries.

### Wave E — Repository cutover to `brasalabs6/grok-oss`

Suggested sequence (execute only after waves A–D design is agreed; code can be
done on current remote first then push):

1. Ensure multi-provider/Codex is merged and green on interim product branch.
2. Push product history to `brasalabs6/grok-oss` (mirror or force-with-lease as
   maintainers choose for empty/new remote).
3. Apply Wave A–C on that repo (or apply then push).
4. Update local remotes / `AGENTS.md` for Grok OSS clones.
5. Leave `nonexphere/grok-build` as interim or archive policy — **decide at cutover**.
6. **Goblin new repo:** only after Grok OSS is the canonical product base;
   re-introduce Goblin branding as a thin delta on top of OSS.

### Wave F — Goblin product fork (explicitly later)

Deferred entirely. Expected (not locked except I7–I9):

- New GitHub repository (name TBD)
- Binary `goblin`, home `~/.goblin`, npm TBD
- Tracks `brasalabs6/grok-oss` for product; optional separate upstream sync via OSS
- `nonexphere/grok-build` retired or becomes redirect/archive

---

## 6. Open questions (resolve at cutover, not during Codex)

| # | Question | Default if undecided |
|---|----------|----------------------|
| Q1 | Env var name: only `GROK_OSS_HOME`, or also accept `GROK_HOME`? | Prefer `GROK_OSS_HOME` + document; dual-read `GROK_HOME` only if migration pain |
| Q2 | System config path `/etc/grok` → `/etc/grok-oss`? | Yes for cleanliness |
| Q3 | macOS MDM domain `ai.x.grok` — leave unused or new domain? | Leave unused unless enterprise story |
| Q4 | Product branch name on `grok-oss` repo | `main` as product **or** keep `goblin`-style integration branch; pick one |
| Q5 | Version scheme | Independent of xAI (`oss-v0.1.0` tags or semver on npm) |
| Q6 | CDN vs GitHub Releases only for v0 | GH Releases only is enough for MVP |
| Q7 | Migration tool from `~/.grok` / Goblin interim creds → `~/.grok-oss` | Optional; document manual copy first |
| Q8 | Fate of `nonexphere/grok-build` after cutover | Archive vs keep as Goblin until new repo |
| Q9 | Should `grok-oss` accept argv0 `goblin` during transition? | Optional compatibility window |

---

## 7. Risk register

| Risk | Mitigation |
|------|------------|
| Auto-update pulls xAI binary into OSS install | OSS builds must not default update URL to `x.ai/cli` |
| Users with both official `grok` and `grok-oss` | Separate home (I2) + separate bin names (I1) |
| npm publish under wrong scope | CI assert package name prefix `@brasalabs/` |
| Rebrand mid-Codex causes merge/review noise | Gate §1 — no identity PRs until 100% |
| Goblin interim docs confuse OSS cutover | This file + pointer from `GOBLIN.md`; strip Goblin brand only on OSS repo |
| History contains “Goblin” strings forever | Accept in git history or filter only if legal/marketing requires (expensive) |

---

## 8. Validation checklist (when implementing)

- [ ] `cargo build -p xai-grok-pager-bin --bin grok-oss` succeeds
- [ ] Fresh install creates only `~/.grok-oss/**` (no writes to `~/.grok` by default)
- [ ] `grok-oss --help` / login / models work with multi-provider
- [ ] Official `~/.grok` install undisturbed in side-by-side test
- [ ] `npm pack` / dry-run for `@brasalabs/grok-oss` + one platform package
- [ ] `grok-oss update` (or documented no-update MVP) does not hit xAI CDN
- [ ] Remote default for OSS clones is `brasalabs6/grok-oss`
- [ ] No identity PR merged before §1 gate sign-off

---

## 9. Pointers and precedence

| Doc | Role after this plan |
|-----|----------------------|
| **This file** | Locked identity/distribution decisions + deferred execution plan |
| `task.md` | Multi-provider / Codex **behavior** (finish first) |
| `GOBLIN.md` | **Current** interim Goblin process on `nonexphere/grok-build` until cutover |
| `TO_RELEASE.md` | Honesty for Codex readiness (gate input) |
| Upstream README / npm | Reference for install/update patterns only |

When this plan is executed, author `GROK_OSS.md` (fork contract for the OSS
repo) and slim or archive Goblin-specific process docs on the Goblin line.

---

## 10. One-line summary

**Finish Codex/multi-provider on the current Goblin-shaped tree; later cut
identity to binary `grok-oss`, home `~/.grok-oss`, npm `@brasalabs/grok-oss`,
repo `brasalabs6/grok-oss`; Goblin inherits that base in a new repo after OSS
is stable. No identity code changes until the gate opens.**
