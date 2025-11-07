.section __TEXT,__text
.extern _puts
.extern _printf
.globl _main
_main:
stp x29, x30, [sp, -16]!
mov x29, sp
sub sp, sp, 80
// @start_function_main:
start_function_main:
// 	%_t0 =w #4
mov w5, 4
mov w0, w5
// 	%a.0 =w %_t0
mov w1, w0
// 	%_t1 =w #80
mov w5, 80
mov w0, w5
// 	%b.0 =w %_t1
mov w2, w0
// 	%a.1 =w %a.0
mov w0, w1
// 	%b.1 =w %b.0
mov w3, w2
// 	%a.3 =w %a.0
mov w4, w1
// 	%b.3 =w %b.0
mov w1, w2
// @_l0:
L_main_0:
// 	%_t2 =w %b.1
mov w2, w3
// 	%_t3 =w #0
mov w5, 0
str w5, [sp, 0]
// 	%_t4 =w %_t2 > %_t3
ldr w6, [sp, 0]
cmp w2, w6
cset w7, gt
str w7, [sp, 8]
// 	branchw %_t4: _l1 _l2
ldr w5, [sp, 8]
cmp w5, 1
beq L_main_1
bne L_main_2
// @_l1:
L_main_1:
// 	%_t5 =l s'A is %d, b is %d\n'
adrp x2, sl0@PAGE
add x2, x2, sl0@PAGEOFF
// 	%_t6 =w %a.1
mov w5, w0
str w5, [sp, 16]
// 	%_t7 =w %b.1
mov w5, w3
str w5, [sp, 24]
// 	%_t8 =w call %printf.0 with (param0 l %_t5, vparam1 w %_t6, vparam2 w %_t7)
sub sp, sp, 48
// Spilling x4 which is in use
str x4, [sp, 0]
// Spilling x3 which is in use
str x3, [sp, 8]
// Spilling x1 which is in use
str x1, [sp, 16]
// Spilling x2 which is in use
str x2, [sp, 24]
// Spilling x0 which is in use
str x0, [sp, 32]
// param0 l %_t5
mov x0, x2
sub sp, sp, 16
// vparam1 w %_t6
ldr w5, [sp, 80]
str x5, [sp, 0]
// vparam2 w %_t7
ldr w5, [sp, 88]
str x5, [sp, 8]
bl _printf
// Variadic parameters pop
add sp, sp, 16
// Popping x4 which was in use
ldr x4, [sp, 0]
// Popping x3 which was in use
ldr x3, [sp, 8]
// Popping x1 which was in use
ldr x1, [sp, 16]
// Popping x2 which was in use
ldr x2, [sp, 24]
// Popping x0 which was in use
ldr x0, [sp, 32]
add sp, sp, 48
mov w5, w0
str w5, [sp, 32]
// 	%_t9 =w #1
mov w5, 1
mov w2, w5
// 	%_t10 =w %a.1 - %_t9
sub w7, w0, w2
str w7, [sp, 40]
// 	%a.2 =w %_t10
ldr w6, [sp, 40]
mov w2, w6
// 	%_t11 =w #4
mov w5, 4
str w5, [sp, 48]
// 	%_t12 =w %b.1 - %_t11
ldr w6, [sp, 48]
sub w7, w3, w6
str w7, [sp, 56]
// 	%b.2 =w %_t12
ldr w6, [sp, 56]
mov w5, w6
str w5, [sp, 64]
// 	%a.1 =w %a.2
mov w0, w2
// 	%b.1 =w %b.2
ldr w6, [sp, 64]
mov w3, w6
// 	%a.3 =w %a.2
mov w4, w2
// 	%b.3 =w %b.2
ldr w6, [sp, 64]
mov w1, w6
// 	jump _l0
b L_main_0
// @_l2:
L_main_2:
// 	%_t13 =w %a.3
mov w0, w4
// 	return w %_t13
mov w0, w0
b return_main
return_main:
add sp, sp, 80
ldp x29, x30, [sp], 16
ret
.section __TEXT,__cstring
sl0:
.asciz "A is %d, b is %d\n"
