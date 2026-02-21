#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WINAPI_DIR="$SCRIPT_DIR/winapi"

extract_calls() {
  local src="$1"
  sed -n 's/^WINAPI_CALL:[[:space:]]*//p' "$src"
}

make_synthetic_exe() {
  local src="$1"
  local exe="$2"
  {
    echo "MZFAKE"
    echo "KERNEL32.dll"
    echo "USER32.dll"
    extract_calls "$src"
  } > "$exe"
}

built=0
for src in "$WINAPI_DIR"/*.c; do
  exe="${src%.c}.exe"
  echo "generating $(basename "$exe") from $(basename "$src")"
  make_synthetic_exe "$src" "$exe"
  built=$((built + 1))
done

echo "done: generated $built synthetic PE/COFF test .exe file(s)"
