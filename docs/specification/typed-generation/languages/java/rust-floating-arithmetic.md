# Java21 binary64 arithmetic mapping

- Status: target foundation implemented and verified (02N-03); Rust source admission pending 02N-04
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
