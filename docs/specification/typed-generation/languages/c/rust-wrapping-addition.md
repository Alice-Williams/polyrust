# Rust wrapping addition in C17

- Status: target foundation complete; compiler-source integration is separate
- Contract: [shared](../../rust-wrapping-addition.md)

## Typed lowering

Use exact I32/U32 or I64/U64 representations on the pinned ABI. Materialize the
original signed operands left-to-right. Convert both to the matching unsigned
width, add them in that width, and materialize the unsigned sum u.

Construct a typed conditional, not a source-text template:

    u <= U(MAX) ? S(u) : -1 - S(~u)

Here S is the original signed width and U its unsigned counterpart. In the first
branch the cast is in range. In the second, u > MAX implies ~u <= MAX; the cast
is in range and -1 minus that nonnegative signed value is representable,
including MIN. Normalize C Int promotion back to exact I32; I64 remains I64.

This formula is our lowering design. Its language premises are unsigned modulo
arithmetic and integer conversion rules in
[C draft N1570, 6.2.5 and 6.3.1.3](https://www.open-std.org/jtc1/sc22/wg14/www/docs/n1570.pdf).
Never substitute an out-of-range unsigned-to-signed cast, signed overflowing
addition, floating widening, compiler wrapping flags or memcpy helper.

## Certification boundary

Admit only the internal scalar categories needed by this construction:
same-width unsigned addition/complement/comparison, matching-width signed-to-
unsigned conversion, range-proved unsigned-to-signed conversion, safe signed
subtraction and exact-width result normalization. Public source signatures stay
I32/I64; this does not enable Rust unsigned types or casts.

Walk all children in source/dependency/resource traversals. Existing numeric
flow must prove both signed casts and subtraction on every reachable path.
If its unsigned complement/refinement machinery cannot establish the bounds,
extend and test those general numeric facts rather than bypass certification or
recognize a special AST spelling. Wrapping loss must never become nonwrapping
allocation/extent evidence. Preserve loop, ownership and arithmetic safety gates.

Missing, reversed, wrong-threshold and wrong-value guards, unsafe signed-add
substitution and unguarded signed casts must fail certification where unsafe.
Safe wrong results and missing exact-width normalization need explicit shape/
semantic controls even when C ABI compatibility would accept them.

## Native and package proof

Separate producer/client compilation, standalone headers, GCC14 and Zig at O0/O2,
GCC undefined-behavior sanitizer, independent modular oracle and strict warnings.
Cover signed extrema, zero, powers/carries and full-width deterministic pairs.
Verify exact typed AST shape, original callable authority, stack/resource bounds,
dependency-derived headers and no extra runtime/math-library files or imports.
No compiler frontend admission change belongs in the target-only checkpoint.

## Implementation structure

The shared profile's iterative package walk delegates scalar expressions to
profile_values.rs and the narrow new integer categories to
profile_integer_operations.rs. Both schedule ordinary child nodes back through
the same bounded walk. Existing numeric flow and range transfer rules remain
the proof authority; no arithmetic pattern receives a trusted bypass.

U32/U64 are registered standard-library type symbols. Their spellings and
stdint dependency are resolved through the same typed catalogue as signed
types. Source-derived size/alignment assertions live in the public header and
therefore guard implementation and clients through inclusion. A signed
forwarding consumer need not repeat those unsigned layout queries.
Public callable signatures retain the earlier signed/Boolean/binary64 inventory;
source-constant admission is unchanged. Closed-call effect summaries visit unsigned literals
and complement operands without treating that summary as a numeric certificate.
Internal scalar types continue to compose under the existing checked local-
storage rules (for example, a private local address); this does not admit new
Rust-source constructs, unsigned public signatures or unchecked storage effects.

Target tests are split into construction, shape, recursive-admission and native
modules. The independently compiled native harness is handwritten test code,
not a production body template; its source-policy exception is an exact path
with adjacent-file rejection controls. No production import string is added.
