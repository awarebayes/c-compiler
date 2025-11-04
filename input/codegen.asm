.section __TEXT,__text
.extern _puts
.globl _other_func
.globl _main
_other_func:
stp x29, x30, [sp, -16]!
mov x29, sp
sub sp, sp, 48
start_function_other_func:
mov w0, 5
str w0, [sp, 0]
ldr w0, [sp, 0]
str w0, [sp, 4]
L0:
ldr w0, [sp, 4]
str w0, [sp, 8]
mov w0, 0
str w0, [sp, 12]
ldr w0, [sp, 8]
ldr w1, [sp, 12]
cmp w0, w1
cset w0, gt
str w0, [sp, 16]
ldr w0, [sp, 16]
cmp w0, 1
beq L1
bne L2
L1:
adrp x0, sl0@PAGE
add x0, x0, sl0@PAGEOFF
str x0, [sp, 20]
ldr x0, [sp, 20]
bl _puts
str w0, [sp, 28]
mov w0, 1
str w0, [sp, 32]
ldr w0, [sp, 4]
ldr w1, [sp, 32]
sub w0, w0, w1
str w0, [sp, 36]
ldr w0, [sp, 36]
str w0, [sp, 4]
b L0
L2:
ldr w0, [sp, 4]
str w0, [sp, 40]
return_other_func:
add sp, sp, 48
ldp x29, x30, [sp], 16
ret
_main:
stp x29, x30, [sp, -16]!
mov x29, sp
sub sp, sp, 64
start_function_main:
bl _other_func
str w0, [sp, 0]
ldr w0, [sp, 0]
str w0, [sp, 4]
adrp x0, sl1@PAGE
add x0, x0, sl1@PAGEOFF
str x0, [sp, 8]
ldr x0, [sp, 8]
bl _puts
str w0, [sp, 16]
bl _other_func
str w0, [sp, 20]
ldr w0, [sp, 20]
str w0, [sp, 24]
ldr w0, [sp, 4]
str w0, [sp, 28]
ldr w0, [sp, 24]
str w0, [sp, 32]
ldr w0, [sp, 28]
ldr w1, [sp, 32]
add w0, w0, w1
str w0, [sp, 36]
ldr w0, [sp, 36]
str w0, [sp, 40]
adrp x0, sl2@PAGE
add x0, x0, sl2@PAGEOFF
str x0, [sp, 44]
ldr x0, [sp, 44]
bl _puts
str w0, [sp, 52]
ldr w0, [sp, 40]
str w0, [sp, 56]
return_main:
add sp, sp, 64
ldp x29, x30, [sp], 16
ret
.section __TEXT,__cstring
sl1:
.asciz "b"
sl0:
.asciz "a"
sl2:
.asciz "c"
