.section __TEXT,__text
.extern _printf
.globl _fib
.globl _main
_fib:
stp x29, x30, [sp, -16]!
mov x29, sp
sub sp, sp, 0
// @start_function_fib:
start_function_fib:
// 	%_t2 =w %n.0 < #2
cmp w0, 2
cset w1, lt
// 	branchw %_t2: _l0 _l1
cmp w1, 1
beq L_fib_0
bne L_fib_1
// @_l0:
L_fib_0:
// 	return w %n.0
mov w0, w0
b return_fib
// @_l1:
L_fib_1:
// 	%_t7 =w %n.0 - #1
sub w1, w0, 1
// 	%_t8 =w call %fib.0 with (param0 w %_t7)
sub sp, sp, 32
// Spilling x2 which is in use
str x2, [sp, 0]
// Spilling w0 which is in use
str x0, [sp, 8]
// Spilling x1 which is in use
str x1, [sp, 16]
// param0 w %_t7
mov w0, w1
bl _fib
mov w5, w0
// Popping x2 which was in use
ldr x2, [sp, 0]
// Popping w0 which was in use
ldr x0, [sp, 8]
// Popping x1 which was in use
ldr x1, [sp, 16]
add sp, sp, 32
mov w2, w5
// 	%_t11 =w %n.0 - #2
sub w1, w0, 2
// 	%_t12 =w call %fib.0 with (param0 w %_t11)
sub sp, sp, 32
// Spilling x2 which is in use
str x2, [sp, 0]
// Spilling w0 which is in use
str x0, [sp, 8]
// Spilling x1 which is in use
str x1, [sp, 16]
// Spilling x3 which is in use
str x3, [sp, 24]
// param0 w %_t11
mov w0, w1
bl _fib
mov w5, w0
// Popping x2 which was in use
ldr x2, [sp, 0]
// Popping w0 which was in use
ldr x0, [sp, 8]
// Popping x1 which was in use
ldr x1, [sp, 16]
// Popping x3 which was in use
ldr x3, [sp, 24]
add sp, sp, 32
mov w3, w5
// 	%_t13 =w %_t8 + %_t12
add w1, w2, w3
// 	return w %_t13
mov w0, w1
b return_fib
return_fib:
add sp, sp, 0
ldp x29, x30, [sp], 16
ret
_main:
stp x29, x30, [sp, -16]!
mov x29, sp
sub sp, sp, 0
// @start_function_main:
start_function_main:
// 	%result.0 =w call %fib.0 with (param0 w #6)
sub sp, sp, 16
// Spilling x1 which is in use
str x1, [sp, 0]
// Spilling x0 which is in use
str x0, [sp, 8]
// param0 w #6
mov w0, 6
bl _fib
mov w5, w0
// Popping x1 which was in use
ldr x1, [sp, 0]
// Popping x0 which was in use
ldr x0, [sp, 8]
add sp, sp, 16
mov w0, w5
// 	%_t3 =l s'Fibonacci of 6 is: %d\n'
adrp x1, sl0@PAGE
add x1, x1, sl0@PAGEOFF
// 	%_t5 =w call %printf.0 with (param0 l %_t3, vparam1 w %result.0)
sub sp, sp, 32
// Spilling x2 which is in use
str x2, [sp, 0]
// Spilling x0 which is in use
str x0, [sp, 8]
// Spilling x1 which is in use
str x1, [sp, 16]
sub sp, sp, 16
// vparam1 w %result.0
str x0, [sp, 0]
// param0 l %_t3
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
// 	return w #0
mov w0, 0
b return_main
return_main:
add sp, sp, 0
ldp x29, x30, [sp], 16
ret
.section __TEXT,__cstring
sl0:
.asciz "Fibonacci of 6 is: %d\n"
