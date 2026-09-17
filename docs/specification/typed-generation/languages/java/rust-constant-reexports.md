# Rust constant re-exports in Java 21

- Status: explicit provenance, certified foreign exports, checked compiler
  lowering and alias-aware bundle publication implemented. Standalone foreign
  owner publication remains rejected.
- Contract: [shared re-export design](../../rust-constant-reexports.md)

## Package provenance and fields

Retain explicit selected-crate provenance in the typed Java compilation unit,
independent of generated source methods and fields. Reconcile it with its
RustCrate package, facade identity, owned source registrations, finite module
bindings and documentation. Caller-supplied metadata alone grants no authority.

An alias-only crate emits its ordinary final Generated facade and private
constructor, with module documentation but no copied constant field or accessor.
The alias resolves to the defining producer's public static final primitive
field. No inheritance, proxy class, runtime helper or raw text is introduced.

## Explicit source-package registration

JavaFileItem::Type retains optional JavaSourcePackage metadata on its selected
facade. JavaSourcePackage owns an immutable Arc<RustCrateExports> through private
fields and exposes only a descriptive constructor and exports() accessor. It is
not a certificate, declaration registration or witness for a foreign value.

An explicit registration requires exactly one canonical public Generated.java
compilation unit in the graph root's JavaPackage::RustCrate namespace, main
placement, and a registered public final synthesized PackageEntryPoint class.
Duplicated or misplaced metadata rejects. Every owned RustSource registration,
including nested record fields, must agree with the explicit selected graph and
crate identity. Absent metadata retains declaration-derived compatibility;
conflicting explicit metadata must never fall back to inferred declarations.

The shared documentation checker accepts the explicit graph independently of its
origin iterator. Both empty and nonempty packages use the same bounded graph,
ancestry, namespace, identity and documentation checks. Distinct immutable graph
allocations are charged and compared; a repeated allocation can be memoized.
No fabricated RustSourceOrigin is used to attach module documentation.

Source registration checks and module-document lowering use this same metadata
selection. Linking retains the exact unresolved registration, and independent
verification reconstructs resolved documentation from the original checked
package. Changing metadata and normalized comments together does not grant
authority. Resource limits still cover graph entries, names, ancestries and
documentation when no methods or fields exist.

Production Rust-source assembly attaches explicit metadata unconditionally.
An empty documented Java class may be syntactically certifiable without being a
JavaDependencyApi: dependency API admission continues to require its separately
validated public owned or foreign binding inventory. Package provenance alone
does not admit foreign exports or enable a new publication schema.

## Certified API and metadata

Keep owned GeneratedValue registrations, JavaImportedValue bindings and public
foreign alias selections distinct. The frozen dependency scope retains original
JavaDependencyConstant authority even for export-only values with no expression
uses. The complete owner closure remains mandatory.

The facade's JavaDependencyApi retains its own crate identity. Resolving a
foreign binding returns the original producer witness/path rather than creating
a replacement witness owned by the facade. Certified alias metadata retains each
module/name/namespace binding and exact defining declaration, owner, qualified
field path, primitive type and lossless value.

JavaDependencyApi::foreign_constants() returns JavaForeignConstantExport views
in deterministic module/name order. Private fields and module(), name() and
dependency() accessors mirror the C distinction: the returned dependency is the
original producer's JavaDependencyConstant, never a facade-owned field. Existing
owned lookup and iteration APIs retain their meaning.

A focused typed selection reconciles explicit export metadata with frozen
JavaImportedValue registrations before dependency linking. Export-only aliases
contribute real dependency roots and exact original qualified field paths without
inventing Java expressions or imported field copies. Independent reconstruction
must reject missing, extra, swapped and jointly altered dependency projections.

An alias-only facade retains the ordinary private constructor and module docs.
Its owned source descriptions are empty; its source-byte bound still covers all
rendered bytes. Owner closure validation uses the selected RustCrate namespace
even without a source method or field. Missing, conflicting, stale or
wrong-consumer producer authority cannot be repaired by matching a string name.

Alias-bearing owner metadata uses schema_version 4 (the bundle index remains
version 1). Its constant_exports array is ordered by (module, namespace, name);
each entry records module, namespace, name, id (defining declaration), owner
(original producer root), path, scalar, readonly: true and lossless value.
constant_imports describes actual declaration/expression references, not aliases
that exist only as file dependency roots. Both arrays may reference the same
defining constant. The complete dependencies inventory covers their union and
all frozen retained authorities. Non-alias owners keep schemas 1/2/3 unchanged.

Projection obtains the selected graph from certified explicit source-package
metadata, falling back to a source description only for existing implicit
packages. Empty declarations are legitimate for an alias-only owner.
Projection and reservation retain exact certificate-derived alias views and
reconstruct them before encoding; changing names, paths, witnesses or the whole
descriptive projection cannot grant authority.

Version the alias-bearing owner schema explicitly. Keep used expression imports
separate from export-only aliases; the dependencies inventory covers both.
Reservation and encoding visit the same checked inventories, and bundle
preflight reconciles each original owner before atomic publication.

## Proof

Compile producer, alias-only intermediate, mixed root and native consumer
separately using strict Java 21 checks. Assert no duplicated field or synthetic
source method and exact producer paths in alias/read evidence. Recompile native
consumers after producer mutations because javac may inline constant variables.
Reject incomplete/replaced owner graphs, forged aliases, wrong types/values,
namespace mismatches and invalid package provenance before publication. Preserve
existing function-only/owned-constant schemas and all prior conformance gates.
