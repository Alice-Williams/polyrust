# M35-03A-02F-02B-05C-06 — Certified Java foreign constant exports

- Status: complete
- Parent: [constant alias closure](M35-03A-02F-02B-05C-constant-alias-closure.md)
- Depends on: M35-03A-02F-02B-05C-05
- Specification: [Java constant re-exports](../../specification/typed-generation/languages/java/rust-constant-reexports.md)

## Contract

A Java facade may own no source definitions and still expose a certified Rust
constant alias inventory. Every alias retains its original JavaDependencyConstant,
defining Rust identity, exact producer certificate and public static final field
path. No copied field, accessor, proxy, inheritance or runtime is emitted.

Keep descriptive source-package metadata, consumer-scoped imported-value handles,
owned source inventory and certified foreign export views separate. An export
name or DefId alone cannot grant dependency authority.

## Implementation

1. Add a focused constant_exports selection module. Reconcile the finite explicit
   source graph with the facade's frozen JavaDependencyBindings. Every foreign
   value binding must select an authenticated JavaImportedValue by exact defining
   declaration identity and original owner. Reject absent/conflicting witnesses,
   wrong namespace/kind and unsupported foreign module/function/type exports.
   Retain each module/name binding; deduplicate dependency references separately.
2. Use typed export-only references in file-item dependency roots even when no
   expression reads the constant. Existing shared linking resolves the defining
   field's qualified path. Reconstruct the same references and selected bindings
   during verification. Never write imports as strings, invent an expression use,
   or rely on a renderer discovering missing dependencies.
3. Allow JavaDependencyApi to take its selected crate from explicit metadata
   without requiring a first source declaration. Require an actual public owned
   function/constant or authenticated foreign constant selection; an ordinary
   empty facade still does not become a dependency API.
4. Add private-field JavaForeignConstantExport values with module(), name() and
   dependency() accessors, exposed through JavaDependencyApi::foreign_constants().
   The dependency is the original JavaDependencyConstant. Keep functions(),
   constants(), function() and constant() owned-only. Validate the complete
   selected owned/foreign union against the finite graph, with deterministic
   binding order and no newly branded facade producer authority.
5. Retain complete original producer closures for export-only dependencies.
   Reconcile consumer identity even when no source methods/fields exist. Preserve
   independent-certificate conflict, duplicate binary name, self/cycle, original
   consumer scope, graph and source resource limits. Audit source_byte_bound and
   source_descriptions for legitimate empty facades without creating source facts.
6. Keep compiler foreign-export admission and bundle schemas unchanged. Those
   require the following cross-language lowering/publication checkpoint.

## Definition of done and tests

- Direct, renamed, repeated, transitive and multiple-producer constant aliases;
  finite local module-alias cycles; mixed owned/foreign and alias-only facades.
- Owned/foreign views stay disjoint, and retrieved foreign witnesses survive
  dropping both the producer and intermediate API handles. Compile-fail probes
  reject constructing foreign export proof views or imports from names/paths.
- Independent Java 21 producer, intermediate, mixed root and consumer compilation
  under strict lint; exact bool/i32/i64 values including signed extrema and wide
  i64 values; reflection confirms no duplicated alias fields, fake source methods
  or runtime classes. Recompile consumers after producer mutations to detect
  stale values despite javac constant inlining.
- Reject missing/replaced/conflicting certificates, wrong consumer scope, wrong
  binding namespace/kind, swapped/deleted references, and coupled projection
  changes. Exercise complete direct/transitive owner closures with zero owned
  source declarations and positive controls.
- The Java source bound covers actual UTF-8 rendered bytes; empty source
  descriptions remain empty rather than claiming aliases are definitions.
- Existing source functions, records, local/private/owned constants, imports and
  legacy portable tests stay enabled. Export real generated examples locally.
- Full Linux Bazel release, Rust/Bazel lint and native gates pass on an exact scoped
  tree. Fresh Sol Extra High review loops leave no unaddressed core findings.
- Document evidence, commit with this ID and push. The parent source publication
  and legacy runtime-removal work remain incomplete.

## Completion evidence

- Implemented typed foreign selection and export-only dependency roots; private
  JavaForeignConstantExport views retain original producer authority. Owned API
  lookups remain owned-only. Empty owned inventories need no fake declarations.
- Exact implementation tree `7f6d5db2dc708c19add038dc92cd3d683a8ea0fc` passed
  Linux dev-container Bazel `//... //:release_gate`: 1,093 targets, **738/738
  tests**, 7 executed / 731 cached, 165.489 seconds; invocation
  `5a1ff1dd-6d85-420f-9d90-ac50ba6052ef`. Rust/Bazel linters, native target tests
  and prior capacity/resource gates stayed enabled.
- Initial implementation tree `fb76decaf8bf38d5bf4e3432031440738e342479` passed
  focused Java unit/native and typed compile-fail targets (2/2), invocation
  `058d2174-f000-4b94-b61a-717fdc5744f2`, then the full 738/738 gate,
  83 executed / 655 cached, 251.688 seconds; invocation
  `c5bb89f5-811f-4a9c-be37-b70de82922be`.
- Native Java 21 proof compiles two producers, alias-only intermediate, mixed
  root, generated typed readers and independent handwritten consumer separately
  with strict lint. Exact eight bool/i32/i64 truths plus two owned values pass.
  Reflection proves zero alias fields/methods and only the normal private
  constructor. All eight compiling producer mutants fail the unchanged truth
  after recompiling readers and consumer, accounting for javac inlining.
- Regressions cover every module/name binding, original authority after handles
  are dropped, missing/wrong-kind/namespace witnesses, direct and transitive
  zero-owned self/conflicting-certificate closure, valid diamonds, consumer-scope
  replacement, deleted and swapped resolved paths, UTF-8 byte bounds and empty
  source descriptions. Private proof construction is compile-fail tested.
- Fresh independent Sol Extra High review found **no core findings**. Its
  optional Macro-namespace and swapped-valid-path regressions were added; the
  follow-up review also cleared those and artifact export. Additional alias-heavy
  boundary duplication was optional: existing declaration-free graph and frozen
  dependency limits retain their existing gates.
- Native generated originals are exported through Bazel's undeclared test
  artifacts under `java-constant-exports/`, including all five Generated.java
  files and Consumer.java; local copies live in ignored
  `generated/examples/java-constant-exports-7f6d5d/`. No generated files are
  committed. All 20 protected unrelated ownership hashes remained unchanged.
- The final documentation-only checkpoint is revalidated before publication.
  Production rustc foreign-export admission, alias-aware bundle schemas and
  legacy runtime retirement remain unfinished; continue with
  [compiler/publication integration](M35-03A-02F-02B-05C-07-compiler-publication.md).
