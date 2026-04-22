#include "serial.h"

void c_main(void) {
    serial_init();
    serial_puts("Hello from protected mode!\n");

    // spin forever
    for (;;) {
        __asm__ volatile("hlt");
    }
}