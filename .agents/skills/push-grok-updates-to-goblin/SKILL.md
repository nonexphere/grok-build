---
name: push-grok-updates-to-goblin
description: >-
  Update the Goblin fork (nonexphere/grok-build) from upstream xai-org/grok-build
  to the latest monorepo sync or a requested upstream commit/SHA. Use when asked
  to bring Goblin forward to a new upstream release, sync fork/main mirror,
  rebase goblin or feature branches onto origin/main, resolve upstream conflicts
  while preserving multi-provider auth / goblin CLI, open or update an upgrade
  PR, or explain the grok-build → Goblin upgrade procedure.
---

# Push Grok Updates To Goblin

## Objective

Move `nonexphere/grok-build` (Goblin) forward from its current upstream base to
the latest (or requested) `xai-org/grok-build` snapshot while:

1. keeping **`main` as a clean upstream mirror**,
2. advancing **`goblin`** (fork principal) via reviewable rebase/merge,
3. preserving the Goblin fork contract (multi-provider auth, `goblin` CLI, docs),
4. leaving a Draft PR + validation evidence.

Use this skill for both:

- "Atualiza o Goblin com o último upstream do grok-build."
- "Sincroniza o fork com `origin/main` / SHA `<sha>`."

Sibling skill in the Codex fork world: `push-codex-updates-to-goblin`
(`~/brainstorm/goblins/.codex/skills/…`). Same intent, different upstream.

## Non-Negotiables

- Verify "latest" before assuming it. Prefer `origin/main` after fetch; record
  tip SHA, commit subject, and `SOURCE_REV` when present.
- **Never land Goblin product commits on `main`.** `main` is mirror-only.
- **All product PRs target `goblin`**, never `main`. See `@create-pr`.
- Preserve Goblin public surface:
  - CLI command **`goblin`** (`xai-grok-pager-bin` `[[bin]]`)
  - crate names `xai-grok-*` (no renames)
  - multi-provider control plane (`xai-grok-multi-auth`, `xai-grok-auth` extensions)
  - feature `native-multi-provider-auth` (default-on for this fork)
  - docs: `GOBLIN.md`, `docs/architecture/multi-provider-auth/**`, skill `add-provider`
- Use a **dedicated worktree and branch**. Do **not** rebase, reset, stash, or
  resolve conflicts in a worktree another agent is using.
- Make destructive Git ops visible before running them. Prefer
  `--force-with-lease` only on dedicated upgrade/mirror refs after intentional
  rewrite-aware sync.
- Upstream may **force-update** `main` (unrelated histories / re-publish). Detect
  that; never treat `merge --allow-unrelated-histories` noise as the semantic
  conflict list.
- Record validation and residual risks in the PR body (and session artifacts if
  the repo uses them).

## Remotes And Branches

| Remote | Repository | Role |
|--------|------------|------|
| `origin` | `https://github.com/xai-org/grok-build.git` | Upstream (read) |
| `fork` | `git@github.com:nonexphere/grok-build.git` | Our fork (push + PRs) |

| Branch | Role |
|--------|------|
| **`main`** | Upstream mirror only. `fork/main` must match `origin/main`. |
| **`goblin`** | Principal / integration branch of the fork. PR base. |
| **`goblin-*` / `upgrade/*`** | Feature or upgrade work. PR **into `goblin`**. |

```text
xai-org/main  ──sync──►  fork/main  (mirror only)
                              │
                         fork/goblin  (fork principal)
                              ▲
                    upgrade / feature PRs
```

If remotes are inverted or missing, fix naming before any push:

```bash
git remote -v
# expect: origin → xai-org/grok-build, fork → nonexphere/grok-build
```

## Release / Tip Resolution

Upstream open-source tree is periodically **synced from the monorepo** (often
one tip commit like `Synced from monorepo`, sometimes via force-update). There
is typically **no** `rust-v*` style release tag. Treat **`origin/main` tip** as
the release unless the user names an exact SHA.

### Latest

```bash
git fetch origin --prune
git fetch fork --prune

UPSTREAM_TIP=$(git rev-parse origin/main)
git log -1 --format='%H%n%s%n%ci' origin/main
git show origin/main:SOURCE_REV 2>/dev/null || true
git rev-parse fork/main fork/goblin 2>/dev/null || true
```

### Explicit SHA

```bash
TARGET_SHA=<sha>
git fetch origin --prune
git rev-parse --verify "${TARGET_SHA}^{commit}"
git merge-base --is-ancestor "${TARGET_SHA}" origin/main \
  || echo "WARNING: TARGET_SHA is not on origin/main; confirm intent"
```

