#include "generated.h"

#include <string.h>

int main(void) {
  static const uint8_t text[] = {'h', 'e', 'l', 'l', 'o'};
  poly_allocator allocator = poly_default_allocator();
  registration_Label label = {0};
  registration_Renderable renderable;
  registration_string_result result;
  if (poly_string_clone(allocator, (poly_string_view){text, sizeof(text)},
                        &label.text) != POLY_OK) {
    return 1;
  }
  renderable = registration_Label_as_Renderable(&label);
  result = registration_call_render(allocator, renderable);
  if (!result.ok || result.value.length != sizeof(text) ||
      memcmp(result.value.data, text, sizeof(text)) != 0) {
    registration_string_result_drop(&result);
    registration_Label_drop(&label);
    return 2;
  }
  registration_string_result_drop(&result);
  registration_Label_drop(&label);
  return 0;
}
