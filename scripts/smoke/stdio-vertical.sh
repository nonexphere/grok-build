#!/usr/bin/env bash
# C7-D — Scripted stdio vertical smoke (Wave 2 evidence).
#
# Builds the `stdio_smoke_server` example (shipped FacadeProcessor +
# run_stdio_loop NDJSON transport over real process stdin/stdout, backed by
# FakeRuntime), then drives it as a real subprocess black-box through:
#
#     initialize -> session/start -> turn/start -> session/read -> session/subscribe
#
# and asserts the primary observables (real session id, real turn id bound to
# the session, non-empty transcript turns/items, replay events).
#
# Full scripted exchange + assertions are written to:
#   /tmp/grok-goal-5598c3040156/implementer/smoke/stdio-vertical.txt
#
# This is NOT a unit-test filter re-run: it captures the actual NDJSON request
# and response lines plus assertions over them.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
OUT_DIR="/tmp/grok-goal-5598c3040156/implementer/smoke"
LOG="$OUT_DIR/stdio-vertical.txt"
mkdir -p "$OUT_DIR"

DRIVER="$REPO_ROOT/scripts/smoke/stdio_vertical_driver.py"

{
  echo "[build] cargo build --example stdio_smoke_server -p xai-grok-app-server"
  cargo build --example stdio_smoke_server -p xai-grok-app-server
} >"$LOG" 2>&1
# Re-open in append mode for the driver output below.
exec 3>>"$LOG"

BIN="$REPO_ROOT/target/debug/examples/stdio_smoke_server"
if [[ ! -x "$BIN" ]]; then
  echo "FATAL: built binary not found at $BIN" >&2
  cat "$LOG" >&2
  exit 1
fi

# Drive the real stdio subprocess and capture full exchange + assertions.
python3 "$DRIVER" "$BIN" >&3
rc=$?

exec 3>&-

# Echo the log to stdout for visibility.
cat "$LOG"
exit $rc
