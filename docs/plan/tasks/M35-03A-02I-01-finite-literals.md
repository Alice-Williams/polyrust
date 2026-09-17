# M35-03A-02I-01 — Dependency-free finite binary64 literal values

- Status: complete
- Parent: [binary64 values](M35-03A-02I-binary64-values.md)

## Contract

Add a focused dependency-free Rust crate with an independently cached Bazel
library/test boundary. FiniteBinary64 has a private representation and a
checked constructor from IEEE binary64 bits. Explicit enums distinguish sign,
finite class and rejected nonfinite categories. Both zeros are retained;
equality is representation equality, not floating arithmetic equality.

Return a private-constructed decomposition consisting of sign, integer
significand and binary exponent, exactly representing sign * significand *
2^exponent. No decimal parsing, host floating arithmetic, target syntax,
templates, runtime helper or third-party dependency belongs in this layer.
Target ASTs will consume the finite witness only in the next child.

## Definition of done and tests

- Constructors reject positive/negative infinity and every sampled quiet or
  signaling NaN payload, never silently canonicalizing them.
- Exact bits and signed zero survive round trips. Both signs, every encoded
  exponent, boundary fraction patterns and a broad deterministic bit corpus
  agree with Rust's independent f64 classification.
- Hand-derived decomposition vectors include both zeros, smallest/largest
  subnormal, smallest normal, one, neighbors of one and maximum finite.
  Independent recomposition from returned parts agrees with input bits.
- Compile-negative documentation proves fields of both witness and parts
  cannot be forged; a positive public API example compiles.
- Unit/doctests, Rust/Bazel linters, full Linux Bazel release gate and a fresh
  Sol Extra High review pass before the scoped commit/push.
- Existing source admission, target rendering and generated schemas stay
  unchanged. Native target proof is explicitly deferred to 02I-02.

## Proof receipt

Implementation tree: `d68009f7fe262edf29f795484342fe223047458b`.
The Linux dev-container Bazel release/lint gate passed all 780 tests across
1,176 targets (6 executed, 774 cached), invocation
`7ec090ad-3bc8-4936-bfb1-2145dc06d0ed`.

The five unit tests exercise 28,672 exponent/sign/boundary combinations and
100,000 deterministic full-width patterns, independent recomposition,
hand-derived boundaries and rejected nonfinite payloads. Three doctests prove
the positive public API and both private-construction compile failures.

A fresh independent Sol Extra High review of that exact implementation tree
against `93e3e13cb6fcb3a2df5e79984a201552ad11a381` found no core issues
and no optional feature needed for this checkpoint. Target syntax and frontend
admission remain unchanged; native C/Java evidence belongs to 02I-02.
