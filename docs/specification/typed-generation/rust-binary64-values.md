# Rust binary64 values and finite literals

- Status: finite value foundation implemented; target/source integration specified
- Plan: [M35-03A-02I](../../plan/tasks/M35-03A-02I-binary64-values.md)
- Targets: [C17](languages/c/rust-binary64-values.md),
  [Java21](languages/java/rust-binary64-values.md)

## Value contract and scope

Rust f64 is IEEE binary64 ([Rust Reference](https://doc.rust-lang.org/reference/types/numeric.html)).
The first increment covers finite literal construction, exact primitive
transport and six ordinary comparisons, not general floating arithmetic.
Finite values compare bit-for-bit in proof, including both zeros. Comparisons
instead obey numeric equality/order: zeros compare equal and NaNs are
unordered. For NaN inputs the initial observable contract covers classification
and comparison, not payload/sign preservation. This restriction must remain
visible; future to_bits/from_bits support requires a separate stronger proof.

No tolerance-based equality, f32 narrowing, integer-backed generated wrapper,
decimal reparsing or silent nonfinite substitution is allowed.

## Shared checked literal layer

FiniteBinary64 is a dependency-free semantic value, not a target AST operation.
Its private bits preserve the exact finite value, including negative zero.
Only checked construction is public. Sign and finite class are closed enums;
nonfinite construction returns a typed error. Equality is exact representation
equality, deliberately distinguishing signed zeros.

Its private-constructed parts expose an unsigned significand and signed binary
exponent with a separate sign. Zero uses significand zero and exponent zero;
subnormals use their nonzero fraction and exponent -1074; normals use the hidden
leading bit plus fraction and encoded exponent minus 1075. The renderer chooses
target syntax; this layer contains no strings, parser or arithmetic lowering.

Unknown data is checked once to obtain the finite witness. Private fields and
typed AST variant payloads then prevent safe code from handing a nonfinite
literal to the renderer. This is a construction invariant, not a claim that
arbitrary input bits are validated by the Rust compiler without evaluation.

## Boundaries

Target profiles explicitly admit only the enabled operators for each numeric
domain. Widening a scalar predicate must not accidentally enable bitwise,
wrapping, shifts, casts or unchecked float arithmetic. Ordinary dependency
certificates retain their producer authority and resource bounds.

Compile-time literal availability and runtime input categories are separate.
NaNs/infinities are legal f64 input values; nonfinite literal/constant generation
remains unsupported until mapped through typed standard-language constructs.
The Rust f64 documentation explains why arithmetic cannot generally promise
NaN payload preservation ([f64](https://doc.rust-lang.org/std/primitive.f64.html)).
