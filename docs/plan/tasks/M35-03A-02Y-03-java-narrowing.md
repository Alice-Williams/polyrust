# M35-03A-02Y-03 — Java checked narrowing

- Status: planned
- Parent: [02Y](M35-03A-02Y-checked-narrowing.md)
- Depends on: [C foundation](M35-03A-02Y-02-c-narrowing.md)
- Specification: [Java21](../../specification/typed-generation/languages/java/rust-checked-narrowing.md)

## Contract and definition of done

Compare an original Long operand against both inclusive bounds, then construct
the certified success or error variant. A Java narrowing cast or thrown
ArithmeticException alone does not implement Rust's fallible result.

## Tests

Strict separately compiled Java21 consumers agree with every oracle outcome in
normal and interpreted execution. Native range/cast/result faults must fail;
typed wrong-owner/variant/payload substitutions reject. Preserve once-only
evaluation, private implementation boundaries, dependency certificates and
resource limits. Full Linux release/lint and fresh review precede commit.
No Java runtime helper, source admission or third-party dependency.
