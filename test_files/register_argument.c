int somefunc(int a, int b) {
    return a + b;
}

int someOtherFunc(char* a, int b, char c) {
    register int x;
    x = a;
    *a = c;
    return a;
}

int main(int argc, char** argv) {
    return argc - 1;
}
