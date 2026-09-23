# Checked Rust signed narrowing

- Status: independent oracle complete; target and source admission planned
- Plan: [02Y](../../plan/tasks/M35-03A-02Y-checked-narrowing.md)
- Targets: [C17](languages/c/rust-checked-narrowing.md), [Java21](languages/java/rust-checked-narrowing.md)

## Semantic contract

For an original i64 value x, conversion succeeds with the same mathematical
integer exactly when -2147483648 <= x <= 2147483647. Otherwise it produces
the standard integer-conversion error. This follows Rust's
[TryFrom contract and i64-to-i32 example](https://doc.rust-lang.org/std/convert/trait.TryFrom.html).
Neither wrapping/truncation, saturation, an error sentinel nor an exception is
an equivalent result. The oracle observes only success/error and the exact
success payload; it makes no claim about error formatting or representation.

## Authority and sequencing

First establish independent integer truth and pinned native Rust evidence.
Then specify and prove the bounded no-heap fallible scalar representation.
Only afterward add typed target conversion bindings and compiler discovery.
Use canonical core implementation/type identities, original checked HIR and
the original once-only operand; do not dispatch from method names.

The intended initial source spelling is i32::try_from(i64_value), returning
Result<i32, core::num::TryFromIntError>. The separate result foundation must
prove its construction/observation and metadata boundary before admission.
Native TryInto is a cross-check, not permission to lower arbitrary trait calls.
Other widths, custom TryFrom, wrapping as-casts, aliases, error formatting and
heap payloads are outside this increment.

## Proof

Unbounded integer truth, two native Rust profiles, actual faulty references,
strict success/error decoding, then separately compiled target consumers and
typed authority/atomic controls. Keep original source boundaries and docs.
No renderer escape hatch, helper runtime or new third-party dependency.
