#include <stdio.h>
#include <Windows.h>

DWORD WINAPI worker(LPVOID arg) {
    (void)arg;
    Sleep(1);
    return 0;
}

int main(void) {
    LARGE_INTEGER freq = {0};
    LARGE_INTEGER ctr = {0};
    SYSTEMTIME sys = {0};
    SYSTEMTIME local = {0};

    puts("thread_time_debug start");

    HANDLE evt = CreateEventA(NULL, TRUE, FALSE, "waygate_evt");
    HANDLE thread = CreateThread(NULL, 0, worker, NULL, 0, NULL);

    SetEvent(evt);
    ResetEvent(evt);

    WaitForSingleObject(thread, 1000);

    QueryPerformanceFrequency(&freq);
    QueryPerformanceCounter(&ctr);
    GetSystemTime(&sys);
    GetLocalTime(&local);

    printf("freq=%lld ctr=%lld sys_h=%u local_h=%u\n",
           (long long)freq.QuadPart,
           (long long)ctr.QuadPart,
           (unsigned int)sys.wHour,
           (unsigned int)local.wHour);

    CloseHandle(thread);
    CloseHandle(evt);

    return 0;
}
