# M35-03A-02R — Wrapping signed-integer subtraction

- Status: in-progress
- Parent: [scalar parity](M35-03A-02-scalar-parity.md)
- Depends on: [wrapping addition](M35-03A-02Q-wrapping-addition.md)
- Specification: [shared](../../specification/typed-generation/rust-wrapping-subtraction.md)

## Ordered checkpoints

1. [02R-01 — Independent subtraction oracle](M35-03A-02R-01-subtraction-oracle.md) — complete.
2. [02R-02 — C target safety foundation](M35-03A-02R-02-c-subtraction.md) — complete.
3. [02R-03 — Java target foundation](M35-03A-02R-03-java-subtraction.md) — complete.
4. [02R-04 — Checked compiler integration](M35-03A-02R-04-compiler-subtraction.md).

## Contract and completion

Admit only actual core i32/i64 wrapping_sub through private compiler witnesses
and executable typed mappings. Preserve ordered once-only source operands.
This is not ordinary Rust subtraction, multiplication, checked arithmetic or
unsigned source support. No custom runtime, helper catalogue or new dependency.

Each checkpoint has its own full isolated Linux Bazel/lint gate, fresh independent
review and tested commit/push. Target foundations must be complete before source
admission. Preserve earlier generated packages and unrelated WIP. Compiler
integration exports actual packages and proves source authority, semantics,
privacy, documentation, diagnostics and evaluation order.
