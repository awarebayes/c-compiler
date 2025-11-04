.section __TEXT,__text
.extern _puts
.extern _printf
.globl _main
_main:
stp x29, x30, [sp, -16]!
mov x29, sp
sub sp, sp, 64
start_function_main:
mov w0, 5
str w0, [sp, 0]
ldr w0, [sp, 0]
str w0, [sp, 4]
ldr w0, [sp, 4]
str w0, [sp, 8]
ldr w0, [sp, 4]
str w0, [sp, 12]
L_main_0:
ldr w0, [sp, 8]
str w0, [sp, 16]
mov w0, 0
str w0, [sp, 20]
ldr w0, [sp, 16]
ldr w1, [sp, 20]
cmp w0, w1
cset w0, gt
str w0, [sp, 24]
ldr w0, [sp, 24]
cmp w0, 1
beq L_main_1
bne L_main_2
L_main_1:
mov w0, 1
str w0, [sp, 28]
ldr w0, [sp, 8]
ldr w1, [sp, 28]
sub w0, w0, w1
str w0, [sp, 32]
ldr w0, [sp, 32]
str w0, [sp, 36]
adrp x0, sl0@PAGE
add x0, x0, sl0@PAGEOFF
str x0, [sp, 40]
ldr w0, [sp, 36]
str w0, [sp, 48]
ldr x0, [sp, 40]
ldr w1, [sp, 48]
bl _printf
str w0, [sp, 52]
ldr w0, [sp, 36]
str w0, [sp, 8]
ldr w0, [sp, 36]
str w0, [sp, 12]
b L_main_0
L_main_2:
mov w0, 0
str w0, [sp, 56]
ldr w0, [sp, 56]
b return_main
b return_main
return_main:
add sp, sp, 64
ldp x29, x30, [sp], 16
ret
.section __TEXT,__cstring
sl0:
.asciz "A is now %d\n"
