# M35-03A-02X-03 — Java scalar constant foundation

- Status: planned
- Parent: [02X](M35-03A-02X-character-constants.md)
- Depends on: [C foundation](M35-03A-02X-02-c-constants.md)
- Specification: [Java21](../../specification/typed-generation/languages/java/rust-character-constants.md)

## Contract

Prove existing typed Int constant declarations/readers/imports across the
character constant corpus. Do not pretend target syntax distinguishes Rust
Char from I32. Keep this as focused reusable proof, with production changes
only if an actual target certification gap is demonstrated.

## Definition of done and tests

Certified fields, local readers and original-owner imported readers produce
the independent scalar values with strict separate Java21 compilation and
normal/-Xint execution. Compiling narrowing/replacement/value mutations are
detected after all dependents are recompiled. Exact type/value/owner/import and
resource negatives pass; negative Int controls remain target-valid. No Java
char/boxing/runtime or source admission. Full release/lint, fresh broad review,
preservation checks and separate commit/push are required.
