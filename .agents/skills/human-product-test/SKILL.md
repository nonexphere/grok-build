---
name: human-product-test
description: >-
  Test the Goblin fork product (grok-oss CLI/TUI) like a human: rebuild the real
  binary, run non-interactive CLI, headless mode, interactive TUI via
  anomalyco/terminal-control (termctrl MCP/CLI: show, PNG screenshot, optional
  MP4 video) or tmux fallback, gated auth/provider smokes, install/PATH checks,
  and evidence-backed verdicts. Use when asked to human-test, product QA, manual
  QA, CLI/TUI smoke, verify grok-oss, test like a human, terminal-control,
  termctrl, interactive terminal test, rebuild and run the binary, or
  /human-product-test. Does not replace unit tests or code review.
---

# Human Product Test (Goblin / grok-oss)

## When To Use

Use this skill when the goal is to **verify the shipped product behaves for a
human**, not only that libraries compile or unit tests pass:

- “Test like a human”, product QA, manual QA, CLI/TUI smoke.
- After a change that affects the binary, pager, shell, auth UI, install script,
  headless flags, or multi-provider surfaces.
- Before claiming a feature or release readiness for `grok-oss`.
- User runs `/human-product-test`.

### Quando NÃO usar

- **Implementing a code fix** without a product-behavior gate →
  `@implementation-loop` (then return here to prove the fix).
- **Adding a provider end-to-end** → `@add-provider` (this skill is the human
  observation layer on top of its verification list).
- **Unit/integration cargo proof only** → package `cargo test` / CI gates;
  still optional L0 here, never sufficient alone.
- **Code review of a diff** → code-review skill / `@review`.
- **Delivery status reconstruction** → `@delivery-report`.
- **Opening a PR** → `@create-pr` after QA, not as part of this skill.

## Prerequisites

