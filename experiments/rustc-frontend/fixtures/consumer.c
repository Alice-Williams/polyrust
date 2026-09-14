#include <inttypes.h>
#include <stdint.h>
#include <stdio.h>

int32_t poly_score(int32_t value);

int main(void) {
    int32_t value;
    while (scanf("%" SCNd32, &value) == 1) {
        printf("%" PRId32 "\n", poly_score(value));
    }
    return 0;
}
