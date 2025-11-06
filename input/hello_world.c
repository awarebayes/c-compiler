extern int puts(char *str);
extern int printf( const char * format, ... );

int main() {
   int b = 4;
   int c = 6;
   int a = 4;
   while (a > 0) {
      printf("A is %d\n", a);
      a -= 1;
      b -= 4;
   } 
   a += b;
   a += c;
   return 0;
}

