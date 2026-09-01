#include <stdint.h>

int main(void) {
  const int32_t value = INT32_C(42);
  return value == INT32_C(42) ? 0 : 1;
}
