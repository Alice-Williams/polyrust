# M35-03A-02N-03 — Java binary64 arithmetic target foundation

- Status: planned
- Parent: [binary64 arithmetic](M35-03A-02N-floating-arithmetic.md)
- Depends on: 02N-01 and 02N-02
- Specification: [Java mapping](../../specification/typed-generation/languages/java/rust-floating-arithmetic.md)

## Contract

Admit exact Double binary arithmetic in certified dependency bodies, with
operator-specific precedence and recursively authenticated original operands.
Do not infer integer arithmetic or general library-call permission.

## Definition of done and tests

Java21 strict-lint original/importing packages execute all four operations and
nested expressions against 02N-01 bit/category expectations. Include wrong
type/precedence/operator/authority controls, operand evaluation traces,
value-preserving dropped/duplicated calls, grouping and fused-result controls.
Rendered bytes stay within actual source reservations; existing package bytes,
full Linux release/lint gate and independent review pass. No runtime helper or
new Rust source admission is claimed by this target-only checkpoint.
