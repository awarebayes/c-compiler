.section __TEXT,__text
.extern _printf
.globl _main
_main:
stp x29, x30, [sp, -16]!
mov x29, sp
sub sp, sp, 128
// @start_function_main:
start_function_main:
// 	%a.0 =w #4
mov w5, 4
mov w0, w5
// 	%b.0 =w #80
mov w5, 80
mov w1, w5
// 	%c.0 =w #50
mov w5, 50
mov w2, w5
// 	%a.1 =w %a.0
mov w3, w0
// 	%b.1 =w %b.0
mov w4, w1
// 	%c.1 =w %c.0
mov w5, w2
str w5, [sp, 0]
// 	%a.3 =w %a.0
mov w5, w0
str w5, [sp, 8]
// 	%b.3 =w %b.0
mov w0, w1
// 	%c.3 =w %c.0
mov w1, w2
// @_l0:
L_main_0:
// 	%_t3 =w %a.1
mov w2, w3
// 	%_t4 =w #0
mov w5, 0
str w5, [sp, 16]
// 	%_t5 =w %_t3 > %_t4
ldr w6, [sp, 16]
cmp w2, w6
cset w7, gt
str w7, [sp, 24]
// 	branchw %_t5: _l1 _l2
ldr w5, [sp, 24]
cmp w5, 1
beq L_main_1
bne L_main_2
// @_l1:
L_main_1:
// 	%_t6 =l s'A is %d, b is %d, c is %d\n'
adrp x2, sl0@PAGE
add x2, x2, sl0@PAGEOFF
// 	%_t7 =w %a.1
mov w5, w3
str w5, [sp, 32]
// 	%_t8 =w %b.1
mov w5, w4
str w5, [sp, 40]
// 	%_t9 =w %c.1
ldr w6, [sp, 0]
mov w5, w6
str w5, [sp, 48]
// 	%_t10 =w call %printf.0 with (param0 l %_t6, vparam1 w %_t7, vparam2 w %_t8, vparam3 w %_t9)
sub sp, sp, 48
// Spilling x4 which is in use
str x4, [sp, 0]
// Spilling x0 which is in use
str x0, [sp, 8]
// Spilling x3 which is in use
str x3, [sp, 16]
// Spilling x1 which is in use
str x1, [sp, 24]
// Spilling x2 which is in use
str x2, [sp, 32]
// param0 l %_t6
mov x0, x2
sub sp, sp, 32
// vparam1 w %_t7
ldr w5, [sp, 112]
str x5, [sp, 0]
// vparam2 w %_t8
ldr w5, [sp, 120]
str x5, [sp, 8]
// vparam3 w %_t9
ldr w5, [sp, 128]
str x5, [sp, 16]
bl _printf
mov w5, w0
// Variadic parameters pop
add sp, sp, 32
// Popping x4 which was in use
ldr x4, [sp, 0]
// Popping x0 which was in use
ldr x0, [sp, 8]
// Popping x3 which was in use
ldr x3, [sp, 16]
// Popping x1 which was in use
ldr x1, [sp, 24]
// Popping x2 which was in use
ldr x2, [sp, 32]
add sp, sp, 48
str w5, [sp, 56]
// 	%_t11 =w #3
mov w5, 3
mov w2, w5
// 	%_t12 =w %a.1 - %_t11
sub w7, w3, w2
str w7, [sp, 64]
// 	%a.2 =w %_t12
ldr w6, [sp, 64]
mov w2, w6
// 	%_t13 =w #4
mov w5, 4
str w5, [sp, 72]
// 	%_t14 =w %b.1 - %_t13
ldr w6, [sp, 72]
sub w7, w4, w6
str w7, [sp, 80]
// 	%b.2 =w %_t14
ldr w6, [sp, 80]
mov w5, w6
str w5, [sp, 88]
// 	%_t15 =w #1
mov w5, 1
str w5, [sp, 96]
// 	%_t16 =w %c.1 - %_t15
ldr w5, [sp, 0]
ldr w6, [sp, 96]
sub w7, w5, w6
str w7, [sp, 104]
// 	%c.2 =w %_t16
ldr w6, [sp, 104]
mov w5, w6
str w5, [sp, 112]
// 	%a.1 =w %a.2
mov w3, w2
// 	%b.1 =w %b.2
ldr w6, [sp, 88]
mov w4, w6
// 	%c.1 =w %c.2
ldr w6, [sp, 112]
mov w5, w6
str w5, [sp, 0]
// 	%a.3 =w %a.2
mov w5, w2
str w5, [sp, 8]
// 	%b.3 =w %b.2
ldr w6, [sp, 88]
mov w0, w6
// 	%c.3 =w %c.2
ldr w6, [sp, 112]
mov w1, w6
// 	jump _l0
b L_main_0
// @_l2:
L_main_2:
// 	%_t17 =w #1
mov w5, 1
mov w2, w5
// 	%_t18 =w %c.3 + %_t17
add w3, w1, w2
// 	%c.4 =w %_t18
mov w1, w3
// 	%_t19 =w #1
mov w5, 1
mov w2, w5
// 	%_t20 =w %a.3 + %_t19
ldr w5, [sp, 8]
add w3, w5, w2
// 	%a.4 =w %_t20
mov w2, w3
// 	%_t21 =l s'Final A is %d, b is %d, c is %d\n'
adrp x3, sl1@PAGE
add x3, x3, sl1@PAGEOFF
// 	%_t22 =w %a.4
mov w4, w2
// 	%_t23 =w %b.3
mov w2, w0
// 	%_t24 =w %c.4 check
mov w0, w1
// 	%_t25 =w call %printf.0 with (param0 l %_t21, vparam1 w %_t22, vparam2 w %_t23, vparam3 w %_t24)
sub sp, sp, 48
// Spilling x3 which is in use
str x3, [sp, 0]
// Spilling x1 which is in use
str x1, [sp, 8]
// Spilling x0 which is in use
str x0, [sp, 16]
// Spilling x4 which is in use
str x4, [sp, 24]
// Spilling x2 which is in use
str x2, [sp, 32]





HERE IS THE BUG!

// param0 l %_t21
mov x0, x3







sub sp, sp, 32
// vparam1 w %_t22
str x4, [sp, 0]
// vparam2 w %_t23
str x2, [sp, 8]
// vparam3 w %_t24
str x0, [sp, 16]
bl _printf
mov w5, w0
// Variadic parameters pop
add sp, sp, 32
// Popping x3 which was in use
ldr x3, [sp, 0]
// Popping x1 which was in use
ldr x1, [sp, 8]
// Popping x0 which was in use
ldr x0, [sp, 16]
// Popping x4 which was in use
ldr x4, [sp, 24]
// Popping x2 which was in use
ldr x2, [sp, 32]
add sp, sp, 48
mov w1, w5
// 	return w #0
mov w0, 0
b return_main
return_main:
add sp, sp, 128
ldp x29, x30, [sp], 16
ret
.section __TEXT,__cstring
sl0:
.asciz "A is %d, b is %d, c is %d\n"
sl1:
.asciz "Final A is %d, b is %d, c is %d\n"
