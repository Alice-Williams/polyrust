# M35-03A-02 — Scalar, constant and numeric feature parity

- Status: in-progress
- Parent: [M35-03A](M35-03A-runtime-free-parity.md)
- Depends on: M35-03A-01

## Contract

The first bounded implementation,
[M35-03A-02A — Boolean negation](M35-03A-02A-boolean-negation.md), is complete.
Other scalar operations remain separate work; this does not claim full parity.

Add missing i64/f64/char/unit values, constants/aliases, Boolean operations,
integer checked/wrapping/bitwise/shift/conversion operations, comparisons and
floating arithmetic/inspection to the Rust-source C and Java mappings. Existing
i32/bool support is only partial parity. Specify source forms and target rules
per operation; register real typed lowering functions, never support markers.

## Definition of done and tests

- Separate operation-specific implementation tasks before enabling support.
- Native Rust/C/Java agree on boundaries, overflow/division/shift/conversion
  failures, short-circuit evaluation, NaN, infinities, negative zero and bits.
- No C signed-overflow UB or Java masking/narrowing approximation.
- Unsupported source rejects before output; constant/type/call provenance and
  compile-negative mapping contracts remain enforced.
- No copied/custom runtime artifact; generated support uses ordinary typed AST.
- Full isolated gates, fresh review and separate tested commits per operation.
