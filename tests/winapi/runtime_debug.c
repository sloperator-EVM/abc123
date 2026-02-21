#include <stdio.h>

/*
WINAPI_CALL: SetLastError(code=42)
WINAPI_CALL: GetLastError()
WINAPI_CALL: GetTickCount()
WINAPI_CALL: LoadLibrary(path="user32.dll")
WINAPI_CALL: GetProcAddress(module=1, name="SetCursorPos")
WINAPI_CALL: GetModuleHandle(module="kernel32.dll")
WINAPI_CALL: FreeLibrary(module=1)
WINAPI_CALL: Sleep(ms=10)
*/

int main(void) {
    puts("runtime_debug spec");
    return 0;
}
