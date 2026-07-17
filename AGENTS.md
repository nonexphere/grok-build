# AGENTS.md — Goblin fork (`grok-goblin`)

Local always-on policy for agents working in this repository. Complements
`~/.grok/AGENTS.md` (global). More specific rules here win on conflict.

## Remotes

| Remote | URL | Use |
|--------|-----|-----|
| `origin` | `xai-org/grok-build` | Upstream **read**; source of truth for `main` |
| `fork` | `nonexphere/grok-build` | **Push** and PRs for this fork |

## Branch policy

| Branch | Tracks | Role |
|--------|--------|------|
| **`main`** | `origin/main` | Always mirrors **upstream**. Never land feature work here. Never open feature PRs into `main`. |
| **`goblin`** | `fork/goblin` | **Principal branch of the fork** — integration line. All product PRs target `goblin`. |
| **`goblin-*` / feature** | `fork/<feature>` | Topic work. Open PRs **into `goblin` only**. |

```text
xai-org/main ──fetch──► local main ──push --force-with-lease──► fork/main
                              │
                         fork/goblin  ◄── PRs from feature branches
```

### Syncing main (mirror only)

```bash
git fetch origin
git branch -f main origin/main    # do not checkout if worktree is dirty
git push fork main --force-with-lease
```

### Creating or resetting `goblin` from main

Only when starting the fork line or deliberately re-aligning empty integration:

```bash
git branch -f goblin origin/main
git push fork goblin --force-with-lease
git branch -u fork/goblin goblin
```

Once `goblin` has unique history, advance it via **merged PRs**, not blind reset.

## Pull requests

- **Skill:** `@create-pr` (`.agents/skills/create-pr/SKILL.md`)
- Base: **`goblin`**
- Head: feature branch on `fork`
- Repo: `nonexphere/grok-build`
- Do **not** set base to `main`

If history diverged (upstream rewrote “Publish…” commits), rebase/cherry-pick the feature onto `goblin` in an **isolated worktree** before opening the PR.

## Shared worktree safety

This tree may have **another agent** mid-edit with uncommitted changes.

- Prefer **local, reversible** edits only on files you own for the task.
- Do **not** `git checkout`, `reset --hard`, `stash`, or force-checkout the checked-out branch to “clean” someone else’s WIP.
- Rebases, force-pushes of feature branches, and conflict resolution for PR: use a **separate git worktree**.
- Do not commit or stage unrelated dirty files.

## Goblin / Grok OSS product docs

- [`GOBLIN.md`](GOBLIN.md) — fork contract + **public surface `grok-oss`**
- [`task.md`](task.md) — multi-provider / Codex plan
- [`TO_RELEASE_NPM.md`](TO_RELEASE_NPM.md) — npm publish remaining step (`NPM_TOKEN`)
- [`TO_RELEASE.md`](TO_RELEASE.md) — release honesty (when present)
- [`docs/architecture/GROK_OSS_IDENTITY_AND_DISTRIBUTION_PLAN.md`](docs/architecture/GROK_OSS_IDENTITY_AND_DISTRIBUTION_PLAN.md) —
  identity cutover (**in progress**): binary `grok-oss`, home `~/.grok-oss`,
  npm `@brasalabs/grok-oss`. Only registry auth is external.

## Default agent behavior

1. Implement on a `goblin-*` feature branch based on `goblin` when possible.
2. Validate with package-scoped `cargo test` / `cargo check`.
3. Product CLI for humans: **`grok-oss`** (`PROFILE=debug ./scripts/install-grok-oss.sh`).
3. Open/update PRs with `@create-pr` (base `goblin`).
4. Leave `main` as upstream mirror only.
