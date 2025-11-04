extern int puts(char *str);

int main() {
   int a = 0;
   int b = 5;
   if (a == 0) {
      a += 1;
      b -= 1;
   } else {
      a -= 1;
      b += 1;
   }
   int b = a;
   return a;
}

