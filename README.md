# winrun + waygate (prototype)

This repository provides a Linux compatibility prototype for Win32-style binaries.

## What runs where

- **Native ELF target**
  1. `winrun` reads file metadata + magic bytes.
  2. If file is ELF, debug mode optionally runs gdb trace (`rbreak -> run -> frame/info args/backtrace -> continue`).
  3. The ELF is executed directly on Linux.
- **Non-native target (`.exe`/custom `.bin`)**
  1. `winrun` skips gdb trace (non-native binaries cannot run before compatibility translation).
  2. It extracts strings, scans for known Win32 API names, and lists unresolved `.so` references.
  3. Detected API calls are replayed in order through `waygate` stubs.

## Components

- `winrun/`: CLI runner.
  - `winrun <binary>`: normal mode.
  - `winrun -d <binary>`: debug mode (format, native capability, trace/scan details, dispatch logs).
- `waygate/`: compatibility library with placeholder WinAPI-shaped stubs.
- `tests/samples/`:
  - `native.c` -> built as `native_sample`
  - `winapi_sample.bin`
  - `broken_sample.bin`

## Win32 API scan coverage

The scanner and waygate dispatcher include:

- Process/runtime: `GetLastError`, `SetLastError`, `ExitProcess`, `GetCurrentProcess`, `Sleep`, `GetTickCount`, `GetModuleHandle`, `GetProcAddress`, `LoadLibrary`, `FreeLibrary`
- Input/cursor: `SendInput`, `mouse_event`, `keybd_event`, `GetCursorPos`, `SetCursorPos`, `GetAsyncKeyState`, `GetKeyState`, `MapVirtualKey`, `ShowCursor`, `ClipCursor`
- Thread/sync/time: `CreateThread`, `WaitForSingleObject`, `CreateEvent`, `SetEvent`, `ResetEvent`, `CloseHandle`, `QueryPerformanceCounter`, `QueryPerformanceFrequency`, `GetSystemTime`, `GetLocalTime`

## Nix shell

Use this command to enter a shell with all tools needed for setup and running the samples:

```bash
nix-shell --run './setup.sh && ./target/debug/winrun tests/samples/native_sample && ./target/debug/winrun -d tests/samples/winapi_sample.bin && ./target/debug/winrun -d tests/samples/broken_sample.bin'
```

If you only want an interactive environment first:

```bash
nix-shell
```

## Quick start

```bash
./setup.sh
./target/debug/winrun tests/samples/native_sample
./target/debug/winrun -d tests/samples/winapi_sample.bin
./target/debug/winrun -d tests/samples/broken_sample.bin
```

## Notes

- `winrun -d` uses gdb tracing only for native ELF executables.
- Setup uses a temporary working directory (`.setup-tmp`) and removes it before exit.
