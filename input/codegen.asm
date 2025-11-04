.section __TEXT,__text
.extern _puts
.globl _main
_main:
stp x29, x30, [sp, -16]!
mov x29, sp
sub sp, sp, 48
start_function_main:
mov w0, 0
str w0, [sp, 0]
ldr w0, [sp, 0]
str w0, [sp, 4]
L0:
ldr w0, [sp, 4]
str w0, [sp, 8]
mov w0, 5
str w0, [sp, 12]
ldr w0, [sp, 8]
ldr w1, [sp, 12]
cmp w0, w1
cset w0, lt
str w0, [sp, 16]
ldr w0, [sp, 24]
cmp w0, 1
beq L1
bne L2
L1:
mov w0, 1
str w0, [sp, 20]
ldr w0, [sp, 4]
ldr w1, [sp, 20]
add w0, w0, w1
str w0, [sp, 24]
ldr w0, [sp, 24]
str w0, [sp, 28]
b L0
L2:
ldr w0, [sp, 28]
str w0, [sp, 32]
return_main:
add sp, sp, 48
ldp x29, x30, [sp], 16
ret
.section __TEXT,__cstring
