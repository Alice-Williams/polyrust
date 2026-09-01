#include "runtime.h"

#include <math.h>
#include <stdlib.h>
#include <string.h>

static void *default_allocate(void *context, size_t size) {
  (void)context;
  return malloc(size);
}

static void *default_reallocate(void *context, void *pointer, size_t size) {
  (void)context;
  return realloc(pointer, size);
}

static void default_deallocate(void *context, void *pointer) {
  (void)context;
  free(pointer);
}

poly_allocator poly_default_allocator(void) {
  poly_allocator result = {NULL, default_allocate, default_reallocate,
                           default_deallocate};
  return result;
}

double poly_f64_from_bits(uint64_t bits) {
  double result;
  memcpy(&result, &bits, sizeof(result));
  return result;
}

uint64_t poly_f64_bits(double value) {
  uint64_t result;
  memcpy(&result, &value, sizeof(result));
  return result;
}

double poly_f64_trunc(double value) { return trunc(value); }

bool poly_f64_is_nan(double value) { return isnan(value); }

double poly_f64_rem_trunc(double left, double right) {
  return fmod(left, right);
}

bool poly_f64_test_equal(double left, double right) {
  return (isnan(left) && isnan(right)) ||
         poly_f64_bits(left) == poly_f64_bits(right);
}

poly_string_view poly_string_borrow(const poly_string *value) {
  poly_string_view result = {value->data, value->length};
  return result;
}

poly_bytes_view poly_bytes_borrow(const poly_bytes *value) {
  poly_bytes_view result = {value->data, value->length};
  return result;
}

static bool valid_scalar(const uint8_t *data, size_t remaining, size_t *width) {
  uint8_t first;
  uint32_t scalar;
  size_t count;
  size_t index;
  if (remaining == 0U) {
    return false;
  }
  first = data[0];
  if (first <= UINT8_C(0x7f)) {
    *width = 1U;
    return true;
  }
  if (first >= UINT8_C(0xc2) && first <= UINT8_C(0xdf)) {
    count = 2U;
    scalar = (uint32_t)(first & UINT8_C(0x1f));
  } else if (first >= UINT8_C(0xe0) && first <= UINT8_C(0xef)) {
    count = 3U;
    scalar = (uint32_t)(first & UINT8_C(0x0f));
  } else if (first >= UINT8_C(0xf0) && first <= UINT8_C(0xf4)) {
    count = 4U;
    scalar = (uint32_t)(first & UINT8_C(0x07));
  } else {
    return false;
  }
  if (remaining < count) {
    return false;
  }
  for (index = 1U; index < count; ++index) {
    if ((data[index] & UINT8_C(0xc0)) != UINT8_C(0x80)) {
      return false;
    }
    scalar = (scalar << 6U) | (uint32_t)(data[index] & UINT8_C(0x3f));
  }
  if ((count == 3U && scalar < UINT32_C(0x800)) ||
      (count == 4U && scalar < UINT32_C(0x10000)) ||
      (scalar >= UINT32_C(0xd800) && scalar <= UINT32_C(0xdfff)) ||
      scalar > UINT32_C(0x10ffff)) {
    return false;
  }
  *width = count;
  return true;
}

bool poly_utf8_valid(poly_string_view value, size_t *scalar_count) {
  size_t offset = 0U;
  size_t count = 0U;
  if (value.length != 0U && value.data == NULL) {
    return false;
  }
  while (offset < value.length) {
    size_t width = 0U;
    if (!valid_scalar(value.data + offset, value.length - offset, &width)) {
      return false;
    }
    offset += width;
    ++count;
  }
  if (scalar_count != NULL) {
    *scalar_count = count;
  }
  return true;
}

