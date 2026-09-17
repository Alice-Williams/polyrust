# Rust binary64 unary negation in Java21

- Status: implemented and verified for the bounded f64 unary-negation contract
- Contract: [shared](../../rust-floating-negation.md)

## Target mapping

Construct JavaExprKind::Unary with JavaUnaryOperator::Negate,
JavaPrecedence::Unary and exactly JavaPrimitive::Double for operand/result.
The dependency owner profile admits only this floating unary operator; existing
Boolean/int/long rules, original-call authority and budgets remain unchanged.
No Double boxing, known library callable or generated Runtime is involved.

Java unary floating minus changes the sign, unlike zero-minus for signed zero;
NaN produces NaN ([JLS 15.15.4](https://docs.oracle.com/javase/specs/jls/se21/html/jls-15.html#jls-15.15.4)).
Render the primitive unary AST structurally, including nested negation
parentheses. Materialize the original source operand exactly once as required
by existing evaluation-order lowering.

## Verification

Compile certified producer/consumer classes separately with Java21
-Xlint:all -Werror. Exact non-NaN bits and NaN classification must agree with
an independent sign-bit oracle. Source/classfile limits and two-slot Double
signatures remain checked. Counterfeit operand/result types or precedence reject.
Test-only no-negation/zero-minus and dropped/duplicated-call faults must fail
their respective value or trace oracles.

## Compiler adapter

JavaFloatingNegation consumes the private checked FloatingInput and validates
its Reader context. Lower the original operand once, materialize it under the
existing prelude mechanism, require TypePlan::F64, and construct primitive
Double Unary Negate with Unary precedence. Value::new verifies representation
agreement. Imported calls retain their original Java dependency witnesses.
