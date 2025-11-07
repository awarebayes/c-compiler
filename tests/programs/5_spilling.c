// === Source ===
extern int printf( const char * format, ... );

int main() {
   int a = 4;
   int b = 80;
   int c = 5;
   while (b > 0) {
      printf("A is %d, b is %d, c is %d\n", a, b, c);
      a -= 1;
      b -= 4;
      c -= 1;
   }
   c += 1;
   a += 1;
   c += 4;
   printf("Final A is %d, b is %d, c is %d\n", a, b, c);
   return 0;
}
// === End Source ===

// === Output ===
// A is 4, b is 80, c is 5
// A is 3, b is 76, c is 4
// A is 2, b is 72, c is 3
// A is 1, b is 68, c is 2
// A is 0, b is 64, c is 1
// A is -1, b is 60, c is 0
// A is -2, b is 56, c is -1
// A is -3, b is 52, c is -2
// A is -4, b is 48, c is -3
// A is -5, b is 44, c is -4
// A is -6, b is 40, c is -5
// A is -7, b is 36, c is -6
// A is -8, b is 32, c is -7
// A is -9, b is 28, c is -8
// A is -10, b is 24, c is -9
// A is -11, b is 20, c is -10
// A is -12, b is 16, c is -11
// A is -13, b is 12, c is -12
// A is -14, b is 8, c is -13
// A is -15, b is 4, c is -14
// Final A is -15, b is 0, c is -10
// === End Output ===