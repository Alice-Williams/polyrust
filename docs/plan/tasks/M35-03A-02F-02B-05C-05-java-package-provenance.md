# M35-03A-02F-02B-05C-05 — Explicit Java source-package provenance

- Status: planned
- Parent: [constant alias closure](M35-03A-02F-02B-05C-constant-alias-closure.md)
- Depends on: M35-03A-02F-02B-05C-04
- Specification: [Java constant re-exports](../../specification/typed-generation/languages/java/rust-constant-reexports.md)

## Contract

Retain the selected Rust crate's finite export graph and module documentation in
its typed Java facade, even when it owns no source methods or fields. This is
descriptive unresolved metadata, not compiler acceptance or dependency authority.
Every existing source declaration must agree with it. No fake declaration,
constant field, method, inheritance or runtime may carry the provenance.

This checkpoint supplies provenance and documentation only. The later certified
foreign-export checkpoint must authorize alias-only dependency APIs; production
rustc foreign-export admission and publication schemas stay closed meanwhile.

## Implementation

1. Extend the shared checked documentation API with an explicit export-graph
   input alongside declaration origins. Reuse its existing graph, ancestry,
   equality and resource checks; do not fabricate a RustSourceOrigin to make an
   empty graph visible. Keep the existing declaration-derived entry point
   compatible. Charge distinct immutable allocations before comparisons.
2. Add JavaSourcePackage with private Arc<RustCrateExports> storage, a descriptive
   constructor and a read-only exports accessor. Retain an optional value on the
   owning JavaFileItem::Type. None preserves existing legacy callers.
3. Verify that explicit provenance belongs to exactly one canonical public
   Generated.java compilation unit in JavaPackage::RustCrate(root.crate_id),
   main placement, and a registered public final PackageEntryPoint class. Reject
   duplicate, misplaced, foreign-namespace or conflicting metadata. Reconcile all
   source origins, including record fields, against the explicit graph.
4. Centralize documentation metadata selection: explicit graph plus origins if
   present; declaration-derived compatibility only when absent. Use it for both
   source-registration verification and documentation lowering. Preserve root
   docs and explanatory module docs without inventing declaration documentation.
5. Retain the exact metadata through linking and independent linked verification.
   A coupled alteration to resolved metadata and rendered docs must not bypass
   comparison with the original checked package. Resource accounting includes
   explicit graph/ancestry/name/document payloads, including empty facades.
6. Make production Rust-to-Java assembly always attach the selected source graph.
   Preserve existing public API acceptance and unsupported-export diagnostics.
   Do not loosen JavaDependencyApi's required certified public inventory here.

Keep files focused; introduce dedicated source_package and documentation test
modules rather than expanding existing large verifier files. Mechanical additions
of absent metadata to old AST constructors must not change their behavior.

## Definition of done and tests

- Positive: explicit nonempty function/constant/record packages; declaration-free
  facade with root and nested module documentation; local module-alias cycles;
  equal graphs in distinct Arc allocations; unchanged legacy construction.
- Negative: wrong package/root, role, path or placement; wrong/unregistered facade;
  duplicate metadata owners; conflicting graph or documentation; body-local and
  foreign declaration origins; incomplete/unreachable graphs and bad ancestry.
- Shared checker tests cover explicit graphs without origins, explicit/inferred
  conflicts, exact resource boundaries and allocation deduplication. Do not
  silently accept an over-budget explicit graph because it has no declarations.
- Independently mutate linked provenance and projected documentation, separately
  and together; certification must reject mismatches with the original package.
- Compile the actual generated empty documented facade using the pinned Java 21
  toolchain and strict checks. Assert no synthetic source field/method and no
  runtime file. Export the generated artifact locally, not into Git.
- Existing source dependency APIs, docs, records, constants, legacy portable
  outputs and Rust-source integration gates remain enabled.
- Full Linux Bazel release, Rust/Bazel lint and native gates pass on an exact scoped
  tree. Fresh Sol Extra High review loops leave no unaddressed core errors.
- Record evidence, commit this task ID and push. Do not mark Java foreign exports,
  compiler publication, parent parity or legacy runtime retirement complete.
