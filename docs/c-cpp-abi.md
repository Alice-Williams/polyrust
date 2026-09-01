# C and C++ generated API contract

## Shared semantic boundary

Both backends consume only `CheckedProgram`. They preserve the portable IR's
fixed-width integers, Unicode-scalar strings, immutable values, explicit
`Option`/`Result` shapes, bounded iteration, and structured runtime errors.
Neither backend uses exceptions to represent a portable failure.

The generated implementations may use a target runtime internally, but the
public APIs below are stable source contracts. Unsupported ownership or ABI
combinations produce backend diagnostics before files are emitted.

## C++20 mapping

| Portable type | C++20 representation |
| --- | --- |
| `Unit` | `std::monostate` |
| `Bool` | `bool` |
| `I32` / `I64` | `std::int32_t` / `std::int64_t` |
| `F64` | `double` with raw-bit conformance |
| `Char` | `char32_t` |
| `String` | UTF-8 `std::string` |
| `Bytes` | `std::vector<std::uint8_t>` |
| `List<T>` | value-semantic `std::vector<T>` |
| `Option<T>` | `std::optional<T>` |
| `Result<T, E>` | generated tagged `value_result<T, E>` |
| record | aggregate class with value equality |
| enum | `std::variant` of generated variant records |
| contract | abstract interface with a virtual destructor |
| callable failure | `poly_result<T>`; never a portable exception |

Public values use RAII. Generated methods accept immutable references where a
copy is not required and return owned values. Checked arithmetic is implemented
without signed-overflow undefined behavior. Builds use `-std=c++20`,
`-Wall -Wextra -Wpedantic -Werror`, and sanitizer gates.

## C17 mapping

M22B is implementing this normative mapping in slices. The checked scalar,
UTF-8 string, allocator, contract-vtable, and concrete aggregate ABI slices are
active. Aggregate layouts and recursive ownership operations are generated and
tested; expression construction, matching, container intrinsics, arithmetic,
and bounded iteration remain diagnostic-only. The backend never exposes
placeholder `void *` containers.

### Names and monomorphization

Every public symbol is prefixed with the sanitized module name. Each reachable
`List`, `Option`, `Result`, record, and enum shape receives one
deterministic concrete C type. C output never exposes an untyped generic
container or relies on `void *` element casts.

### Strings, bytes, and lists

Inputs use borrowed views:

`polyrust_string_view { const uint8_t *data; size_t length; }`

The bytes must be well-formed UTF-8 for `String` and may contain zero bytes.
Owned string/byte/list results contain package-allocated storage plus length and
capacity. Every owned public type has generated `_clone` and `_drop`
functions. Empty values use a null pointer with zero length/capacity.

### Records, enums, options, and results

- Records own each owned field and are destroyed recursively.
- Enums contain a generated tag plus a union of variant payload structs.
- Options contain a `has_value` flag plus a payload union member.
- Portable value results contain an `is_ok` flag plus an ok/error union.
- Callable results contain an `is_ok` flag plus either the return value or
  `polyrust_error`.

The active union member is always determined by its adjacent tag. Constructors
fully initialize the active member; drop functions inspect the tag before
recursive destruction.

### Allocation and failure

Generated packages use an explicit allocator table:

`polyrust_allocator { context, allocate, reallocate, deallocate }`.

The default uses `malloc`/`realloc`/`free`. Tests inject a
deterministically failing allocator. Allocation failure returns
`POLY_ALLOCATION_FAILED` and leaves every output in a valid droppable
state. Allocation ownership never crosses allocators: a value is dropped with
the allocator that created it.

Malformed borrowed string input returns `POLY_INVALID_UTF8` before any
allocation is attempted. A non-empty view with a null data pointer is malformed;
an empty null view is the canonical empty value.

### Contracts

A contract value contains a borrowed context pointer and a generated const
vtable of method callbacks. The context remains borrowed for the duration of a
call; generated code does not retain it. Callbacks return the same generated
result structs as ordinary functions.

### Aliasing rules

Borrowed inputs remain valid only for the duration of the call. Owned outputs do
not alias input storage. Clone operations create independent storage. Drop is
idempotent for a zero-initialized or already-dropped value and resets the value
to that state.

## Required ABI tests

- empty, ASCII, embedded-zero, astral Unicode, and invalid UTF-8 strings;
- empty and nested owned lists;
- every option/result/enum tag;
- record clone and recursive destruction;
- contract callback dispatch;
- allocation failure at every allocation point;
- checked overflow, division by zero, and out-of-range indexing; and
- AddressSanitizer plus UndefinedBehaviorSanitizer leak/double-free/UB runs.