static bool bytes_clone(poly_allocator allocator, const uint8_t *source,
                        size_t length, uint8_t **output) {
  uint8_t *data;
  *output = NULL;
  if (length == 0U) {
    return true;
  }
  if (source == NULL || allocator.allocate == NULL ||
      allocator.deallocate == NULL) {
    return false;
  }
  data = (uint8_t *)allocator.allocate(allocator.context, length);
  if (data == NULL) {
    return false;
  }
  memcpy(data, source, length);
  *output = data;
  return true;
}

poly_error_code poly_string_clone(poly_allocator allocator,
                                  poly_string_view source,
                                  poly_string *output) {
  poly_string result = {0};
  if (output == NULL) {
    return POLY_INVARIANT_VIOLATION;
  }
  *output = result;
  if (!poly_utf8_valid(source, NULL)) {
    return POLY_INVALID_UTF8;
  }
  if (!bytes_clone(allocator, source.data, source.length, &result.data)) {
    return POLY_ALLOCATION_FAILED;
  }
  result.length = source.length;
  result.capacity = source.length;
  result.allocator = allocator;
  *output = result;
  return POLY_OK;
}

bool poly_bytes_clone(poly_allocator allocator, poly_bytes_view source,
                      poly_bytes *output) {
  poly_bytes result = {0};
  if (output == NULL) {
    return false;
  }
  *output = result;
  if (!bytes_clone(allocator, source.data, source.length, &result.data)) {
    return false;
  }
  result.length = source.length;
  result.capacity = source.length;
  result.allocator = allocator;
  *output = result;
  return true;
}

void poly_string_drop(poly_string *value) {
  if (value == NULL) {
    return;
  }
  if (value->data != NULL && value->allocator.deallocate != NULL) {
    value->allocator.deallocate(value->allocator.context, value->data);
  }
  *value = (poly_string){0};
}

void poly_bytes_drop(poly_bytes *value) {
  if (value == NULL) {
    return;
  }
  if (value->data != NULL && value->allocator.deallocate != NULL) {
    value->allocator.deallocate(value->allocator.context, value->data);
  }
  *value = (poly_bytes){0};
}

static bool view_equal(const uint8_t *left, size_t left_length,
                       const uint8_t *right, size_t right_length) {
  return left_length == right_length &&
         (left_length == 0U || memcmp(left, right, left_length) == 0);
}

bool poly_string_equal(poly_string_view left, poly_string_view right) {
  return view_equal(left.data, left.length, right.data, right.length);
}

bool poly_bytes_equal(poly_bytes_view left, poly_bytes_view right) {
  return view_equal(left.data, left.length, right.data, right.length);
}

bool poly_string_starts_with(poly_string_view source, poly_string_view prefix) {
  return prefix.length <= source.length &&
         view_equal(source.data, prefix.length, prefix.data, prefix.length);
}

bool poly_string_ends_with(poly_string_view source, poly_string_view suffix) {
  if (suffix.length == 0U) {
    return true;
  }
  return suffix.length <= source.length &&
         view_equal(source.data + source.length - suffix.length, suffix.length,
                    suffix.data, suffix.length);
}

bool poly_string_contains(poly_string_view source, poly_string_view needle) {
  size_t offset;
  if (needle.length == 0U) {
    return true;
  }
  if (needle.length > source.length) {
    return false;
  }
  for (offset = 0U; offset <= source.length - needle.length; ++offset) {
    if (view_equal(source.data + offset, needle.length, needle.data,
                   needle.length)) {
      return true;
    }
  }
  return false;
}

poly_error_code poly_string_strip_prefix(poly_allocator allocator,
                                         poly_string_view source,
                                         poly_string_view prefix,
                                         poly_string *output) {
  if (!poly_utf8_valid(source, NULL) || !poly_utf8_valid(prefix, NULL)) {
    return POLY_INVALID_UTF8;
  }
  if (poly_string_starts_with(source, prefix)) {
    source.data += prefix.length;
    source.length -= prefix.length;
  }
  return poly_string_clone(allocator, source, output);
}

