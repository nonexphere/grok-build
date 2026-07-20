# Goblin / grok-oss surfaces under test

Canonical product contract: repository root `GOBLIN.md`. This file is a QA-oriented
map so agents do not invent binary names or homes.

## Public surface

| Surface | Value | QA note |
| --- | --- | --- |
| CLI binary | `grok-oss` | Build: `cargo build -p xai-grok-pager-bin --bin grok-oss` |
| Artifact path (debug) | `./target/debug/grok-oss` | Prefer this path in evidence commands |
| Artifact path (release) | `./target/release/grok-oss` | Only when perf/release in scope |
| User home | `~/.grok-oss` | Override with `GROK_OSS_HOME` (fallback `GROK_HOME`) |
| Install | `./scripts/install-grok-oss.sh` | Writes PATH launcher under `~/.local/bin/grok-oss` |
| npm | `@brasalabs/grok-oss` (+ platform packages) | Publish needs external `NPM_TOKEN` |
| Internal crates | `xai-grok-*` | Unit tests prove libraries, not the ship binary |

Clap accepts argv0 aliases: `grok-oss` / `goblin` / `grok` / `agent` (per GOBLIN.md).
Test aliases only when install or argv0 behavior is in scope.

## Build loop (human truth)

Rust does **not** hot-reload the TUI. After code changes that affect the CLI:

```text
edit → cargo build -p xai-grok-pager-bin --bin grok-oss → run target/debug/grok-oss
```

| Command | Runnable TUI? | Role in QA |
| --- | --- | --- |
| `cargo check -p …` | No | L0 informational |
| `cargo test -p …` | No | L0 automated |
| `cargo build -p xai-grok-pager-bin --bin grok-oss` | Yes | **Required** for binary claims |
| `PROFILE=debug ./scripts/install-grok-oss.sh` | Yes + PATH | L5 |

### PATH wrapper behavior

`install-grok-oss.sh` installs a **launcher**, not a frozen copy. Preference order:

1. Newest among repo `target/debug/grok-oss` and `target/release/grok-oss` (path baked into wrapper).
2. Else copy under `~/.local/lib/grok-oss/` if present.

QA should still invoke `./target/debug/grok-oss` when proving “this checkout”.

### Version / SHA check

```bash
./target/debug/grok-oss --version
git rev-parse --short HEAD
```

If SHA in version does not match HEAD, rebuild or document that an external
binary was intentionally tested.

## Isolation: host agent vs product under test

| Role | Home | Binary |
| --- | --- | --- |
| Host agent (running this skill) | user’s normal home | session agent binary |
| Product under test | `GROK_OSS_HOME=/tmp/grok-oss-qa-…/home` | `./target/debug/grok-oss` |

Never default product QA to the host’s live config/sessions. Dogfood mode
requires explicit user opt-in.

## Major behavioral surfaces (map to layers)

| Surface | Layers | Notes |
| --- | --- | --- |
| Clap CLI flags / help | L1 | Non-interactive |
| Headless `-p` / scripting | L2 | Cost/auth gated |
| Interactive TUI / pager | L3 | Needs PTY observation |
| Multi-provider auth | L4 | Gated; see `@add-provider` |
| Model picker / binding | L3–L4 | UI + wire identity |
| Skills / MCP (product loads) | L2–L3 | Config discovery; optional |
| Install / PATH | L5 | Wrapper freshness |
| npm distribution | L5 external | Token required |
| App-server / stdio protocol | L0 scripts | `scripts/smoke/*` — not a human TUI substitute |

## Related repo paths

| Path | Why it matters for QA |
| --- | --- |
| `GOBLIN.md` | Public surface + build loop |
| `AGENTS.md` | Branch/PR policy; product name |
| `scripts/install-grok-oss.sh` | Install / wrapper |
| `scripts/smoke/` | Scripted protocol smokes |
| `.agents/issues/testing/` | Known testing honesty bugs |
| `docs/architecture/` | Auth/provider architecture (L4 depth) |
| `~/.grok/docs/user-guide/14-headless-mode.md` | Headless flags reference (upstream-shaped docs may say `grok`; map to `grok-oss`) |
| `~/.grok/docs/user-guide/07-mcp-servers.md` | How **host** configures MCP (e.g. `termctrl mcp`) |
| [anomalyco/terminal-control](https://github.com/anomalyco/terminal-control) | Preferred L3: screen, PNG, video for TUI QA |

## Branch policy (context only)

Product work lives on `goblin` / `goblin-*`. `main` is upstream mirror only.
QA does not open PRs; after green product QA, `@create-pr` targets **`goblin`**.

## What “100%” means (honesty)

This skill aims for **100% of the in-scope human product checklist**, not 100%
of every crate in the monorepo. Expand scope only with explicit triage, and
list remaining surfaces in the report.
