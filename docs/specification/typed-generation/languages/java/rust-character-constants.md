# Rust character constants in Java21

- Status: target proof and checked source integration complete
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

The compiler-authenticated RustConstantValue map is reconciled against every
owned certified field. Char(c) maps to I32(u32::from(c) as i32), retaining the
separate Char source variant and TypePlan::Char. JavaDependencyConstant exposes
the exact retained source value from its immutable owning certificate, not a
value inferred from Java Int syntax. Import lowering compares it with the
consumer's compiler-evaluated source value; same-valued Char/I32 substitution
must fail even though target field types and integer literals match.
Use owner schema 8 when character-aware source facts include constants; retain
schema 7 for existing character-value owners with no constant facts. Other
schema selections and output bytes remain unchanged.
