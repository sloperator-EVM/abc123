#include <stdio.h>
#include <Windows.h>

int main(void) {
    DWORD code = 42;
    DWORD last = 0;
    DWORD tick = 0;

    puts("runtime_debug start");

    SetLastError(code);
    last = GetLastError();
    tick = GetTickCount();

    HMODULE user = LoadLibraryA("user32.dll");
    FARPROC proc = GetProcAddress(user, "SetCursorPos");
    HMODULE k32 = GetModuleHandleA("kernel32.dll");

    printf("GetLastError=%lu tick=%lu proc=%p module=%p\n",
           (unsigned long)last,
           (unsigned long)tick,
           (void*)proc,
           (void*)k32);

    if (user) {
        FreeLibrary(user);
    }

    Sleep(10);
    return 0;
}
