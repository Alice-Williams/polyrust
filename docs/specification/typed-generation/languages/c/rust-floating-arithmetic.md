# C17 binary64 arithmetic mapping

- Status: target foundation implemented and verified (02N-02); Rust source integration verified with 879 release/lint tests and two clean reviews (02N-04)
- Parent: [shared arithmetic](../../rust-floating-arithmetic.md)

## Target AST

Use CBinaryOperator::{Add, Subtract, Multiply, Divide} with exact scalar F64
operands and result. Shared-profile admission recursively visits both operands;
existing generated-call authority, ownership and numeric checks remain active.
Do not permit integer arithmetic, pointer arithmetic, float casts or remainder
through this rule. Render only existing structural nodes, retaining grouping.
Pure arithmetic introduces no header, system library or generated runtime.

## Execution profile

Retain the pinned Linux LP64 binary64 requirements: 53-bit precision, exact
exponent/subnormal properties, FLT_EVAL_METHOD=0, nontrapping nearest-even
environment and no flush-to-zero. Compile with no fast math or contraction.
No optimization may reassociate source operators or fuse multiply/add.
Verify GCC14 and Zig behavior explicitly, including zero division and invalid
operations, under the existing supported floating profile. This is not a
promise for arbitrary ISO C implementations or foreign floating environments.

The source mapper emits ordered materialization statements so C operand
evaluation order cannot reorder source calls. Each intermediate has double
storage/type and preserves the original expression grouping.

## Required evidence

Apply the shared integer/rational oracle to separately compiled producer and
consumer packages under GCC/Zig O0/O2. Include exact boundary/zero/nonfinite
results, sequential rounding, nested original calls and independent call traces.
Metadata and resource bounds must remain authentic through imports. Existing
integer UB rejection is a regression requirement. Unsupported target shapes
remain diagnostic; arithmetic admission is not permission for all C operators.

Reference: [N1570](https://www.open-std.org/jtc1/sc22/wg14/www/docs/n1570.pdf),
5.2.4.2.2, 6.5.5, 6.5.6 and Annex F; concrete claims remain pinned-profile tested.

## Checked source integration

The executable FloatingArithmetic slot maps only the canonical checked
ArithmeticInput. It validates the original compiler context, materializes the
left operand completely before beginning the right, then constructs the
structural operator node. Compiler-owned built-in identity and exact f64 types
exclude overloaded operators and implicit conversion. Nested arithmetic,
ordinary calls, immutable locals and admitted inspection/negation compose.

Source proofs are separate from the earlier hand-built target tests:
`arithmetic_native_test`, `arithmetic_ast_test`, `arithmetic_contract_test`
and `arithmetic_rejection_test`. The three-crate example is exported as actual
unmodified generated files by the native test, not a handwritten illustration.
