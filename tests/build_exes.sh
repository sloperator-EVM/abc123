#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WINAPI_DIR="$SCRIPT_DIR/winapi"

if command -v x86_64-w64-mingw32-gcc >/dev/null 2>&1; then
  WIN_GCC="x86_64-w64-mingw32-gcc"
elif command -v i686-w64-mingw32-gcc >/dev/null 2>&1; then
  WIN_GCC="i686-w64-mingw32-gcc"
else
  echo "error: mingw cross compiler not found (expected x86_64-w64-mingw32-gcc or i686-w64-mingw32-gcc)" >&2
  exit 1
fi

echo "using compiler: $WIN_GCC"

for src in "$WINAPI_DIR"/*.c; do
  exe="${src%.c}.exe"
  echo "building $(basename "$src") -> $(basename "$exe")"
  "$WIN_GCC" "$src" -Wall -Wextra -O0 -g -o "$exe"
done

echo "done: built $(find "$WINAPI_DIR" -maxdepth 1 -name '*.exe' | wc -l | tr -d ' ') exe samples"
