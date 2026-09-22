# M35-03A-02T — Lossless signed integer widening

- Status: in-progress
- Parent: [scalar parity](M35-03A-02-scalar-parity.md)
- Depends on: [multiplication](M35-03A-02S-wrapping-multiplication.md)
- Specification: [shared](../../specification/typed-generation/rust-signed-widening.md)

## Ordered checkpoints

1. [02T-01 — Independent widening oracle](M35-03A-02T-01-widening-oracle.md) — complete.
2. [02T-02 — C target foundation](M35-03A-02T-02-c-widening.md) — complete.
3. [02T-03 — Java target foundation](M35-03A-02T-03-java-widening.md) — complete.
4. [02T-04 — Checked compiler integration](M35-03A-02T-04-compiler-widening.md).

## Contract and completion

Implement the lossless i32-to-i64 conversion already represented by the legacy
IntegerConversions capability, using ordinary Rust `operand as i64`. The signed
mathematical value is unchanged, including negative values. Checked narrowing
requires its own result/failure contract and is not silently replaced by a cast.

Each checkpoint requires its own full Linux Bazel/native/lint gate, independent
review and tested commit/push. Do not admit source casts until both target
foundations are complete. Preserve old packages and unrelated WIP; export actual
source-owned packages at compiler integration. No runtime/helper or dependency.
This is partial conversion parity, not legacy-retirement approval.
