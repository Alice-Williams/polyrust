# M35-03A-02O — Binary64 truncating remainder

- Status: in-progress
- Parent: [scalar parity](M35-03A-02-scalar-parity.md)
- Depends on: [binary64 arithmetic](M35-03A-02N-floating-arithmetic.md)
- Specification: [remainder contract](../../specification/typed-generation/rust-floating-remainder.md)

## Ordered checkpoints

1. [02O-01 — Independent remainder oracle](M35-03A-02O-01-remainder-oracle.md) — complete.
2. [02O-02 — C remainder foundation](M35-03A-02O-02-c-remainder.md) — complete.
3. [02O-03 — Java remainder foundation](M35-03A-02O-03-java-remainder.md).
4. [02O-04 — Checked source integration](M35-03A-02O-04-compiler-remainder.md).

Each checkpoint requires an isolated tested tree, independent review and a
separate commit. Source admission follows both verified target foundations.
No legacy runtime removal or broad family parity is inferred from one operator.

## Definition of done

The original built-in f64 remainder expression preserves both ordered operands,
truncating-quotient semantics, signed zeros, nonfinite categories and original
dependencies across ordinary generated C/Java packages. Pure integer operations,
overloads, Euclidean remainder and IEEE nearest-quotient remainder are not
admitted by this capability. Native independent proof and actual AST/compile
contracts pass; older artifacts and unrelated work remain unchanged.
