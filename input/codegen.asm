.section __TEXT,__text
.extern _puts
.extern _printf
.globl _main
_main:
stp x29, x30, [sp, -16]!
mov x29, sp
sub sp, sp, 0
// @start_function_main:
start_function_main:
// 	%_t0 =w #5
mov w5, 5
mov w0, w5
// 	%a.0 =w %_t0
mov w1, w0
// 	%_t1 =l s'A is %d\n'
adrp x0, sl0@PAGE
add x0, x0, sl0@PAGEOFF
// 	%_t2 =w %a.0
mov w2, w1
// 	%_t3 =w call %printf.0 with (param0 l %_t1, vparam1 w %_t2)
sub sp, sp, 16
str w2, [sp, 0]
bl _printf
add sp, sp, 16
mov w5, w0
mov w1, w5
// 	%_t4 =w #0
mov w5, 0
mov w0, w5
// 	return w %_t4
mov w0, w0
b return_main
b return_main
return_main:
add sp, sp, 0
ldp x29, x30, [sp], 16
ret
.section __TEXT,__cstring
sl0:
.asciz "A is %d\n"
