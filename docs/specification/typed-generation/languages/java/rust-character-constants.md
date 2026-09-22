# Rust character constants in Java21

- Status: target proof complete; checked source admission planned
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

The target proof covers all 4,127 independent character-constant inputs plus
six deliberately non-character Int controls, including both signed extrema.
Each has a field, a local reader and an imported reader selected from an
original-owner alias facade. Certify batches of at most 64 values, compile the
three owners and client separately, and check both JVM execution modes. Actual
producer mutations exercise byte/code-unit narrowing, replacement and changed
values after dependent recompilation. Field shape, exact certificate identity,
long-name/read reservations, JVM name capacity and readonly assignment controls
must remain active. These tests add no new production capability.
