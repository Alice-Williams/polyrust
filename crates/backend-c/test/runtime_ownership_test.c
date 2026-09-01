#ifndef POLYRUST_RUNTIME_TEMPLATE_INCLUDED
#include "runtime.h"
#endif

#include <stdlib.h>
#include <string.h>

typedef struct failing_context {
  size_t calls;
  size_t fail_at;
} failing_context;

static void *failing_allocate(void *opaque, size_t size) {
  failing_context *context = (failing_context *)opaque;
  context->calls += 1U;
  if (context->calls == context->fail_at) {
    return NULL;
  }
  return malloc(size);
}

static void *failing_reallocate(void *opaque, void *pointer, size_t size) {
  failing_context *context = (failing_context *)opaque;
  context->calls += 1U;
  if (context->calls == context->fail_at) {
    return NULL;
  }
  return realloc(pointer, size);
}

static void failing_deallocate(void *opaque, void *pointer) {
  (void)opaque;
  free(pointer);
}

int main(void) {
  static const uint8_t embedded_zero[] = {'a', 0, 'b'};
  static const uint8_t astral[] = {UINT8_C(0xf0), UINT8_C(0x9f),
                                   UINT8_C(0xa6), UINT8_C(0x80)};
  static const uint8_t invalid[] = {UINT8_C(0xff)};
  static const uint8_t source[] = {'a', '-', 'b', '-', 'c'};
  static const uint8_t needle[] = {'-'};
  static const uint8_t replacement[] = {'/', '/'};
  static const uint8_t truncate_source[] = {
      'a', UINT8_C(0xe2), UINT8_C(0x98), UINT8_C(0x83)};
  static const uint8_t many_source[] = {'&', 'a', 'm', 'p', ';', 'l', 't', ';',
                                        '&', 'l', 't', ';'};
  static const uint8_t amp[] = {'&', 'a', 'm', 'p', ';'};
  static const uint8_t lt[] = {'&', 'l', 't', ';'};
  static const uint8_t amp_replacement[] = {'&'};
  static const uint8_t lt_replacement[] = {'<'};
  static const poly_string_view needles[] = {
      {amp, sizeof(amp)}, {lt, sizeof(lt)}};
  static const poly_string_view replacements[] = {
      {amp_replacement, sizeof(amp_replacement)},
      {lt_replacement, sizeof(lt_replacement)}};
  poly_allocator allocator = poly_default_allocator();
  poly_string first = {0};
  poly_string clone = {0};
  poly_string replaced = {0};
  poly_string replaced_many = {0};
  poly_string failed_many = {0};
  poly_string truncated = {0};
  poly_string failed_truncate = {0};
  size_t scalars = 0U;
  failing_context context = {0};
  poly_allocator failing = {&context, failing_allocate, failing_reallocate,
                            failing_deallocate};

  if (!poly_utf8_valid((poly_string_view){astral, sizeof(astral)}, &scalars) ||
      scalars != 1U ||
      poly_utf8_valid((poly_string_view){invalid, sizeof(invalid)}, NULL)) {
    return 1;
  }
  if (poly_string_clone(
          allocator,
          (poly_string_view){embedded_zero, sizeof(embedded_zero)}, &first) !=
          POLY_OK ||
      poly_string_clone(allocator, poly_string_borrow(&first), &clone) !=
          POLY_OK ||
      first.data == clone.data ||
      !poly_string_equal(poly_string_borrow(&first),
                         poly_string_borrow(&clone))) {
    return 2;
  }
  if (poly_string_replace_all(
          allocator, (poly_string_view){source, sizeof(source)},
          (poly_string_view){needle, sizeof(needle)},
          (poly_string_view){replacement, sizeof(replacement)}, &replaced) !=
          POLY_OK ||
      replaced.length != 7U || memcmp(replaced.data, "a//b//c", 7U) != 0) {
    return 3;
  }
  if (poly_string_replace_many(
          allocator, (poly_string_view){many_source, sizeof(many_source)},
          needles, replacements, 2U, &replaced_many) != POLY_OK ||
      replaced_many.length != 5U ||
      memcmp(replaced_many.data, "&lt;<", 5U) != 0) {
    return 6;
  }
  context.calls = 0U;
  context.fail_at = 1U;
  if (poly_string_replace_many(
          failing, (poly_string_view){many_source, sizeof(many_source)},
          needles, replacements, 2U, &failed_many) !=
          POLY_ALLOCATION_FAILED ||
      failed_many.data != NULL || failed_many.length != 0U ||
      failed_many.capacity != 0U) {
    return 7;
  }
  if (poly_string_truncate_utf8_bytes(
          allocator,
          (poly_string_view){truncate_source, sizeof(truncate_source)}, 2.0,
          &truncated) != POLY_OK ||
      truncated.length != 1U || memcmp(truncated.data, "a", 1U) != 0) {
    return 8;
  }
  context.calls = 0U;
  context.fail_at = 1U;
  if (poly_string_truncate_utf8_bytes(
          failing,
          (poly_string_view){truncate_source, sizeof(truncate_source)}, 4.0,
          &failed_truncate) != POLY_ALLOCATION_FAILED ||
      failed_truncate.data != NULL || failed_truncate.length != 0U ||
      failed_truncate.capacity != 0U) {
    return 9;
  }
  context.calls = 0U;
  context.fail_at = 1U;
  if (poly_string_clone(failing, (poly_string_view){source, sizeof(source)},
                        &(poly_string){0}) != POLY_ALLOCATION_FAILED) {
    return 4;
  }
  if (poly_string_clone(allocator,
                        (poly_string_view){invalid, sizeof(invalid)},
                        &(poly_string){0}) != POLY_INVALID_UTF8 ||
      poly_string_concat(allocator,
                         (poly_string_view){invalid, sizeof(invalid)},
                         (poly_string_view){source, sizeof(source)},
                         &(poly_string){0}) != POLY_INVALID_UTF8 ||
      poly_string_truncate_utf8_bytes(
          allocator, (poly_string_view){invalid, sizeof(invalid)}, 1.0,
          &(poly_string){0}) != POLY_INVALID_UTF8 ||
      poly_utf8_valid((poly_string_view){NULL, 1U}, NULL)) {
    return 5;
  }
  poly_string_drop(&replaced);
  poly_string_drop(&replaced_many);
  poly_string_drop(&failed_many);
  poly_string_drop(&truncated);
  poly_string_drop(&failed_truncate);
  poly_string_drop(&clone);
  poly_string_drop(&first);
  poly_string_drop(&first);
  return 0;
}
