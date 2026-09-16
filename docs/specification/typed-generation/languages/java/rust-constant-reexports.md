# Rust constant re-exports in Java 21

- Status: planned; existing foreign-export rejection remains active.
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
