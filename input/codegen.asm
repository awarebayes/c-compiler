.section __TEXT,__text
.extern _puts
.extern _printf
.globl _main
_main:
stp x29, x30, [sp, -16]!
mov x29, sp
sub sp, sp, 0
start_function_main:
mov w5, 5
mov w0, w5
mov x1, x0
mov w1, w1
mov x0, x1
mov w0, w0
mov x2, x1
mov w2, w2
L_main_0:
mov x1, x0
mov w1, w1
mov w5, 0
mov w3, w5
cmp w1, w3
cset w4, gt
mov w4, w4
cmp w4, 1
beq L_main_1
bne L_main_2
L_main_1:
mov w5, 1
mov w1, w5
sub w3, w0, w1
mov w3, w3
mov x1, x3
mov w1, w1
adrp x3, sl0@PAGE
add x3, x3, sl0@PAGEOFF
mov x4, x1
mov w4, w4
sub sp, sp, 32
str x0, [sp, 0]
mov x0, x3
sub sp, sp, 32
str x1, [sp, 0]
mov w1, w4
bl _printf
mov w5, w0
mov w3, w5
ldr x1, [sp, 0]
add sp, sp, 32
ldr x0, [sp, 0]
add sp, sp, 32
mov x0, x1
mov w0, w0
mov x2, x1
mov w2, w2
b L_main_0
L_main_2:
mov w5, 0
mov w0, w5
mov w0, w0
b return_main
b return_main
return_main:
add sp, sp, 0
ldp x29, x30, [sp], 16
ret
.section __TEXT,__cstring
sl0:
.asciz "Hello world %d\n"
