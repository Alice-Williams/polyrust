# Rust constant re-exports in C17

- Status: explicit provenance, certified foreign exports, checked compiler
  lowering and alias-aware bundle publication implemented. Standalone foreign
  owner publication remains rejected.
- Contract: [shared re-export design](../../rust-constant-reexports.md)

## Package provenance and storage

Retain explicit selected-crate provenance in the typed C package independently
of function/object origins. Check it against every owned declaration, file role,
module graph and documentation attachment. It is descriptive unresolved metadata
until the containing package has passed certification, not a proof constructor.

An alias-only crate keeps its ordinary public header/source pair. Its public
header derives required defining-producer headers from certified export
witnesses. The source includes its own header. The implementation-to-header edge is an
explicit typed file requirement derived from the selected package registration;
its existence must not depend on an owned function/object symbol. The shared
linker authenticates the exact module handle and output path, checks role/cycle
rules, deduplicates it with any symbol-derived edge and reconstructs its directive
during verification. Do not emit a second object
definition, macro rename, accessor or runtime. Native consumers use the defining
ordinary object symbol resolved by the preserved alias metadata.

## Explicit source-package registration

CRegistry::register_source_package registers one CSourcePackage with private
fields: the registry-branded public-header reference and Arc<RustCrateExports>.
The read-only accessor is descriptive metadata; neither registration nor freezing
makes it renderable. Reject wrong file roles, foreign registry references and
duplicate/replacement registrations. The production public Rust frontend always
supplies this registration.

Projection reconciles it with the exact header/source layout and every owned
Rust-source registration. Declaration-derived callers remain compatible only when
the explicit registration is absent. Explicit and inferred provenance may not
disagree, and no fallback may mask such a conflict.

Documentation lowering accepts the graph independently of declarations. It
validates finite graph bounds, root, module ancestry, documentation consistency
and file routing, including graphs with no owned declarations. Rebuilding the
shared projection during verification repeats these checks; resource accounting
includes normalized comments. The semantic profile admits zero owned definitions only when the separate
certified export selection authenticates a nonempty foreign constant inventory.

## Certified API and metadata

Keep owned constant definitions, imported registrations and public alias bindings
as distinct typed inventories. Every foreign binding must match a retained
CDependencyConstant and exact original producer authority. Export-only references
still retain dependency closure and require their header even without body reads.
Namespace/collision checks include the complete relevant producer inventories.

The selected crate remains a distinct CDependencyApi owner; resolving one of its
foreign bindings yields the original defining witness, not a newly branded
constant whose owner is the facade. Owned constants continue using existing APIs.
CDependencyApi::foreign_constants() returns a distinct read-only sequence of
CForeignConstantExport values in deterministic module/name order. Each has
private fields and module(), name() and dependency() accessors. The dependency
is the original CDependencyConstant, not a newly owned facade constant.
Construction remains internal to certificate-derived API inventory collection.

Export selection reconciles the finite graph with registry-authenticated imports
before projection. For each foreign binding it derives a header-owned
DependencyValue reference, even without a body read. Independent projection
reconstruction rejects deleted, swapped, extra or jointly altered per-unit and
package import maps. File-layout ordering is structural; semantic profile
admission separately requires public owned declarations or authenticated foreign
bindings and actual owned definitions or authenticated foreign bindings.

Alias-bearing bundled owner metadata uses schema_version 6; non-alias packages
retain versions 1 through 5. The bundle index stays version 1. constant_exports
records each module/namespace/name binding in deterministic order with id,
original owner, symbol, defining header, scalar type, readonly and lossless value.
constant_imports remains the inventory of actual expression-used constants;
export-only aliases are not misreported as body reads. The same constant may
occur in both. Preflight reconciles their union against original member
certificates. Standalone output cannot omit required foreign owners.

New alias schema records binding module/name/namespace and defining declaration,
producer, symbol/header, scalar type and exact value. Reconstruct both directions
from certified package evidence before publication.

## Proof

Compile each generated header independently and link separately compiled
producer/facade/consumer files with GCC and Zig at O0/O2 under existing strict
flags. Assert one defining external object, no synthetic function, no runtime
files and preserved module docs. Exercise alias-only packages and zero owned
definitions. Reject missing/replaced owners, missing or forged bindings,
wrong files/type/value/symbol, declaration-kind mismatches, collisions and
over-budget graphs before creating/replacing outputs.
