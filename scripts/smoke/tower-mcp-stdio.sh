#!/usr/bin/env bash
# Product MCP stdio smoke: build the real grok-oss launcher and drive it as a
# subprocess. Protocol bytes must remain JSON-RPC-only on stdout.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
TMP_ROOT="$(mktemp -d /var/tmp/grok-mcp-stdio.XXXXXX)"
OUT="${TMP_ROOT}/stdout.ndjson"
ERR="${TMP_ROOT}/stderr.log"
IDEMPOTENCY_KEY="stdio-smoke-${TMP_ROOT##*/}"
trap 'rm -rf "$TMP_ROOT"' EXIT

cargo build -p xai-grok-pager-bin --features mcp-stdio --bin grok-oss

printf '%s\n' \
  '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' \
  "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/call\",\"params\":{\"name\":\"tower_agent_start\",\"arguments\":{\"workspaceRoot\":\"${TMP_ROOT}/workspace\",\"agentType\":\"orchestrator\",\"idempotencyKey\":\"${IDEMPOTENCY_KEY}\"}}}" \
  | XAI_API_KEY=test-api-key timeout 30 "${REPO_ROOT}/target/debug/grok-oss" tower --stdio \
  >"${OUT}" 2>"${ERR}"

python3 - "${OUT}" "${ERR}" <<'PY'
import json
import sys

out_path, err_path = sys.argv[1:]
lines = open(out_path, encoding="utf-8").read().splitlines()
assert len(lines) == 2, lines
responses = [json.loads(line) for line in lines]
assert len(responses[0]["result"]["tools"]) == 9
assert responses[1]["result"]["structuredContent"]["state"] == "completed"
assert all(line.lstrip().startswith("{") for line in lines)
assert "mcp stdio eof" in open(err_path, encoding="utf-8").read()
print("tower MCP stdio smoke: PASS (tools/list=9, start=completed, stdout=JSON-RPC-only, EOF=stderr)")
PY
