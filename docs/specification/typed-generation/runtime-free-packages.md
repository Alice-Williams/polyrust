# Ordinary generated packages without a custom runtime

- Status: accepted migration contract; feature parity and legacy removal incomplete
- Plan: [M35-03A](../../plan/tasks/M35-03A-runtime-free-parity.md)

## Required outcome

The Rust-source C and Java paths emit ordinary libraries/packages. They must
not require copied handwritten target-language runtime templates, a mandatory
Polyrust support library, or a privileged generated Runtime class. Changing a
runtime filename, moving it to another directory, or rebuilding the same
mandatory runtime through typed AST nodes does not satisfy this requirement.

Necessary implementation code is not forbidden: allocation checks, cleanup,
data representations, Unicode processing and error branches may be required
to preserve the source program. Produce them through checked executable
capability mappings into ordinary target declarations and statements, using
the existing target AST, dependency linker, certification and renderer.
Imports derive from referenced declarations or authenticated standard-library
symbols. No raw target-source/import escape hatch is introduced.

Prefer equivalent native operations and standard-library facilities. Where
these differ semantically, lower an explicit checked implementation rather
than substituting an approximate operation. Any reusable source dependency
is an ordinary admitted Rust crate, translated through the same pipeline;
it does not receive a magic runtime identity or bypass compiler checks.

## Feature-preserving migration

The [coverage inventory](runtime-parity-inventory.md) records the baseline,
per-target limits, evidence anchors and primary task for each legacy family.

The user requires functionality missing from the new system to be implemented,
not removed along with the old runtime. Therefore retain existing portable-IR
C/Java entry points, examples and tests until their replacement evidence passes.
Their presence during migration is an explicitly tracked exception, not the
desired architecture. Do not disable a failing regression to claim parity.

Inventory actual implementation and successful tests, not just advertised
capability names. A partial C ABI or a capability enum is not full executable
support. Record support per target and per operation/shape. In particular,
legacy C has narrower callable-container and enum support than legacy Java.

The current Rust-source output path admits i32/bool, closed scalar-field
records/shared borrows, structured branches/local bindings, direct calls,
compiler-authenticated crate/API boundaries and documentation attributes.
Owned-source evidence is separate from target heap output; completed compiler
observations do not establish a supported C/Java allocation mapping.

Parity means equivalent behavior and declared public/private boundaries for
the migrated source corpus, not identical text or the old Runtime ABI.
Portable-model semantics that differ from ordinary Rust must be represented
explicitly in the replacement Rust fixture/library. Do not silently equate
portable deep equality, checked-operation results, Unicode scalar indexing or
floating-point edge policies with superficially similar native operations.

## C specification

Emit ordinary .c implementation files and .h public declarations according to
the established crate boundary policy. No automatic runtime.c/runtime.h,
poly_allocator wrapper ABI, or unused core helper group is permitted in the
Rust-source output path. Use authenticated C standard-library calls where
appropriate, with typed allocation identity, null checks and evidenced cleanup.
Never copy Rust object layout into C or render MIR edges as gotos.

Keep the C registry, declaration/import provenance, ABI rules, ownership checks,
target certification and structural renderer. The legacy string generator and
embedded runtime templates may be deleted only after every supported feature
and affected native consumer has replacement evidence.

## Java specification

Emit ordinary classes, records, enums, interfaces and methods in their mapped
packages. No reserved Runtime.java location, RuntimeMembers composition path,
fixed Runtime helper catalogue or privileged Runtime shell should be necessary
in the completed architecture. Use standard JDK symbols through typed mappings
where they preserve the source contract; generate ordinary declared types for
source types that have no equivalent JDK representation.

Keep generated symbol identities, interfaces/composition, type checking,
resource checks, imports, certification and structural rendering. Preserve
foreign/public-boundary validation when required by the mapped contract.
Replacing custom result/error/byte/list support requires its semantic tests;
renaming its class is not a migration.

## Proof and deletion rule

Every replacement needs pinned rustc admission, typed executable bindings,
negative unsupported/invalid-source cases, target certification/privacy tests,
ordinary consumer tests and native Rust/C/Java differential cases. Include
boundary values, failure behavior, ownership, aliasing, Unicode and floating
point edges appropriate to the feature. Add counterexamples to new invariants.

Generated-bundle tests must reject extra support files and undeclared dependency
owners, and retain all expected crate members. Fixed-corpus include/import and
manifest guards complement typed binding checks and native compilation/linking;
file inventories alone cannot prove the absence of arbitrary code dependencies.
Native and structural tests remain mandatory. Keep old and new tests
side by side until the feature-specific cutover is proven.

Only then remove obsolete generator/runtime code, special AST/catalogue/render
paths, old build actions/dependencies, stale examples and obsolete policy
exceptions. Preserve useful independent tests by porting them. Historical
completion records remain history, with current-status links rather than
rewritten claims. Never remove unrelated unfinished M34 work as cleanup.
