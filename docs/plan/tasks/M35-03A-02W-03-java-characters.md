# M35-03A-02W-03 — Java Unicode scalar target foundation

- Status: planned
- Depends on: [C foundation](M35-03A-02W-02-c-characters.md)
- Specification: [Java21](../../specification/typed-generation/languages/java/rust-character-values.md)

## Contract

Represent a checked scalar as primitive int, never Java char, Character,
String, surrogate pair storage or a custom runtime wrapper. Keep source Char
identity separate from I32 even though their target primitive types coincide.

## Definition of done and tests

Certify real target literal/identity/conditional/comparison packages and verify
the complete scalar corpus plus comparison pairs with strict Java21 separate
compilation, normal execution and -Xint. Detect compiling truncation, UTF-16
ordering, surrogate acceptance and reversed-comparison faults. Retain exact
source registration, resource accounting and imports. Existing Int expression
admission must not be mislabeled as a general Unicode-validation guarantee.
Full Linux release/lint, old-output/WIP preservation and fresh broad review
precede a separate commit/push. Source admission remains disabled here.
