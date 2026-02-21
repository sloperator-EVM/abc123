#include <stdint.h>
#include <stdio.h>

int main(void) {
    int cInputs = 4;
    LPINPUT pInputs = inputs;
    int cbSize = 40;

    SendInput(cInputs, pInputs, cbSize);
    keybd_event(65, 0, 0, 0);
    keybd_event(65, 0, 2, 0);
    mouse_event(1, 1, 1, 0, 0);
    MapVirtualKey(65, 0);
    GetAsyncKeyState(16);
    GetKeyState(20);

    return 0;
}
