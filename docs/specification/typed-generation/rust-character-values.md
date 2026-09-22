# Checked Rust Unicode scalar values

- Status: independent oracle, C/Java foundations and checked source integration complete
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

## Original type metadata boundary

Use the existing private LiteralInput and exhaustive LiteralValue::Char(char)
mapping under Supports<LiteralValues>. Character admission is not a string
capability flag or a duplicate generic literal subsystem.

RustSourceTypes is bounded descriptive data: an exact crate-root identity,
declaration-keyed scalar function parameters/results and record-field facts.
Each RustFieldTypes retains both its scalar kind and its original enclosing
record declaration. Both target inventories must authenticate that field-owner
join, including two distinct records with identical scalar field shapes.
Its scalar enum distinguishes Char, I32, I64, Bool and F64; result kinds
distinguish Unit from scalar values. It is not a source-analysis certificate.
Read facts from canonical rustc signatures and field declarations, then
authenticate them against original compiler queries at package attachment.
At foreign calls, additionally join the producer declaration's retained source
signature to the consumer's original rustc signature before target import.

Reconcile these facts with the complete certified target function/field
inventory. Target-only constructors may describe either Char or I32 over Java
Int; only the compiler adapter authenticates which source type it actually was.
No serialized manifest or descriptive facts object grants callable authority.
Serialize a character-aware schema only when original facts contain Char;
ordinary pre-character packages retain their exact existing bytes.

The current public package contract supports local declaration aliases and
named dependency imports, not foreign public function/module re-exports.
The selected-entry harness remains exactly fn(i32) -> i32, even on Java.
Direct character references, character constants, casts and methods remain
explicit rejection boundaries for this increment.
