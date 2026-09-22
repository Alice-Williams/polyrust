# M35-03A-02X-02 — C U32 constant foundation

- Status: planned
- Parent: [02X](M35-03A-02X-character-constants.md)
- Depends on: [constant oracle](M35-03A-02X-01-constant-oracle.md)
- Specification: [C17](../../specification/typed-generation/languages/c/rust-character-constants.md)

## Contract

Extend the closed certified constant inventory to exact U32 values using
existing typed syntax. Keep source character validation outside the target.
Preserve original object/dependency identity, qualifiers and resource bounds.

## Definition of done and tests

Test exact literal/inventory round trips, mismatched types/initializers,
readonly storage, aliases, original owners and forged imports. Native certified
objects/readers agree with the independent scalar corpus and U32 extrema under
strict GCC14/Zig O0/O2, UBSan, standalone headers and separate consumers.
Compiling narrowing/replacement/value faults must be detected. No raw source,
new runtime, broader integer capability or source admission. Full release/lint,
fresh broad review, old output/WIP preservation and separate commit/push.
