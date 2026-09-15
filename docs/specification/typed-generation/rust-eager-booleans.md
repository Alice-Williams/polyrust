# Checked Rust eager Boolean mapping

Status: implemented and independently reviewed (M35-03A-02E).

## Compiler boundary

EagerBooleans is independent of ShortCircuitBooleans and IntegerBitwise.
EagerBooleanInput has private fields. Its sole checked constructor requires
built-in, unadjusted bool operands and bool result, with no overloaded operator.
EagerBooleanOperator is a closed And/Or/Xor enum; mappings receive read-only
compiler-origin operands, not source strings. The consuming backend builder
requires a correctly typed executable mapping. Unknown source still undergoes
compiler and admission checks, not compile-time checking of runtime data.

Rust evaluates both operands of these Boolean operators, unlike lazy `&&` and
`||`. See the [Rust operator reference](https://doc.rust-lang.org/reference/expressions/operator-expr.html#arithmetic-and-logical-binary-operators).

## C17 mapping

Lower left then right using the existing once-only call materialization.
Build CBinaryOperator::BitAnd/BitOr/BitXor over Bool operands, whose integer
promotion produces Int, then explicitly normalize the result to Bool.
These operands have canonical zero/one values; no signed overflow or runtime
helper is involved. The profile admits this Bool/Bool -> Int form separately
from existing equal-width I32/I64 bitwise forms, never mixed operands.
Keep source casts closed. Ordinary typed dependency discovery supplies headers.

## Java 21 mapping

Materialize left then right, then emit JavaBinaryOperator::BitAnd/BitOr/BitXor
over primitive Boolean types, with the corresponding exact JavaPrecedence.
Dependency-body admission and resource accounting traverse both operands.
No boxing, runtime helper, inheritance or source fragments are introduced.
See [JLS 15.22.2](https://docs.oracle.com/javase/specs/jls/se21/html/jls-15.html#jls-15.22.2).

## Proof boundary

Truth tables alone cannot distinguish eager evaluation from lazy or reordered
evaluation. Require independent call traces on the actual generated code and
deliberate value-preserving evaluation faults. Probe the actual mapped AST,
not a separately reconstructed approximation. Native consumers compile
separately and cross a real crate/package boundary.

This adds scalar built-in Boolean forms within the existing no-heap grammar,
not every legacy operand shape, arbitrary Rust programs or full runtime parity.
