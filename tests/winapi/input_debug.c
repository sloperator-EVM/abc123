#include <stdio.h>
#include <Windows.h>

int main(void) {
    WORD map = 0;
    SHORT async_state = 0;
    SHORT key_state = 0;
    INPUT in = {0};

    puts("input_debug start");

    map = (WORD)MapVirtualKeyA('A', MAPVK_VK_TO_VSC);
    async_state = GetAsyncKeyState(VK_SHIFT);
    key_state = GetKeyState(VK_CAPITAL);

    keybd_event((BYTE)'A', 0, 0, 0);
    keybd_event((BYTE)'A', 0, KEYEVENTF_KEYUP, 0);

    mouse_event(MOUSEEVENTF_MOVE, 1, 1, 0, 0);

    in.type = INPUT_MOUSE;
    in.mi.dwFlags = MOUSEEVENTF_MOVE;
    in.mi.dx = 1;
    in.mi.dy = 1;
    SendInput(1, &in, sizeof(in));

    printf("MapVirtualKey=%u GetAsyncKeyState=%d GetKeyState=%d\n",
           (unsigned int)map,
           (int)async_state,
           (int)key_state);

    return 0;
}
