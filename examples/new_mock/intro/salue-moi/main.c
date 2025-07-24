#include <stdio.h>

#define NAME_MAX_SIZE 100
int main(int argc, char *argv[]) {
    char firstname[NAME_MAX_SIZE];
    char lastname[NAME_MAX_SIZE];

    printf("Quel est ton prénom ? ");
    fflush(stdout);
    scanf("%s", firstname);

    printf("Salut %s, qu'est est ton nom de famille ? ", firstname);
    fflush(stdout);
    scanf("%s", lastname);

    printf("Passe une belle journée %s %s !\n", firstname, lastname);
    return 0;
}
