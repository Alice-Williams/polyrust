# M35-03A-02N — Binary64 arithmetic

- Status: in-progress
- Parent: [scalar parity](M35-03A-02-scalar-parity.md)
- Depends on: [truncation](M35-03A-02M-floating-truncation.md)
- Specification: [checked arithmetic](../../specification/typed-generation/rust-floating-arithmetic.md)

## Ordered checkpoints

1. [02N-01 — Independent arithmetic oracle](M35-03A-02N-01-arithmetic-oracle.md).
2. [02N-02 — C target arithmetic](M35-03A-02N-02-c-arithmetic.md).
3. [02N-03 — Java target arithmetic](M35-03A-02N-03-java-arithmetic.md).
4. [02N-04 — Checked Rust capability](M35-03A-02N-04-compiler-arithmetic.md).

Each checkpoint needs a focused tested tree, independent review and its own
commit. The oracle alone admits no new generated operation. Complete C/Java
target evidence before enabling Rust source. Preserve older output bytes and
unrelated ownership work. Publish only when authorized.

## Definition of done

Original checked Rust built-in f64 addition, subtraction, multiplication and
division lower through one closed executable capability to ordinary typed target
operators. Exact non-NaN result bits, signed zeros, gradual underflow, infinities,
NaN categories and source evaluation order agree with an independent rational
oracle and native Rust. Public/private crate boundaries, authenticated imports,
resource accounting and atomic unsupported-source rejection remain intact.
No copied runtime, custom arithmetic library, raw fragment or disabled test.
Remainder, fused operations, casts, assignments and other widths remain separate.
