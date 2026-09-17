# Rust binary64 unary negation in C17

- Status: implemented and verified for the bounded f64 unary-negation contract
- Contract: [shared](../../rust-floating-negation.md)

## Target mapping

Construct CValueKind::Unary with CUnaryOperator::Negate and exact
CScalarType::F64 operand/result. Both the shared package profile and scalar-call
shape analysis admit this exact pair independently of integer bitwise rules.
Retain all existing ownership, numeric, canonical dependency and platform checks.

C unary minus is the negative of its promoted operand; no integer promotion
changes double ([N1570 6.5.3.3](https://www.open-std.org/jtc1/sc22/wg14/www/docs/n1570.pdf)).
The existing pinned binary64/default-environment C profile applies. Rendering
uses ordinary unary minus with structural parentheses; it must not rewrite
negation into subtraction from zero or call a runtime/bit-conversion helper.

## Verification

Compile generated original-owner producer/consumer units separately with pinned
GCC14 and Zig, C17 strict warnings, O0/O2, no fast-math or contraction.
Check exact non-NaN sign inversion and NaN classification against integer bits.
Preserve subnormals and signed zeros and validate default floating environment.
Instrument only test copies to observe exactly-once imported operand calls.
Keep double bitwise/logical operators, binary arithmetic and casts rejected.

## Compiler adapter

CFloatingNegation consumes the private checked FloatingInput, validates its
Reader context, lowers its original operand once and materializes an exact F64
local. It then calls the typed unary constructor. Calls remain full-expression
initializers under the existing sequencing rules; the renderer adds no
sequencing logic. No promotion, cast, integer guard or helper is needed.
