#include "serial.h"
#include "rng.c"
#include "counter.c"

void c_main(void) {
    serial_init();
    virtio_rng_init();
    virtio_cnt_init();

    uint8_t rnd_buf[16] = {0};
    uint32_t written = virtio_rng_read(rnd_buf, 16);

    serial_puts("bytes written: "); serial_putx(written); serial_puts("\n");
    serial_puts("random bytes: ");
    for (int i = 0; i < 16; i++) {
        serial_putx(rnd_buf[i]);
        serial_putc(' ');
    }
    serial_puts("\n");

    uint8_t cnt_buf[4] = {10, 0, 0, 0};
    bool sucess = virtio_cnt(cnt_buf);

    serial_puts("counter: ");
    for (int i = 0; i < 4; i++) {
        serial_putx(cnt_buf[i]);
        serial_putc(' ');
    }
    serial_puts("\n");

    // spin forever
    while (1) {
        __asm__ volatile("hlt");
    }
}