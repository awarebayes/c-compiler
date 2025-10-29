    .section __TEXT,__text
    .globl _main
    .extern _puts

_main:
    stp     x29, x30, [sp, -16]!   // save frame + link register
    mov     x29, sp

    mov     w0, #0                 // a = 0   <-- initialization

L0:
    cmp     w0, #5                 // a < 5 ?
    bge     L2                     // if a >= 5 -> L2

    add     w0, w0, #1             // a = a + 1
    cmp     w0, #3                 // a > 3 ?
    ble     L4                     // if a <= 3 -> L4

L3:
    adrp    x0, msg@PAGE           // x0 = &">3!"
    add     x0, x0, msg@PAGEOFF
    bl      _puts                  // puts(">3!")

L4:
    b       L0                     // loop

L2:
    ldp     x29, x30, [sp], 16
    ret


.section __TEXT,__cstring
msg:
    .asciz  ">3!"
