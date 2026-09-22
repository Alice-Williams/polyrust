# Checked Rust Unicode scalar values

- Status: independent oracle and C/Java foundations complete; source admission pending
- Plan: [02W](../../plan/tasks/M35-03A-02W-character-values.md)
- Targets: [C17](languages/c/rust-character-values.md), [Java21](languages/java/rust-character-values.md)

## Semantic domain

Rust char denotes every scalar from 0 through 0x10FFFF except 0xD800..0xDFFF.
NUL, unassigned, private-use and noncharacter scalar values remain valid.
Its numeric identity and ordinal comparisons must be preserved. This is not
a grapheme, normalized text, UTF-8 byte or UTF-16 code unit. The fixed domain
does not depend on a Unicode property database. See the authoritative
[Rust char validity and layout contract](https://doc.rust-lang.org/std/primitive.char.html#validity-and-layout).

Use Rust's own char type as the checked value witness. A distinct private
compiler input retains the original expression/type and source Char identity.
Do not accept a raw u32 as an already checked scalar. Any future dynamic
integer-to-character conversion must validate the domain through its own
capability; it cannot reuse a literal constructor to bypass admission.

## Bounded source and lowering

Initially support character literals, immutable places, scalar function
parameters/results, original direct calls, conditional selection and six
same-type comparisons. Preserve original operand order and once-only
evaluation. The source type plan distinguishes Char from I32/U32 even when
a target representation overlaps. No implicit source integer coercion is added.

Character constants and aliases, casts, inherent methods, Unicode properties,
text/byte encodings, normalization, iteration, patterns and heap ownership are
separate increments. Existing scalar-field shapes must retain behavior when
admitted; otherwise diagnose unsupported shapes before publication.

C uses its existing U32 model/uint32_t; Java uses primitive Int. Both can
represent the entire scalar domain without narrowing. Generated foreign APIs
must describe Char and its valid-scalar input precondition in authenticated
source/type metadata. An arbitrary foreign integer is not a valid original
Rust char input. The guarantee concerns checked Rust inputs and well-formed
foreign callers; it does not claim arbitrary Java ints/C uint32_t are chars.
No integer-to-char conversion, runtime validation API or wrapper is implied.

## Certification and proof

Target AST verification/certification remains mandatory. Source provenance,
signatures, type joins, function identity, original APIs/docs/privacy and
dependencies cannot be replaced by rendered text or name matching.
Use structural literals/operators and ordinary package declarations; no raw
target fragments, copied runtime, boxing, encoding helper or third-party crate.

The independent oracle precedes separately gated C/Java target foundations,
then compiler integration. It covers every scalar and surrogate, out-of-range
u32 boundaries, exact transport and all six comparisons. Real compiling
byte/UTF-16 truncation, BMP-only filtering, surrogate admission, reversed
comparisons and UTF-16 lexicographic ordering must be distinguishable.
Native source/target profiles, typed and compile-negative controls, atomic
publication, resource limits and actual exported examples complete the proof.
