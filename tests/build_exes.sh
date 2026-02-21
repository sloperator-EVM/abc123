#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WINAPI_DIR="$SCRIPT_DIR/winapi"

make_synthetic_exe() {
  local src="$1"
  local exe="$2"

  python - "$src" "$exe" <<'PY'
import re
import sys
from pathlib import Path

src = Path(sys.argv[1])
exe = Path(sys.argv[2])
text = src.read_text()

# Per-function argument names used when source call uses positional arguments.
ARG_NAMES = {
    "SetCursorPos": ["x", "y"],
    "GetCursorPos": ["point"],
    "ShowCursor": ["show"],
    "SetLastError": ["code"],
    "GetLastError": [],
    "GetTickCount": [],
    "LoadLibrary": ["path"],
    "GetProcAddress": ["module", "name"],
    "GetModuleHandle": ["module"],
    "FreeLibrary": ["module"],
    "Sleep": ["ms"],
    "MapVirtualKey": ["vk", "map_type"],
    "GetAsyncKeyState": ["vk"],
    "GetKeyState": ["vk"],
    "keybd_event": ["vk", "scan", "flags", "extra"],
    "mouse_event": ["flags", "dx", "dy", "data", "extra"],
    "SendInput": ["cInputs", "pInputs", "cbSize"],
    "CreateEvent": ["attrs", "manual", "initial", "name"],
    "CreateThread": ["attrs", "stack", "func", "param"],
    "SetEvent": ["handle"],
    "ResetEvent": ["handle"],
    "WaitForSingleObject": ["handle", "timeout"],
    "QueryPerformanceFrequency": ["counter"],
    "QueryPerformanceCounter": ["counter"],
    "GetSystemTime": ["time"],
    "GetLocalTime": ["time"],
    "CloseHandle": ["handle"],
}

SKIP = {"if", "for", "while", "switch", "return", "puts", "printf", "sizeof"}

# Parse simple variable assignments so args can resolve symbolic names -> values.
vars_map = {}
assign_re = re.compile(r"\b(?:const\s+)?(?:unsigned\s+|signed\s+)?(?:int|short|long|char|float|double|size_t|uint\d+_t|int\d+_t|UINT|DWORD|WORD|BYTE|BOOL|LPINPUT|INPUT\s*\*)\s+([A-Za-z_]\w*)\s*=\s*([^;]+);")
for m in assign_re.finditer(text):
    name, value = m.group(1), m.group(2).strip()
    vars_map[name] = value

calls = []
call_re = re.compile(r"\b([A-Za-z_]\w*)\s*\(([^;()]*)\)\s*;")
for m in call_re.finditer(text):
    func = m.group(1)
    if func in SKIP:
        continue

    raw_args = [a.strip() for a in m.group(2).split(",") if a.strip()]
    named = []
    name_hints = ARG_NAMES.get(func, [])

    for i, arg in enumerate(raw_args):
        # Keep explicitly named args as-is.
        if "=" in arg and not arg.strip().startswith(('"', "'")):
            named.append(arg)
            continue

        key = name_hints[i] if i < len(name_hints) else f"arg{i + 1}"
        if re.fullmatch(r"[A-Za-z_]\w*", arg) and arg in vars_map:
            named.append(f"{arg}={vars_map[arg]}")
        else:
            named.append(f"{key}={arg}")

    if named:
        calls.append(f"{func}({', '.join(named)})")
    else:
        calls.append(f"{func}()")

with exe.open("w", encoding="utf-8") as f:
    f.write("MZFAKE\n")
    f.write("KERNEL32.dll\n")
    f.write("USER32.dll\n")
    for line in calls:
        f.write(line + "\n")
PY
}

built=0
for src in "$WINAPI_DIR"/*.c; do
  exe="${src%.c}.exe"
  echo "generating $(basename "$exe") from $(basename "$src")"
  make_synthetic_exe "$src" "$exe"
  built=$((built + 1))
done

echo "done: generated $built synthetic PE/COFF test .exe file(s)"
