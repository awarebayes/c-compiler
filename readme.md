# C Compiler 

C Compiler written in rust targeting aarch64 + Mach-O.

- [x] Parsing/Lexing frontend (TreeSitter)
- [x] AST backend
- [x] Semantic analysis, symbol table
- [x] Intermediate Representation
- [x] Single Static Assignment
- [x] IR Optimization
- [x] Phi Elimination
- [x] Assembly codegen
- [x] Register Allocation (Linear Scan)
- [x] Integration tests

Optimisations status

- [x] Constant folding
- [x] Dead code elimination
- [x] Copy elimination on virtual registers
- [ ] Common subexpression elimination

C Language status 

- [x] Init statement
- [x] Function definitions
- [x] Function calls
- [x] Return
- [x] If/Else conditionals
- [x] Loops
- [ ] Struct
- [ ] Arrays / Pointers

source

```c
extern int printf(const char * format, ...);
extern int puts(char *str);

int other_func() {
   int times = 5;
   while (times > 0)
   {
      printf("times is %d\n", times);
      times -= 1;
   }
   return times;
}

int main() {
   int b = other_func();
   puts("b");
   int c = other_func();
   int g = b + c;
   puts("c");
   return g;
}
```

IR SSA ([QBE](https://c9x.me/compile/) inspired)

```c
extern $printf = "printf": (l) -> w
extern $puts = "puts": (l) -> w
function w other_func () {
@start_function_other_func:
        %times.0 =w #5
@_l0:
        %times.1 =w phi [%times.0, @start_function_other_func], [%times.2, @_l1]
        %_t3 =w %times.1 > #0
        branchw %_t3: _l1 _l2
@_l1:
        %_t4 =l s'times is %d\n'
        %_t6 =w call %printf.0 with (param0 l %_t4, vparam1 w %times.1)
        %times.2 =w %times.1 - #1
        jump _l0
@_l2:
        %times.3 =w phi [%times.0, @start_function_other_func], [%times.2, @_l1]
        return w %times.3
}

function w main () {
@start_function_main:
        %_t0 =w call %other_func.0 with ()
        %_t1 =l s'b'
        %_t2 =w call %puts.0 with (param0 l %_t1)
        %_t3 =w call %other_func.0 with ()
        %g.0 =w %b.0 + %c.0
        %_t7 =l s'c'
        %_t8 =w call %puts.0 with (param0 l %_t7)
        return w %g.0
}
```

Phi function elimination

```c
extern $printf = "printf": (l) -> w
extern $puts = "puts": (l) -> w
function w other_func () {
@start_function_other_func:
        %times.0 =w #5
@_l0:
        %_t3 =w %times.0 > #0
        branchw %_t3: _l1 _l2
@_l1:
        %_t4 =l s'times is %d\n'
        %_t6 =w call %printf.0 with (param0 l %_t4, vparam1 w %times.0)
        %times.0 =w %times.0 - #1
        jump _l0
@_l2:
        return w %times.0
}

function w main () {
@start_function_main:
        %_t0 =w call %other_func.0 with ()
        %_t1 =l s'b'
        %_t2 =w call %puts.0 with (param0 l %_t1)
        %_t3 =w call %other_func.0 with ()
        %g.0 =w %b.0 + %c.0
        %_t7 =l s'c'
        %_t8 =w call %puts.0 with (param0 l %_t7)
        return w %g.0
}
```

Graphviz generation

<img width="1096" height="523" alt="Image" src="https://github.com/user-attachments/assets/165f1b2c-02c8-474c-84e9-98acbb17e192" />

Aarch64 Mach-o asm

```asm
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
// Spilling x0 which is in use
str x0, [sp, 0]
// Spilling x2 which is in use
str x2, [sp, 8]
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
// Popping x0 which was in use
ldr x0, [sp, 0]
// Popping x2 which was in use
ldr x2, [sp, 8]
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
// Spilling x1 which is in use
str x1, [sp, 8]
// Spilling x0 which is in use
str x0, [sp, 16]
// param0 l %_t1
mov x0, x1
bl _puts
mov w5, w0
// Popping x2 which was in use
ldr x2, [sp, 0]
// Popping x1 which was in use
ldr x1, [sp, 8]
// Popping x0 which was in use
ldr x0, [sp, 16]
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
// Spilling x0 which is in use
str x0, [sp, 0]
// Spilling x1 which is in use
str x1, [sp, 8]
// Spilling x2 which is in use
str x2, [sp, 16]
// param0 l %_t7
bl _puts
mov w5, w0
// Popping x0 which was in use
ldr x0, [sp, 0]
// Popping x1 which was in use
ldr x1, [sp, 8]
// Popping x2 which was in use
ldr x2, [sp, 16]
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
sl2:
.asciz "c"
sl1:
.asciz "b"
sl0:
.asciz "times is %d\n"
```