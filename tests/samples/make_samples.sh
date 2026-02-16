#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TMP_DIR="${1:-$SCRIPT_DIR/.tmp-generate}"
mkdir -p "$TMP_DIR"

WIN_TMP="$TMP_DIR/winapi_sample.bin"
BROKEN_TMP="$TMP_DIR/broken_sample.bin"

cat > "$WIN_TMP" <<'SAMPLE'
MZFAKE
KERNEL32.dll
USER32.dll
SetLastError(code=5)
GetLastError()
GetCurrentProcess()
Sleep(ms=25)
GetProcAddress(module=1, name="CreateThread")
LoadLibrary(path="user32.dll")
SendInput(count=1, inputs="mouse")
GetCursorPos(point="out")
GetAsyncKeyState(vk=65)
CreateThread(attrs=0, stack=0, func="worker", param=0)
WaitForSingleObject(handle=1, timeout=1000)
QueryPerformanceCounter(counter="out")
GetLocalTime(time="out")
CloseHandle(handle=1)
SAMPLE

cat > "$BROKEN_TMP" <<'SAMPLE'
NOTELF
libmystery.so
libbroken_custom.so
UnknownEntryPoint
SAMPLE

mv "$WIN_TMP" "$SCRIPT_DIR/winapi_sample.bin"
mv "$BROKEN_TMP" "$SCRIPT_DIR/broken_sample.bin"

echo "generated $SCRIPT_DIR/winapi_sample.bin"
echo "generated $SCRIPT_DIR/broken_sample.bin"
