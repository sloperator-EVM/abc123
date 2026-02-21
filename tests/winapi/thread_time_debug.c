#include <stdio.h>

/*
WINAPI_CALL: CreateEvent(attrs=0, manual=true, initial=false, name="waygate_evt")
WINAPI_CALL: CreateThread(attrs=0, stack=0, func="worker", param=0)
WINAPI_CALL: SetEvent(handle=1)
WINAPI_CALL: ResetEvent(handle=1)
WINAPI_CALL: WaitForSingleObject(handle=1, timeout=1000)
WINAPI_CALL: QueryPerformanceFrequency(counter="out")
WINAPI_CALL: QueryPerformanceCounter(counter="out")
WINAPI_CALL: GetSystemTime(time="out")
WINAPI_CALL: GetLocalTime(time="out")
WINAPI_CALL: CloseHandle(handle=1)
*/

int main(void) {
    puts("thread_time_debug spec");
    return 0;
}
