# M35-03A-02U-03 — Java finite-constant foundation

- Status: complete
- Parent: [02U](M35-03A-02U-finite-f64-constants.md)
- Depends on: [C foundation](M35-03A-02U-02-c-constants.md)
- Specification: [Java21](../../specification/typed-generation/languages/java/rust-finite-f64-constants.md)

## Contract

Extend exact public static final source-constant certification to primitive
Double and finite F64 literals, preserving owner/registration/provenance and
resource checks. No source admission or renderer/helper changes.

## Definition of done and tests

Private-reader matrices and public certificate tests reject wrong primitive,
boxed type, initializer, visibility/modifiers, resolved owner and import
authority. Separately compiled rendered producers/consumers agree with the
independent bit oracle under strict Java21 normal and interpreted execution.
Compiling zero-sign, f32-rounding and wrong-value controls are detected.
Source bounds and old constant tests pass. Full gate and fresh review pass;
preserve old output/WIP and commit/push this foundation separately.

## Implementation and proof scope

The dependency inventory adds exactly primitive Double plus finite F64 literal
initialization. Existing declaration/source/owner/modifier checks are unchanged.
No source admission, renderer, runtime or dependency is added.

The private inventory matrix checks seven primitive annotations against nine
literal forms, including both zeros. Additional controls cover boxed types,
missing/duplicate modifiers and unresolved/incorrect owners. Public tests cover
wrong types, initializers, identity, visibility/finality/staticness, signed-zero
and lookalike authority, actual imported readers and repeated long-name bounds.

Native proof uses all 24,576 independent oracle observations as certified fields
in 256-field packages. Producers and external clients compile separately under
strict Java21 and run normally and with -Xint. Eight boundary controls also pass
through generated owned and imported readers. Three compiling value mutations
are observed only after recompiling all dependents against the changed producer,
so javac constant inlining cannot hide a changed value.

The expensive matrix has its own cached Bazel partition. The historical public
test target remains the complete suite; the compiled unit target, Clippy,
Rustfmt and Cargo-oracle inputs are named explicitly. A partition contract
detects omitted, renamed and overlapping cases.

## Verification evidence

All 994 Linux Bazel release/lint targets pass (6 executed, 988 cached),
invocation `42b95de9-518d-40f4-9109-ea9eafca46b9`. The preceding run executed
116 tests: every selected test passed, but a stale Rustfmt reference to the
new suite label caused an analysis failure. Correcting it to the compiled unit
target restored the complete gate; no failed test was disabled.

The independently cached native matrix passes in 387.9 seconds and the
remaining Java unit suite in 252.7 seconds. All 423 prior generated file hashes
and 38 unrelated ownership/conditional-source WIP hashes remain unchanged.
Independent Sol Extra High review of tree
`da02a310ea5eccc985c573addd065e66d99c5b5c` against `3a13c2a` is clean.
A fresh post-fix Sol Extra High reviewer independently audited the same exact
15-file tree and also found no actionable core defects. Final documentation
closure is gated once more before commit. This completes the Java target
foundation only; checked compiler-source admission remains the next checkpoint.
