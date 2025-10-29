extern int puts(char *str);

int other_func() {
   int a = 5;
   while (a > 0)
   {
      puts("Boba");
      a -= 1;
   }
   return a;
}

int main() {
   int b = other_func();
   puts("biba");
   int c = other_func();
   int g = b + c;
   puts("ZeliBoba");
   return g;
}

