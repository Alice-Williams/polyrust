# Rust character constants in C17

- Status: target foundation and checked source integration complete
- Contract: [shared](../../rust-character-constants.md)

Use ordinary const uint32_t objects with existing typed U32 literal nodes.
Extend CScalarConstantValue with target U32(u32), not source Char: valid C
integers include values outside the Unicode scalar domain. Reconstruct exact
inventory membership from certified U32 syntax and retain object qualifiers,
linkage, definition/header ownership and original dependency certificates.

Public declarations/aliases and reads use normal object references; private
or local evaluated reads follow the existing constant mapping. Infer stdint.h
through typed symbols. Do not use C character tokens, strings, casts, a custom
runtime or textual constant folding. Other unsigned widths remain separate.

Prove exact scalar-boundary/interior values and target-only U32 extrema,
including values not admissible as source char. Reject wrong annotations,
qualifiers, initializers, owners and declarations. Compile headers standalone
and producer/consumer objects separately with GCC14/Zig O0/O2 and GCC UBSan.
Native mutation controls must detect narrowed/replaced values and stale readers.
The target foundation alone must not enable Rust character constants.

The compiler manifest reconciles the original RustConstantValue inventory with
every certified public object by declaration, exact value and target type.
Char(c) maps to U32(u32::from(c)); I32 remains a distinct variant. The compiler
graph checks the consumer's original evaluated constant against the producer's
retained source value before the executable import mapping registers an object.
Reserve constant metadata bytes explicitly; serialized descriptions never grant
object authority. Constant-only owners trigger character-aware publication.
Use owner schema 11 when character-aware source facts include constants; retain
schema 10 for existing character-value owners with no constant facts. Other
schema selections and output bytes remain unchanged.
