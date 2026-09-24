# M35-03A-05A-04D — Shared type-owner graph and atomic bundles

- Status: planned
- Parent: [compiler results](M35-03A-05A-04-compiler-results.md)
- Depends on: [Java owner](M35-03A-05A-04C-java-type-owner.md)
- Specification: [canonical owners](../../specification/typed-generation/rust-canonical-type-owners.md)

## Contract

Intern canonical type owners on authenticated encounter, before lowering that
use. Reconcile complete facts on reuse and supply one original authority to all
producers. Freeze and validate the complete graph before any publication.
Extend both target bundle paths to retain explicit type-owner nodes and exact
type/member dependencies, including uses that have no imported function call.

## Implementation and definition of done

1. Reconcile compiler instance facts before target construction; graph descriptors
   and serialized manifests never substitute for checked compiler/package handles.
2. Add explicit type-owner entries to bounded, versioned manifests and complete
   source/metadata reservations. Preserve old scalar-only serialized output.
3. Verify original owner closure and every nominal/member witness before any
   output is published; forbid missing/duplicate/competing owners and cycles.
4. Run fan-in and diamond graphs with permuted traversal and unrelated consumers.
   Compare complete C/Java output bytes, one shared nominal identity and native
   cross-producer calls.
5. Corrupt owner/profile/instance/closure/manifest facts and exceed each relevant
   capacity by one; rejection must leave no partial published package, including
   a late producer conflict after earlier producers successfully lowered.
6. Prove producer-change invalidation and restoration; export actual examples.
   Full Linux Bazel release/lint, independent review and scoped commit/push.

Use explicit target-only graph probes until the next task opens checked Result
HIR operations. Do not expose a production source-admission shortcut for tests.
Recalculate Java publication depth/directory limits from the expanded tree: the
new owner has eight package directories, exceeding the current depth of seven.