poly_error_code poly_string_concat(poly_allocator allocator,
                                   poly_string_view left,
                                   poly_string_view right,
                                   poly_string *output) {
  poly_string result = {0};
  size_t length;
  if (output == NULL) {
    return POLY_INVARIANT_VIOLATION;
  }
  *output = result;
  if (!poly_utf8_valid(left, NULL) || !poly_utf8_valid(right, NULL)) {
    return POLY_INVALID_UTF8;
  }
  if (left.length > SIZE_MAX - right.length) {
    return POLY_ALLOCATION_FAILED;
  }
  length = left.length + right.length;
  if (length != 0U) {
    if (allocator.allocate == NULL || allocator.deallocate == NULL) {
      return POLY_ALLOCATION_FAILED;
    }
    result.data = (uint8_t *)allocator.allocate(allocator.context, length);
    if (result.data == NULL) {
      return POLY_ALLOCATION_FAILED;
    }
  }
  if (left.length != 0U) {
    memcpy(result.data, left.data, left.length);
  }
  if (right.length != 0U) {
    memcpy(result.data + left.length, right.data, right.length);
  }
  result.length = length;
  result.capacity = length;
  result.allocator = allocator;
  *output = result;
  return POLY_OK;
}

static size_t count_occurrences(poly_string_view source,
                                poly_string_view needle) {
  size_t count = 0U;
  size_t offset = 0U;
  if (needle.length == 0U) {
    size_t scalars = 0U;
    (void)poly_utf8_valid(source, &scalars);
    return scalars + 1U;
  }
  while (offset + needle.length <= source.length) {
    if (view_equal(source.data + offset, needle.length, needle.data,
                   needle.length)) {
      ++count;
      offset += needle.length;
    } else {
      ++offset;
    }
  }
  return count;
}

poly_error_code poly_string_replace_all(poly_allocator allocator,
                                        poly_string_view source,
                                        poly_string_view needle,
                                        poly_string_view replacement,
                                        poly_string *output) {
  size_t count;
  size_t length;
  size_t input_offset = 0U;
  size_t output_offset = 0U;
  poly_string result = {0};
  if (output == NULL) {
    return POLY_INVARIANT_VIOLATION;
  }
  *output = result;
  if (!poly_utf8_valid(source, NULL) || !poly_utf8_valid(needle, NULL) ||
      !poly_utf8_valid(replacement, NULL)) {
    return POLY_INVALID_UTF8;
  }
  if (needle.length == 0U) {
    /* Empty-needle insertion is intentionally handled scalar-by-scalar. */
    size_t scalar_offset = 0U;
    size_t scalar_count = 0U;
    (void)poly_utf8_valid(source, &scalar_count);
    if (replacement.length != 0U &&
        scalar_count + 1U > (SIZE_MAX - source.length) / replacement.length) {
      return POLY_ALLOCATION_FAILED;
    }
    length = source.length + (scalar_count + 1U) * replacement.length;
    if (length != 0U) {
      if (allocator.allocate == NULL || allocator.deallocate == NULL) {
        return POLY_ALLOCATION_FAILED;
      }
      result.data = (uint8_t *)allocator.allocate(allocator.context, length);
      if (result.data == NULL) {
        return POLY_ALLOCATION_FAILED;
      }
    }
    if (replacement.length != 0U) {
      memcpy(result.data, replacement.data, replacement.length);
      output_offset += replacement.length;
    }
    while (scalar_offset < source.length) {
      size_t width = 0U;
      (void)valid_scalar(source.data + scalar_offset,
                         source.length - scalar_offset, &width);
      memcpy(result.data + output_offset, source.data + scalar_offset, width);
      output_offset += width;
      scalar_offset += width;
      if (replacement.length != 0U) {
        memcpy(result.data + output_offset, replacement.data,
               replacement.length);
        output_offset += replacement.length;
      }
    }
  } else {
    count = count_occurrences(source, needle);
    if (replacement.length >= needle.length) {
      size_t growth = replacement.length - needle.length;
      if (growth != 0U && count > (SIZE_MAX - source.length) / growth) {
        return POLY_ALLOCATION_FAILED;
      }
      length = source.length + count * growth;
    } else {
      length = source.length - count * (needle.length - replacement.length);
    }
    if (length != 0U) {
      if (allocator.allocate == NULL || allocator.deallocate == NULL) {
        return POLY_ALLOCATION_FAILED;
      }
      result.data = (uint8_t *)allocator.allocate(allocator.context, length);
      if (result.data == NULL) {
        return POLY_ALLOCATION_FAILED;
      }
    }
    while (input_offset < source.length) {
      if (input_offset + needle.length <= source.length &&
          view_equal(source.data + input_offset, needle.length, needle.data,
                     needle.length)) {
        if (replacement.length != 0U) {
          memcpy(result.data + output_offset, replacement.data,
                 replacement.length);
        }
        output_offset += replacement.length;
        input_offset += needle.length;
      } else {
        result.data[output_offset++] = source.data[input_offset++];
      }
    }
  }
  result.length = length;
  result.capacity = length;
  result.allocator = allocator;
  *output = result;
  return POLY_OK;
}

