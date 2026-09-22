# Rust Unicode scalar values in C17

- Status: target foundation complete; source admission disabled
- Contract: [shared](../../rust-character-values.md)

Use CScalarType::U32 and typed unsigned literals for exact scalar numbers.
The existing catalogue supplies uint32_t and symbol-derived stdint.h.
Do not use PlainChar, signed/unsigned byte, wchar_t, multicharacter tokens or
an encoding-dependent character literal. Keep the pinned C execution profile.

Identity transport, locals, conditionals and same-U32 comparisons use existing
typed expression/statement nodes. U32 scalar calls need body-derived effect
evidence, not a caller-supplied purity flag. Numeric and contextual checks
still reject invalid arithmetic, missing bodies and unproved storage.
No integer/character conversion or arithmetic source capability is introduced.

The target profile admits U32 in function signatures, scalar record fields
and same-U32 conditional branches. Dependency API inventories retain U32 as
an exact signature type. Existing numeric proofs still govern conversions and
arithmetic; U64 remains excluded from public signatures. The renderer and
standard-library catalogue do not change. Target certification proves C
validity/safety, not the narrower Unicode domain: raw U32 values outside that
domain remain valid target integers, never checked source characters.

The compiler source type remains Char, with Rust char as the value witness.
Public metadata distinguishes the Unicode-scalar domain from generic U32;
foreign callers must satisfy that original domain. No raw integer is promoted
to a checked source char because its target C type happens to match.

Prove exact full-domain transport and ordinal comparison using certified
packages, GCC14/Zig O0/O2, strict separate compilation/standalone headers and
GCC UBSan. Mutate actual target nodes/artifacts to narrow or misorder values;
compiling faults must disagree with independent truth. Prove inferred imports,
resource accounting and unchanged old bytes before enabling compiler input.
