# M35-03A-02F-02B-05B — Authenticate foreign compiler constant reads

- Status: complete
- Parent: [multi-crate constants](M35-03A-02F-02B-05-constant-bundles.md)
- Depends on: M35-03A-02F-02B-05A

## Contract

Discover foreign public module constant references from checked HIR for reachable
function bodies within the existing traversal/depth/count limits. Preserve exact
DefId identities and normalized compiler values; do not discover dependencies
from emitted text or fold public foreign reads into literals.

Use distinct typed function/constant lookup results, not name strings or a
catch-all ID branch. Join each compiler owner/declaration/type/value to an opaque
CDependencyConstant or JavaDependencyConstant from the actual checked producer.
Import through C registry / Java dependency scope before lowering bodies. Freeze
all Java function/value bindings in one scope and retain exact owner authority.

Add an executable PublicConstantImports capability with a private compiler input
for a resolved foreign ordinary scalar constant. Registration uses a dedicated
package-state mapping and consumes an original producer witness selected by the
checked graph, never a raw target name. The consuming builder requires this slot
independently of owned declarations and reads; add all seven negative binding
controls for both backends.

PublicConstantReads maps the authenticated input to either a registered owned
reference or an authenticated imported reference. Keep package and Reader state
typed and preserve state across functions. Extend C bundle metadata with a
separate constant-import inventory; descriptions cannot reconstruct authority.

## Definition of done and tests

- Real constants-only dependency, mixed consumer, private same-spelled constants,
  local import aliases and transitive dependencies work in Rust/C/Java.
- Actual AST probes prove imported value references, precise producer identity,
  type/value/path and owned/foreign separation; production bytes match probes.
- Wrong/stale owner, declaration, scalar type or value; missing producers; replaced
  independent certificates; incomplete used/retained-owner graphs and forged
  metadata fail before publication, preserving absent/existing destinations.
- Separate native consumers exercise exact boundary values, multiple calls,
  same-authority diamonds and both bools. Producer value mutation invalidates the
  dependent Bazel generation and fails the original independent native oracle.
- Constants do not add call-stack frames. Existing call limits, source budgets,
  file counts, manifest limits, standalone folding and source rejection tests pass.
- Keep unsupported public foreign re-exports diagnosed until 05C.
- Full Linux Bazel/release/lint gate, fresh independent review, real ignored
  examples, evaluated findings and scoped commit/push precede completion.

## Bounded implementation order

1. Define the private foreign declaration input and bounded checked-HIR dependency
   inventory. Discover both direct calls and module constant uses without parsing
   emitted code or traversing unbounded body graphs.
2. Add distinct typed callable/constant producer lookups and executable import
   mapping slots. Register all imported handles before lowering function bodies;
   freeze Java scope only after both function and constant registrations.
3. Preserve owned and imported maps through package/Reader transitions, retaining
   compiler DefId/type/value and original certificate identity. Do not make an
   imported field appear in the owned declaration list.
4. Extend explicit C/Java bundle reference metadata and exact owner preflight;
   version any newly introduced structural fields rather than silently changing
   the meaning of the already published schemas.
5. Add source/probe/native, forged/stale-producer, mutation/rebuild and atomic
   publication proof. Replace foreign-read rejection only for authenticated
   declared producer graphs; standard-library module constants without a
   translated producer stay rejected in public-package mode.

## Evidence and review decisions

- Frozen tree fb715def4f2b2fa067847a647b9111db6b7dc0a5 passed all 18 initial
  focused tests: four-crate native Rust/C GCC+Zig O0/O2/Java21 equivalence,
  fixture and Java bundle lints/tests, and all 14 new import compile-negative
  controls. The native fixture's missing final newline was fixed; strict
  warning flags were not relaxed.
- Frozen tree cf37564ff3753210d202349ed68d2461ffa8428f passed native and atomic
  publication tests, including actual AST reference probes, byte equality to
  production, wrong crate/declaration/type/value, independent recertification,
  missing/stale metadata, refreshed metadata propagation, original-owner graph
  reconciliation and preserved absent/existing output. Replaced-owner tests
  use the single-import intermediate crate so failure specifically exercises
  retained original-authority checks rather than an earlier duplicate import.
- First independent Sol Extra High review found no production correctness
  defect. Its remaining evidence requests are accepted: source/AST probes,
  mutations, graph closure, cache invalidation and limit tests must pass before
  completion. New Java source probes assert call height 1 for direct scalar
  reads and 2 only for actual intermediate calls. Native inventory asserts
  no synthesized callable. The existing C constant-consumer composition test
  explicitly proves zero producer frames and unchanged callable-frame costs.
- Optional review suggestion: include unused Java registered values in the
  reference metadata. Not adopted for this source-integration checkpoint:
  constant_imports is explicitly the used-reference inventory, whereas
  dependencies retains every registered/transitive owner and validates exact
  authority even when unused. Production registers exactly discovered uses.
  Rejecting valid unused bindings would contradict the existing consumer
  contract; changing this descriptive scope is not needed for source safety.
  The language specification now states both inventories' distinct meanings.
- The manual Linux Bazel integration gate in
  test/constant_import_cache_proof.py passed against archived tree cf37564:
  changing the real producer initializer from 62 to 17 changed all four rmeta
  artifacts and both generated bundles; the unchanged independent native oracle
  failed. Restoring source restored every output hash and reused a cached passing
  test result. Only a temporary extracted archive was mutated.
- Real runtime-free examples were exported to the ignored host directory
  generated/examples/foreign-constants-cf37564 (four C header/source pairs, four
  Java facades, manifests and the Rust source fixtures).
- Discovery and target resource limits are independent. The initial attempt to
  publish 4096 imports correctly hit C's smaller AST node budget; it was not
  evidence of a discovery-boundary defect. The focused compiler probe now calls
  both production inventory walkers without rendering/certifying a target, and
  compares their exact inventories and over-limit/depth/traversal diagnostics.
  No production limit or strict warning flag was weakened.
- The discovery probe passed exact 4096/4097 distinct-constant boundaries and
  excessive depth/traversal rejection in both production walkers.
- Full Linux dev-container command: `bazelisk
  --output_user_root=/tmp/polyrust-m34a10w-bazel --batch test //... //:release_gate
  --noshow_progress --noverbose_failures --test_output=errors
  --test_summary=terse --keep_going`.
- Frozen tree 8eb2fa9f8062f02477b8bb45b0cd84a12cdbb085: 1,089 targets,
  736/736 tests passed, 36 executed and 700 cached, 162.721 seconds.
  Invocation: 32e22c65-191f-4c14-b7d2-971343cface3.
- The prior full run passed 734 tests and exposed only two build-hygiene issues:
  missing Bazel macro argument docs and an unused verifier in the standalone
  manifest probe. Both were fixed; verify_constants delegates to the complete
  reconstruction path rather than suppressing a production warning.
- A fresh independent Sol Extra High reviewer found no core correctness defect
  or missing required feature after reviewing authority, compiler joins,
  inventories, scope freezing, metadata, closure and the complete proof suite.
  Its two build-hygiene observations are fixed and the full gate above is green.
- Twenty pre-existing unrelated file hashes were verified unchanged. Generated
  output, unrelated C ownership changes and next-step plans are excluded from
  this milestone's scoped checkpoint. Completion documentation is rechecked in
  the final exact-tree Linux gate before commit/push.

Foreign public re-exports remain rejected until child05C. This completes
authenticated body reads only, not all Rust constants or legacy runtime removal.
