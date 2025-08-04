#include <stdio.h>
#include <stdlib.h>
int main() {
    FILE *file = fopen("test.txt", "r");
    if (!file) {
        printf("file test.txt not present in working directory...\n");
        exit(2);
    }
    char buffer[100];
    int length = fread(buffer, 1, 100, file);
    buffer[length] = '\0';
    printf("test.txt: %s", buffer);
}
