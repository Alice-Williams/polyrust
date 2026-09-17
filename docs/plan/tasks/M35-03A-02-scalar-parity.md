# M35-03A-02 — Scalar, constant and numeric feature parity

- Status: in-progress
- Parent: [M35-03A](M35-03A-runtime-free-parity.md)
- Depends on: M35-03A-01

## Contract

The first bounded implementation,
[M35-03A-02A — Boolean negation](M35-03A-02A-boolean-negation.md), is complete.
Other scalar operations remain separate work; this does not claim full parity.
[M35-03A-02B — short-circuit Boolean expressions](M35-03A-02B-short-circuit-booleans.md)
is also complete, including native operand-call traces and typed AST probes.
[M35-03A-02C — exact i64 values and comparisons](M35-03A-02C-i64-values.md)
is complete, including mixed-width signatures, native call-order controls and
target-AST admission tests. Other scalar families below remain outstanding.

[M35-03A-02D — integer bitwise operations](M35-03A-02D-integer-bitwise.md)
is complete, including native value/trace controls and direct mapper AST probes.

[M35-03A-02E — eager Boolean operators](M35-03A-02E-eager-booleans.md)
is complete, with exhaustive truth tables, native eager/lazy evaluation controls
and exact typed mapper probes. Other scalar families remain outstanding.

[M35-03A-02F — scalar constants](M35-03A-02F-scalar-constants.md) is complete
for bool/i32/i64 reads, local/public declarations, authenticated imports and
aliases, with native/atomic/cache/review evidence. Wider families remain open.

[M35-03A-02G — unit function results](M35-03A-02G-unit-results.md) is complete:
typed void foundation and checked compiler integration, with 760/760 release
tests, original-authority/native/AST proof and clean independent review.
Unit storage/parameters remain a separate capability.

[M35-03A-02H — wrapping signed negation](M35-03A-02H-wrapping-negation.md)
is complete: certified C/Java target foundation and identity-checked source
capability, with 778/778 release tests and clean fresh compiler review.
Method/associated i32/i64 wrapping_neg retain exact receiver evaluation and
original dependencies. Other arithmetic remains separate work.

[M35-03A-02I — binary64 values](M35-03A-02I-binary64-values.md) is complete:
a dependency-free finite literal witness, C/Java target proof and checked
compiler value/comparison integration, with 786/786 release tests and clean
independent review. Floating arithmetic/constants/casts remain separate work.

[02J — built-in binary64 negation](M35-03A-02J-floating-negation.md) is complete:
primitive target admission and canonical compiler integration, 804/804 release
tests, exact value/dataflow/atomic proof and clean independent reviews.
Remaining floating inspection and arithmetic require separate bounded contracts.

Add remaining f64/char/unit values, constants/aliases, Boolean operations,
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
