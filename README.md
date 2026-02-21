# winrun + waygate (prototype)

Linux compatibility prototype for Win32-style binaries.

## CLI modes

- `winrun <file>`: run mode.
- `winrun -d <file>`: run mode + detailed debug logs.
- `winrun -c <file>`: compile-only mode (writes `.waygate.plan`, no execution).
- `winrun -cd <file>`: compile-only mode + debug logs.

## Test layout

`tests/winapi/*.c` are **debug specs** (plain C files) that list expected Win32 calls via lines like:

```c
WINAPI_CALL: SetCursorPos(x=500, y=300)
```

`tests/build_exes.sh` turns those specs into synthetic `.exe` fixtures (PE-like text blobs with `MZFAKE` + API call lines) so `winrun` can exercise the non-native scan/plan/dispatch path without Windows SDK headers or real Windows compilation.

Included specs currently validate scanner/dispatch coverage for:

- cursor APIs (`SetCursorPos`, `GetCursorPos`, `ShowCursor`)
- input APIs (`SendInput`, `mouse_event`, `keybd_event`, `MapVirtualKey`, `GetAsyncKeyState`, `GetKeyState`)
- runtime APIs (`SetLastError`, `GetLastError`, `Sleep`, `GetTickCount`, `GetModuleHandle`, `GetProcAddress`, `LoadLibrary`, `FreeLibrary`)
- threading/time APIs (`CreateThread`, `WaitForSingleObject`, `CreateEvent`, `SetEvent`, `ResetEvent`, `CloseHandle`, `QueryPerformanceCounter`, `QueryPerformanceFrequency`, `GetSystemTime`, `GetLocalTime`)

## Setup / installer

```bash
./setup.sh
```

`setup.sh` now:

1. formats Rust code
2. builds workspace
3. generates all `tests/winapi/*.exe` fixtures from spec `.c` files
4. runs `tests/test.sh` (auto-discovers all `.exe` and runs `winrun -d`)

## Manual commands

```bash
./tests/build_exes.sh
./tests/test.sh
./target/debug/winrun -d tests/winapi/setpos_debug.exe
./target/debug/winrun -c tests/winapi/setpos_debug.exe
```

## Nix shell

```bash
nix-shell --run './setup.sh'
```
