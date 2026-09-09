/* Independent constant/layout oracle, never generated or certified source. */
#include <float.h>
#include <limits.h>
#include <math.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>

_Static_assert(CHAR_BIT == 8 && CHAR_MIN == -128, "poly_char");
_Static_assert(INT_MAX == 2147483647 && INT_MIN == (-2147483647 - 1), "poly_int");
_Static_assert(UINT64_MAX == UINT64_C(18446744073709551615), "poly_u64");
_Static_assert(SIZE_MAX == UINT64_MAX && EOF == -1, "poly_size_eof");
_Static_assert(FLT_RADIX == 2 && DBL_MANT_DIG == 53 && DBL_MIN_EXP == -1021
    && DBL_MAX_EXP == 1024 && FLT_EVAL_METHOD == 0, "poly_binary64");
_Static_assert((uint8_t)255 + (uint8_t)255 == 510, "poly_narrow_promotion");
_Static_assert(UINT32_MAX + UINT32_C(1) == 0, "poly_u32_wrap");
_Static_assert(UINT64_MAX * UINT64_MAX == 1, "poly_u64_product");
_Static_assert(UINT32_MAX + INT64_C(-1) == INT64_C(4294967294), "poly_mixed_rank");
_Static_assert(UINT64_MAX == (uint64_t)INT64_C(-1), "poly_unsigned_conversion");
_Static_assert(-7 / 3 == -2 && -7 % 3 == -1, "poly_truncated_division");
_Static_assert((-3 >> 1) == -2 && (INT64_MIN >> 63) == -1, "poly_signed_right_shift");
_Static_assert((UINT64_C(1) << 63) == UINT64_C(9223372036854775808), "poly_unsigned_left_shift");
_Static_assert((1 << 30) == 1073741824, "poly_signed_left_shift");
_Static_assert((uint8_t)-1 == 255 && (_Bool)2 == 1, "poly_narrow_bool");
_Static_assert((0 && 1 / 0) == 0 && (1 || 1 / 0) == 1, "poly_short_circuit");
_Static_assert((1 ? 7 : 1 / 0) == 7, "poly_conditional");

struct poly_record { uint8_t first; uint64_t middle; uint16_t last; };
union poly_union { uint8_t first; uint64_t middle; uint16_t last; };
enum poly_enum { poly_negative = -1, poly_positive = INT_MAX };
typedef struct poly_record poly_alias;
_Static_assert(offsetof(poly_alias, first) == 0 && offsetof(poly_alias, middle) == 8
    && offsetof(poly_alias, last) == 16, "poly_offsets");
_Static_assert(sizeof(poly_alias) == 24 && _Alignof(poly_alias) == 8, "poly_struct");
_Static_assert(sizeof(union poly_union) == 8 && _Alignof(union poly_union) == 8, "poly_union");
_Static_assert(sizeof(poly_alias[2][3]) == 144 && _Alignof(poly_alias[2][3]) == 8, "poly_array");
_Static_assert(sizeof(enum poly_enum) == 4 && _Alignof(enum poly_enum) == 4, "poly_enum");
_Static_assert(sizeof(max_align_t) == 32 && _Alignof(max_align_t) == 16, "poly_max_align");

#ifdef POLY_UNSAFE_ARITHMETIC
_Static_assert(INT_MAX + 1 > 0, "poly_signed_overflow");
#endif

int main(void) {
    volatile uint64_t maximum = UINT64_MAX;
    volatile uint64_t tie = UINT64_C(9007199254740993);
    if ((double)maximum != 0x1p64 || (double)tie != 0x1p53) return 1;
    volatile double signed_inside = 0x1.fffffffffffffp62;
    volatile double unsigned_inside = 0x1.fffffffffffffp63;
    volatile double minimum = -0x1p63;
    if ((int64_t)signed_inside != INT64_MAX - INT64_C(1023)) return 2;
    if ((uint64_t)unsigned_inside != UINT64_MAX - UINT64_C(2047)) return 3;
    if ((int64_t)minimum != INT64_MIN) return 4;
    volatile double fraction = -0.75;
    if ((uint64_t)fraction != 0) return 5;
    volatile double zero = 0.0;
    volatile double one = 1.0;
    double infinity = one / zero;
    double nan = zero / zero;
    if (!isinf(infinity) || !isnan(nan) || (_Bool)nan != 1) return 6;
    volatile double negative_zero = -0.0;
    if (!signbit(negative_zero * 2.0) || (_Bool)negative_zero != 0) return 7;
    return 0;
}
