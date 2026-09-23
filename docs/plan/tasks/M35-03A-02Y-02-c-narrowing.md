# M35-03A-02Y-02 — C checked narrowing

- Status: planned
- Parent: [02Y](M35-03A-02Y-checked-narrowing.md)
- Depends on: 02Y-01 and [05A](M35-03A-05A-scalar-results.md)
- Specification: [C17](../../specification/typed-generation/languages/c/rust-checked-narrowing.md)

## Contract and definition of done

Bind a typed fallible conversion to the certified scalar-result representation.
Compare the original I64 value against both inclusive bounds before narrowing;
never execute an out-of-range signed conversion. Preserve one operand evaluation.

## Tests

Reuse independent oracle successes/errors in separately compiled GCC14/Zig
O0/O2 and UBSan consumers. Check negative and positive endpoints, all native
fault families, exact result type/variant authority, foreign readers and
resource bounds. Reject disconnected casts/guards, wrong result owner and
foreign variant tokens. Full Linux release/lint and fresh review precede commit.
This target step alone enables no Rust source forms or custom runtime.
