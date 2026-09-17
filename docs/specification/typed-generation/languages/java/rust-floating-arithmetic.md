# Java21 binary64 arithmetic mapping

- Status: target foundation implemented and verified (02N-03); Rust source integration verified with 879 release/lint tests and two clean reviews (02N-04)
- Parent: [shared arithmetic](../../rust-floating-arithmetic.md)

## Target AST

Use JavaBinaryOperator::{Add, Subtract, Multiply, Divide} with primitive Double
operands/result. Add/Subtract require Additive precedence; Multiply/Divide
require Multiplicative precedence. Reject mixed types, wrong precedence and
unadmitted operations. Recursively authenticate operands and imported calls.
Retain structural rendering and exact source-byte bounds, with no helper class,
known library call or new import required.

Lower/materialize ordered source operands once before constructing the node.
Preserve every source operation/grouping; do not rewrite as Math.fma, reciprocal
multiplication, reassociated trees or integer operations. Java21 strict floating
evaluation supplies separate binary64 rounding; do not add obsolete strictfp.

## Required evidence

Strict Java21 compilation (-Xlint:all -Werror) and execution across separately
compiled owners/importers must agree with the shared exact oracle. Include
normal/subnormal/overflow, zero signs, zero division, infinities and NaN category.
Prove original receiver/operand dataflow and call traces independently; reject
malformed type, precedence, operator, dependency and resource metadata.
Compiled wrong-operator/grouping/evaluation faults must be observable.

Reference: [Java SE 21, 15.4](https://docs.oracle.com/javase/specs/jls/se21/html/jls-15.html#jls-15.4),
15.17 and 15.18. Numeric promotion is not permission to admit mixed source types.

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
