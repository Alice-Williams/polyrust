# M35-03A-02S — Wrapping signed-integer multiplication

- Status: in-progress
- Parent: [scalar parity](M35-03A-02-scalar-parity.md)
- Depends on: [wrapping subtraction](M35-03A-02R-wrapping-subtraction.md)
- Specification: [shared](../../specification/typed-generation/rust-wrapping-multiplication.md)

## Ordered checkpoints

1. [02S-01 — Independent multiplication oracle](M35-03A-02S-01-multiplication-oracle.md) — complete.
2. [02S-02 — C target safety foundation](M35-03A-02S-02-c-multiplication.md) — complete.
3. [02S-03 — Java target foundation](M35-03A-02S-03-java-multiplication.md).
4. [02S-04 — Checked compiler integration](M35-03A-02S-04-compiler-multiplication.md).

## Contract and completion

Admit actual core i32/i64 wrapping_mul only through private compiler witnesses
and executable typed bindings. Preserve original left-before-right, once-only
operand evaluation even though multiplication is commutative. Ordinary Rust *,
checked/saturating arithmetic, other widths and unsigned source remain outside.
No custom runtime, helper catalogue or dependency is required.

Each checkpoint has an isolated full Linux Bazel/native/lint gate, independent
review and separate tested commit/push. Source admission waits for both target
foundations. Preserve old output bytes and unrelated WIP; export actual packages
at the final source checkpoint. This does not retire legacy arithmetic or runtimes.
