#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 3 ]]; then
  echo "usage: $0 <expected-test-name-fragment> cargo test <cargo-args...>" >&2
  exit 2
fi

expected_test=$1
shift
if [[ $1 != cargo || $2 != test ]]; then
  echo "gate accepts only a cargo test command" >&2
  exit 2
fi

gate_output=$(mktemp)
trap 'rm -f "$gate_output"' EXIT
"$@" -- --nocapture 2>&1 | tee "$gate_output"

if ! grep -E "^test .*${expected_test}.* \.\.\. ok$" "$gate_output" >/dev/null; then
  echo "test gate failed: no passing test matched '$expected_test'" >&2
  exit 1
fi
