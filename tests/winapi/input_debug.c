#include <stdio.h>

/*
WINAPI_CALL: MapVirtualKey(vk=65, map_type=0)
WINAPI_CALL: GetAsyncKeyState(vk=16)
WINAPI_CALL: GetKeyState(vk=20)
WINAPI_CALL: keybd_event(vk=65, scan=0, flags=0, extra=0)
WINAPI_CALL: keybd_event(vk=65, scan=0, flags=2, extra=0)
WINAPI_CALL: mouse_event(flags=1, dx=1, dy=1, data=0, extra=0)
WINAPI_CALL: SendInput(count=1, inputs="mouse")
*/

int main(void) {
    puts("input_debug spec");
    return 0;
}
