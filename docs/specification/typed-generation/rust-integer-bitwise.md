# Checked Rust signed-integer bitwise mapping

Status: implemented and independently reviewed (M35-03A-02D).

## Shared compiler boundary

`IntegerBitwise` consumes a privately constructed `BitwiseInput`. Its reader
requires an unadjusted, non-overloaded built-in operation with i32 or i64
operands and exactly the same result type. Binary operands have identical
types. The closed operation set is complement (`!`), and (`&`), or (`|`) and
xor (`^`). Boolean `!` remains the separate BooleanNegation capability.

Mappings receive compiler-origin operand expressions through read-only access,
not target source strings. Each backend registers a complete executable mapping
signature through its consuming builder; missing/duplicate/incorrect mappings
are compile errors. Compiler checks at this input boundary are not represented
as a claim that arbitrary runtime input is verified by Rust type checking alone.

## C17 mapping

Use CUnaryOperator::BitNot and CBinaryOperator::{BitAnd,BitOr,BitXor} with exact
I32/I64 values; never substitute logical operators, shifts or arithmetic.
C promotes I32 to Int, so the mapping uses the existing Int-to-I32 identity
normalization at its boundary. I64 is never converted. Source casts stay closed.
Calls are materialized in source order before the enclosing operator so C's
unspecified binary operand evaluation order cannot reorder source calls.
The shared target profile admits only equal-width I32/I64 operands, with the
C-mandated Int result for I32 and I64 result for I64. Existing dependency-derived
stdint imports and layout
obligations apply; no runtime helper or source fragment is added.

C exact-width signed typedefs require two's complement without padding, unlike
arbitrary C signed integers. The pinned C profile also fixes 32-bit int and
existing layout constraints; this is not an all-C-implementations claim.
See [C committee draft N1570, 6.5.3.3, 6.5.10–12 and 7.20.1.1](https://www.open-std.org/jtc1/sc22/wg14/www/docs/n1570.pdf).

## Java 21 mapping

Map I32/I64 to primitive int/long. Use JavaUnaryOperator::BitNot or the matching
JavaBinaryOperator::{BitAnd,BitOr,BitXor}, with their specific precedence enums.
Materialize binary operands left to right before constructing the operator.
Dependency-body validation and source-size reservation explicitly recognize
these nodes; neither widens the admitted types or enables numeric conversions.
See [JLS 15.15.5 and 15.22.1](https://docs.oracle.com/javase/specs/jls/se21/html/jls-15.html#jls-15.15.5).

## Evidence boundary

Native value tests must be independent of generated metadata for signatures and
truth. Native call traces additionally prove ordering/count, including mutations
that preserve commutative results. Full IntegerBitwise catalogue parity is not
claimed for unsupported source forms, Boolean eager operators or other widths.
Legacy runtimes remain until all feature-family cutover tasks are complete.

Read-only mapper probes inspect 28 actual nodes per backend on a closed local
fixture assembled from the native corpus sources. They require exact Java
operator/type/precedence metadata and C promotion/normalization structure.
Probe and production adapters must emit byte-identical packages. This is
separate from the native test's real two-crate dependency boundaries.
