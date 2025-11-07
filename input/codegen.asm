.section __TEXT,__text
.extern _printf
.globl _four
.globl _five
.globl _main
_four:
stp x29, x30, [sp, -16]!
mov x29, sp
sub sp, sp, 0
// @start_function_four:
start_function_four:
// 	%_t0 =w #4
mov w5, 4
mov w0, w5
// 	%a.0 =w %_t0
mov w1, w0
// 	%_t1 =w %a.0
mov w0, w1
// 	return w %_t1
mov w0, w0
b return_four
return_four:
add sp, sp, 0
ldp x29, x30, [sp], 16
ret
_five:
stp x29, x30, [sp, -16]!
mov x29, sp
sub sp, sp, 0
// @start_function_five:
start_function_five:
// 	%_t0 =w #5
mov w5, 5
mov w0, w5
// 	return w %_t0
mov w0, w0
b return_five
return_five:
add sp, sp, 0
ldp x29, x30, [sp], 16
ret
_main:
stp x29, x30, [sp, -16]!
mov x29, sp
sub sp, sp, 0
// @start_function_main:
start_function_main:
// 	%_t0 =w call %four.0 with ()
sub sp, sp, 16
// Spilling x0 which is in use
str x0, [sp, 0]
bl _four
mov w5, w0
// Popping x0 which was in use
ldr x0, [sp, 0]
add sp, sp, 16
mov w0, w5
// 	%b.0 =w %_t0
mov w1, w0
// 	%_t1 =w call %five.0 with ()
sub sp, sp, 16
// Spilling x0 which is in use
str x0, [sp, 0]
// Spilling x1 which is in use
str x1, [sp, 8]
bl _five
mov w5, w0
// Popping x0 which was in use
ldr x0, [sp, 0]
// Popping x1 which was in use
ldr x1, [sp, 8]
add sp, sp, 16
mov w0, w5
// 	%c.0 =w %_t1
mov w2, w0
// 	%_t2 =w %b.0
mov w0, w1
// 	%_t3 =w %c.0
mov w1, w2
// 	%_t4 =w %_t2 + %_t3
add w2, w0, w1
// 	%d.0 =w %_t4
mov w0, w2
// 	%_t5 =l s'Four plus five is %d\n'
adrp x1, sl0@PAGE
add x1, x1, sl0@PAGEOFF
// 	%_t6 =w %d.0
mov w2, w0
// 	%_t7 =w call %printf.0 with (param0 l %_t5, vparam1 w %_t6)
sub sp, sp, 32
// Spilling x2 which is in use
str x2, [sp, 0]
// Spilling x1 which is in use
str x1, [sp, 8]
// Spilling x0 which is in use
str x0, [sp, 16]
// param0 l %_t5
mov x0, x1
sub sp, sp, 16
// vparam1 w %_t6
str x2, [sp, 0]
bl _printf
mov w5, w0
// Variadic parameters pop
add sp, sp, 16
// Popping x2 which was in use
ldr x2, [sp, 0]
// Popping x1 which was in use
ldr x1, [sp, 8]
// Popping x0 which was in use
ldr x0, [sp, 16]
add sp, sp, 32
mov w0, w5
// 	%_t8 =w #0
mov w5, 0
mov w0, w5
// 	return w %_t8
mov w0, w0
b return_main
return_main:
add sp, sp, 0
ldp x29, x30, [sp], 16
ret
.section __TEXT,__cstring
sl0:
.asciz "Four plus five is %d\n"
