# Human checklist — grok-oss product QA

Use with `@human-product-test`. Mark each in-scope row:

| Verdict | Meaning |
| --- | --- |
| `proven` | Command ran; artifact proves expected behavior |
| `contradicted` | Ran; behavior wrong (FAIL) |
| `incomplete` | Partial run; not enough to judge |
| `weak` | Ran; signal noisy or flaky |
| `missing` | Required but not executed |
| `blocked` | Could not run (env, auth, tools) |
| `skipped` | Out of scope by triage (must state reason) |
| `n/a` | Not applicable to this product surface |

**Rule:** `skipped` / `blocked` / `missing` are never auto-promoted to `proven`.

Copy a working table into the session report (see
`templates/product-qa-report.md`).

---

## L_build — Binary freshness

| ID | Check | How | Pass criteria |
| --- | --- | --- | --- |
| B1 | Workspace buildable for product bin | `cargo build -p xai-grok-pager-bin --bin grok-oss` | exit 0; binary exists |
| B2 | Binary path known | `ls -la ./target/debug/grok-oss` | executable |
| B3 | Version string | `./target/debug/grok-oss --version` | prints version; no panic |
| B4 | SHA alignment | compare version SHA vs `git rev-parse --short HEAD` | match or mismatch documented |
| B5 | Release binary (optional) | `cargo build … --release` + `--version` | only if perf/release in scope |

---

## L0 — Targeted automated (optional)

| ID | Check | How | Pass criteria |
| --- | --- | --- | --- |
| U1 | Package unit/lib tests for touched crates | `cargo test -p <crate> --lib` | exit 0 |
| U2 | Repo smoke scripts if surface matches | e.g. `scripts/smoke/stdio-vertical.sh` | exit 0 + log artifact |
| U3 | Typecheck-only (informational) | `cargo check -p …` | never alone as product PASS |

---

## L1 — Non-interactive CLI

| ID | Check | How | Pass criteria |
| --- | --- | --- | --- |
| C1 | Help | `$BIN --help` | exit 0; useful usage; mentions product identity |
| C2 | Version | `$BIN --version` | exit 0; stable format |
| C3 | Bad flag | `$BIN --not-a-real-flag` | non-zero; error message; no panic/backtrace required-as-crash |
| C4 | Subcommand / argv0 aliases (if claimed) | invoke as documented (`grok-oss`, legacy aliases only if testing install) | documented aliases work |
| C5 | Non-TTY safety | pipe stdin closed / `script` or plain shell | no hang waiting for TTY when flags imply non-interactive |
| C6 | `--cwd` / workdir (if flag exists) | headless or status with `--cwd` scratch | uses provided cwd |
| C7 | Config on empty home | `GROK_OSS_HOME=$scratch` first run | no panic; creates expected layout or clear error |

Save: `l1-help.txt`, `l1-version.txt`, `l1-bad-flag.txt`.

---

## L2 — Headless / scripted agent

| ID | Check | How | Pass criteria |
| --- | --- | --- | --- |
| H1 | One-shot prompt | `$BIN -p "…" --max-turns N` (or equivalent) | process terminates; stdout artifact |
| H2 | Machine output (if supported) | `--output-format json` or `streaming-json` | parseable or documented plain fallback |
| H3 | Tool allow/deny (if supported) | restrict to read-only tools | does not execute disallowed tools |
| H4 | Max turns respected | small `--max-turns` | stops without infinite loop |
| H5 | Offline / no-auth honesty | no credentials in scratch home | clear auth error or skip path — **not** silent fake success |
| H6 | Live inference (gated) | authorized only | durable redacted log; real model response |

Save: `l2-headless.txt`, optional `l2-json.json`.

---

## L3 — Interactive TUI (must observe screen)

Prefer **[terminal-control](https://github.com/anomalyco/terminal-control)** (`termctrl`).

| ID | Check | How | Pass criteria |
| --- | --- | --- | --- |
| T1 | Starts under real terminal | `termctrl start` / `save` / MCP | session running or shot non-empty |
| T2 | Recognizable UI | `termctrl show` or `save --format txt` | chrome/status/input visible in dump |
| T3 | Screenshot evidence | `termctrl save --format png` (+ `txt`) | `l3-tui.png` (+ `.txt`) in evidence dir |
| T4 | No immediate panic | show/save + any stderr | no panic, no "thread panicked" |
| T5 | Minimal interaction (safe) | `termctrl send` quit / `ctrl-c` / `stop` | clean stop; no orphan session |
| T6 | Slash help (safe) | `send … text:/help enter` + `wait`/`show` | help surface in frame |
| T7 | Model menu (safe/gated) | `/model` only if safe | list or clear empty-state |
| T8 | Resize (optional) | `termctrl resize` | no crash |
| T9 | Demo video (optional) | `--record` + `mark` + `termctrl video` | MP4 when user asks / release demo; needs `ffmpeg` |

**Hard rule:** without a `termctrl show`/`save` frame (or tmux pane dump), T1–T7
cannot be `proven`.

Save at minimum: `l3-tui.png`, `l3-tui.txt` (or `l3-tui-pane.txt` on tmux fallback).
Optional: `l3-demo.termctrl`, `l3-demo.mp4` (redact secrets in recordings).

---

## L4 — Auth / multi-provider (gated)

Require explicit user authorization. Prefer scratch credentials.

| ID | Check | How | Pass criteria |
| --- | --- | --- | --- |
| A1 | Auth surface discoverable | help / TUI / CLI accounts | documented entry points exist |
| A2 | Login path (gated) | device/browser/API key as product supports | completes or fails loudly with typed error |
| A3 | Account list isolation | two accounts if available | selection is explicit; no silent first-wins without doc |
| A4 | Model binding | pick model for provider | UI/wire identity coherent (see `@add-provider`) |
| A5 | Request succeeds (gated live) | one short completion | durable evidence; no secrets in logs |
| A6 | Logout honesty | logout / revoke | `remote_revoked` / warnings honest if product exposes them |
| A7 | Secret redaction | inspect evidence logs | no full token; no 4/8/12/20 char secret prefixes |

Default without auth: entire L4 = `skipped` with reason `no authorization`.

---

## L5 — Install / PATH / distribution

| ID | Check | How | Pass criteria |
| --- | --- | --- | --- |
| I1 | Install script | `PROFILE=debug ./scripts/install-grok-oss.sh` | exit 0 |
| I2 | Wrapper on PATH | `command -v grok-oss` | finds launcher |
| I3 | Wrapper prefers target | rebuild → `grok-oss --version` SHA | tracks newest target build for this clone |
| I4 | Home layout | inspect `$GROK_OSS_HOME` after first run | expected dirs; no write to host home when env set |
| I5 | npm package (optional/external) | only if release in scope + token | usually `blocked` / external |

---

## Honesty / regression guards

| ID | Check | Pass criteria |
| --- | --- | --- |
| R1 | No skip-as-pass | any skipped live test labeled skipped/blocked |
| R2 | Evidence durability | PASS rows cite `.agents/evidence/product-qa/...` not only `/tmp` |
| R3 | Scope honesty | report lists untested surfaces |
| R4 | Known issue awareness | skim `.agents/issues/testing/` for related open issues |

---

## Suggested minimal profiles

### Smoke (default after small binary change)

B1–B4, C1–C3, C7, H1, H5, T1–T5, R1–R3.

### Auth wave

Smoke + A1–A7 (as authorized) + U1 for auth crates.

### Release readiness

Smoke + L5 + broader L2 + authorized L4 subset + U2 smokes + R1–R4.

### Docs-only claim of CLI behavior

C1–C2 only (or N/A if pure prose).
