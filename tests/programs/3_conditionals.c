// === Source ===
extern int printf( const char * format, ... );

int main() {
    int a = 42;
    if (a < 30) {
        printf("less than 30\n");
    } else {
        printf("more than 30\n");
    }
    return 0;
}
// === End Source ===

// === Output ===
// more than 30
// === End Output ===