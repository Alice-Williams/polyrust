# Checked Rust binary64 unary negation

- Status: target foundation complete; compiler integration specified
- Plan: [M35-03A-02J](../../plan/tasks/M35-03A-02J-floating-negation.md)
- Targets: [C17](languages/c/rust-floating-negation.md), [Java21](languages/java/rust-floating-negation.md)

## Source and observable contract

The Rust built-in unary minus accepts floating operands; overloaded Neg is a
different operation ([Rust Reference](https://doc.rust-lang.org/reference/expressions/operator-expr.html#negation-operators)).
The initial capability supports exactly f64 built-in unary negation, not trait
calls, overloaded references, f32, arithmetic, conversions or float constants.

For all non-NaN binary64 inputs, output bits equal input bits with the sign bit
flipped. This includes both zeros, subnormals, finite extremes and infinities.
For NaN input, output remains NaN; payload, quiet/signaling representation and
sign are not promised. The existing default nontrapping floating environment
contract remains in force. A primitive operand is evaluated exactly once.

## Typed layers

FloatingNegation has a private-constructed compiler input carrying canonical
HIR source/operand identities and checked exact f64 types. The original
TypeckResults must be verified both at construction and when mapping. Supports
registers an actual lowering implementation, never a Boolean support marker.

Each mapping lowers its original operand once and constructs the target's
primitive unary Negate node with exact f64/Double result type. No string token,
sign-bit helper, copied runtime, cast or target-independent emitter semantics
are introduced. Finite negative literals retain their existing checked literal
path, including compiler rounding and negative zero.

## Proof boundaries

The target-only checkpoint does not enable new compiler source forms.
The compiler checkpoint adds real multi-crate native/AST/compile-negative and
atomic-rejection proof. Native oracles use independent integer-bit expectations;
test-only dropped/duplicated calls and sign-loss mutants establish observability.
Broader FloatNeg legacy parity is not inferred from an operation name alone.
