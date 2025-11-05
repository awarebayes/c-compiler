extern int puts(char *str);
extern int printf( const char * format, ... );

int main() {
   int a = 5;
   while (a > 0)
   {
      a -= 1;
      printf("Hello world %d\n", a);
   }
   return 0;
}
