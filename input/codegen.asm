.section __TEXT,__text
.extern _puts
.extern _printf
.globl _main
_main:
stp x29, x30, [sp, -16]!
mov x29, sp
sub sp, sp, 96
// @start_function_main:
start_function_main:
// 	%_t0 =w #4
mov w5, 4
mov w0, w5
// 	%b.0 =w %_t0
mov w1, w0
// 	%_t1 =w #6
mov w5, 6
mov w0, w5
// 	%c.0 =w %_t1
mov w2, w0
// 	%_t2 =w #4
mov w5, 4
mov w0, w5
// 	%a.0 =w %_t2
mov w3, w0
// 	%a.1 =w %a.0
mov w0, w3
// 	%a.3 =w %a.0
mov w4, w3
// 	%b.2 =w %b.0
mov w3, w1
// @_l0:
L_main_0:
// 	%_t3 =w %a.1
mov w5, w0
str w5, [sp, 0]
// 	%_t4 =w #0
mov w5, 0
str w5, [sp, 8]
// 	%_t5 =w %_t3 > %_t4
ldr w5, [sp, 0]
ldr w6, [sp, 8]
cmp w5, w6
cset w7, gt
str w7, [sp, 16]
// 	branchw %_t5: _l1 _l2
ldr w5, [sp, 16]
cmp w5, 1
beq L_main_1
bne L_main_2
// @_l1:
L_main_1:
// 	%_t6 =l s'A is %d\n'
adrp x5, sl0@PAGE
add x5, x5, sl0@PAGEOFF
str x5, [sp, 24]
// 	%_t7 =w %a.1
mov w5, w0
str w5, [sp, 32]
// 	%_t8 =w call %printf.0 with (param0 l %_t6, vparam1 w %_t7)
// param0 l %_t6
ldr x5, [sp, 24]
// Register is occupied, spilling!
sub sp, sp, 32
str x0, [sp, 0]
mov x0, x5
sub sp, sp, 16
// vparam1 w %_t7
ldr w5, [sp, 80]
str w5, [sp, 0]
bl _printf
add sp, sp, 16
ldr x0, [sp, 0]
add sp, sp, 32
mov w5, w0
str w5, [sp, 40]
// 	%_t9 =w #1
mov w5, 1
str w5, [sp, 48]
// 	%_t10 =w %a.1 - %_t9
ldr w6, [sp, 48]
sub w7, w0, w6
str w7, [sp, 56]
// 	%a.2 =w %_t10
ldr w6, [sp, 56]
mov w5, w6
str w5, [sp, 64]
// 	%_t11 =w #4
mov w5, 4
str w5, [sp, 72]
// 	%_t12 =w %b.0 - %_t11
ldr w6, [sp, 72]
sub w7, w1, w6
str w7, [sp, 80]
// 	%b.1 =w %_t12
ldr w6, [sp, 80]
mov w1, w6
// 	%a.1 =w %a.2
ldr w6, [sp, 64]
mov w0, w6
// 	%a.3 =w %a.2
ldr w6, [sp, 64]
mov w4, w6
// 	%b.2 =w %b.1
mov w3, w1
// 	jump _l0
b L_main_0
// @_l2:
L_main_2:
// 	%_t13 =w %b.2
mov w0, w3
// 	%_t14 =w %a.3 + %_t13
add w1, w4, w0
// 	%a.4 =w %_t14
mov w0, w1
// 	%_t15 =w %c.0
mov w1, w2
// 	%_t16 =w %a.4 + %_t15
add w2, w0, w1
// 	%a.5 =w %_t16
mov w0, w2
// 	%_t17 =w #0
mov w5, 0
mov w0, w5
// 	return w %_t17
mov w0, w0
b return_main
b return_main
return_main:
add sp, sp, 96
ldp x29, x30, [sp], 16
ret
.section __TEXT,__cstring
sl0:
.asciz "A is %d\n"
