#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TMP_DIR="$ROOT_DIR/.setup-tmp"

cleanup() {
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

rm -rf "$TMP_DIR"
mkdir -p "$TMP_DIR"

cd "$ROOT_DIR"

echo "[1/6] formatting Rust code"
cargo fmt --all

echo "[2/6] building workspace"
cargo build --workspace

echo "[3/6] building native sample"
"$ROOT_DIR/tests/samples/build_native.sh"

echo "[4/6] generating compatibility samples"
"$ROOT_DIR/tests/samples/make_samples.sh" "$TMP_DIR"

echo "[5/6] removing setup temporary files"
cleanup

echo "[6/6] done"
echo "Use: ./target/debug/winrun [-d] tests/samples/<file>"
