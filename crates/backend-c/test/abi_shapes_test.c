#include "generated.h"

#include <stdlib.h>
#include <string.h>

typedef struct fail_context {
  size_t calls;
  size_t fail_at;
} fail_context;

static void *fail_allocate(void *opaque, size_t size) {
  fail_context *context = (fail_context *)opaque;
  ++context->calls;
  if (context->calls == context->fail_at) {
    return NULL;
  }
  return malloc(size);
}

static void *fail_reallocate(void *opaque, void *pointer, size_t size) {
  fail_context *context = (fail_context *)opaque;
  ++context->calls;
  if (context->calls == context->fail_at) {
    return NULL;
  }
  return realloc(pointer, size);
}

static void fail_deallocate(void *opaque, void *pointer) {
  (void)opaque;
  free(pointer);
}

static bool set_text(poly_allocator allocator, const char *text,
                     poly_string *output) {
  return poly_string_clone(
             allocator,
             (poly_string_view){(const uint8_t *)text, strlen(text)}, output) ==
         POLY_OK;
}

int main(void) {
  poly_allocator allocator = poly_default_allocator();
  abi_shapes_Leaf leaves[2] = {0};
  abi_shapes_list__named_1 borrowed = {leaves, 2U, 2U, {0}};
  abi_shapes_list__named_1 owned = {0};
  abi_shapes_option__named_1 maybe = {0};
  abi_shapes_option__named_1 maybe_clone = {0};
  abi_shapes_option__named_1 none_clone = {0};
  abi_shapes_result__option__string__bytes outcome = {0};
  abi_shapes_result__option__string__bytes outcome_clone = {0};
  abi_shapes_result__option__string__bytes failure = {0};
  abi_shapes_result__option__string__bytes failure_clone = {0};
  abi_shapes_Choice choice = {0};
  abi_shapes_Choice choice_clone = {0};
  abi_shapes_Choice empty_choice = {0};
  abi_shapes_Choice empty_choice_clone = {0};
  abi_shapes_Choice label_choice = {0};
  abi_shapes_Choice label_choice_clone = {0};
  abi_shapes_list__list__string empty_nested = {0};
  abi_shapes_list__list__string empty_nested_clone = {0};
  abi_shapes_list__named_1 failed_output = {leaves, 1U, 1U, {0}};
  fail_context context = {0};
  poly_allocator failing = {&context, fail_allocate, fail_reallocate,
                            fail_deallocate};

  if (!set_text(allocator, "one", &leaves[0].text) ||
      !set_text(allocator, "two", &leaves[1].text) ||
      !abi_shapes_list__named_1_clone(allocator, &borrowed, &owned) ||
      owned.length != 2U || owned.data == leaves ||
      owned.data[0].text.data == leaves[0].text.data) {
    return 1;
  }

  maybe.has_value = true;
  if (!abi_shapes_Leaf_clone(allocator, &leaves[0], &maybe.payload.value) ||
      !abi_shapes_option__named_1_clone(allocator, &maybe, &maybe_clone) ||
      !maybe_clone.has_value ||
      maybe_clone.payload.value.text.data == maybe.payload.value.text.data) {
    return 2;
  }
  if (!abi_shapes_option__named_1_clone(
          allocator, &(abi_shapes_option__named_1){0}, &none_clone) ||
      none_clone.has_value) {
    return 8;
  }

  outcome.is_ok = true;
  outcome.payload.ok.has_value = true;
  if (!set_text(allocator, "ok", &outcome.payload.ok.payload.value) ||
      !abi_shapes_result__option__string__bytes_clone(
          allocator, &outcome, &outcome_clone) ||
      !outcome_clone.is_ok || !outcome_clone.payload.ok.has_value) {
    return 3;
  }
  failure.is_ok = false;
  if (!poly_bytes_clone(allocator,
                        (poly_bytes_view){(const uint8_t *)"bad", 3U},
                        &failure.payload.error) ||
      !abi_shapes_result__option__string__bytes_clone(
          allocator, &failure, &failure_clone) ||
      failure_clone.is_ok || failure_clone.payload.error.length != 3U) {
    return 9;
  }

  choice.tag = ABI_SHAPES_CHOICE_LEAVES;
  choice.payload.Leaves.items = borrowed;
  if (!abi_shapes_Choice_clone(allocator, &choice, &choice_clone) ||
      choice_clone.tag != ABI_SHAPES_CHOICE_LEAVES ||
      choice_clone.payload.Leaves.items.length != 2U) {
    return 4;
  }
  empty_choice.tag = ABI_SHAPES_CHOICE_EMPTY;
  label_choice.tag = ABI_SHAPES_CHOICE_LABEL;
  if (!set_text(allocator, "label", &label_choice.payload.Label.value) ||
      !abi_shapes_Choice_clone(allocator, &empty_choice,
                               &empty_choice_clone) ||
      empty_choice_clone.tag != ABI_SHAPES_CHOICE_EMPTY ||
      !abi_shapes_Choice_clone(allocator, &label_choice,
                               &label_choice_clone) ||
      label_choice_clone.tag != ABI_SHAPES_CHOICE_LABEL ||
      label_choice_clone.payload.Label.value.data ==
          label_choice.payload.Label.value.data) {
    return 10;
  }

  if (!abi_shapes_list__list__string_clone(allocator, &empty_nested,
                                            &empty_nested_clone) ||
      empty_nested_clone.data != NULL || empty_nested_clone.length != 0U) {
    return 5;
  }

  context.fail_at = 1U;
  if (abi_shapes_list__named_1_clone(failing, &borrowed, &failed_output) ||
      failed_output.data != NULL || failed_output.length != 0U ||
      failed_output.capacity != 0U) {
    return 6;
  }
  context.calls = 0U;
  context.fail_at = 2U;
  if (abi_shapes_list__named_1_clone(failing, &borrowed,
                                     &(abi_shapes_list__named_1){0})) {
    return 7;
  }

  abi_shapes_list__list__string_drop(&empty_nested_clone);
  abi_shapes_Choice_drop(&label_choice_clone);
  abi_shapes_Choice_drop(&label_choice);
  abi_shapes_Choice_drop(&empty_choice_clone);
  abi_shapes_Choice_drop(&choice_clone);
  abi_shapes_result__option__string__bytes_drop(&failure_clone);
  abi_shapes_result__option__string__bytes_drop(&failure);
  abi_shapes_result__option__string__bytes_drop(&outcome_clone);
  abi_shapes_result__option__string__bytes_drop(&outcome);
  abi_shapes_option__named_1_drop(&none_clone);
  abi_shapes_option__named_1_drop(&maybe_clone);
  abi_shapes_option__named_1_drop(&maybe);
  abi_shapes_list__named_1_drop(&owned);
  poly_string_drop(&leaves[1].text);
  poly_string_drop(&leaves[0].text);

  abi_shapes_list__named_1_drop(&owned);
  abi_shapes_Choice_drop(&choice_clone);
  return 0;
}
