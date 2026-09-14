# M35-01D-04C-03 — Foreign typed bindings and package publication

- Status: complete
- Depends on: [M35-01D-04C-02](M35-01D-04C-02-declared-crate-driver.md)
- Parent: [M35-01D-04C](M35-01D-04C-rust-crate-dependency-driver.md)

## Definition of done

Implement the following ordered, independently reviewable tasks:

1. [04C-03-01 — Compiler-to-certificate foreign bindings](M35-01D-04C-03-01-compiler-foreign-bindings.md).
2. [04C-03-02 — Imported inventories and atomic bundle publication](M35-01D-04C-03-02-bundle-publication.md).

The first task joins compiler evidence to existing typed imports in memory. It
does not publish a partial bundle. The second adds exact descriptive inventories
and whole-graph output. Native differential proof remains the separate 04D task.

Resolve compiler direct-call targets as local definitions or exact checked
foreign definitions. Reuse scalar signature capability mappings for foreign
DefIds; join signatures and defining identities to CDependencyApi witnesses.
Register those witnesses through the existing typed import API before lowering
consumer bodies. Never visit a foreign body as an owned consumer definition.

Keep compiler aliases, owned definition inventories and imported references
distinct and verify exact membership in both directions. Extend descriptive
manifests without treating their strings as authority. Retain the scalar public
header policy; unsupported foreign public re-exports diagnose unless an explicit
typed representation and proof are added, rather than synthesized wrappers.

Publish all owning packages only after the entire graph is checked, certified
and rendered. Define a bounded collision-free bundle layout and preserve the
existing new-directory atomic publication guarantees and single-crate layout.

## Tests and proof

- Actual two-crate and transitive/diamond Rust source fixtures bind exact calls;
  aliases retain defining identities and dependency bodies remain separate.
- Missing/private/extra/wrong-signature/wrong-owner/coordinated substitutions
  fail; ordinary unsupported calls remain diagnostics rather than fallbacks.
- Manifest owned/imported inventories agree with compiler and C certificates.
- Failure at every phase leaves absent output absent and existing output intact;
  deterministic bundles contain no duplicate owning implementations.
- Full release/frontend/C/shared/Java, lint/format/policy gates and fresh broad
  independent review pass before 04D native differential integration.

## Completion evidence

Both child tasks are complete. The exact compiler/certificate join and separate
owned/imported inventories passed their fresh reviews. Complete bundle output,
atomic publication, deterministic declared actions, failure/authority mutations
and budget boundaries passed fresh review and the final 340-test full gate
`5195cc51-42df-4ffc-afc0-4d4dba87cf2d`. See 03-02 for action and
inspectable-artifact evidence. Native multi-crate equivalence remains 04D.
