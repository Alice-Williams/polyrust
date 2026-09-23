# M35-03A-02Y-04 — Checked narrowing source integration

- Status: planned
- Parent: [02Y](M35-03A-02Y-checked-narrowing.md)
- Depends on: [Java foundation](M35-03A-02Y-03-java-narrowing.md)
- Specification: [shared](../../specification/typed-generation/rust-checked-narrowing.md)

## Contract and definition of done

Authenticate the selected core i32 TryFrom<i64> implementation, its original
operand and Result<i32, TryFromIntError> identity using checked compiler facts.
Use executable capability bindings; spelling, erased generic names and target
integer syntax grant no authority. General TryInto calls remain unsupported
unless separately specified and proved; their oracle use does not admit them.

## Tests

Multi-crate Rust/C/Java match the independent success/error corpus. Verify
once-only operand traces, source-owned imports/docs/privacy, match/return of
results, typed source/target/callee/variant forgery, and atomic source rejection.
Prove real producer-change cache invalidation and restored success; export
actual packages. Preserve previous outputs/WIP, pass full release/lint and fresh
review before separate commit. No legacy retirement or unchecked narrowing.
