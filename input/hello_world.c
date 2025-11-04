extern int puts(char *str);

int other_func() {
   int a = 5;
   while (a > 0)
   {
      puts("a");
      a -= 1;
   }
   return 0;
}

int main() {
   int b = other_func();
   puts("b");
   int c = 0;
   if (b == 0) {
      c = other_func();
   }
   int g = b + c;
   puts("c");
   return g;
}