If the user said "latest" but `origin/main` moved via force-push relative to
`fork/main` (no common merge-base, or tip rewrite), **stop and state that**
before rewriting the mirror.

```bash
git merge-base fork/main origin/main >/dev/null 2>&1 \
  || echo "NO common merge-base: upstream history rewrite likely"
git log --oneline -3 fork/main origin/main
```

Do not guess the target tip. Do not sync from a dirty local checkout of `main`.

## Setup

1. Read contracts first: `GOBLIN.md`, `AGENTS.md`, `.agents/skills/create-pr/SKILL.md`,
   and any prior upgrade PR comments / session notes.
2. Fetch remotes (above).
3. Identify current fork bases:

```bash
git rev-parse origin/main fork/main fork/goblin 2>/dev/null
git log --oneline --decorate --graph --max-count=20 origin/main fork/goblin 2>/dev/null
```

4. Create a dedicated worktree (never the agent-busy product worktree):

```bash
UPSTREAM_TIP=$(git rev-parse --short origin/main)
BRANCH=upgrade/grok-${UPSTREAM_TIP}
WORKTREE=/tmp/goblin-upgrade-grok-${UPSTREAM_TIP}

# Start from current fork integration tip when it exists; else from origin/main
if git rev-parse --verify fork/goblin >/dev/null 2>&1; then
  BASE_REF=fork/goblin
else
  BASE_REF=origin/main
fi

git worktree add -b "${BRANCH}" "${WORKTREE}" "${BASE_REF}"
```

All rebase/conflict work happens in `${WORKTREE}` only.

## Phase A — Mirror `main` (no product commits)

Update the local ref and push the mirror. Prefer updating the ref **without**
checking out `main` in a shared worktree:

```bash
git branch -f main origin/main
git push fork main --force-with-lease
```

Only use force-with-lease after confirming `origin/main` is the intended tip
and that `fork/main` must remain a pure mirror (no Goblin commits).

Verification:

```bash
test "$(git rev-parse main)" = "$(git rev-parse origin/main)"
test "$(git ls-remote fork refs/heads/main | awk '{print $1}')" = "$(git rev-parse origin/main)"
```

## Phase B — Advance `goblin` onto the new upstream

### B1. No Goblin-unique history yet

If `goblin` is still identical to an old upstream publish and has no product
merges worth preserving as history, recreating from the mirror is OK **only
with explicit maintainer intent**:

```bash
git branch -f goblin origin/main
git push fork goblin --force-with-lease
```

### B2. Goblin already has product commits (normal case)

Replay fork commits onto the new upstream tip. Prefer **cherry-pick / rebase
onto `origin/main`** over `merge --allow-unrelated-histories` when upstream
rewrote history.

When the old base and new base share no merge-base (force-push of "Publish…"):

```bash
# In ${WORKTREE} starting from origin/main:
git -C "${WORKTREE}" reset --hard origin/main

# Replay Goblin product range from the OLD upstream parent of the first
# Goblin commit, e.g. old_base..fork/goblin or explicit SHAs:
OLD_BASE=<previous-origin-main-or-parent-of-first-goblin-commit>
git -C "${WORKTREE}" cherry-pick "${OLD_BASE}..fork/goblin"
# or: git rebase --onto origin/main "${OLD_BASE}" fork/goblin
```

When histories still share a merge-base:

```bash
git -C "${WORKTREE}" rebase origin/main
# or: merge origin/main into goblin with normal 3-way merge
```

**Semantic conflict method (preferred evidence):**

```bash
# True overlap between "our Goblin delta" and "upstream delta" from OLD_BASE:
comm -12 \
  <(git diff --name-only "${OLD_BASE}" fork/goblin | sort) \
  <(git diff --name-only "${OLD_BASE}" origin/main | sort)
```

Unrelated-histories merges that list hundreds of `add/add` files are **noise**;
do not use that list as the resolution checklist.

### B3. During conflicts — preserve Goblin over upstream defaults

