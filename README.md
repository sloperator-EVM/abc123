# winrun + waygate (prototype)

This repository provides a Linux compatibility prototype for Win32-style binaries.

## CLI modes

- `winrun <file>`: run mode.
- `winrun -d <file>`: run mode + debug logs.
- `winrun -c <file>`: compile-only mode (do not run), generate a `.waygate.plan` file.
- `winrun -cd <file>`: compile-only mode + debug logs.

## How it runs (end-to-end)

### 1) Input inspection
`winrun` reads the target file, checks format (`ELF`, `PE/COFF`, `unknown`), and chooses a path.

### 2) Native ELF path
- `winrun` / `winrun -d`: runs the ELF directly on Linux.
- Debug mode optionally attempts gdb tracing for known Win32-style symbols.
- `winrun -c` / `-cd`: does not execute; writes a note plan (`<file>.waygate.plan`) saying translation is not required for native ELF.

### 3) Non-native path (`.exe` / custom `.bin`)
- gdb trace is skipped (non-native files can’t run before compatibility translation).
- Strings are scanned for known Win32 API symbols and inline argument snippets like `Sleep(ms=10)`.
- A plan file is written beside the target: `<file>.waygate.plan`.
- In run mode (`winrun` / `-d`) the plan is executed immediately through `waygate` stubs in detected order.
- In compile-only mode (`-c` / `-cd`) the plan is generated only; no execution happens.

## Plan file format

`<target>.waygate.plan` is line-based:

- Header: `# waygate execution plan`
- Each call: `<index>\t<FunctionName>\t<arg_name:type=value||...>`

Example:

```text
1	SetLastError	code:int=5
2	Sleep	ms:int=25
3	CreateThread	attrs:int=0||stack:int=0||func:string="worker"||param:int=0
```

## Components

- `winrun/`: CLI runner and scanner/plan generator.
- `waygate/`: placeholder Linux-side implementations (stubs) for detected Win32 API names.
- `tests/samples/`:
  - `native.c` -> built as `native_sample`
  - `winapi_sample.bin`
  - `broken_sample.bin`

## Win32 API scan coverage

- Process/runtime: `GetLastError`, `SetLastError`, `ExitProcess`, `GetCurrentProcess`, `Sleep`, `GetTickCount`, `GetModuleHandle`, `GetProcAddress`, `LoadLibrary`, `FreeLibrary`
- Input/cursor: `SendInput`, `mouse_event`, `keybd_event`, `GetCursorPos`, `SetCursorPos`, `GetAsyncKeyState`, `GetKeyState`, `MapVirtualKey`, `ShowCursor`, `ClipCursor`
- Thread/sync/time: `CreateThread`, `WaitForSingleObject`, `CreateEvent`, `SetEvent`, `ResetEvent`, `CloseHandle`, `QueryPerformanceCounter`, `QueryPerformanceFrequency`, `GetSystemTime`, `GetLocalTime`

## Setup

```bash
./setup.sh
```

## Quick start

```bash
./target/debug/winrun tests/samples/native_sample
./target/debug/winrun -d tests/samples/winapi_sample.bin
./target/debug/winrun -c tests/samples/winapi_sample.bin
./target/debug/winrun -cd tests/samples/winapi_sample.bin
```

## Nix shell

```bash
nix-shell --run './setup.sh && ./target/debug/winrun tests/samples/native_sample && ./target/debug/winrun -d tests/samples/winapi_sample.bin && ./target/debug/winrun -c tests/samples/winapi_sample.bin'
```

## Notes

- Setup uses a temporary working directory (`.setup-tmp`) and removes it before exit.
- Generated sample binaries/plans are local artifacts and should remain untracked.
