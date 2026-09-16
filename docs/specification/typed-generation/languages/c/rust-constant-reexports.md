# Rust constant re-exports in C17

- Status: explicit source-package provenance and typed file requirements implemented;
  certified foreign exports remain planned. Existing foreign-export rejection is active.
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
includes normalized comments. The existing nonempty-definition profile remains
closed until the separate certified export inventory admits alias-only packages.

## Certified API and metadata

Keep owned constant definitions, imported registrations and public alias bindings
as distinct typed inventories. Every foreign binding must match a retained
CDependencyConstant and exact original producer authority. Export-only references
still retain dependency closure and require their header even without body reads.
Namespace/collision checks include the complete relevant producer inventories.

The selected crate remains a distinct CDependencyApi owner; resolving one of its
foreign bindings yields the original defining witness, not a newly branded
constant whose owner is the facade. Owned constants continue using existing APIs.
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
