.section __TEXT,__text
.extern _printf
.extern _puts
.globl _other_func
.globl _main
_other_func:
stp x29, x30, [sp, -16]!
mov x29, sp
sub sp, sp, 0
// @start_function_other_func:
start_function_other_func:
// 	%times.0 =w #5
mov w5, 5
mov w0, w5
// @_l0:
L_other_func_0:
// 	%_t3 =w %times.0 > #0
cmp w0, 0
cset w1, gt
// 	branchw %_t3: _l1 _l2
cmp w1, 1
beq L_other_func_1
bne L_other_func_2
// @_l1:
L_other_func_1:
// 	%_t4 =l s'times is %d\n'
adrp x1, sl0@PAGE
add x1, x1, sl0@PAGEOFF
// 	%_t6 =w call %printf.0 with (param0 l %_t4, vparam1 w %times.0)
sub sp, sp, 32
// Spilling x2 which is in use
str x2, [sp, 0]
// Spilling x0 which is in use
str x0, [sp, 8]
// Spilling x1 which is in use
str x1, [sp, 16]
sub sp, sp, 16
// vparam1 w %times.0
str x0, [sp, 0]
// param0 l %_t4
mov x0, x1
bl _printf
mov w5, w0
// Variadic parameters pop
add sp, sp, 16
// Popping x2 which was in use
ldr x2, [sp, 0]
// Popping x0 which was in use
ldr x0, [sp, 8]
// Popping x1 which was in use
ldr x1, [sp, 16]
add sp, sp, 32
mov w2, w5
// 	%times.0 =w %times.0 - #1
sub w0, w0, 1
// 	jump _l0
b L_other_func_0
// @_l2:
L_other_func_2:
// 	return w %times.0
mov w0, w0
b return_other_func
return_other_func:
add sp, sp, 0
ldp x29, x30, [sp], 16
ret
_main:
stp x29, x30, [sp, -16]!
mov x29, sp
sub sp, sp, 0
// @start_function_main:
start_function_main:
// 	%b.0 =w call %other_func.0 with ()
sub sp, sp, 16
// Spilling x0 which is in use
str x0, [sp, 0]
bl _other_func
mov w5, w0
// Popping x0 which was in use
ldr x0, [sp, 0]
add sp, sp, 16
mov w0, w5
// 	%_t1 =l s'b'
adrp x1, sl1@PAGE
add x1, x1, sl1@PAGEOFF
// 	%_t2 =w call %puts.0 with (param0 l %_t1)
sub sp, sp, 32
// Spilling x2 which is in use
str x2, [sp, 0]
// Spilling x0 which is in use
str x0, [sp, 8]
// Spilling x1 which is in use
str x1, [sp, 16]
// param0 l %_t1
mov x0, x1
bl _puts
mov w5, w0
// Popping x2 which was in use
ldr x2, [sp, 0]
// Popping x0 which was in use
ldr x0, [sp, 8]
// Popping x1 which was in use
ldr x1, [sp, 16]
add sp, sp, 32
mov w2, w5
// 	%c.0 =w call %other_func.0 with ()
sub sp, sp, 16
// Spilling x0 which is in use
str x0, [sp, 0]
// Spilling x1 which is in use
str x1, [sp, 8]
bl _other_func
mov w5, w0
// Popping x0 which was in use
ldr x0, [sp, 0]
// Popping x1 which was in use
ldr x1, [sp, 8]
add sp, sp, 16
mov w1, w5
// 	%g.0 =w %b.0 + %c.0
add w2, w0, w1
// 	%_t7 =l s'c'
adrp x0, sl2@PAGE
add x0, x0, sl2@PAGEOFF
// 	%_t8 =w call %puts.0 with (param0 l %_t7)
sub sp, sp, 32
// Spilling x2 which is in use
str x2, [sp, 0]
// Spilling x0 which is in use
str x0, [sp, 8]
// Spilling x1 which is in use
str x1, [sp, 16]
// param0 l %_t7
bl _puts
mov w5, w0
// Popping x2 which was in use
ldr x2, [sp, 0]
// Popping x0 which was in use
ldr x0, [sp, 8]
// Popping x1 which was in use
ldr x1, [sp, 16]
add sp, sp, 32
mov w1, w5
// 	return w %g.0
mov w0, w2
b return_main
return_main:
add sp, sp, 0
ldp x29, x30, [sp], 16
ret
.section __TEXT,__cstring
sl0:
.asciz "times is %d\n"
sl1:
.asciz "b"
sl2:
.asciz "c"
