# M35-03A-02I-02 — Certified C/Java binary64 target values

- Status: in-progress
- Parent: [binary64 values](M35-03A-02I-binary64-values.md)
- Depends on: [finite witness](M35-03A-02I-01-finite-literals.md)
- Specifications: [C](../../specification/typed-generation/languages/c/rust-binary64-values.md),
  [Java](../../specification/typed-generation/languages/java/rust-binary64-values.md)

## Contract

Use FiniteBinary64 in actual target literal enums. Emit finite hexadecimal
floating syntax from typed sign/significand/exponent data, without decimal
rounding or helper calls. Extend only primitive value/field/signature transport
and six comparison operators, including original-owner dependency admission.
Explicitly restrict integer-only operators after widening any shared scalar
predicate; float arithmetic/conversion must not leak through a broad match.

## Definition of done and tests

- Both signs, signed zeros, subnormals, boundary exponents and finite random
  payloads compile with exactly matching bits in GCC/Zig O0/O2 and Java 21
  strict lint. Public producers and consumers compile separately.
- Native inputs include infinities and NaNs; comparison results use independent
  category/order expectations. NaN payload identity is not claimed.
- C binary64 platform assertions and pinned execution assumptions are explicit;
  wrong/missing assertions reject before certification.
- Primitive double remains exact in signatures, fields, locals, conditions,
  source bounds and original dependency witnesses. Wrong widths/operators,
  narrowing, invalid authorities and insufficient bounds reject.
- Deliberate sign/exponent/subnormal corruption and dropped/reordered operand
  calls are caught by bit/trace oracles. No numeric epsilon is used.
- Full Linux release/lint gate and fresh independent review pass before push.
  Rust-source admission stays unchanged in this checkpoint.

## Integration prerequisite

Inspection found that the shared linker supports named symbol imports and
generated-file imports, but not an unnamed standard-library requirement.
[M35-03A-02I-02A](M35-03A-02I-02A-library-imports.md) adds that independently
reviewed boundary first. The remaining C/Java target implementation stays in
progress until every parent exit criterion has its own evidence.
