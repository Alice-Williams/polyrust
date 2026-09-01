#ifndef POLYRUST_RUNTIME_H
#define POLYRUST_RUNTIME_H

/* Dependency-free C17 ownership runtime copied into generated packages. */

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

typedef void *(*poly_allocate_fn)(void *context, size_t size);
typedef void *(*poly_reallocate_fn)(void *context, void *pointer, size_t size);
typedef void (*poly_deallocate_fn)(void *context, void *pointer);

typedef struct poly_allocator {
  void *context;
  poly_allocate_fn allocate;
  poly_reallocate_fn reallocate;
  poly_deallocate_fn deallocate;
} poly_allocator;

typedef struct poly_string_view {
  const uint8_t *data;
  size_t length;
} poly_string_view;

typedef struct poly_bytes_view {
  const uint8_t *data;
  size_t length;
} poly_bytes_view;

typedef struct poly_string {
  uint8_t *data;
  size_t length;
  size_t capacity;
  poly_allocator allocator;
} poly_string;

typedef struct poly_bytes {
  uint8_t *data;
  size_t length;
  size_t capacity;
  poly_allocator allocator;
} poly_bytes;

typedef enum poly_error_code {
  POLY_OK = 0,
  POLY_ALLOCATION_FAILED = 1,
  POLY_CHECKED_OVERFLOW = 2,
  POLY_DIVISION_BY_ZERO = 3,
  POLY_REMAINDER_BY_ZERO = 4,
  POLY_INVALID_SHIFT = 5,
  POLY_NARROWING_OUT_OF_RANGE = 6,
  POLY_INDEX_OUT_OF_BOUNDS = 7,
  POLY_INVALID_UTF8 = 8,
  POLY_INVARIANT_VIOLATION = 9
} poly_error_code;

typedef struct poly_error {
  poly_error_code code;
  const char *message;
} poly_error;

poly_allocator poly_default_allocator(void);
double poly_f64_from_bits(uint64_t bits);
poly_string_view poly_string_borrow(const poly_string *value);
poly_bytes_view poly_bytes_borrow(const poly_bytes *value);
bool poly_utf8_valid(poly_string_view value, size_t *scalar_count);
poly_error_code poly_string_clone(poly_allocator allocator,
                                  poly_string_view source,
                                  poly_string *output);
bool poly_bytes_clone(poly_allocator allocator, poly_bytes_view source,
                      poly_bytes *output);
void poly_string_drop(poly_string *value);
void poly_bytes_drop(poly_bytes *value);
bool poly_string_equal(poly_string_view left, poly_string_view right);
bool poly_bytes_equal(poly_bytes_view left, poly_bytes_view right);
bool poly_string_starts_with(poly_string_view source, poly_string_view prefix);
bool poly_string_ends_with(poly_string_view source, poly_string_view suffix);
bool poly_string_contains(poly_string_view source, poly_string_view needle);
poly_error_code poly_string_strip_prefix(poly_allocator allocator,
                                         poly_string_view source,
                                         poly_string_view prefix,
                                         poly_string *output);
poly_error_code poly_string_concat(poly_allocator allocator,
                                   poly_string_view left,
                                   poly_string_view right,
                                   poly_string *output);
poly_error_code poly_string_replace_all(poly_allocator allocator,
                                        poly_string_view source,
                                        poly_string_view needle,
                                        poly_string_view replacement,
                                        poly_string *output);
poly_error_code poly_string_trim_start(poly_allocator allocator,
                                       poly_string_view source,
                                       poly_string_view characters,
                                       poly_string *output);
poly_error_code poly_string_trim_end(poly_allocator allocator,
                                     poly_string_view source,
                                     poly_string_view characters,
                                     poly_string *output);

#endif
