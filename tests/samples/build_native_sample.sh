#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cc "$SCRIPT_DIR/native_ok.c" -o "$SCRIPT_DIR/native_ok"
chmod +x "$SCRIPT_DIR/native_ok"
echo "built $SCRIPT_DIR/native_ok"
