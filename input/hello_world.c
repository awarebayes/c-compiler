extern int puts(char *str);
extern int printf( const char * format, ... );

int spill_me() {
   int a = 4;
   int b = 80;
   while (b > 0) {
      printf("A is %d, b is %d\n", a, b);
      a -= 1;
      b -= 4;
   } 
   return a;
}

int main() {
   int a = spill_me();
   printf("Spilling in process...\n");
   int b = spill_me();
   return a + b;
}