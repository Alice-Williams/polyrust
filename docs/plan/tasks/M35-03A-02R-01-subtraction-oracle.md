# M35-03A-02R-01 — Independent wrapping-subtraction oracle

- Status: complete
- Parent: [02R](M35-03A-02R-wrapping-subtraction.md)
- Depends on: [02Q](M35-03A-02Q-wrapping-addition.md)
- Specification: [shared](../../specification/typed-generation/rust-wrapping-subtraction.md)

## Contract

Compute signed modulo-2^32/2^64 differences with unbounded integers, independently
of native/generated subtraction. Reuse the deterministic signed pair corpus if
its borrow/underflow coverage is explicitly checked; never reuse addition truth.
Keep support test-only, focused and independently cached.

## Definition of done and tests

Known extrema and borrow-boundary vectors, range/congruence, x-0=x, x-x=0 and
modular antisymmetry pass. Corpus includes both subtraction-overflow directions,
unequal asymmetric operands, full-width deterministic pairs and every power-of-two
borrow boundary. Detect addition, reversed subtraction, saturation and narrowing
faults independently at both widths. Pinned native Rust wrapping_sub agrees at
optimization/checks 0/yes and 2/no. Rustfmt, Clippy, buildifier and the full gate
pass; fresh review is clean; commit/push separately. No target/source admission.

## Completion evidence

The oracle reuses only the reviewed signed input corpus/domain helpers from
addition; it computes every expected difference independently with unbounded
Python subtraction and canonical residue-to-signed conversion. All 15,790 unique
width/operand pairs agree with native Rust wrapping_sub under O0/checks-on and
O2/checks-off. Known boundary identities, both overflow directions, every bit's
borrow boundary, range/congruence and modular antisymmetry pass. Five distinct
fault families are detected at both widths: saturation, addition, reversal,
borrowless XOR and narrowing. The reviewed corpus count is pinned against shrinkage.

Focused oracle, Clippy, rustfmt, buildifier and documentation targets passed:
a5cc11b8-97dd-4abc-b49e-3c3e2fe7f831. Exact tree
2de5d22741d769e9bc9c00c82a743c5308fa3682 passes all 927 release/lint targets,
d669b66a-fced-4eda-b241-f6888c145008 (10 executed, 917 cached).
All 372 prior generated files and all 38 protected unrelated WIP hashes match.

Independent Sol Extra High review found no core math, corpus, native-reference,
Bazel-wiring or specification defect. Its optional count guard was accepted;
activation/guard re-review is clean. No target certificate, renderer, compiler
admission, legacy test/runtime path or third-party dependency changed. The C,
Java and source checkpoints remain planned. Final documentation closure is gated
again before this checkpoint's separate commit/push.
