# M35-03A-02V-01 — Independent signed-infinity constant oracle

- Status: complete
- Parent: [02V](M35-03A-02V-infinite-f64-constants.md)
- Depends on: [finite constants](M35-03A-02U-finite-f64-constants.md)
- Specification: [shared](../../specification/typed-generation/rust-infinite-f64-constants.md)

## Contract

Define exact sign/bit truth with integer operations independently of any
backend. Compare named and computed native Rust constants at both pinned
optimization/check settings. Challenge exponent-only misclassification with
finite values and signed NaN payloads. No target/source admission changes.

## Definition of done and tests

Pin the original expression list, classification corpus and boundary membership.
Both native binaries agree bit-for-bit; actual sign-loss, finite-clamp and zero
fault columns match independent faulty models and disagree with truth. Native
is_infinite/is_nan checks agree with integer classification. Run full Linux
Bazel release/lint, fresh review and preservation checks before separate push.

## Implemented proof and gate

The 20 named/computed native expressions cover both signs through original
constants, negation, signed-zero division, overflow, exact bits and aliases.
Independent integer expectations match actual sign-loss, finite-clamp and
zero-replacement fault columns. The pinned classification corpus contains
24,566 finite patterns, 366 signed NaN payloads and the two infinities. Every
single fraction bit at the all-ones exponent is challenged under both signs.
Three deliberately incorrect classifiers disagree with the corpus.

Focused native/rustfmt/Bazel/docs tests passed. Clippy initially rejected
multiplication by -1 as redundant negation; using -2 keeps actual multiplication
coverage without suppressing the lint or changing expected results.

Implementation tree `7aeef00a246ccb28b19d3f6aa678714ef766ebdb` passes all
1,004 Linux release/lint targets, invocation
`a7329d0f-1b16-494a-9661-2b935780368c` (11 executed, 993 cached).
All 462 previous generated file hashes and 38 unrelated WIP hashes match.
Independent whole-scope Sol Extra High review found no core error. Both optional
hardening suggestions were implemented: out-of-range/invalid oracle inputs
must reject, and native is_infinite/is_nan flags are observed independently
for every classification pattern, including NaNs. The reviewer checked this
bounded delta and found no implementation errors.

The hardened implementation tree `319425543cb771ef05786aed99659143f3276c54`
passes all 1,004 release/lint targets, invocation
`81bc264e-f86d-48ac-90ea-519a4009ce42` (12 executed, 992 cached).
Documentation-only closure is gated again before the separate commit/push.
No production target or source admission has changed. C/Java foundations and
checked source integration remain separate unfinished checkpoints.
