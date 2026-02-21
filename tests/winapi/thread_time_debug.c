#include <stdint.h>
#include <stdio.h>

int main(void) {
    int handle = 1;
    int timeout = 1000;

    CreateEvent(0, true, false, "waygate_evt");
    CreateThread(0, 0, worker, 0);
    SetEvent(handle);
    ResetEvent(handle);
    WaitForSingleObject(handle, timeout);
    QueryPerformanceFrequency(counterOut);
    QueryPerformanceCounter(counterOut);
    GetSystemTime(systemTimeOut);
    GetLocalTime(localTimeOut);
    CloseHandle(handle);

    return 0;
}
