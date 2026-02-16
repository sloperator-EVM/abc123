#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cat > "$SCRIPT_DIR/win32_api_sample.bin" <<'SAMPLE'
MZFAKE
KERNEL32.dll
USER32.dll
CreateFileA
ReadFile
WriteFile
CloseHandle
MessageBoxA
SAMPLE

cat > "$SCRIPT_DIR/broken_lib_sample.bin" <<'SAMPLE'
NOTELF
libmystery.so
libbroken_custom.so
UnknownEntryPoint
SAMPLE

echo "generated $SCRIPT_DIR/win32_api_sample.bin"
echo "generated $SCRIPT_DIR/broken_lib_sample.bin"
