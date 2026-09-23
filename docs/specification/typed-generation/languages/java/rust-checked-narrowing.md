# Checked signed narrowing in Java21

- Status: planned; no production admission
- Contract: [shared](../../rust-checked-narrowing.md)

Require the independently certified no-heap fallible result representation.
Materialize the Long operand once and compare both inclusive bounds before
casting to Int in the success branch. Return the error variant otherwise.
Java's truncating cast and Math.toIntExact's exception are not interchangeable
with an explicit Rust Result. No boxed-null or numeric error sentinel.

Retain source/callee/result-instance authority separately from Java syntax.
Use ordinary source-derived certified types, not Runtime.Result. Prove exact
success/error values, result visibility and dependency calls under strict
separate Java21 compilation and normal/interpreted execution.