| Surface | Keep Goblin unless upstream truly supersedes |
|---------|-----------------------------------------------|
| `crates/codegen/xai-grok-multi-auth/**` | Entire crate (fork-only) |
| `crates/codegen/xai-grok-auth/**` multi-provider types | Fork extensions |
| `xai-grok-shell` features | `native-multi-provider-auth` (+ alias `native-codex-auth`), optional dep `xai-grok-multi-auth` |
| `xai-grok-shell` **version / dev-deps** | Prefer **upstream** version bump (`0.2.x`) and `test-support` feature wiring; re-apply Goblin features on top |
| `pager-bin` | Keep `[[bin]] name = "goblin"` + multi-auth deps; keep upstream entrypoint/headless fixes |
| `pager` CLI | Keep `Auth`, `login --provider`, argv0 `goblin`; keep upstream flags (e.g. reasoning-effort) |
| `agent/config.rs` / `models.rs` / `sampler_turn` | Keep multi-provider catalog keys, BearerResolver, 401 recover, short-slug resolution |
| `sampling-types` / `sampler` responses | Keep Codex wire fixes (system→instructions, empty completed.output recovery) |
| `GOBLIN.md`, `task.md`, architecture docs, `add-provider` skill | Fork-owned |
| `README.md` | Merge: upstream DotSlash/SOURCE_REV **and** Goblin fork section |
| `Cargo.lock` | Regenerate via cargo after resolving manifests — avoid hand edits |

After resolving manifests:

```bash
# versions / features sanity
rg -n 'native-multi-provider-auth|name = "goblin"|xai-grok-multi-auth' \
  crates/codegen/xai-grok-shell/Cargo.toml \
  crates/codegen/xai-grok-pager-bin/Cargo.toml \
  crates/codegen/xai-grok-pager/Cargo.toml \
  Cargo.toml
rg -n '^(<{7}|>{7}|={7}$)' . || true
```

If Git drops commits as empty/already applied, record which and why.

### B4. Integration risks without text conflicts

Always re-check after a successful auto-merge:

- **`AuthManager` / `SharedAuthKeyProvider`** vs Goblin `AuthManagerBearerResolver`
  in `sampler_turn` (`current_or_expired` vs `current_wire_valid` + static API key).
- **Headless** `task_backgrounded` drain / `--no-wait-for-background`.
- **Workspace permission** / session spawn binding changes near multi-provider session path.
- **OAuth scopes** / managed config / voice STT changes that share auth surfaces.

Document the decision for bearer semantics in the PR if both policies remain.

## Metadata Alignment

```bash
git -C "${WORKTREE}" log -1 --format='%H %s' origin/main
git -C "${WORKTREE}" show origin/main:SOURCE_REV 2>/dev/null || true
rg -n 'version = ' crates/codegen/xai-grok-version/Cargo.toml \
  crates/codegen/xai-grok-shell/Cargo.toml \
  crates/codegen/xai-grok-pager/Cargo.toml \
  crates/codegen/xai-grok-pager-bin/Cargo.toml | head -40
```

Update `GOBLIN.md` **Stable Base** / sync notes with the new upstream SHA and
`SOURCE_REV` when cutting or documenting a sync. Do not invent release tags;
first product tag remains `goblin-v0.1.0` per `GOBLIN.md` unless policy changes.

## Validation

Narrow checks first (in `${WORKTREE}`):

```bash
git diff --check
rg -n '^(<{7}|>{7}|={7}$)' . || true
rg -n 'native-multi-provider-auth|\[\[bin\]\]' \
  crates/codegen/xai-grok-shell/Cargo.toml \
  crates/codegen/xai-grok-pager-bin/Cargo.toml
```

