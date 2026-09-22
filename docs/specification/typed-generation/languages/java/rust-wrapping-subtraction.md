# Rust wrapping subtraction in Java21

- Status: target foundation and checked compiler-source integration complete
- Contract: [shared](../../rust-wrapping-subtraction.md)

## Typed lowering

Materialize left completely before right into matching primitive Int or Long
values. Build JavaBinaryOperator::Subtract with Additive precedence and identical
operand/result types. Java's low-width integer difference matches this explicit
wrapping operation; see [JLS21 15.18.2](https://docs.oracle.com/javase/specs/jls/se21/html/jls-15.html#jls-15.18.2).
Do not enable ordinary checked/panicking Rust subtraction, widen, narrow, box,
coerce to Double, or invoke Math.subtractExact/custom runtime helpers.

## Certification and proof

The dependency-body traversal admits this exact primitive shape and recursively
checks both children, including original call/import authority and budgets.
Mixed widths, boxed/String/Boolean values, wrong result types or precedence and
unrelated operators remain rejected. Existing Double/Add contracts are unchanged.
The renderer prints the existing typed operator; no special text generation.

Strict separately compiled Java21 producers/clients must match independent modular
truth at both widths, including borrow/overflow boundaries and full-width samples.
Compiled reversal, addition, saturation, narrowing and operand-disconnection faults
must differ. Preserve depth, call-height, byte-bound, arity and both-child import
checks. Original once-only source evaluation and compiler witness registration are
proved by the separate completed source checkpoint, not inferred from target
admission. Its three-crate native/dataflow and API/docs/privacy gates stay active.

## Test organization

Addition and subtraction have sibling operation-specific test modules and share
test-only integer package/dependency fixtures. The shared native harness selects
an independent subtraction oracle, verifies the rendered operation and compiles
each producer before its forwarding consumer and external client. Shared test
fixtures are not compiler witnesses or production lowering functions. Preserve
existing addition package bytes while giving subtraction its own method names.
