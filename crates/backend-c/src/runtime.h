/* POLYRUST-BEGIN runtime.core.types */
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
/* POLYRUST-END runtime.core.types */
/* POLYRUST-BEGIN runtime.feature.f64 */
double poly_f64_from_bits(uint64_t bits);
uint64_t poly_f64_bits(double value);
double poly_f64_trunc(double value);
bool poly_f64_is_nan(double value);
double poly_f64_rem_trunc(double left, double right);
bool poly_f64_test_equal(double left, double right);
/* POLYRUST-END runtime.feature.f64 */
/* POLYRUST-BEGIN runtime.feature.string-utf16-length */
poly_error_code poly_string_utf16_length(poly_string_view value,
                                           int64_t *output);
/* POLYRUST-END runtime.feature.string-utf16-length */
/* POLYRUST-BEGIN runtime.feature.string-index-of-literal */
poly_error_code poly_string_index_of_literal(poly_string_view source,
                                             poly_string_view needle,
                                             int64_t *index,
                                             bool *found);
/* POLYRUST-END runtime.feature.string-index-of-literal */
/* POLYRUST-BEGIN runtime.feature.string-slice-scalars */
poly_error_code poly_string_slice_scalars(poly_allocator allocator,
                                          poly_string_view source,
                                          int64_t start,
                                          int64_t end,
                                          poly_string *output);
/* POLYRUST-END runtime.feature.string-slice-scalars */
/* POLYRUST-BEGIN runtime.feature.string-predicates */
bool poly_string_starts_with(poly_string_view source, poly_string_view prefix);
bool poly_string_ends_with(poly_string_view source, poly_string_view suffix);
bool poly_string_contains(poly_string_view source, poly_string_view needle);
/* POLYRUST-END runtime.feature.string-predicates */
/* POLYRUST-BEGIN runtime.feature.string-strip-prefix */
poly_error_code poly_string_strip_prefix(poly_allocator allocator,
                                         poly_string_view source,
                                         poly_string_view prefix,
                                         poly_string *output);
/* POLYRUST-END runtime.feature.string-strip-prefix */
/* POLYRUST-BEGIN runtime.feature.string-concat */
poly_error_code poly_string_concat(poly_allocator allocator,
                                   poly_string_view left,
                                   poly_string_view right,
                                   poly_string *output);
/* POLYRUST-END runtime.feature.string-concat */
/* POLYRUST-BEGIN runtime.feature.string-replace-all */
poly_error_code poly_string_replace_all(poly_allocator allocator,
                                        poly_string_view source,
                                        poly_string_view needle,
                                        poly_string_view replacement,
                                        poly_string *output);
/* POLYRUST-END runtime.feature.string-replace-all */
/* POLYRUST-BEGIN runtime.feature.bytes-replace-all */
poly_error_code poly_bytes_replace_all(poly_allocator allocator,
                                       poly_bytes_view source,
                                       poly_bytes_view needle,
                                       poly_bytes_view replacement,
                                       poly_bytes *output);
/* POLYRUST-END runtime.feature.bytes-replace-all */
/* POLYRUST-BEGIN runtime.feature.string-replace-many */
poly_error_code poly_string_replace_many(poly_allocator allocator,
                                         poly_string_view source,
                                         const poly_string_view *needles,
                                         const poly_string_view *replacements,
                                         size_t mapping_count,
                                         poly_string *output);
/* POLYRUST-END runtime.feature.string-replace-many */
/* POLYRUST-BEGIN runtime.feature.string-truncate-utf8 */
poly_error_code poly_string_truncate_utf8_bytes(poly_allocator allocator,
                                                poly_string_view source,
                                                double budget,
                                                poly_string *output);
/* POLYRUST-END runtime.feature.string-truncate-utf8 */
/* POLYRUST-BEGIN runtime.feature.string-trim */
poly_error_code poly_string_trim_start(poly_allocator allocator,
                                       poly_string_view source,
                                       poly_string_view characters,
                                       poly_string *output);
poly_error_code poly_string_trim_end(poly_allocator allocator,
                                     poly_string_view source,
                                     poly_string_view characters,
                                     poly_string *output);
/* POLYRUST-END runtime.feature.string-trim */
