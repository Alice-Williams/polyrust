# M35-03A-02U-01 — Independent finite-constant oracle

- Status: complete
- Parent: [02U](M35-03A-02U-finite-f64-constants.md)
- Depends on: [signed widening](M35-03A-02T-signed-widening.md)
- Specification: [shared](../../specification/typed-generation/rust-finite-f64-constants.md)

## Contract

Build an independent bit-pattern corpus for finite compiler-evaluated f64
constants before changing target or source admission. Reuse FiniteBinary64 as a
checked representation only; expected values must not be derived from emitters.

## Definition of done and tests

Native Rust constant reads at O0/O2 agree with integer-bit truth, including
negative zero, subnormal boundaries, exponent/significand variation, extrema,
rounded decimal literals and independently specified computed constants.
The oracle rejects zero-sign loss, f32 rounding and wrong-value controls.
Nonfinite values remain distinct rejected cases. Document corpus size and
coverage honestly; a finite corpus is not exhaustive binary64 verification.
Full Linux release/lint gate and fresh independent review pass. Preserve old
output/WIP, then commit and push only this checkpoint.

## Implemented proof

The independent integer/rational oracle contains 24,566 distinct finite patterns:
five significand patterns at every finite exponent and both signs, plus 4,096
deterministic finite full-width samples. Ten separately specified constant
expressions cover decimal rounding, a halfway addition, computed decimal sum,
division, subnormal/normal boundaries, maximum magnitude and negative zero.
Fixed hexadecimal anchors cross-check the rational expectations.

Native Rust builds the corpus through constant evaluation into a static array
and observes the same values at O0/checks-on and O2/checks-off. Every output
includes three actual faulty computations: zero-sign collapse, f32 rounding
and a changed low bit. A separate integer model checks every fault column,
including gradual f32 underflow, ties-to-even and overflow. Four nonfinite
controls are categorized natively and rejected by the finite oracle.

No backend, renderer or compiler-source admission is changed.

## Completion evidence

All 991 Linux Bazel release/lint targets passed (12 executed, 979 cached),
invocation `5491c4dd-02b4-467b-8d14-e9ceab0c6ffe`. Fresh independent Sol
Extra High review of implementation tree
`0b5fa6f24a88459972d1bcda5a8afdc45ce4a927` against `a1892f3` found no
substantiated defects across all 16 scoped files. The final documentation-only
closure is gated again before commit. All 423 prior generated file hashes and
38 unrelated ownership/conditional-source WIP hashes remain unchanged.

This completes only the independent oracle. C/Java constant foundations and
checked source admission remain separate unfinished checkpoints.
