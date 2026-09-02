#include "generated.h"

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

int main(void) {
  static const struct {
    uint64_t bits;
    bool expected;
  } cases[] = {
      {UINT64_C(0x8000000000000000), true},
      {UINT64_C(0x0000000000000000), false},
      {UINT64_C(0x8000000000000001), false},
      {UINT64_C(0x8010000000000000), false},
      {UINT64_C(0xfff0000000000000), false},
      {UINT64_C(0xfff0000000000001), false},
      {UINT64_C(0xfff8000000000001), false},
      {UINT64_C(0xffffffffffffffff), false},
  };
  poly_allocator allocator = poly_default_allocator();
  size_t index;

  for (index = 0U; index < sizeof(cases) / sizeof(cases[0]); ++index) {
    stdlib_is_negative_zero_bool_result result =
        stdlib_is_negative_zero_is_negative_zero(
            allocator, poly_f64_from_bits(cases[index].bits));
    if (!result.ok || result.value != cases[index].expected) {
      return 1;
    }
  }
  return 0;
}
