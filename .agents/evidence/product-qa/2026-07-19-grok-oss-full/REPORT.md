# Product QA report — grok-oss / Goblin

| Field | Value |
| --- | --- |
| Date | 2026-07-19 |
| Slug | 2026-07-19-grok-oss-full |
| Branch | `goblin-implement-epic-tree` |
| HEAD short SHA | `d0ea385` |
| Binary path | `/home/guilherme/github/grok-goblin/target/debug/grok-oss` |
| Binary version line | `grok 0.2.102 (d0ea385)` |
| `GROK_OSS_HOME` | `/tmp/grok-oss-qa-20260719-full-1855854/home` |
| Dogfood host home? | no |
| Observability | portable tmux socket under scratch QA root |
| Scope profile | smoke + npm packaging |
| Overall verdict | **BLOCKED/PARTIAL** |

## Intent

Validate the actual `grok-oss` binary as a human would, in addition to the
library gates already run for App Server/MCP/Tower.

## Evidence

| ID | Verdict | Artifact | Notes |
| --- | --- | --- | --- |
| B1 | proven | `build.txt` | `cargo build -p xai-grok-pager-bin --bin grok-oss` succeeded |
| B2 | proven | `l1-version.txt` | executable binary exists at the recorded path |
| B3 | proven | `l1-version.txt` | version exits successfully and includes SHA |
| B4 | proven | `l1-version.txt` | SHA matches `git rev-parse --short HEAD` |
| C1 | proven | `l1-help.txt` | useful help and `grok-oss` usage |
| C2 | proven | `l1-version.txt` | stable version output |
| C3 | proven | `l1-bad-flag.txt` | exit 2, useful error, no panic/backtrace |
| C7 | proven | `l2-headless.txt` | scratch home does not crash; auth error is explicit |
| H1 | blocked | `l2-headless.txt` | no credentials; process exits with typed “Not signed in” |
| H5 | proven | `l2-headless.txt` | no-auth path is honest; no fake success |
| T1 | proven | `l3-tui-pane-portable.txt` | real TUI rendered in a 120x40 tmux pane |
| T2 | proven | `l3-tui-pane-portable.txt` | sign-in screen, one-time code, and waiting state are recognizable |
| T3 | proven | `l3-tui-pane-portable-after-ctrl-c.txt` | `Ctrl-C` terminates the session cleanly |
| T4 | proven | `l3-tui-pane-portable.txt` | no panic/backtrace; unauthenticated state is explicit |
| A1–A7 | skipped | `l4-auth.txt` | no user authorization or real credentials |
| I1 | proven | `l5-install.txt` | isolated `PREFIX` install exited 0 |
| I2 | proven | `l5-install.txt` | isolated prefix launcher found by PATH |
| I3 | proven | `l5-install.txt` | launcher invoked rebuilt target and version matched SHA |
| I5 | proven | npm dry-run logs | meta and Linux x64 `npm pack --dry-run` succeeded |

## Honesty guards

- No credentials, tokens, OAuth codes, or host home were used.
- L2 auth failure and L4 authorization absence remain blocked/skipped; none
  were promoted to PASS. L3 used a portable tmux extracted into the scratch
  evidence root, so no system package installation or sudo was needed.
- Automated App Server/Tower/Shell/MCP evidence remains in the R6 report.

## Remaining work

- Provide explicitly authorized scratch credentials/browser path for L4/live
  inference if that surface is required.
- Product actor factory, remote TLS acceptance, npm publish, and commit remain
  external or product-context dependent.

## Conclusion

The rebuilt binary is healthy for local CLI behavior and the TUI is proven
through a real portable-tmux session. A 100% product verdict is not proven
because live auth/inference requires credentials and authorization not present
in this session.
