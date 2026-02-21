# winrun + waygate (prototype)

Linux compatibility prototype for Win32-style binaries.

## CLI modes

- `winrun <file>`: run mode.
- `winrun -d <file>`: run mode + detailed debug logs.
- `winrun -c <file>`: compile-only mode (writes `.waygate.plan`, no execution).
- `winrun -cd <file>`: compile-only mode + debug logs.

## Test layout

`tests/winapi/*.c` contains debug Win32 API programs compiled into `.exe` files with MinGW.

Included debug programs currently validate scanner/dispatch coverage for:

- cursor APIs (`SetCursorPos`, `GetCursorPos`, `ShowCursor`)
- input APIs (`SendInput`, `mouse_event`, `keybd_event`, `MapVirtualKey`, `GetAsyncKeyState`, `GetKeyState`)
- runtime APIs (`SetLastError`, `GetLastError`, `Sleep`, `GetTickCount`, `GetModuleHandleA`, `GetProcAddress`, `LoadLibraryA`, `FreeLibrary`)
- threading/time APIs (`CreateThread`, `WaitForSingleObject`, `CreateEventA`, `SetEvent`, `ResetEvent`, `CloseHandle`, `QueryPerformanceCounter`, `QueryPerformanceFrequency`, `GetSystemTime`, `GetLocalTime`)

## Setup / installer

```bash
./setup.sh
```

`setup.sh` now:

1. formats Rust code
2. builds workspace
3. compiles all `tests/winapi/*.c` into `.exe`
4. runs `tests/test.sh` (auto-discovers all `.exe` and runs `winrun -d`)

## Manual commands

```bash
./tests/build_exes.sh
./tests/test.sh
./target/debug/winrun -d tests/winapi/setpos_debug.exe
```

## Nix shell (with MinGW)

The provided `shell.nix` includes Rust + MinGW, so setup works end-to-end:

```bash
nix-shell --run './setup.sh'
```
