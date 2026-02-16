#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TMP_DIR="${1:-$SCRIPT_DIR/.tmp-generate}"
mkdir -p "$TMP_DIR"

WIN_TMP="$TMP_DIR/win32_api_sample.bin"
BROKEN_TMP="$TMP_DIR/broken_lib_sample.bin"

cat > "$WIN_TMP" <<'SAMPLE'
MZFAKE
KERNEL32.dll
USER32.dll
CreateFileA(path="fake.txt", mode="r")
ReadFile(handle=1, size=128)
WriteFile(handle=1, data="hello")
CloseHandle(handle=1)
MessageBoxA(title="winrun", text="demo")
SAMPLE

cat > "$BROKEN_TMP" <<'SAMPLE'
NOTELF
libmystery.so
libbroken_custom.so
UnknownEntryPoint
SAMPLE

mv "$WIN_TMP" "$SCRIPT_DIR/win32_api_sample.bin"
mv "$BROKEN_TMP" "$SCRIPT_DIR/broken_lib_sample.bin"

echo "generated $SCRIPT_DIR/win32_api_sample.bin"
echo "generated $SCRIPT_DIR/broken_lib_sample.bin"
