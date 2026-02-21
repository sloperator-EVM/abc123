#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cd "$ROOT_DIR"

echo "[1/5] formatting Rust code"
cargo fmt --all

echo "[2/5] building workspace"
cargo build --workspace

echo "[3/5] building Windows debug test executables"
"$ROOT_DIR/tests/build_exes.sh"

echo "[4/5] running debug compatibility tests"
"$ROOT_DIR/tests/test.sh"

echo "[5/5] done"
echo "Use: ./target/debug/winrun -d tests/winapi/<sample>.exe"
