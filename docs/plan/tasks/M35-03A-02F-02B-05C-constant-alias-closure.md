# M35-03A-02F-02B-05C — Preserve cross-crate constant exports and close proof

- Status: in-progress
- Parent: [multi-crate constants](M35-03A-02F-02B-05-constant-bundles.md)
- Depends on: M35-03A-02F-02B-05B
- Specifications: [shared](../../specification/typed-generation/rust-constant-reexports.md),
  [C17](../../specification/typed-generation/languages/c/rust-constant-reexports.md),
  [Java21](../../specification/typed-generation/languages/java/rust-constant-reexports.md)

## Contract

Extend the checked public inventory with a distinct foreign-constant binding
case authorized by the exact producer certificate. A public re-export preserves
the defining Rust identity and ordinary producer C/Java symbol; it does not mint
a second owned declaration, duplicate storage or create a synthetic accessor.

Retain complete finite module/alias graphs and original module/doc ownership.
A facade consisting only of foreign constant exports is a valid selected API;
do not invent a function or falsely claim ownership of the producer field/object.
Specify and check this mapping in both target source inventories and versioned
bundle metadata before enabling the compiler path. Unsupported foreign modules
or other declaration kinds remain diagnosed, not silently omitted.

## Definition of done and tests

- Direct, renamed and transitive constant re-exports retain one producer identity
  across C/Java metadata and actual imported read nodes. Re-export-only roots and
  mixed roots work without dummy functions or duplicate constant definitions.
- Native independent Rust/C/Java consumers agree for exact values and aliases;
  target files remain separated by defining crate and docs stay inspectable.
- Stale/replaced owner, wrong alias binding, missing export, foreign private value,
  wrong declaration kind, duplicate identity, cyclic/unbounded expansion and
  unsupported schema controls reject before atomic publication.
- Complete whole-graph inventory and producer mutation/rebuild proof pass.
  Existing function, local/private constant and owned-source tests stay enabled.
- Fresh independent review loops, full Linux Bazel/release/lint tests, exported
  generated examples and scoped push complete this checkpoint.
- Close 05/02B/02 only after all required evidence is recorded. Do not claim wider
  constant families, all Rust syntax or legacy runtime retirement is complete.

## Bounded implementation order

1. [Typed compiler inventory](M35-03A-02F-02B-05C-01-source-export-inventory.md)
   — complete. Retain checked DefId bindings alongside the finite shared
   export graph; expose a distinct private foreign-module-constant declaration
   type, separate from owned LocalDefId declarations. Resolve direct, renamed and
   transitive aliases to one defining identity. The standalone constructor stays
   strict; authenticated bundle admission is integrated in step 7.
2. [C package provenance](M35-03A-02F-02B-05C-02-c-package-provenance.md)
   — complete; Java counterpart is completed in step 5 below. Package provenance: preserve the selected crate's export graph independently
   of any owned function/field. A re-export-only crate needs no fabricated source
   declaration. C header/source and Java facade metadata must retain this explicit
   package provenance and reconcile it with every owned declaration.
3. [Symbol-independent file requirements](M35-03A-02F-02B-05C-03-file-requirements.md)
   — complete. Preserve the implementation-to-header dependency even when an
   alias-only package has no owned symbol to create an edge. Use typed file
   requirements, shared role/cycle checks and independently reconstructed imports.
4. [C target export evidence](M35-03A-02F-02B-05C-04-c-constant-export-evidence.md)
   — complete (Java counterpart follows). Certify a separate alias inventory backed by original
   imported constant witnesses. Aliases point to defining symbols/paths; they do
   not enter owned constant definitions or create new producer authority. Check
   complete local/foreign binding union, retained dependency closure, documentation,
   identifier collisions and existing bounds, including zero-owned-item packages.
5. [Java package provenance](M35-03A-02F-02B-05C-05-java-package-provenance.md)
   — complete. Retain explicit selected-crate metadata and module documentation
   independently of owned Java methods/fields; reconcile all existing origins and
   reconstruct the metadata during certification. Foreign-export API admission
   remains a separate checkpoint.
6. [Java target export evidence](M35-03A-02F-02B-05C-06-java-constant-export-evidence.md)
   — complete. Implemented the corresponding certified foreign
   binding inventory, original producer authority and export-only dependency
   closure, with separate Java 21 native and tamper proofs.
7. [Compiler lowering and schemas](M35-03A-02F-02B-05C-07-compiler-publication.md)
   — complete. Register exports even when no body reads them;
   preserve direct/transitive owner authority and all finite module/name bindings.
   Explicitly version new alias metadata and reconstruct it from certified target
   evidence before publication. Authenticated source bundle foreign constant
   exports are now enabled; standalone foreign-owner publication stays rejected.
8. [End-to-end proof](M35-03A-02F-02B-05C-08-end-to-end-closure.md): native separate Rust/C/Java consumers, direct/transitive and
   re-export-only fixtures, stale/replaced/missing/wrong-kind controls, exact
   publication inventory, independent source mutation and Bazel invalidation,
   examples, fresh review, and complete release/lint gate.

Finite cycles in local module aliases are represented as graph edges and need
not be rejected or expanded into paths. Cyclic crate dependency graphs and
unbounded/over-budget expansion remain rejected. Producer declaration docs stay
on the producer; facade/module docs remain with the re-exporting crate.
