# Rust wrapping negation in Java 21

- Status: target foundation implemented (M35-03A-02H-01); compiler integration planned (02H-02)
- Contract: [shared](../../rust-wrapping-negation.md)

## Typed mapping

I32 maps to primitive int and I64 to primitive long. Materialize the source
receiver once and emit JavaUnaryOperator::Negate with unary precedence and the
same primitive result type. No widening, boxing, checked-math call or support
class is required.

Java's integer negation preserves the minimum value on overflow; see
[JLS 15.15.4](https://docs.oracle.com/javase/specs/jls/se21/html/jls-15.html#jls-15.15.4).
This matches explicit Rust wrapping_neg, not all possible source negation.

## Certificate and publication

Closed source/dependency admission recognizes exact int/long negation and
traverses the operand for dependency, local-scope, type and call-height evidence.
Bounded source-size reservation accounts for the operator and operand.
Wrong result widths or forged/missing/recertified receiver-call authorities
remain rejected. Other numeric domains are not enabled by this change.

Render ordinary certified AST. Public/private ownership, methods, scalar
signatures and manifest schemas stay unchanged. No runtime or helper artifacts
are generated.

## Required evidence

Typed unary/result/precedence and nested-call/source-bound assertions; separately
compiled Java 21 producers/clients with strict lint; independent boundary and
deterministic modular truth and deliberate wrong-operation/width controls.
Compiler source admission is deferred until the checked capability exists.
