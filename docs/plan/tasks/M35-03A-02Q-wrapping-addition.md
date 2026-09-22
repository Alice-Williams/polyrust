# M35-03A-02Q — Wrapping signed-integer addition

- Status: complete
- Parent: [scalar parity](M35-03A-02-scalar-parity.md)
- Depends on: [negative-zero composition](M35-03A-02P-negative-zero-composition.md)
- Specification: [shared](../../specification/typed-generation/rust-wrapping-addition.md)

## Ordered checkpoints

1. [02Q-01 — Independent modular oracle](M35-03A-02Q-01-addition-oracle.md) — complete.
2. [02Q-02 — C target safety foundation](M35-03A-02Q-02-c-addition.md) — complete.
3. [02Q-03 — Java target foundation](M35-03A-02Q-03-java-addition.md) — complete.
4. [02Q-04 — Checked compiler integration](M35-03A-02Q-04-compiler-addition.md) — complete.

## Contract and completion

Implement only actual core i32/i64 wrapping_add through a private typed source
capability and normal C/Java ASTs. No custom runtime, unsigned source API or
ordinary Rust + admission is implied. Both source operands retain their original
ordered once-only evaluation; addition's commutativity is not trace evidence.

Complete, test and independently review each target foundation before enabling
source admission. Every checkpoint has its own full isolated Linux Bazel/lint
gate and commit/push, preserving unrelated WIP and all earlier generated outputs.
The compiler checkpoint exports actual source-owned packages and proves native
Rust/C/Java modular values, per-input traces, exact APIs and atomic rejection.

All four checkpoints are complete. Compiler integration passes all 924
release/lint targets with independent native/value/trace/dataflow proof, actual
exported examples and two clean whole-scope reviews. Task 02Q-04 records exact
tree/gate receipts and the two corrected historical test expectations. Remaining
scalar families and legacy-runtime retirement are still separate work.
