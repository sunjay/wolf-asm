#include <stdio.h>
#include <stdlib.h>

typedef long long intl;

int main(int argc, char *argv[]) {
    if (argc < 2) {
        printf("Usage: ./oscillating_sum n\n");
        return 1;
    }

    intl n = atoi(argv[1]);

    intl total = 0;
    intl multiplier = 1;
    for (intl i = 1; i <= n; i++) {
        total += i * multiplier;

        multiplier *= -1;
    }

    printf("%lld\n", total);
}
