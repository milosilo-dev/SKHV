org 0x1000

cli

xor ax, ax
mov ds, ax

; set IVT entry for IRQ0 (interrupt 0x08)
mov word [0x08 * 4], timer_handler
mov word [0x08 * 4 + 2], 0x0000

; ICW1
mov al, 0x11
out 0x20, al
out 0xA0, al

; ICW2 (vector offsets)
mov al, 0x08      ; master PIC → 0x08–0x0F
out 0x21, al
mov al, 0x70      ; slave PIC → 0x70–0x77
out 0xA1, al

; ICW3
mov al, 0x04
out 0x21, al
mov al, 0x02
out 0xA1, al

; ICW4
mov al, 0x01
out 0x21, al
out 0xA1, al

; unmask IRQ0
mov al, 0xFE
out 0x21, al
mov al, 0xFD
out 0xA1, al

sti

main:
    hlt
    jmp main

timer_handler:
    mov dx, 0x3F8
    mov al, 'X'
    out dx, al

    mov al, 0x20
    out 0x20, al

    iret