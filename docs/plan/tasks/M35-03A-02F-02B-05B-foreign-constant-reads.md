# M35-03A-02F-02B-05B — Authenticate foreign compiler constant reads

- Status: planned
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
