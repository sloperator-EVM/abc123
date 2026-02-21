#include <stdio.h>
#include <Windows.h>

int main(void) {
    POINT p = {0};

    puts("setpos_debug start");
    SetCursorPos(500, 300);
    GetCursorPos(&p);
    ShowCursor(TRUE);
    ShowCursor(FALSE);

    printf("cursor after SetCursorPos: x=%ld y=%ld\n", (long)p.x, (long)p.y);
    return 0;
}