static size_t first_mapping_at(poly_string_view source, size_t offset,
                               const poly_string_view *needles,
                               size_t mapping_count) {
  size_t index;
  for (index = 0U; index < mapping_count; ++index) {
    poly_string_view needle = needles[index];
    const uint8_t *location =
        offset == source.length ? NULL : source.data + offset;
    if (needle.length <= source.length - offset &&
        view_equal(location, needle.length, needle.data, needle.length)) {
      return index;
    }
  }
  return mapping_count;
}

static bool add_length(size_t *length, size_t added) {
  if (added > SIZE_MAX - *length) {
    return false;
  }
  *length += added;
  return true;
}

poly_error_code poly_string_replace_many(poly_allocator allocator,
                                         poly_string_view source,
                                         const poly_string_view *needles,
                                         const poly_string_view *replacements,
                                         size_t mapping_count,
                                         poly_string *output) {
  size_t index;
  size_t input_offset = 0U;
  size_t output_offset = 0U;
  size_t length = 0U;
  poly_string result = {0};
  if (output == NULL || mapping_count == 0U || needles == NULL ||
      replacements == NULL) {
    return POLY_INVARIANT_VIOLATION;
  }
  *output = result;
  if (!poly_utf8_valid(source, NULL)) {
    return POLY_INVALID_UTF8;
  }
  for (index = 0U; index < mapping_count; ++index) {
    if (!poly_utf8_valid(needles[index], NULL) ||
        !poly_utf8_valid(replacements[index], NULL)) {
      return POLY_INVALID_UTF8;
    }
  }

  while (true) {
    size_t mapping = first_mapping_at(source, input_offset, needles,
                                      mapping_count);
    if (mapping != mapping_count) {
      size_t needle_length = needles[mapping].length;
      if (!add_length(&length, replacements[mapping].length)) {
        return POLY_ALLOCATION_FAILED;
      }
      if (needle_length != 0U) {
        input_offset += needle_length;
        continue;
      }
      if (input_offset == source.length) {
        break;
      }
    } else if (input_offset == source.length) {
      break;
    }
    {
      size_t width = 0U;
      (void)valid_scalar(source.data + input_offset,
                         source.length - input_offset, &width);
      if (!add_length(&length, width)) {
        return POLY_ALLOCATION_FAILED;
      }
      input_offset += width;
    }
  }

  if (length != 0U) {
    if (allocator.allocate == NULL || allocator.deallocate == NULL) {
      return POLY_ALLOCATION_FAILED;
    }
    result.data = (uint8_t *)allocator.allocate(allocator.context, length);
    if (result.data == NULL) {
      return POLY_ALLOCATION_FAILED;
    }
  }
  input_offset = 0U;
  while (true) {
    size_t mapping = first_mapping_at(source, input_offset, needles,
                                      mapping_count);
    if (mapping != mapping_count) {
      poly_string_view needle = needles[mapping];
      poly_string_view replacement = replacements[mapping];
      if (replacement.length != 0U) {
        memcpy(result.data + output_offset, replacement.data,
               replacement.length);
        output_offset += replacement.length;
      }
      if (needle.length != 0U) {
        input_offset += needle.length;
        continue;
      }
      if (input_offset == source.length) {
        break;
      }
    } else if (input_offset == source.length) {
      break;
    }
    {
      size_t width = 0U;
      (void)valid_scalar(source.data + input_offset,
                         source.length - input_offset, &width);
      memcpy(result.data + output_offset, source.data + input_offset, width);
      output_offset += width;
      input_offset += width;
    }
  }
  result.length = length;
  result.capacity = length;
  result.allocator = allocator;
  *output = result;
  return POLY_OK;
}

