# Rust character constants in Java21

- Status: planned
- Contract: [shared](../../rust-character-constants.md)

Use existing primitive Int literals and public static final int fields.
JavaScalarConstantValue::I32 remains an honest target inventory value; do not
add a fake source-Char variant reconstructed from indistinguishable Java syntax.
Do not use Java char, Character, String, code-unit pairs or runtime conversion.

Retain original Rust Char facts separately from target inventory. The compiler
adapter authenticates the declaration/type/value; the dependency API reconciles
facts with its exact certified field. A consumer checks original Char identity
even when a forged I32 description has identical Java Int type and value.
Constant-only owners and aliases retain these facts and exact original owners.

Prove existing Int constant storage/read/import certification at Unicode scalar
boundaries and deterministic interior values. Target-only negative/arbitrary
Int constants remain legal Java and do not become source character witnesses.
Use strict separate Java21 compilation and normal/-Xint observations of fields,
local readers and imported readers. Recompile every dependent after producer
mutations so constant inlining cannot hide a bad mapping. No renderer changes
or production source admission are implied by this target proof.
