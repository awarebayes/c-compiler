// === Source ===
extern int printf( const char * format, ... );

int four() {
   int a = 4;
   return a;
}

int five() {
   return 5;
}

int main() {
   int b = four();
   int c = five();
   int d = b + c;

   printf("Four plus five is %d\n", d);
   return 0;
}
// === End Source ===

// === Output ===
// Four plus five is 9
// === End Output ===