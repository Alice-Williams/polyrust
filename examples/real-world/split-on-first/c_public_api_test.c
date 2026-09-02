#include "generated.h"

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

typedef struct tracking_context {
  size_t calls;
  size_t fail_at;
  size_t live;
  bool invalid_free;
} tracking_context;

static void *tracking_allocate(void *opaque, size_t size) {
  tracking_context *context = (tracking_context *)opaque;
  void *result;
  context->calls += 1U;
  if (context->calls == context->fail_at) {
    return NULL;
  }
  result = malloc(size);
  if (result != NULL) {
    context->live += 1U;
  }
  return result;
}

static void *tracking_reallocate(void *opaque, void *pointer, size_t size) {
  tracking_context *context = (tracking_context *)opaque;
  bool was_null = pointer == NULL;
  void *result;
  context->calls += 1U;
  if (context->calls == context->fail_at) {
    return NULL;
  }
  result = realloc(pointer, size);
  if (result != NULL && was_null) {
    context->live += 1U;
  }
  return result;
}

static void tracking_deallocate(void *opaque, void *pointer) {
  tracking_context *context = (tracking_context *)opaque;
  if (pointer == NULL) {
    return;
  }
  if (context->live == 0U) {
    context->invalid_free = true;
  } else {
    context->live -= 1U;
  }
  free(pointer);
}

static bool equals(const poly_string *actual, const uint8_t *expected,
                   size_t length) {
  return actual->length == length &&
         (length == 0U || memcmp(actual->data, expected, length) == 0);
}

static int require_success(poly_allocator allocator, poly_string_view input,
                           poly_string_view separator,
                           const uint8_t *before, size_t before_length,
                           const uint8_t *after, size_t after_length) {
  split_on_first_list__string_call_result result =
      split_on_first_split_on_first(allocator, input, separator);
  if (!result.ok || result.value.length != 2U || result.value.data == NULL ||
      !equals(&result.value.data[0], before, before_length) ||
      !equals(&result.value.data[1], after, after_length)) {
    split_on_first_list__string_call_result_drop(&result);
    return 1;
  }
  split_on_first_list__string_call_result_drop(&result);
  split_on_first_list__string_call_result_drop(&result);
  return 0;
}

int main(void) {
  static const uint8_t ascii[] = {'l', 'e', 'f', 't', ':', ':',
                                  'r', 'i', 'g', 'h', 't'};
  static const uint8_t separator[] = {':', ':'};
  static const uint8_t before[] = {'l', 'e', 'f', 't'};
  static const uint8_t after[] = {'r', 'i', 'g', 'h', 't'};
  static const uint8_t unicode[] = {
      UINT8_C(0xf0), UINT8_C(0x9f), UINT8_C(0xa6), UINT8_C(0x80),
      ':', ':', UINT8_C(0xe5), UINT8_C(0xb0), UINT8_C(0xbe)};
  static const uint8_t unicode_before[] = {
      UINT8_C(0xf0), UINT8_C(0x9f), UINT8_C(0xa6), UINT8_C(0x80)};
  static const uint8_t unicode_after[] = {
      UINT8_C(0xe5), UINT8_C(0xb0), UINT8_C(0xbe)};
  poly_allocator standard = poly_default_allocator();
  tracking_context context = {0};
  poly_allocator tracking = {&context, tracking_allocate, tracking_reallocate,
                             tracking_deallocate};
  split_on_first_list__string_call_result result;
  size_t total_calls;
  size_t fail_at;

  if (require_success(standard, (poly_string_view){ascii, sizeof(ascii)},
                      (poly_string_view){separator, sizeof(separator)}, before,
                      sizeof(before), after, sizeof(after)) != 0 ||
      require_success(standard, (poly_string_view){unicode, sizeof(unicode)},
                      (poly_string_view){separator, sizeof(separator)},
                      unicode_before, sizeof(unicode_before), unicode_after,
                      sizeof(unicode_after)) != 0) {
    return 1;
  }

  result = split_on_first_split_on_first(
      standard, (poly_string_view){NULL, 0U},
      (poly_string_view){separator, sizeof(separator)});
  if (!result.ok || result.value.length != 0U || result.value.data != NULL) {
    split_on_first_list__string_call_result_drop(&result);
    return 2;
  }
  split_on_first_list__string_call_result_drop(&result);

  context.fail_at = SIZE_MAX;
  result = split_on_first_split_on_first(
      tracking, (poly_string_view){ascii, sizeof(ascii)},
      (poly_string_view){separator, sizeof(separator)});
  total_calls = context.calls;
  if (!result.ok || total_calls == 0U) {
    split_on_first_list__string_call_result_drop(&result);
    return 3;
  }
  split_on_first_list__string_call_result_drop(&result);
  if (context.live != 0U || context.invalid_free) {
    return 4;
  }

  for (fail_at = 1U; fail_at <= total_calls; ++fail_at) {
    context.calls = 0U;
    context.fail_at = fail_at;
    context.live = 0U;
    context.invalid_free = false;
    result = split_on_first_split_on_first(
        tracking, (poly_string_view){ascii, sizeof(ascii)},
        (poly_string_view){separator, sizeof(separator)});
    if (result.ok || result.error.code != POLY_ALLOCATION_FAILED) {
      split_on_first_list__string_call_result_drop(&result);
      return 5;
    }
    split_on_first_list__string_call_result_drop(&result);
    if (context.live != 0U || context.invalid_free) {
      return 6;
    }
  }

  return 0;
}
