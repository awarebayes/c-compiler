# C Compiler 

C Compiler written in rust targeting aarch64 + Mach-O.

- [x] Parsing/Lexing frontend (TreeSitter)
- [x] AST backend
- [x] Semantic analysis, symbol table
- [x] Intermediate Representation
- [x] Single Static Assignment
- [ ] IR Optimization
- [x] Phi Elimination
- [x] Assembly codegen
- [x] Register Allocation (Linear Scan)
- [x] Integration tests

Optimisations status

- [x] Constant folding
- [x] Dead code elimination

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

IR SSA

```asm
extern $puts = "puts": (l) -> w
function w other_func () {
@start_function_other_func:
        %_t0 =w #5
        %a.0 =w %_t0
@_l0:
        %a.1 =w phi [%a.0, @start_function_other_func], [%a.2, @_l1]
        %_t1 =w %a.1
        %_t2 =w #0
        %_t3 =w %_t1 > %_t2
        branch %_t0: _l1 _l2
@_l1:
        %_t4 =l s'a'
        param0 l %_t4
        %_t5 =w call %puts.0
        %_t6 =w #1
        %_t7 =w %a.1 - %_t6
        %a.2 =w %_t7
        jump _l0
@_l2:
        %a.3 =w phi [%a.0, @start_function_other_func], [%a.2, @_l1]
        %_t8 =w %a.3
        return w %_t8
}

function w main () {
@start_function_main:
        %_t0 =w call %other_func.0
        %b.0 =w %_t0
        %_t1 =l s'b'
        param0 l %_t1
        %_t2 =w call %puts.0
        %_t3 =w call %other_func.0
        %c.0 =w %_t3
        %_t4 =w %b.0
        %_t5 =w %c.0
        %_t6 =w %_t4 + %_t5
        %g.0 =w %_t6
        %_t7 =l s'c'
        param0 l %_t7
        %_t8 =w call %puts.0
        %_t9 =w %g.0
        return w %_t9
}
```

Graphviz generation

<img width="1096" height="523" alt="Image" src="https://github.com/user-attachments/assets/165f1b2c-02c8-474c-84e9-98acbb17e192" />

Aarch64 Mach-o asm

```asm
.section __TEXT,__text
.extern _puts
.globl _other_func
.globl _main
_other_func:
stp x29, x30, [sp, -16]!
mov x29, sp
sub sp, sp, 48
mov w0, 5
str w0, [sp, 0]
ldr w0, [sp, 0]
str w0, [sp, 4]
L0:
ldr w0, [sp, 4]
str w0, [sp, 8]
mov w0, 0
str w0, [sp, 12]
ldr w0, [sp, 8]
ldr w1, [sp, 12]
cmp w0, w1
cset w0, gt
str w0, [sp, 16]
ldr w0, [sp, 16]
cmp w0, 1
beq L1
bne L2
L1:
adrp x0, sl0@PAGE
add x0, x0, sl0@PAGEOFF
str x0, [sp, 20]
ldr x0, [sp, 20]
bl _puts
str w0, [sp, 28]
mov w0, 1
str w0, [sp, 32]
ldr w0, [sp, 4]
ldr w1, [sp, 32]
sub w0, w0, w1
str w0, [sp, 36]
ldr w0, [sp, 36]
str w0, [sp, 4]
b L0
L2:
ldr w0, [sp, 4]
str w0, [sp, 40]
return_other_func:
add sp, sp, 48
ldp x29, x30, [sp], 16
ret
_main:
stp x29, x30, [sp, -16]!
mov x29, sp
sub sp, sp, 64
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
bl _other_func
str w0, [sp, 20]
ldr w0, [sp, 20]
str w0, [sp, 24]
ldr w0, [sp, 4]
str w0, [sp, 28]
ldr w0, [sp, 24]
str w0, [sp, 32]
ldr w0, [sp, 28]
ldr w1, [sp, 32]
add w0, w0, w1
str w0, [sp, 36]
ldr w0, [sp, 36]
str w0, [sp, 40]
adrp x0, sl2@PAGE
add x0, x0, sl2@PAGEOFF
str x0, [sp, 44]
ldr x0, [sp, 44]
bl _puts
str w0, [sp, 52]
ldr w0, [sp, 40]
str w0, [sp, 56]
return_main:
add sp, sp, 64
ldp x29, x30, [sp], 16
ret
.section __TEXT,__cstring
sl2:
.asciz "c"
sl0:
.asciz "a"
sl1:
.asciz "b"

```