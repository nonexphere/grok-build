# Terminal observability — how the agent “sees” the product

Human product QA requires **seeing** what a human sees: version lines, help text,
headless stdout, and especially the **interactive TUI grid**. A non-TTY pipe is
not a TUI.

## Decision matrix

| Need | Preferred | Fallback | Insufficient alone |
| --- | --- | --- | --- |
| `--version` / `--help` | `run_terminal_command` | same | — |
| Headless `-p` | `run_terminal_command` + tee | `termctrl` session + `logs` | — |
| Interactive TUI | **[terminal-control](https://github.com/anomalyco/terminal-control)** (`termctrl`) | tmux `capture-pane` | bare pipe to TUI |
| Screenshot / PNG / SVG | `termctrl save --format png` | — | text-only guess |
| Demo video MP4 | `termctrl` record + `video` (+ `ffmpeg`) | — | — |
| Long-running server | background + monitor | `termctrl start` session | blocking forever without timeout |
| Browser OAuth | chrome-devtools MCP (optional) | human completes browser | guessing tokens |

---

## Tier A — terminal-control / `termctrl` (preferred)

Upstream: **[anomalyco/terminal-control](https://github.com/anomalyco/terminal-control)**  
CLI: `termctrl` · crates.io: `terminal-control`

Purpose-built for **agents and TUI review**: real PTY, settled visible screen,
screenshots, shared human+agent sessions, timeline recording, and **MP4 export**.

### Why this is better for grok-oss QA

| Capability | Use in `@human-product-test` |
| --- | --- |
| Ghostty-backed screen model | Faithful full-screen TUI (not line-oriented pipes) |
| `show` | Read settled visible screen (text / json / svg) |
| `save --format png` (+ `txt`) | Durable L3 evidence humans can open |
| `start` / `send` / `wait` / `stop` | Multi-step interactive flows without sleep loops |
| `run` | Human watches the same PTY the agent drives |
| `--record` + `mark` + `video` | Short demo MP4 of the product under test |
| `termctrl mcp` | MCP stdio server — tools return current frame immediately |
| Optional `settleMs` / `deadlineMs` | Quiet-output settling when a transition needs it |

### Install (host machine)

Source builds need a recent Rust toolchain, Zig (pinned version per upstream),
Git, and network while Ghostty core builds. Video export needs **`ffmpeg`**.

```bash
# Stable crate
cargo install --locked terminal-control

# Or repository head
cargo install --locked --git https://github.com/anomalyco/terminal-control terminal-control

termctrl --help
```

Optional agent skill from upstream (workflow tips):

```bash
npx skills add anomalyco/terminal-control --skill terminal-control
```

### MCP setup (host agent)

Expose sessions as structured MCP tools (stdio):

```bash
termctrl mcp
```

Example host config (`~/.grok/config.toml` or equivalent):

```toml
[mcp_servers.terminal-control]
command = "termctrl"
args = ["mcp"]
enabled = true
startup_timeout_sec = 30
```

If `termctrl` is not on PATH, use the absolute path from `cargo install`
(typically `~/.cargo/bin/termctrl`).

**Security:** full terminal control as the OS user. Trusted local env only.
Prefer scratch `GROK_OSS_HOME` and non-production credentials. Recordings
(`.termctrl`) can contain prompts and secrets — treat as sensitive; redaction
rules apply before copying into `.agents/evidence/`.

### CLI vs MCP

Either path is valid for L3:

1. **Shell:** agent runs `termctrl …` via `run_terminal_command` (always works if installed).
2. **MCP:** agent calls tools from `termctrl mcp` (discover names with host MCP
   tool list; screen reads return the current frame immediately).

Prefer CLI in evidence commands so reports stay reproducible without MCP.

---

## Product-under-test recipes (grok-oss)

Assume rebuild done: `./target/debug/grok-oss` exists.  
Use absolute paths in sessions. Isolate home.

```bash
REPO="$(pwd)"
BIN="$REPO/target/debug/grok-oss"
export GROK_OSS_HOME="${GROK_OSS_HOME:-/tmp/grok-oss-qa-home}"
EVIDENCE="${EVIDENCE:-$REPO/.agents/evidence/product-qa/latest}"
mkdir -p "$EVIDENCE" "$GROK_OSS_HOME"
```

### One-shot screen read

```bash
# Visible text to stdout (no files)
env GROK_OSS_HOME="$GROK_OSS_HOME" termctrl show \
  --cols 112 --rows 34 \
  -- "$BIN"

# Or wait for a known string after start
env GROK_OSS_HOME="$GROK_OSS_HOME" termctrl show \
  --cols 112 --rows 34 \
  --wait-for "grok" \
  -- "$BIN"
```

Formats: default text; `--format json`; `--format svg`.

### Save L3 evidence (PNG + text) — recommended

```bash
env GROK_OSS_HOME="$GROK_OSS_HOME" termctrl save \
  --cols 112 --rows 34 \
  --format png --format txt \
  --out "$EVIDENCE/l3-tui" \
  -- "$BIN"
# writes l3-tui.png and l3-tui.txt
```

One-off `show` / `save` **terminate** the launched process after the shot. Use
`start` for multi-step flows.

### Named live session (multi-step)

```bash
SESSION="goss-qa-$$"

env GROK_OSS_HOME="$GROK_OSS_HOME" termctrl start "$SESSION" \
  --cols 112 --rows 34 \
  --cwd "$REPO" \
  -- "$BIN"

termctrl wait "$SESSION" "grok" --timeout 8000 || true
termctrl show "$SESSION" | tee "$EVIDENCE/l3-tui-pane.txt"
termctrl save "$SESSION" --format png --format txt --out "$EVIDENCE/l3-tui"

# Exact input: text:…, enter, escape, arrows, tab, ctrl-c, …
# termctrl send "$SESSION" text:/help enter
# termctrl wait "$SESSION" "help" --timeout 5000

termctrl stop "$SESSION"
```

Notes from upstream:

- `send` accepts `text:<value>`, named keys (`enter`, `escape`, arrows, `tab`,
  `shift-tab`, `backspace`, `delete`, `home`, `end`, `page-up`, `page-down`),
  and `ctrl-a` … `ctrl-z`. Pipe exact bytes with `--stdin`.
- Prefer `wait` for visible text over `sleep`.
- `status` / `list` for session hygiene; `resize` for responsive layout checks.
- Full-screen alternate-screen TUIs: use **`show` / `save`**, not `logs`.
- Log-like / normal-screen tools: `termctrl logs "$SESSION"`.

### Share with a human (same PTY)

```bash
# Human sees the app in their pane; agent drives via named session commands
termctrl run goss-shared --cwd "$REPO" -- env GROK_OSS_HOME="$GROK_OSS_HOME" "$BIN"
```

No tmux required for shared control.

### Record + export video (demo evidence)

Requires `ffmpeg` for MP4 export.

```bash
SESSION="goss-demo-$$"
REC="$EVIDENCE/l3-demo.termctrl"

env GROK_OSS_HOME="$GROK_OSS_HOME" termctrl start "$SESSION" \
  --cols 112 --rows 34 \
  --record "$REC" \
  --cwd "$REPO" \
  -- "$BIN"

termctrl wait "$SESSION" "grok" --timeout 8000 || true
termctrl mark "$SESSION" ui-ready
# optional paced typing:
# termctrl send "$SESSION" --pace-ms 35 'text:/help' enter
# termctrl wait "$SESSION" "Help" --timeout 10000
termctrl mark "$SESSION" after-action
termctrl stop "$SESSION"

termctrl markers "$REC"
termctrl show --recording "$REC" --at-marker ui-ready \
  > "$EVIDENCE/l3-at-ui-ready.txt" || true

# Full timing (no edit plan):
termctrl video "$REC" --out "$EVIDENCE/l3-demo.mp4" --tail-ms 0

# Or edited clips with captions (optional JSON plan):
# termctrl video "$REC" --edit "$EVIDENCE/l3-edit.json" --footer --out "$EVIDENCE/l3-demo.mp4"
```

Example edit plan:

```json
{
  "clips": [
    {
      "from": "ui-ready",
      "to": "after-action",
      "speed": 2,
      "caption": "grok-oss TUI smoke"
    }
  ]
}
```

Keep speeds low enough that text stays readable. Copy only redacted or
non-secret recordings into the evidence tree.

### OpenTUI note

Apps using OpenTUI (e.g. OpenCode) may need `--host opentui`. **grok-oss** is a
different stack (ratatui / Grok pager); **do not** pass `--host opentui` unless
you are testing an OpenTUI binary. If a product needs a host handshake later,
document it in the report.

---

## Tier B — tmux (fallback only)

Use when `termctrl` is not installed and cannot be installed in-session.

```bash
SESSION="goss-qa-$$"
tmux new-session -d -s "$SESSION" -x 120 -y 40 \
  "env GROK_OSS_HOME=$GROK_OSS_HOME $BIN"
sleep 2
tmux capture-pane -t "$SESSION" -p -e -S -5000 > "$EVIDENCE/l3-tui-pane.txt"
tmux kill-session -t "$SESSION" 2>/dev/null || true
```

tmux cannot produce PNG/MP4 like terminal-control. Prefer installing `termctrl`.

---

## Tier B2 — Harness built-ins

| Tool | Use |
| --- | --- |
| `run_terminal_command` | `termctrl` recipes, cargo, L1/L2 |
| `background` / `monitor` | long headless; not a TUI screenshot |
| MCP `search_tool` / `use_tool` | when `termctrl mcp` is configured |

---

## Capability preflight (start of L3)

```bash
command -v termctrl && termctrl --help | head -5
command -v ffmpeg && echo "ffmpeg:yes" || echo "ffmpeg:no (video export blocked)"
command -v tmux && echo "tmux:yes" || echo "tmux:no"
```

| Availability | L3 policy |
| --- | --- |
| `termctrl` present | Required path for `proven` TUI claims when possible |
| only `tmux` | Text pane only; note weak fidelity vs PNG |
| neither | L3 = **blocked**, not pass |

---

## Interpreting screens

- Pass = recognizable UI structure (chrome, input, status), not pixel perfection.
- Empty screen after wait = fail or incomplete; do not invent content.
- PNG + `.txt` from `save` is stronger evidence than chat paste alone.
- MP4 is optional demo/hand-off; not required for every smoke unless user asks.

---

## Failure modes

| Symptom | Likely cause | Action |
| --- | --- | --- |
| `termctrl: not found` | not installed | install crate; else tmux fallback |
| Empty show | app not ready / wrong binary | `--wait-for`, longer timeout, check `status` |
| Build of termctrl fails | Zig/Rust version | report blocked; use tmux |
| `video` fails | missing ffmpeg | install or skip video (`blocked`) |
| Secrets in `.termctrl` | recorded prompts | redact; do not commit raw recording |
| Wrong host flag | `--host opentui` on non-OpenTUI | remove flag for grok-oss |

---

## Historical / non-preferred MCPs

Older docs mentioned `interactive-terminal-mcp` or shared PTY `terminal-mcp`.
For this fork skill, **do not prefer them** when terminal-control is available.
They remain emergency alternatives only if `termctrl` cannot run.
