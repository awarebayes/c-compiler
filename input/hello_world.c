extern int puts(char *str);
extern int printf( const char * format, ... );

int main() {
   int a = 4;
   int b = 80;
   while (b > 0) {
      printf("A is %d, b is %d\n", a, b);
      a -= 1;
      b -= 4;
   } 
   return a;
}