poly_error_code poly_string_truncate_utf8_bytes(poly_allocator allocator,
                                                poly_string_view source,
                                                double budget,
                                                poly_string *output) {
  size_t offset = 0U;
  size_t prefix_length = source.length;
  if (output == NULL) {
    return POLY_INVARIANT_VIOLATION;
  }
  *output = (poly_string){0};
  if (!poly_utf8_valid(source, NULL)) {
    return POLY_INVALID_UTF8;
  }
  while (offset < source.length) {
    size_t width = 0U;
    size_t end;
    double consumed;
    (void)valid_scalar(source.data + offset, source.length - offset, &width);
    end = offset + width;
    consumed = (double)end;
    if (consumed == budget) {
      prefix_length = end;
      break;
    }
    if (consumed > budget) {
      prefix_length = offset;
      break;
    }
    offset = end;
  }
  return poly_string_clone(
      allocator,
      (poly_string_view){source.data, prefix_length},
      output);
}

static bool scalar_in(poly_string_view characters, const uint8_t *scalar,
                      size_t width) {
  size_t offset = 0U;
  while (offset < characters.length) {
    size_t item_width = 0U;
    (void)valid_scalar(characters.data + offset, characters.length - offset,
                       &item_width);
    if (view_equal(characters.data + offset, item_width, scalar, width)) {
      return true;
    }
    offset += item_width;
  }
  return false;
}

poly_error_code poly_string_trim_start(poly_allocator allocator,
                                       poly_string_view source,
                                       poly_string_view characters,
                                       poly_string *output) {
  size_t offset = 0U;
  if (!poly_utf8_valid(source, NULL) || !poly_utf8_valid(characters, NULL)) {
    return POLY_INVALID_UTF8;
  }
  while (offset < source.length) {
    size_t width = 0U;
    (void)valid_scalar(source.data + offset, source.length - offset, &width);
    if (!scalar_in(characters, source.data + offset, width)) {
      break;
    }
    offset += width;
  }
  source.data += offset;
  source.length -= offset;
  return poly_string_clone(allocator, source, output);
}

poly_error_code poly_string_trim_end(poly_allocator allocator,
                                     poly_string_view source,
                                     poly_string_view characters,
                                     poly_string *output) {
  size_t offset = 0U;
  size_t keep = 0U;
  if (!poly_utf8_valid(source, NULL) || !poly_utf8_valid(characters, NULL)) {
    return POLY_INVALID_UTF8;
  }
  while (offset < source.length) {
    size_t width = 0U;
    (void)valid_scalar(source.data + offset, source.length - offset, &width);
    if (!scalar_in(characters, source.data + offset, width)) {
      keep = offset + width;
    }
    offset += width;
  }
  source.length = keep;
  return poly_string_clone(allocator, source, output);
}
