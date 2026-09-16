# Rust constant re-exports in C17

- Status: planned; existing foreign-export rejection remains active.
- Contract: [shared re-export design](../../rust-constant-reexports.md)

## Package provenance and storage

Retain explicit selected-crate provenance in the typed C package independently
of function/object origins. Check it against every owned declaration, file role,
module graph and documentation attachment. It is descriptive unresolved metadata
until the containing package has passed certification, not a proof constructor.

An alias-only crate keeps its ordinary public header/source pair. Its public
header derives required defining-producer headers from certified export
witnesses. The source includes its own header. Do not emit a second object
definition, macro rename, accessor or runtime. Native consumers use the defining
ordinary object symbol resolved by the preserved alias metadata.

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