1. Repo root of the Goblin fork; read `AGENTS.md` and `GOBLIN.md` public surface.
2. Permission to run builds and execute the product binary locally.
3. For live network / paid inference / real OAuth: **explicit user authorization**.
4. Terminal observability path chosen (see
   [terminal-observability.md](./references/terminal-observability.md)):
   - **Preferred:** [anomalyco/terminal-control](https://github.com/anomalyco/terminal-control)
     (`termctrl` CLI and/or `termctrl mcp`) — PTY screen, PNG/SVG/txt, optional MP4.
   - **Fallback:** `tmux` + `capture-pane` + `send-keys` (text only; no PNG/video).
5. Evidence directory writable:
   `.agents/evidence/product-qa/<YYYY-MM-DD>-<slug>/`.

If interactive TUI claims are required and neither `termctrl` nor tmux is available,
**block L3** and report the gap; do not invent green.

## Responsibility Boundary

**Do:**

- Rebuild `grok-oss` before any claim that depends on binary behavior.
- Run layered human-style checks (L0–L5) appropriate to the change scope.
- Isolate product home from the host agent home (`GROK_OSS_HOME` scratch).
- Observe the terminal via `termctrl` (screen/PNG/video) or tmux pane capture for
  interactive claims.
- Persist redacted evidence and an honest verdict matrix.
- File durable testing findings with `@issue-lifecycle` when gaps remain.

**Do not:**

- Treat `cargo check` / library unit tests alone as product verification.
- Use the host agent’s live `~/.grok` / `~/.grok-oss` as the product-under-test
  home unless the user explicitly requests dogfood mode (high risk).
- Log, paste, or commit secrets, tokens, or OAuth codes.
- Upgrade skip/blocked/incomplete to PASS for narrative convenience.
- Open PRs, push remotes, publish npm, or mutate production accounts as part of QA.
- Run destructive git (`reset --hard`, force-checkout) to “clean for test”.
- Claim 100% monorepo coverage; scope waves and surfaces honestly.

## Contract Precedence

1. Explicit user instruction and applicable `AGENTS.md`.
2. Security / secret / credential safety.
3. `GOBLIN.md` public surface and build loop.
4. This skill’s checklists and evidence rules.
5. Headless/CLI user-guide behavior for the product version under test.
6. Existing automated tests (inform scope; never replace L1–L3 for product claims).

## Complementaridade

| Skill | Role | When |
| --- | --- | --- |
| **@human-product-test** | Human-like product observation + verdict | Prove CLI/TUI/install/auth UX |
| **@implementation-loop** | Implement a bounded code change | Before re-running this skill |
| **@add-provider** | Provider contract E2E implementation | Provider work; this skill observes |
| **@session-evidence-gate** | Machine gate plan vs evidence | After a wave claims completion |
| **@issue-lifecycle** | Persist testing findings | FAIL/BLOCKER that outlives the session |
| **@delivery-report** | Reconstruct what was done | Status, not execution of QA |
| **@create-pr** | Open PR base `goblin` | After product QA for the branch |

Typical pipeline: implement → **@human-product-test** → fix gaps → re-test →
review → PR.

## Workflow

Follow steps in order. Expand checklists in
[human-checklist.md](./references/human-checklist.md). Surfaces in
[goblin-surfaces.md](./references/goblin-surfaces.md). Evidence rules in
[evidence-and-verdict.md](./references/evidence-and-verdict.md). Terminal
methods in [terminal-observability.md](./references/terminal-observability.md).
Write the final report from
[templates/product-qa-report.md](./templates/product-qa-report.md).

### 1. Triage scope

From the user request and `git diff` / changed packages, select surfaces:

| Scope signal | Minimum layers |
| --- | --- |
| Docs-only | L1 help text if docs claim CLI behavior; else N/A |
| Library-only (no binary path) | L0 targeted + skip L1–L5 with reason |
| Binary / shell / pager / CLI flags | Rebuild + L1 + L2; L3 if TUI touched |
| Auth / provider / login UI | Rebuild + L1 + L2 + gated L4 |
| Install / PATH / npm packaging | Rebuild + L5 (+ L1 version) |
| “Full product QA” / release readiness | Rebuild + L0 (targeted) + L1–L3 + L4 if authorized + L5 |

Record scope in the report. Do not silently expand into unrelated monorepo areas.

### 2. Preflight

1. `pwd` / confirm repo root; note branch (`goblin-*` preferred for product work).
2. `git status` — preserve unrelated dirty files; never clean others’ WIP.
3. Disk/toolchain: enough space under `target/` and `/tmp` for a debug build.
4. Choose **product binary path**: prefer
   `./target/debug/grok-oss` after rebuild (not a mystery PATH binary).
5. Choose **scratch home**:
   ```bash
   export QA_ROOT="/tmp/grok-oss-qa-$(date +%Y%m%d-%H%M%S)-$$"
   export GROK_OSS_HOME="$QA_ROOT/home"
   mkdir -p "$GROK_OSS_HOME" "$QA_ROOT/evidence" "$QA_ROOT/cwd"
   ```
6. Choose observability: `termctrl` (CLI or MCP) if available; else tmux.
7. Create evidence dir:
   `.agents/evidence/product-qa/<date>-<slug>/` and mirror key logs there.

### 3. Rebuild the product (mandatory for binary claims)

```bash
cargo build -p xai-grok-pager-bin --bin grok-oss
./target/debug/grok-oss --version
git rev-parse --short HEAD
```

- Version string should include a git short SHA consistent with HEAD (or document
  intentional mismatch).
- Optional PATH wrapper (once per machine / after install script changes):
  `PROFILE=debug ./scripts/install-grok-oss.sh`
- Still prefer invoking `./target/debug/grok-oss` in QA for path honesty.

**FAIL L_build** if build fails or you cannot produce a runnable binary. Stop
interactive claims.

### 4. Layer L0 — Targeted automated (optional, never sufficient)

Run package-scoped tests that match the change:

```bash
cargo test -p <crate> --lib
# or repo scripts when relevant, e.g. scripts/smoke/stdio-vertical.sh
```

Record commands and exit codes. L0 PASS does **not** complete product QA.

### 5. Layer L1 — Non-interactive CLI (human typing flags)

With scratch home and the rebuilt binary:

```bash
BIN=./target/debug/grok-oss
$BIN --version
$BIN --help
# invalid flag should be non-zero and useful, not panic
$BIN --definitely-not-a-real-flag 2>&1 | tee "$EVIDENCE/l1-bad-flag.txt" || true
```

Assert: exit codes, no panic/backtrace for ordinary misuse, help names public
surface (`grok-oss` / Goblin-relevant terms as applicable). Save stdout/stderr
artifacts.

### 6. Layer L2 — Headless mode (scripted human prompt)

Headless is how a human scripts the agent. Prefer short, **read-only**, low-cost
prompts unless the user authorized live inference.

```bash
export GROK_OSS_HOME=...   # scratch
# Example shape — adjust flags to the binary under test:
$BIN -p "Reply with exactly: ping" \
  --max-turns 2 \
  --output-format plain \
  --cwd "$QA_ROOT/cwd" \
  2>&1 | tee "$EVIDENCE/l2-headless.txt"
```

When possible restrict tools (product flag names may differ by version — verify
with `$BIN --help`):

- Prefer allowlisting read-only tools for smoke.
- Prefer `--max-turns` small.
- Do not pass real secrets on the command line.

Verdict: process exits, output is coherent, no crash. If auth/network missing,
mark **blocked/skip with reason**, never PASS for live inference.

### 7. Layer L3 — Interactive TUI (see the screen)

A human opens the TUI and looks at it. The agent must **observe a real terminal
frame**, not only a non-TTY pipe.

**Preferred ([terminal-control](https://github.com/anomalyco/terminal-control)):**
full recipes in [terminal-observability.md](./references/terminal-observability.md).

```bash
# Minimal durable evidence (PNG + text)
env GROK_OSS_HOME="$GROK_OSS_HOME" termctrl save \
  --cols 112 --rows 34 \
  --format png --format txt \
  --out "$EVIDENCE/l3-tui" \
  -- ./target/debug/grok-oss
```

Multi-step: `termctrl start` → `wait` → `send` → `show`/`save` → `stop`.  
Optional demo: `--record` + `mark` + `termctrl video` → MP4 (needs `ffmpeg`).  
MCP alternative: configure `termctrl mcp` and use structured tools (frame returned immediately).

**Fallback (tmux):** text pane only — see terminal-observability.md.

Assert: recognizable UI (not empty, not panic), clean exit/`stop`, no orphan sessions.

**Without `termctrl` and without tmux:** L3 = **BLOCKED**, not PASS.

### 8. Layer L4 — Auth / multi-provider (gated)

Only with user authorization and a clear account strategy.

- Prefer fixture / already-configured scratch credentials when available.
- Never print tokens; redaction rules from `@add-provider` apply.
- Observe login CLI/TUI surfaces, account list, model pick, logout honesty.
- Live inference: durable redacted log under evidence; skip ≠ pass.

If unauthorized: **SKIP with reason** and continue other layers.

### 9. Layer L5 — Install / distribution

When install or PATH is in scope:

```bash
PROFILE=debug ./scripts/install-grok-oss.sh
command -v grok-oss
grok-oss --version
# confirm wrapper picks newest target/ artifact for this clone
```

npm publish is **out of band** (needs `NPM_TOKEN`); document as external if
release packaging is in scope but not executable.

### 10. Evidence + report

1. Copy/move key logs into
   `.agents/evidence/product-qa/<date>-<slug>/`.
2. Fill [templates/product-qa-report.md](./templates/product-qa-report.md).
3. Every checklist row: command → artifact path → verdict
   (`proven` / `contradicted` / `incomplete` / `weak` / `missing` / `blocked` / `skipped`).
4. Overall: **PASS only if every in-scope required row is proven**. Partial work
   is PARTIAL; blockers are BLOCKED.

### 11. Issues and handoff

- New durable defects → `@issue-lifecycle` under `.agents/issues/testing/`.
- Do not fix out-of-scope product bugs unless the user also asked to fix; report
  the smallest next action.

## Stop Conditions

Stop and ask the user when:

- Live OAuth/browser login is required and no human/browser path is available.
- Paid API quota would be spent without authorization.
- Product binary cannot be built and no alternative artifact is authorized.
- Overlapping dirty files would be corrupted by a “test fix” edit.
- The same failure survives two distinct, evidence-based hypotheses.

Do **not** stop the whole skill for a single gated L4 skip: finish L1–L3/L5 and
report L4 blocked.

## Conventions

- Binary under test: **`grok-oss`** (`xai-grok-pager-bin`).
- Product home: **`GROK_OSS_HOME`** (fallback `GROK_HOME` per `GOBLIN.md`).
- Prefer absolute path to `./target/debug/grok-oss` in evidence commands.
- Evidence is durable and redacted; `/tmp` alone is not enough for PASS claims
  that must survive the session — copy into `.agents/evidence/product-qa/`.
- Severity × confidence for findings: e.g. `[HIGH][Confirmed]`, `[BLOCKER][Likely]`.
- Language of reports: English unless the user asks for PT-BR.

## Common Mistakes

- **Stale binary:** running PATH `grok-oss` without rebuild after code changes.
- **Check ≠ ship:** `cargo check` green treated as TUI verified.
- **Skip as pass:** live test skipped for missing credentials reported as PASS
  (see existing testing issues in `.agents/issues/testing/`).
- **Host home bleed:** wiping or reading the host agent’s real sessions/creds.
- **Blind TUI:** claiming interactive UI works from a non-TTY pipe with no
  `termctrl show`/`save` (or tmux pane) capture.
- **Secret leakage:** tokens or prefixes in evidence logs.
- **Infinite scope:** “100% of the monorepo” instead of scoped surfaces + honest
  remaining list.

## Verification (skill applied correctly)

- [ ] Scope and layers selected and recorded?
- [ ] Rebuild + version/sha evidence for any binary claim?
- [ ] Scratch `GROK_OSS_HOME` used (or dogfood mode explicitly labeled)?
- [ ] L1 artifacts present for in-scope CLI claims?
- [ ] L2 run or honestly blocked/skipped?
- [ ] L3 evidence via `termctrl` (prefer PNG+txt; optional MP4) or tmux pane, or L3 blocked?
- [ ] L4 gated with authorization statement?
- [ ] Report filled; no skip→pass upgrades?
- [ ] Secrets redacted; evidence under `.agents/evidence/product-qa/`?
- [ ] Residual issues filed or listed?

## Provenance

Type: project-scoped `workflow` / product QA method for the Goblin fork.
Grounded in `GOBLIN.md` human build loop, headless user-guide patterns, existing
`scripts/smoke/*`, and testing honesty issues under `.agents/issues/testing/`.
Terminal observability: [anomalyco/terminal-control](https://github.com/anomalyco/terminal-control)
(`termctrl` / `termctrl mcp`) with tmux fallback. Optional demo video via
`termctrl video` + `ffmpeg`. Agent profile and AGENTS.md wiring intentionally deferred.
