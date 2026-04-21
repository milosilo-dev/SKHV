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
mov al, 0x08
out 0x21, al
mov al, 0x70
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

; unmask IRQ0 only on master, mask all slave
mov al, 0xFE
out 0x21, al
mov al, 0xFF
out 0xA1, al

; program PIT channel 0
mov al, 0x34        ; channel 0, lobyte/hibyte, rate generator
out 0x43, al
mov al, 0xA9        ; low byte of 1193
out 0x40, al
mov al, 0x04        ; high byte
out 0x40, al

mov al, 0x0B
out 0x70, al
mov al, 0b00000110
out 0x71, al

sti

main:
    jmp main

timer_handler:
    mov al, 0x04
    out 0x70, al
    in al, 0x71
    xor ah, ah
    call print_int

    mov dx, 0x3F8
    mov al, ':' ; print ":"
    out dx, al

    mov al, 0x02
    out 0x70, al
    in al, 0x71
    xor ah, ah
    call print_int

    mov dx, 0x3F8
    mov al, ':' ; print ":"
    out dx, al

    mov al, 0x00
    out 0x70, al
    in al, 0x71
    xor ah, ah
    call print_int

    mov dx, 0x3F8
    mov al, 0x0A ; print newline
    out dx, al

    mov al, 0x20    ; EOI
    out 0x20, al

    iret

; input: ax = integer to print
print_int:
    mov bx, 0

.digit_loop:
    xor dx, dx
    mov cx, 10
    div cx
    add dl, '0'
    push dx
    inc bx
    test ax, ax
    jnz .digit_loop

.print_loop:
    pop ax
    mov dx, 0x3F8
    out dx, al
    dec bx
    jnz .print_loop

    ret