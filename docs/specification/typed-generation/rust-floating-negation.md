# Checked Rust binary64 unary negation

- Status: implemented and verified for the bounded f64 unary-negation contract
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

## Executable compiler implementation

FloatingInput::read authenticates the exact HIR node and original TypeckResults
before querying operand/result types. Both are unadjusted built-in f64; a
type-dependent operator definition rejects overloaded Neg. Private fields retain
the canonical source and operand, and require_context repeats that validation
at the mapping boundary.

Both consuming builders now require FloatingNegation's actual Reader-to-value
mapping independently of WrappingNegation. The traversal selects this capability
for non-literal unary-minus f64 operands. Negative literals retain LiteralValues,
including compiler rounding and negative zero; signed integer non-literal
negation remains unsupported outside its explicit wrapping capability.
