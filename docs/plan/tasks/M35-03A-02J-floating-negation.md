# M35-03A-02J — Built-in binary64 negation

- Status: in-progress
- Parent: [scalar parity](M35-03A-02-scalar-parity.md)
- Depends on: [binary64 values](M35-03A-02I-binary64-values.md)
- Specification: [shared contract](../../specification/typed-generation/rust-floating-negation.md)

## Ordered checkpoints

1. [02J-01 — C/Java target negation](M35-03A-02J-01-target-negation.md):
   exact primitive unary nodes, closed certification and separate native proof.
2. [02J-02 — Checked Rust negation](M35-03A-02J-02-compiler-negation.md):
   canonical executable capability, registration, original operand identity,
   crate-native/atomic/AST proof and actual exported examples.

## Definition of done

Built-in Rust f64 unary negation translates to native C/Java unary minus without
runtime helpers, casts, integer payloads or subtraction-from-zero substitution.
Every non-NaN binary64 sign flips exactly, including zeros/subnormals/infinities;
NaN classification is preserved without a payload/sign guarantee. Evaluation
occurs exactly once. Each child has passing Linux release/lint/native gates,
independent review and its own scoped commit/push. Other floating operations,
constants and runtime retirement remain separate work.

## Progress

02J-01 is complete: exact primitive C/Java target admission, 278-input native
oracle with four outputs per input, detected value/call-count faults,
786/786 release tests and a clean independent Sol Extra High review.
02J-02 remains the next compiler integration checkpoint. No additional
Rust source forms are enabled by target admission alone.
