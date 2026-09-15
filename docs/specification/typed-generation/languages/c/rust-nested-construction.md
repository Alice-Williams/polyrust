# Typed nested-record construction input

- Status: implemented operation identity; whole-body admission remains separate
- Plan: [M35-02B-03L-02](../../../../plan/tasks/M35-02B-03L-02-nested-construction.md)
- Parent evidence: [nested owned records](rust-nested-owned-records.md)

## Input and authority

Introduce a separate `OwnedNestedRecordConstruction` capability whose input is
private compiler-session evidence for one canonical complete HIR struct
initializer. Reading that evidence requires the successful-analysis TyCtxt,
actual owner and canonical expression. Reject wrong owners, fabricated copies
of a HIR node, update syntax and adjustments. This input does not certify a body
or admit a target call, borrow, move or cleanup by itself.

Retain the canonical initializer and its exact result Ty, AdtDef and generic
arguments. Use a closed field-kind enum: a full standard Box<i32, Global> leaf,
or a nested local record layout. No field-name string, integer invented by the
adapter, erased Any value or supported-feature Boolean selects that mapping.

The layout is metadata over compiler-owned declarations, not a replacement
language AST. Each field occurrence retains its actual FieldIdx, DefId and full
Ty; each nested node retains its compiler record identity. Query-only projections
may expose these compiler types without making evidence constructible.

## Closed type grammar and budgets

Every record node must be a local nongeneric named-field struct with at least
one field, default representation and no custom Drop. Reject enums, unions,
tuple/unit structs, non-default representation/packing/alignment, generic arguments
or parameters, external records and fields of other kinds. Normalized compiler
field types must exactly match the layout's admitted type.

Explicit `#[repr(Rust)]` retains default Rust representation and is accepted,
including on a nested node. Reject C/transparent/SIMD and explicit packing or
alignment metadata. The compiler's semantic representation options are the
authority; do not parse source attributes merely to reject a spelling that
does not change representation.

Box leaves are equal to the full allocator-bearing scalar Box type derived from
the existing authenticated standard constructor signature. A pointer, reference,
same-named user Box, other payload, nested Box or custom allocator is not that
leaf. Do not strip generic arguments before comparing.

Limits are eight record levels (root is level one), 128 total field occurrences
across the expanded layout, and nonempty membership at every node. Count a
nested record-valued field itself and its descendant fields. Repeated use of the
same nominal record in two sibling fields is permitted, counted twice and not
a cycle. Reject repeated record identity on the current ancestor path. Check
limits before descending or allocating unbounded metadata. No source text or
debug-string parsing defines the layout.

The root must have at least one record-valued field. A flat all-Box root remains
the existing `OwnedRecordConstruction` input, not a second accidental route.

## Initializers and registration

For the root expression, preserve source evaluation order as a separate list
from declaration-ordered layout. Every initializer resolves through typeck to
one declared field, with complete unique membership. Its unadjusted expression
type must equal that field's full type. Retain canonical expressions; operation
identity makes no claim about the ownership effects of their bodies. L-03 will
separately restrict and authenticate complete source evaluation and transfers.

Add an optional consuming builder slot for a Mapping of this capability.
Existing five slots and Box-only consumers remain valid. A built binding that
claims Supports must provide its executable mapping, with associated typed
input/context/output, not an independent marker. Missing/duplicate/wrong slots
and forged evidence receive exact compile-negative tests.

## Required proof

Positive cases include two levels, deeper bounded layouts, repeated nominal
siblings, reversed root initializer order, aliases that retain canonical
compiler type identity, and distinct same-spelled declarations. Test depth eight
versus nine and 128 versus 129 expanded field occurrences independently.

Negatives cover the closed type grammar, wrong initializer membership/type,
wrong owner/canonical node, generic/tuple/representation/custom Drop and full
allocator type mismatches. Use coherent compiler types for isolated allocator
substitutions using the actual `std::alloc::System` allocator type, not a plain
struct without an Allocator implementation and not only invalid Rust source.
Authenticate the positive compiler trait bound and positive concrete impl;
negative or reservation impl inventory entries are not implementation proof.
Reference, locality, packing and alignment controls must reach their intended
predicate independently (e.g. a nongeneric static reference or external ADT).
Whole-body evidence, conditional
initialization and target rendering remain explicitly separate obligations.
