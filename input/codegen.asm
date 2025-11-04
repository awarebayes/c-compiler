.section __TEXT,__text
.extern _puts
.globl _other_func
.globl _main
_other_func:
stp x29, x30, [sp, -16]!
mov x29, sp
sub sp, sp, 64
start_function_other_func:
mov w0, 5
str w0, [sp, 0]
ldr w0, [sp, 0]
str w0, [sp, 4]
ldr w0, [sp, 4]
str w0, [sp, 8]
ldr w0, [sp, 4]
str w0, [sp, 12]
L_other_func_0:
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
beq L_other_func_1
bne L_other_func_2
L_other_func_1:
adrp x0, sl0@PAGE
add x0, x0, sl0@PAGEOFF
str x0, [sp, 28]
ldr x0, [sp, 28]
bl _puts
str w0, [sp, 36]
mov w0, 1
str w0, [sp, 40]
ldr w0, [sp, 8]
ldr w1, [sp, 40]
sub w0, w0, w1
str w0, [sp, 44]
ldr w0, [sp, 44]
str w0, [sp, 48]
ldr w0, [sp, 48]
str w0, [sp, 8]
ldr w0, [sp, 48]
str w0, [sp, 12]
b L_other_func_0
L_other_func_2:
mov w0, 0
str w0, [sp, 52]
ldr w0, [sp, 52]
b return_other_func
b return_other_func
return_other_func:
add sp, sp, 64
ldp x29, x30, [sp], 16
ret
_main:
stp x29, x30, [sp, -16]!
mov x29, sp
sub sp, sp, 96
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
mov w0, 0
str w0, [sp, 20]
ldr w0, [sp, 20]
str w0, [sp, 24]
ldr w0, [sp, 4]
str w0, [sp, 28]
mov w0, 0
str w0, [sp, 32]
ldr w0, [sp, 28]
ldr w1, [sp, 32]
cmp w0, w1
cset w0, eq
str w0, [sp, 36]
ldr w0, [sp, 24]
str w0, [sp, 40]
ldr w0, [sp, 36]
cmp w0, 1
beq L_main_0
bne L_main_1
L_main_0:
bl _other_func
str w0, [sp, 44]
ldr w0, [sp, 44]
str w0, [sp, 48]
ldr w0, [sp, 48]
str w0, [sp, 40]
L_main_1:
ldr w0, [sp, 4]
str w0, [sp, 52]
ldr w0, [sp, 40]
str w0, [sp, 56]
ldr w0, [sp, 52]
ldr w1, [sp, 56]
add w0, w0, w1
str w0, [sp, 60]
ldr w0, [sp, 60]
str w0, [sp, 64]
adrp x0, sl2@PAGE
add x0, x0, sl2@PAGEOFF
str x0, [sp, 68]
ldr x0, [sp, 68]
bl _puts
str w0, [sp, 76]
ldr w0, [sp, 64]
str w0, [sp, 80]
ldr w0, [sp, 80]
b return_main
b return_main
return_main:
add sp, sp, 96
ldp x29, x30, [sp], 16
ret
.section __TEXT,__cstring
sl0:
.asciz "a"
sl1:
.asciz "b"
sl2:
.asciz "c"
