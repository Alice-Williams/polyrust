/* Independent library oracle, not generated/certified package source. */
#include <math.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define POLY_TYPE(expression, type) \
    _Static_assert(_Generic((expression), type: 1, default: 0), "poly_callable_type_" #type)

POLY_TYPE(&malloc, void *(*)(size_t));
POLY_TYPE(&free, void (*)(void *));
POLY_TYPE(&memcpy, void *(*)(void *, const void *, size_t));
POLY_TYPE(&memcmp, int (*)(const void *, const void *, size_t));
POLY_TYPE(&fmod, double (*)(double, double));
POLY_TYPE(&trunc, double (*)(double));
POLY_TYPE(&fwrite, size_t (*)(const void *, size_t, size_t, FILE *));
POLY_TYPE(&ferror, int (*)(FILE *));
POLY_TYPE(isnan(0.0), int);
POLY_TYPE(signbit(0.0), int);
POLY_TYPE(stdin, FILE *);
POLY_TYPE(stdout, FILE *);
POLY_TYPE(stderr, FILE *);

#ifdef POLY_EXPECT_BOOL
_Static_assert(_Generic(isnan(0.0), _Bool: 1, default: 0), "poly_predicate_result");
#endif

int main(void) {
    const unsigned char source[3] = {1, 2, 3};
    void *storage = malloc(sizeof source);
    if (storage == NULL) return 1;
    if (memcpy(storage, source, sizeof source) != storage) {
        free(storage);
        return 2;
    }
    if (memcmp(storage, source, sizeof source) != 0) {
        free(storage);
        return 3;
    }
    free(storage);
    free(NULL);
    /* Nonconstant arguments retain actual libm calls even under optimization. */
    volatile double input = 5.5;
    volatile double divisor = 2.0;
    if (fmod(input, divisor) != 1.5 || trunc(input) != 5.0) return 4;
    if (isnan(NAN) == 0 || isnan(input) != 0) return 5;
    if (signbit(-0.0) == 0 || signbit(0.0) != 0) return 6;
    if (fwrite(source, 1, 0, stdout) != 0 || ferror(stdout) != 0) return 7;
    return 0;
}