Focused Cargo checks (adjust if disk is tight; use a dedicated `CARGO_TARGET_DIR`
under `/tmp` or the worktree when the main tree's `target/` is owned by another agent):

```bash
export CARGO_TARGET_DIR="${WORKTREE}/target-upgrade"
cargo check -p xai-grok-multi-auth
cargo check -p xai-grok-shell
cargo check -p xai-grok-pager-bin
cargo test -p xai-grok-multi-auth --test current_thread_no_panic
cargo test -p xai-grok-multi-auth provider_model_key token_manager
cargo test -p xai-grok-sampling-types inject_streaming_text hoist_
cargo test -p xai-grok-shell --test codex_effort_after_merge
```

Optional live smoke (only with credentials / policy env present):

```bash
# after install-goblin.sh from the upgrade tree, if safe:
goblin auth status
goblin models
# goblin -p '…' --model <codex-or-xai> --permission-mode dontAsk --max-turns 1
```

If disk fills: remove only regenerable build/temp artifacts after stating what
will be deleted; never wipe another agent's worktree or uncommitted sources.

## PR And Handoff

Push the upgrade branch to the fork (not to `main`):

```bash
git -C "${WORKTREE}" push -u fork "${BRANCH}" --force-with-lease
```

Open or update a **Draft** PR with **base `goblin`**:

```bash
gh pr create --repo nonexphere/grok-build \
  --base goblin \
  --head "${BRANCH}" \
  --draft \
  --title "chore(goblin): sync upstream ${UPSTREAM_TIP}" \
  --body "$(cat <<EOF
## Summary
- Sync Goblin onto upstream \`origin/main\` (\`${UPSTREAM_TIP}\`).
- Preserve multi-provider auth, \`goblin\` CLI, and fork docs.

## Upstream
- Tip: \`${UPSTREAM_TIP}\`
- Subject: $(git log -1 --format=%s origin/main)
- SOURCE_REV: $(git show origin/main:SOURCE_REV 2>/dev/null || echo n/a)
- History rewrite detected: yes/no

## Method
- Worktree: \`${WORKTREE}\`
- Branch: \`${BRANCH}\`
- Strategy: cherry-pick / rebase onto origin/main (not unrelated-histories merge)
- OLD_BASE: \`…\`

## Conflicts And Decisions
- Hard conflicts: …
- Auto-merged overlap review: …
- BearerResolver / AuthManager alignment: …

## Dropped Commits
- …

## Validation
- [ ] commands + outcomes
- [ ] residual risks / CI not yet green

## Base Policy
- **Base must be \`goblin\`**. Never \`main\` (upstream mirror).
EOF
)"
```

If a feature PR (e.g. multi-provider) is still open against a stale base,
retarget/rebase it **after** `goblin` advances — use `@create-pr` and isolated
worktrees; do not hijack another agent's dirty tree.

Before handoff, verify:

```bash
gh pr view <N> --repo nonexphere/grok-build \
  --json baseRefName,headRefName,state,isDraft,mergeStateStatus,url
test "$(gh pr view <N> --repo nonexphere/grok-build --jq .baseRefName)" = "goblin"
```

## Completion Checklist

- [ ] Upstream tip verified (`origin/main` SHA + subject + `SOURCE_REV` if any).
- [ ] History rewrite (if any) detected and strategy chosen (cherry-pick/rebase onto tip).
- [ ] `fork/main` mirror matches `origin/main` (force-with-lease only for mirror).
- [ ] Dedicated worktree/branch used; agent-busy worktree untouched.
- [ ] `goblin` advanced without putting product commits on `main`.
- [ ] Goblin surfaces preserved: multi-auth crate, shell feature, `goblin` bin, docs.
- [ ] Upstream version / test-support / headless / auth-manager fixes retained where applicable.
- [ ] No conflict markers remain.
- [ ] Focused cargo check/test results recorded.
- [ ] Draft PR into **`goblin`** with evidence and residual risks.
- [ ] Follow-up feature branches noted for rebase (e.g. `goblin-multi-provider-codex`).

## Stop Conditions

- Shared worktree has another agent's uncommitted work and the operation needs
  checkout/reset/stash → **stop**, use isolated worktree.
- GitHub "latest" vs local `origin/main` disagree and cannot be reconciled → stop.
- Force-push of `goblin` would drop unmerged product history without maintainer
  decision → stop and present options.
- Semantic conflicts in auth/session cannot be resolved without product choice
  (e.g. bearer wire-valid vs multi-provider TokenManager) → document and ask.

## Common Mistakes

- Opening the upgrade PR with base **`main`** (pollutes mirror / wrong policy).
- `merge --allow-unrelated-histories` then "resolving" hundreds of fake `add/add`s.
- Blind `git reset --hard origin/main` on `goblin` after product merges exist.
- Rebasing inside the multi-provider WIP worktree.
- Dropping `native-multi-provider-auth` or the `goblin` bin while taking upstream
  `Cargo.toml` version bumps.
- Forgetting to re-validate `AuthManagerBearerResolver` after upstream auth changes.
- Pushing Goblin commits to `fork/main`.

## Complementarity

| Skill | Role |
|-------|------|
| **@push-grok-updates-to-goblin** (this) | Upstream → Goblin sync / upgrade |
| `@create-pr` | PR base policy (`goblin` only) |
| `@add-provider` | New auth providers after base is current |
| `push-codex-updates-to-goblin` (Codex/Goblins repo) | Analog for OpenAI Codex → Goblins |

## Verification (agent self-check)

- [ ] `git rev-parse main` == `git rev-parse origin/main`
- [ ] `git ls-remote fork refs/heads/main` == `origin/main`
- [ ] Upgrade PR `baseRefName` == `goblin`
- [ ] Original product worktree `git status` unchanged by this skill
- [ ] PR body lists hard conflicts, auto-merge reviews, and test commands
