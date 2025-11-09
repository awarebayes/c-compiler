extern int printf( const char * format, ... );

int main() {
   int a = 4;
   int b = 80;
   int c = 50;
   while (a > 0) {
      printf("A is %d, b is %d, c is %d\n", a, b, c);
      a -= 3;
      b -= 4;
      c -= 1;
   }
   c += 1;
   a += 1;
   printf("Final A is %d, b is %d, c is %d\n", a, b, c);
   return 0;
}