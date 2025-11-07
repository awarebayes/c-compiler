// === Source ===
extern int printf( const char * format, ... );

int main() {
   int a = 5;
   while (a > 0) {
      printf("A is %d\n", a);
      a -= 1;
   } 
   return a;
}
// === End Source ===

// === Output ===
// A is 5
// A is 4
// A is 3
// A is 2
// A is 1
// === End Output ===