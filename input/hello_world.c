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