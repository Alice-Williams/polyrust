# M35-03A-02M — Binary64 truncation

- Status: complete
- Parent: [scalar parity](M35-03A-02-scalar-parity.md)
- Depends on: [absolute value](M35-03A-02L-floating-absolute.md)
- Specification: [checked truncation](../../specification/typed-generation/rust-floating-truncation.md)

## Ordered checkpoints

1. [02M-01 — C standard-call foundation](M35-03A-02M-01-c-truncation.md).
2. [02M-02 — Java standard-call foundation](M35-03A-02M-02-java-truncation.md).
3. [02M-03 — Compiler capability and package integration](M35-03A-02M-03-compiler-truncation.md).

Each checkpoint requires its own tested tree, independent review and scoped
commit. Push after completion when publishing authority is available. Do not
combine these checkpoints or delete the legacy runtime on partial evidence.

## Definition of done

Original checked Rust f64::trunc method/associated calls map through a private
typed witness and executable capability to ordinary C and Java. Native values,
single evaluation, original dependency authority, publication metadata, atomic
rejections and compile-negative registration contracts are proven. Actual
generated examples are exported locally. Full Linux Bazel release and lint gates
pass with no disabled tests. Wider rounding/arithmetic remains separate.

## Completion

All three checkpoints are implemented and independently reviewed. The compiler
integration passes 858 Linux release/lint tests, native Rust/C/Java equivalence,
typed AST and call-dataflow probes, capability compile negatives, atomic
rejections and authenticated library metadata controls. See 02M-03 for exact
tree/invocation and local example receipts. Existing bundle bytes and unrelated
ownership work are preserved. No full arithmetic or legacy retirement claim.
