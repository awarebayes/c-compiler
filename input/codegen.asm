.section __TEXT,__text
.extern _puts
.extern _printf
.globl _spill_me
.globl _main
_spill_me:
stp x29, x30, [sp, -16]!
mov x29, sp
sub sp, sp, 64
// @start_function_spill_me:
start_function_spill_me:
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
// 	%b.1 =w %b.0
mov w0, w2
// 	%a.2 =w %a.0
mov w3, w1
// 	%b.3 =w %b.0
mov w4, w2
// @_l0:
L_spill_me_0:
// 	%_t2 =w %b.1
mov w2, w0
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
beq L_spill_me_1
bne L_spill_me_2
// @_l1:
L_spill_me_1:
// 	%_t5 =l s'A is %d, b is %d\n'
adrp x2, sl0@PAGE
add x2, x2, sl0@PAGEOFF
// 	%_t6 =w %a.0
mov w5, w1
str w5, [sp, 16]
// 	%_t7 =w %b.1
mov w5, w0
str w5, [sp, 24]
// 	%_t8 =w call %printf.0 with (param0 l %_t5, vparam1 w %_t6, vparam2 w %_t7)
// param0 l %_t5
// Register is occupied, spilling!
sub sp, sp, 16
str x0, [sp, 0]
mov x0, x2
sub sp, sp, 16
// vparam1 w %_t6
ldr w5, [sp, 48]
str x5, [sp, 0]
// vparam2 w %_t7
ldr w5, [sp, 56]
str x5, [sp, 8]
bl _printf
// Variadic parameters pop
add sp, sp, 16
// Restoring register x0 that was spilled
ldr x0, [sp, 0]
add sp, sp, 16
mov w5, w0
str w5, [sp, 32]
// 	%_t9 =w #1
mov w5, 1
mov w2, w5
// 	%_t10 =w %a.0 - %_t9
sub w7, w1, w2
str w7, [sp, 40]
// 	%a.1 =w %_t10
ldr w6, [sp, 40]
mov w1, w6
// 	%_t11 =w #4
mov w5, 4
mov w2, w5
// 	%_t12 =w %b.1 - %_t11
sub w7, w0, w2
str w7, [sp, 48]
// 	%b.2 =w %_t12
ldr w6, [sp, 48]
mov w2, w6
// 	%b.1 =w %b.2
mov w0, w2
// 	%a.2 =w %a.1
mov w3, w1
// 	%b.3 =w %b.2
mov w4, w2
// 	jump _l0
b L_spill_me_0
// @_l2:
L_spill_me_2:
// 	%_t13 =w %a.2
mov w0, w3
// 	return w %_t13
mov w0, w0
b return_spill_me
return_spill_me:
add sp, sp, 64
ldp x29, x30, [sp], 16
ret
_main:
stp x29, x30, [sp, -16]!
mov x29, sp
sub sp, sp, 0
// @start_function_main:
start_function_main:
// 	%_t0 =w call %spill_me.0 with ()
bl _spill_me
mov w5, w0
mov w0, w5
// 	%a.0 =w %_t0
mov w1, w0
// 	%_t1 =l s'Spilling in process...\n'
adrp x0, sl1@PAGE
add x0, x0, sl1@PAGEOFF
// 	%_t2 =w call %printf.0 with (param0 l %_t1)
// param0 l %_t1
bl _printf
mov w5, w0
mov w2, w5
// 	%_t3 =w call %spill_me.0 with ()
bl _spill_me
mov w5, w0
mov w0, w5
// 	%b.0 =w %_t3
mov w2, w0
// 	%_t4 =w %a.0
mov w0, w1
// 	%_t5 =w %b.0
mov w1, w2
// 	%_t6 =w %_t4 + %_t5
add w2, w0, w1
// 	return w %_t6
mov w0, w2
b return_main
return_main:
add sp, sp, 0
ldp x29, x30, [sp], 16
ret
.section __TEXT,__cstring
sl0:
.asciz "A is %d, b is %d\n"
sl1:
.asciz "Spilling in process...\n"
