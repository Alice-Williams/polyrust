# M35-03A-02I — Exact binary64 values before floating arithmetic

- Status: in-progress
- Parent: [scalar parity](M35-03A-02-scalar-parity.md)
- Depends on: [wrapping negation](M35-03A-02H-wrapping-negation.md)
- Specification: [shared](../../specification/typed-generation/rust-binary64-values.md)

## Ordered checkpoints

1. [02I-01 — Finite literal value foundation](M35-03A-02I-01-finite-literals.md):
   dependency-free checked binary64 payload and exact integer decomposition.
2. [02I-02 — C/Java target foundation](M35-03A-02I-02-target-values.md):
   typed finite literals, primitive value transport/comparison, certified target
   grammar/platform/authority and separately compiled native proof.
3. [02I-03 — Checked Rust values](M35-03A-02I-03-compiler-values.md):
   compiler-evaluated literals, exact source types/signatures/fields, dependency
   metadata, original-owner calls, atomic diagnostics and real generated examples.

Every child is separately gated, independently reviewed, committed and pushed.
Do not enable source f64 until target proof is complete. Arithmetic, casts,
bit conversions, nonfinite constant construction and float methods need later
operation-specific increments. This is not full JavaF64Values parity or
permission to delete legacy runtimes.

## Definition of done

Exact finite values and signed zero survive all enabled boundaries; NaN and
infinity input transport/comparison follow the explicit shared contract.
No decimal tolerance, float narrowing, integer-payload runtime wrapper, copied
helper, bit reinterpretation escape or weakened existing test is acceptable.
All child proof receipts must identify exact tested/reviewed trees.
