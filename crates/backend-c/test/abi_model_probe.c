/* Independent native ABI oracle, not generated/certified package source. */
#include <float.h>
#include <limits.h>
#include <stddef.h>
#include <stdint.h>
#include <string.h>

#define POLY_LAYOUT(type, size, alignment) \
    _Static_assert(sizeof(type) == (size), "poly_size_" #type); \
    _Static_assert(_Alignof(type) == (alignment), "poly_align_" #type)
#define POLY_TYPE(expression, type) \
    _Static_assert(_Generic((expression), type: 1, default: 0), "poly_type_" #type)

_Static_assert(CHAR_BIT == 8 && CHAR_MIN == -128 && CHAR_MAX == 127, "poly_char_signed");
_Static_assert(INT_MIN == (-2147483647 - 1) && INT_MAX == 2147483647, "poly_int_range");
_Static_assert(DBL_MANT_DIG == 53 && DBL_MAX_EXP == 1024 && DBL_MIN_EXP == -1021, "poly_binary64");
_Static_assert(FLT_RADIX == 2 && FLT_EVAL_METHOD == 0, "poly_float_evaluation");

POLY_LAYOUT(_Bool, 1, 1);
POLY_LAYOUT(char, 1, 1);
POLY_LAYOUT(int8_t, 1, 1);
POLY_LAYOUT(uint8_t, 1, 1);
POLY_LAYOUT(int16_t, 2, 2);
POLY_LAYOUT(uint16_t, 2, 2);
POLY_LAYOUT(int, 4, 4);
POLY_LAYOUT(int32_t, 4, 4);
POLY_LAYOUT(uint32_t, 4, 4);
POLY_LAYOUT(int64_t, 8, 8);
POLY_LAYOUT(uint64_t, 8, 8);
POLY_LAYOUT(size_t, 8, 8);
POLY_LAYOUT(double, 8, 8);
POLY_LAYOUT(void *, 8, 8);
POLY_LAYOUT(int *, 8, 8);
typedef void (*poly_callback)(void);
POLY_LAYOUT(poly_callback, 8, 8);
/* C17 6.7.6.3: incomplete by-value types are legal in a mere prototype.
 * This intentionally has no definition or invocation using incomplete storage. */
struct poly_incomplete;
typedef struct poly_incomplete (*poly_incomplete_callback)(struct poly_incomplete);
POLY_LAYOUT(poly_incomplete_callback, 8, 8);
POLY_LAYOUT(max_align_t, 32, 16);

POLY_TYPE((int8_t)0, signed char);
POLY_TYPE((uint8_t)0, unsigned char);
POLY_TYPE((int16_t)0, short);
POLY_TYPE((uint16_t)0, unsigned short);
POLY_TYPE((int32_t)0, int);
POLY_TYPE((uint32_t)0, unsigned int);
POLY_TYPE((int64_t)0, long);
POLY_TYPE((uint64_t)0, unsigned long);
POLY_TYPE((size_t)0, unsigned long);
POLY_TYPE(+(char)0, int);
POLY_TYPE(+(_Bool)0, int);
POLY_TYPE(+(uint16_t)0, int);
POLY_TYPE((uint32_t)0 + (int32_t)0, unsigned int);
POLY_TYPE((uint32_t)0 + (int64_t)0, long);
POLY_TYPE((uint64_t)0 + (int64_t)0, unsigned long);

/* Independent compiler oracle for every pair in the 13-scalar AST inventory. */
#define POLY_ARITHMETIC_ROW(left, small, u32, i64, u64, floating) \
    POLY_TYPE((left)0 + (_Bool)0, small); \
    POLY_TYPE((left)0 + (char)0, small); \
    POLY_TYPE((left)0 + (int)0, small); \
    POLY_TYPE((left)0 + (int8_t)0, small); \
    POLY_TYPE((left)0 + (uint8_t)0, small); \
    POLY_TYPE((left)0 + (int16_t)0, small); \
    POLY_TYPE((left)0 + (uint16_t)0, small); \
    POLY_TYPE((left)0 + (int32_t)0, small); \
    POLY_TYPE((left)0 + (uint32_t)0, u32); \
    POLY_TYPE((left)0 + (int64_t)0, i64); \
    POLY_TYPE((left)0 + (uint64_t)0, u64); \
    POLY_TYPE((left)0 + (size_t)0, u64); \
    POLY_TYPE((left)0 + (double)0, floating)

POLY_ARITHMETIC_ROW(_Bool, int, unsigned int, long, unsigned long, double);
POLY_ARITHMETIC_ROW(char, int, unsigned int, long, unsigned long, double);
POLY_ARITHMETIC_ROW(int, int, unsigned int, long, unsigned long, double);
POLY_ARITHMETIC_ROW(int8_t, int, unsigned int, long, unsigned long, double);
POLY_ARITHMETIC_ROW(uint8_t, int, unsigned int, long, unsigned long, double);
POLY_ARITHMETIC_ROW(int16_t, int, unsigned int, long, unsigned long, double);
POLY_ARITHMETIC_ROW(uint16_t, int, unsigned int, long, unsigned long, double);
POLY_ARITHMETIC_ROW(int32_t, int, unsigned int, long, unsigned long, double);
POLY_ARITHMETIC_ROW(uint32_t, unsigned int, unsigned int, long, unsigned long, double);
POLY_ARITHMETIC_ROW(int64_t, long, long, long, unsigned long, double);
POLY_ARITHMETIC_ROW(uint64_t, unsigned long, unsigned long, unsigned long, unsigned long, double);
POLY_ARITHMETIC_ROW(size_t, unsigned long, unsigned long, unsigned long, unsigned long, double);
POLY_ARITHMETIC_ROW(double, double, double, double, double, double);

enum poly_positive { poly_zero = 0, poly_one = 1 };
enum poly_signed { poly_negative = -1, poly_positive_max = INT_MAX };
POLY_LAYOUT(enum poly_positive, 4, 4);
POLY_LAYOUT(enum poly_signed, 4, 4);
POLY_TYPE((enum poly_positive)0, unsigned int);
POLY_TYPE((enum poly_signed)0, int);
POLY_TYPE(+((enum poly_positive)0), unsigned int);
POLY_TYPE(+((enum poly_signed)0), int);
POLY_TYPE(poly_one, int);

struct poly_padded { uint8_t first; uint64_t second; uint16_t third; };
union poly_union { uint8_t byte; uint64_t wide; };
_Static_assert(offsetof(struct poly_padded, second) == 8, "poly_member_padding");
_Static_assert(offsetof(struct poly_padded, third) == 16, "poly_member_offset");
POLY_LAYOUT(struct poly_padded, 24, 8);
POLY_LAYOUT(union poly_union, 8, 8);

int main(void) {
    const uint64_t endian = UINT64_C(1);
    unsigned char bytes[8] = {0};
    const double one = 1.0;
    uint64_t bits = 0;
    const int64_t minimum = INT64_MIN;
    memcpy(bytes, &endian, sizeof bytes);
    if (bytes[0] != 1 || bytes[7] != 0) { return 1; }
    memcpy(&bits, &one, sizeof bits);
    if (bits != UINT64_C(0x3ff0000000000000)) { return 2; }
    memcpy(&bits, &minimum, sizeof bits);
    if (bits != UINT64_C(0x8000000000000000)) { return 3; }
    return 0;
}
