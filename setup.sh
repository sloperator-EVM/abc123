#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT_DIR"

echo "[1/5] formatting Rust code"
cargo fmt --all

echo "[2/5] building workspace"
cargo build --workspace

echo "[3/5] building native sample"
"$ROOT_DIR/tests/samples/build_native_sample.sh"

echo "[4/5] generating non-native sample inputs"
"$ROOT_DIR/tests/samples/generate_non_native_samples.sh"

echo "[5/5] done"
echo "Use: ./target/debug/winrun [-d] tests/samples/<file>"
