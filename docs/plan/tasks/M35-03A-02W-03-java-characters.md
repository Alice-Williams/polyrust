# M35-03A-02W-03 — Java Unicode scalar target foundation

- Status: complete
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
ordering and reversed-comparison faults. Retain the independent Rust oracle's
surrogate-admission negatives; this target step does not add an integer-to-char
validator and must not claim that Java Int certification rejects surrogates.
Retain exact
source registration, resource accounting and imports. Existing Int expression
admission must not be mislabeled as a general Unicode-validation guarantee.
Full Linux release/lint, old-output/WIP preservation and fresh broad review
precede a separate commit/push. Source admission remains disabled here.

## Independent fixture preparation

Preparation while the C gate runs is limited to test fixtures, native consumer
infrastructure and documentation. No production Java admission/renderer change
is needed for the representation: primitive Int already supports it.
The checkpoint cannot complete before its C dependency and its own gates pass.

Construct 19 typed scalar literals and actual certified identity, immutable-local,
Boolean selection and six comparison functions. Two independently certified
consumers forward through original dependency witnesses and final locals.
Drive every valid scalar and the independent pair corpus through normal and
interpreted Java21 runs, with strict separate compilation of each owner.
Mutate the real producer to narrow to eight/sixteen bits, reverse comparisons
or compare UTF-16 strings; compare every result with independent correct and
fault-specific integer models. Malformed packets and missing-owner/syntax
negative compilation are distinct controls.

Use a separate java_characters_native_test Bazel action and the existing
partition-contract test. Keep the native consumer's policy exception exact;
do not relax import restrictions for generated body templates.
Unit tests must positively certify valid packages and reject counterfeit
type/precedence/conditional annotations, unbound/uninitialized locals, excessive
depth and wrong dependency authority/arity. Check source-byte bounds and composed
call height. Explicitly certify ordinary Int values outside Unicode to document
that source-domain validity belongs to checked Rust Char input, not target Int.

## Validation-driven correction

The first focused gate compiled the fixtures and passed 426 Java unit cases,
but the new comparison-precedence negative exposed a target-profile omission.
The source-owned comparison profile accepted a false Primary annotation even
though arithmetic and conditional profiles required exact precedence. Structural
rendering already parenthesizes binaries, so this did not corrupt output.
The profile now also requires Equality for equality operators and Relational
for ordering operators. Existing valid output remains unchanged; all six
operators have positive and forged-annotation controls. This is consistency
hardening of target evidence, not a new source capability or Unicode validator.
The full gate then identified two older i64 comparison fixtures that had used
Primary as placeholder metadata for their valid controls. Those fixtures now
use actual operator precedence; mixed-width and wrong-result rejection
assertions are unchanged. No production predicate was weakened to accommodate
the stale annotations.

## Review and preservation evidence

The first independent Sol Extra High review of e910e5b88a0988aa17d6cda0f81656b7e89dd895
found no remaining core defects or optional-feature gaps after the comparison
predicate correction. A fresh Sol Extra High reviewer independently reviewed
the complete updated scope at 6338675f5a420d766c3c5984d77920d01f5c9b1c, including
the two corrected older fixtures, and found no defects. No findings were
dismissed. Both reviews explicitly distinguish source-domain evidence from
ordinary Java Int certification and defer execution authority to the full gate.

All 501 baseline compiler-generated file hashes and all 38 unrelated WIP hashes
are unchanged after the production correction. The first full run's only
reported failed target was the Java unit suite with the two stale fixtures;
the corrected full Linux `//... //:release_gate` passes all 1,017 targets
(13 executed, the rest cached), including Rust and Bazel lint. All 427 Java
unit cases pass and the separate full-domain character native matrix passes.
Tested implementation tree: 3c559418dd7d825d45f8cfe6c4ecf15672711f51;
invocation: 972f05c4-8717-48a1-9790-a257b924ee3e. Source character admission
is still disabled; checked compiler integration remains the next checkpoint.
