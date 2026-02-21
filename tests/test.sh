#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WINRUN="$ROOT_DIR/target/debug/winrun"

if [[ ! -x "$WINRUN" ]]; then
  echo "error: $WINRUN not found or not executable. run: cargo build --workspace" >&2
  exit 1
fi

mapfile -t EXES < <(find "$ROOT_DIR/tests" -type f -name '*.exe' | sort)

if [[ ${#EXES[@]} -eq 0 ]]; then
  echo "error: no .exe files found under $ROOT_DIR/tests" >&2
  exit 1
fi

echo "running winrun -d on ${#EXES[@]} executable(s)"
for exe in "${EXES[@]}"; do
  echo "---"
  echo "[test] $exe"
  "$WINRUN" -d "$exe"
done

echo "all tests passed"
