/* Test-only guarded native stack; never copied into generated packages. */
#define _GNU_SOURCE
#include "polyrust_remainder_603.h"
#include <pthread.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mman.h>
#include <unistd.h>

struct test_case { uint64_t left; uint64_t right; uint64_t expected; };
static const struct test_case cases[] = { PROBE_CASES };
struct result { int correct; volatile unsigned char *guard; };

static void *exercise(void *opaque) {
    struct result *result = opaque;
    if (result->guard != NULL) {
        *result->guard = 0;
        result->correct = 1;
        return NULL;
    }
    result->correct = 1;
    for (size_t i = 0; i < sizeof cases / sizeof cases[0]; ++i) {
        double left, right;
        memcpy(&left, &cases[i].left, sizeof left);
        memcpy(&right, &cases[i].right, sizeof right);
        double output = poly_remainder_603_0(left, right);
        uint64_t bits;
        memcpy(&bits, &output, sizeof bits);
        uint64_t expected = cases[i].expected;
        int nan = (expected & UINT64_C(0x7fffffffffffffff)) > UINT64_C(0x7ff0000000000000);
        if (nan ? (bits & UINT64_C(0x7fffffffffffffff)) <= UINT64_C(0x7ff0000000000000) : bits != expected)
            result->correct = 0;
    }
    return NULL;
}

int main(int argc, char **argv) {
    if (argc < 3 || argc > 4) return 2;
    size_t size = (size_t)strtoul(argv[1], NULL, 10);
    size_t allowance = (size_t)strtoul(argv[2], NULL, 10);
    long page_size = sysconf(_SC_PAGESIZE);
    if (page_size <= 0 || size < 65536 || size > 1048576 || allowance > size) return 2;
    size_t page = (size_t)page_size;
    if (size % page != 0) return 2;
    unsigned char *mapping = mmap(NULL, size + 2 * page, PROT_NONE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
    if (mapping == MAP_FAILED) return 2;
    unsigned char *stack = mapping + page;
    if (mprotect(stack, size, PROT_READ | PROT_WRITE) != 0) return 2;
    memset(stack, 0xa5, size);
    struct result result = { 0, NULL };
    if (argc == 4) result.guard = strcmp(argv[3], "lower") == 0 ? stack - 1 : stack + size;
    pthread_attr_t attributes;
    pthread_t thread;
    if (pthread_attr_init(&attributes) != 0
        || pthread_attr_setguardsize(&attributes, 0) != 0
        || pthread_attr_setstack(&attributes, stack, size) != 0
        || pthread_create(&thread, &attributes, exercise, &result) != 0
        || pthread_attr_destroy(&attributes) != 0
        || pthread_join(thread, NULL) != 0) return 2;
    size_t untouched = 0;
    const volatile unsigned char *bytes = stack;
    while (untouched < size && bytes[untouched] == 0xa5) ++untouched;
    size_t used = size - untouched;
    if (printf("%zu\n", used) < 0 || munmap(mapping, size + 2 * page) != 0) return 2;
    if (!result.correct || used == 0) return 4;
    return used <= allowance ? 0 : 3;
}
