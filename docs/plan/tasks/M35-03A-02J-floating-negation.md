# M35-03A-02J — Built-in binary64 negation

- Status: complete
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
02J-02 is complete: canonical compiler capability, executable C/Java mappings,
14 compile-negative contracts, 36 atomic rejections, native 578-result proof,
eight typed AST observations and two dataflow fault controls per target.
804/804 release tests pass, with clean first and fresh independent Sol Extra High
reviews. All 138 prior generated bundle files remain byte-identical and 24 real
example files are available outside Docker. Broader floating parity is not claimed.
