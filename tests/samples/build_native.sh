#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cc "$SCRIPT_DIR/native.c" -o "$SCRIPT_DIR/native_sample"
chmod +x "$SCRIPT_DIR/native_sample"
echo "built $SCRIPT_DIR/native_sample"
