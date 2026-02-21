#include <stdint.h>
#include <stdio.h>

int main(void) {
    int code = 42;
    int ms = 10;

    SetLastError(code);
    GetLastError();
    GetTickCount();
    LoadLibrary("user32.dll");
    GetProcAddress(1, "SetCursorPos");
    GetModuleHandle("kernel32.dll");
    FreeLibrary(1);
    Sleep(ms);

    return 0;
}
