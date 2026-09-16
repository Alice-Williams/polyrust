# M35-03A-02F-02B-05C-05 — Explicit Java source-package provenance

- Status: complete
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

## Implementation and focused proof

Implemented the explicit graph entry point on CheckedRustDocumentation and
JavaSourcePackage metadata on the typed facade. Source registration and module
documentation share one selector; production compiler assembly always attaches
the compiler export graph. Existing portable constructors retain None, and their
module mapping contract requires None explicitly rather than silently ignoring
unexpected source metadata.

Focused Linux Bazel on tree `c1b47deb443a4c914379dacfb2940d7643516cbf` passed
**3/3 targets**: shared codegen, Java unit/native and Java typed compile-fail;
137.794 seconds, invocation `a047726d-c161-437b-81b3-cf81e3a3dbd3`.
An earlier constructor-migration build caught one exhaustive match and three
non-file module-input initializers; corrected before this passing tree.

Final implementation tree `7b5b0e0a7da40cf02872f783d94c82e9d6678e78` adds
fake-facade, field-only graph conflict and 100,001-binding resource regressions.
The compiler-backed Java AST probe now requires explicit package metadata and
compares its root directly with rustc's crate definition identity and every owned
source graph. Java's unit/native suite passed **336 cases** on this tree.

An independent Sol Extra High review reported **no core findings**, covering
coherence, bounded allocation traversal, facade identity, field origins,
documentation, reconstruction, production registration and compatibility.
The linked-mutation test compares the exact resolved-item equality used by the
shared verifier: safe callers cannot mutate a LinkedFile slot. Metadata-only,
docs-only and jointly substituted copies differ from the independently
reconstructed original item. Existing shared tests exercise real linked-item
tampering; no unsafe mutation API was introduced solely for tests.

The actual pinned Java 21 native test compiles the generated empty documented
facade and independent reflection consumer with --release 21 -Xlint:all -Werror.
Reflection verifies zero fields, zero methods and exactly one private constructor.
Actual generated sources are exported locally to
`generated/examples/java-source-package-7b5b0e/{Generated.java,Consumer.java}`;
they are ignored, not committed. The facade contains only its package declaration,
root/nested module docs, final class and private constructor, with no runtime.

An empty facade still cannot become JavaDependencyApi without the subsequent
certified public binding inventory. Foreign-export admission, bundle schemas and
legacy runtime removal remain outside this checkpoint.

## Complete release gate

Exact implementation tree `7b5b0e0a7da40cf02872f783d94c82e9d6678e78`
passed Linux dev-container command
`bazelisk --output_user_root=/tmp/polyrust-m34a10w-bazel --batch test //... //:release_gate --noshow_progress --noverbose_failures --test_output=errors --test_summary=terse --keep_going`:
**738/738 tests**, 1,093 targets, 133 executed and 605 cached,
807.780 seconds, invocation `370237b3-d19e-49d0-8ac3-79388e1b8080`.
This includes Rust/Bazel lint, the compiler-backed Java source AST assertions,
generated native-language checks and all existing capacity cases.
The shared suite passed 142 cases, Java 336 and C 765 plus its five separately
scheduled expensive tests. No test was disabled or weakened. All 20 preserved
ownership-work hashes remained unchanged.
