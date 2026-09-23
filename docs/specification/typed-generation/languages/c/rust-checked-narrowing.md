# Checked signed narrowing in C17

- Status: planned; no production admission
- Contract: [shared](../../rust-checked-narrowing.md)

Require the independently certified no-heap fallible result representation.
Materialize the source I64 operand once. Use typed inclusive I64 comparisons
before a Numeric(I32) conversion inside the success branch. Construct the
error variant otherwise. Never use an out-of-range signed cast or rely on
implementation-defined truncation. No unchecked inactive result payload read.

Target checks retain exact operand, guard, branch, result owner and payload
identity; rendering only prints certified nodes. Ordinary source-owned types
and standard includes replace runtime helpers. Native GCC14/Zig O0/O2 and
UBSan consumers must distinguish errors from every valid I32 value.
