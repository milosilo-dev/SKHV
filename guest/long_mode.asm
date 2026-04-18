org 0x1000

; -----------------------------
; Fix GDT pointer at runtime
; -----------------------------
start:
    call get_base
get_base:
    pop ebx
    sub ebx, get_base        ; EBX = actual load base

    ; patch GDT base into gdt_ptr
    lea eax, [ebx + gdt]
    mov [ebx + gdt_ptr + 2], eax

    lgdt [ebx + gdt_ptr]

    ; -----------------------------
    ; Enter protected mode
    ; -----------------------------
    mov eax, cr0
    or eax, 0x1
    mov cr0, eax

    jmp 0x08:protected_mode   ; far jump using 32-bit code segment

; -----------------------------
; 32-bit protected mode
; -----------------------------
[bits 32]
protected_mode:
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov ss, ax

    ; test output (serial)
    mov dx, 0x3F8
    mov al, 'P'
    out dx, al

.hang:
    jmp .hang

; -----------------------------
; GDT (MUST be 32-bit here)
; -----------------------------
[bits 16]
gdt:
    dq 0x0000000000000000        ; null
    dq 0x00CF9A000000FFFF        ; 32-bit code segment
    dq 0x00CF92000000FFFF        ; data segment

gdt_ptr:
    dw gdt_end - gdt - 1
    dd 0x0                       ; patched at runtime

gdt_end: