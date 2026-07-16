#!/usr/bin/env bash
# Install the goblin fork CLI onto PATH as `goblin`.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PREFIX="${PREFIX:-$HOME/.local}"
BIN_DIR="${PREFIX}/bin"
mkdir -p "$BIN_DIR"

export PROTOC="${PROTOC:-}"
if [[ -z "${PROTOC}" && -x /tmp/grok-goal-b582657efabf/implementer/tools/protoc/bin/protoc ]]; then
  export PROTOC=/tmp/grok-goal-b582657efabf/implementer/tools/protoc/bin/protoc
  export PATH="$(dirname "$PROTOC"):$PATH"
fi

PROFILE="${PROFILE:-release}"
echo "Building goblin ($PROFILE) from $ROOT ..."
cd "$ROOT"
if [[ "$PROFILE" == "release" ]]; then
  cargo build -p xai-grok-pager-bin --bin goblin --release
  SRC="$ROOT/target/release/goblin"
else
  cargo build -p xai-grok-pager-bin --bin goblin
  SRC="$ROOT/target/debug/goblin"
fi

install -m 755 "$SRC" "$BIN_DIR/goblin"
echo "Installed: $BIN_DIR/goblin"
"$BIN_DIR/goblin" --help | head -5
echo
echo "Ensure $BIN_DIR is on your PATH, then run: goblin login --provider codex"
