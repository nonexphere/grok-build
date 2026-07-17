#!/usr/bin/env bash
# Local npm pack for @brasalabs/grok-oss without publishing (no NPM_TOKEN required).
#
# Uses a real host binary when small enough; otherwise a stub file so `npm pack`
# stays usable in dev (debug grok-oss can be 500MB+). CI release builds use
# release binaries + brotli via assemble-platform-packages.mjs without --host-only.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
NPM="$ROOT/npm"
OUT="${OUT:-$ROOT/npm/dist}"
mkdir -p "$OUT"

host_key="$(node -e "process.stdout.write(process.platform+'-'+process.arch)")"
plat_dir="$NPM/grok-oss-$host_key"
mkdir -p "$plat_dir/bin"

real_bin=""
for c in "$ROOT/target/release/grok-oss" "$ROOT/target/debug/grok-oss"; do
  if [[ -x "$c" ]]; then real_bin="$c"; break; fi
done

if [[ -n "$real_bin" ]]; then
  size=$(stat -c%s "$real_bin" 2>/dev/null || stat -f%z "$real_bin")
else
  size=0
fi

# 80 MiB threshold: above this, pack a stub so local smoke stays fast.
if [[ -n "$real_bin" && "$size" -lt 83886080 ]]; then
  echo "Assembling host package from $real_bin ($size bytes)"
  export "GROK_OSS_BIN_$(echo "$host_key" | tr 'a-z-' 'A-Z_')=$real_bin" 2>/dev/null || true
  # Map common host keys explicitly
  case "$host_key" in
    linux-x64) export GROK_OSS_BIN_LINUX_X64="$real_bin" ;;
    linux-arm64) export GROK_OSS_BIN_LINUX_ARM64="$real_bin" ;;
    darwin-x64) export GROK_OSS_BIN_DARWIN_X64="$real_bin" ;;
    darwin-arm64) export GROK_OSS_BIN_DARWIN_ARM64="$real_bin" ;;
    win32-x64) export GROK_OSS_BIN_WIN32_X64="$real_bin" ;;
    win32-arm64) export GROK_OSS_BIN_WIN32_ARM64="$real_bin" ;;
  esac
  node "$NPM/scripts/assemble-platform-packages.mjs" --host-only
else
  echo "Using stub platform binary for pack smoke (real bin size=${size:-missing})"
  printf '#!/bin/sh\necho "grok-oss stub for npm pack"\n' > "$plat_dir/bin/grok-oss"
  chmod +x "$plat_dir/bin/grok-oss"
  # Stamp platform version to meta
  node -e "
    const fs=require('fs');
    const meta=JSON.parse(fs.readFileSync('$NPM/grok-oss/package.json','utf8'));
    const p='$plat_dir/package.json';
    const j=JSON.parse(fs.readFileSync(p,'utf8'));
    j.version=meta.version;
    fs.writeFileSync(p, JSON.stringify(j,null,4)+'\n');
  "
fi

echo "Packing meta package…"
(cd "$NPM/grok-oss" && npm pack --pack-destination "$OUT")

if [[ -f "$plat_dir/package.json" ]]; then
  echo "Packing platform package $host_key…"
  (cd "$plat_dir" && npm pack --pack-destination "$OUT")
fi

echo "Artifacts in $OUT:"
ls -la "$OUT" | sed -n '1,30p'

# Identity assertions (shipped package.json, not reinvented)
node "$NPM/scripts/assert-package-identity.mjs"
