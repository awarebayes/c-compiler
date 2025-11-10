// === Source ===
extern int printf(const char *format, ...);

int fib(int n) {
    if (n < 2) {
        return n;
    }
    return fib(n - 1) + fib(n - 2);
}

int main() {
    int six = 6;
    int result = fib(six);
    printf("Fibonacci of 6 is: %d\n", result);
    return 0;
}
// === End Source ===

// === Output ===
// Fibonacci of 6 is: 8
// === End Output ===