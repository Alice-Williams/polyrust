# C17 storage, extents and ownership evidence

- Status: normative for M34A-11-02D-04
- Prerequisite: authenticated converged numeric facts from 02D-03

## Scope and module boundaries

The storage layer consumes the same immutable registry, AST and control graph
as numeric analysis. Private observation views expose actual operands and
derived facts, never writable Number/State values or user-supplied proof flags.
Use focused modules under ownership/storage for extent checks, storage paths,
initialization, pointer provenance, allocation/lifecycle and borrow/boundary
composition. Share identities at a genuine internal boundary instead of
inventing parallel name-based storage IDs. No public proof constructor.

The ordered implementation checkpoints are fixed-array index extents (04A),
storage/pointer/initialization flow (04B), allocation/lifecycle (04C), and
borrow/allocator/boundary composition (04D). Intermediate diagnostic success
is not complete C safety, a callable summary or a rendering certificate.

## Index and pointer formation

Each evaluated index uses the retained numeric fact for that exact actual
place at that exact graph action. Facts reconstructed from a later use or an
identically spelled variable are invalid. All reachable Read, Write and
AddressOf paths are covered, including guarded expression children.

File-scope address initializers have a separate authenticated initializer/place
origin, not a fabricated function-graph point. Their integer operands use the
same numeric transfer and retained wrap history after constant-tree checking.
The bound consumer covers both static and runtime observations. Missing or
cloned static occurrences cannot validate against the actual input package.

For a fixed array, derive the bound from its actual base type. Require the
entire index domain to be nonnegative and strictly less than that bound, with
no unresolved wrap/storage history. Nested arrays check each dimension
independently. The admitted subset conservatively excludes one-past indexed
places even in address-only contexts; it does not claim C itself forbids every
one-past pointer. Pointer-based indexing additionally requires actual storage
provenance and extent; unknown pointer extent is a diagnostic, never a skipped
check. Numeric bounds alone do not establish pointer validity.

Address formation and memory access retain distinct obligations. Taking the
address of uninitialized but live storage does not read its value. A write can
initialize valid fresh storage. A read requires the exact initialized subobject,
live allocation, correct alignment and active union member. Pointer type,
nonnull state and declared array length cannot authenticate backing storage.

## Storage and lifetime

Storage identity retains actual declarations and typed subobject paths.
Allocation identity additionally retains the actual producing call, successful
path, checked extent/alignment and exact allocator descriptor. Conversion to
void and back preserves provenance; AllocationRestore cannot create it.
Read-only qualification cannot extend lifetime or confer ownership.

Initialization and lifecycle are separate: Uninitialized/Prefix/Complete
storage does not substitute for Empty/Live/Moved/Dropped handle state.
Aggregate zero initializes representation slots, not inhabited portable owners.
Active union membership is separate from containing-object initialization.
Joins retain only facts established on every incoming path. Scope exits,
including crossed scopes on jumps, expire storage/borrows. Aliased writes,
opaque effects and owner transitions invalidate dependent observations.

## Allocators, calls and proof composition

### 04B diagnostic subset

The initial storage checker uses shared authenticated declaration roots and
member/index selectors. An actual numeric observation may give an index
interval; every selected element must be valid. Pointer indexing admits
nonnegative offsets within the same array, or zero for a scalar subobject.
A member pointer does not inherit its containing array's extent.

Storage values compress zero arrays and sparse initialized elements. A
non-singleton write weakly updates possible targets and never initializes a
particular previously uninitialized element. Reads prove complete selected
coverage, including the active union member. Joins retain exact pointer
agreement; ambiguous or imported pointer values remain unproved, even for a
read of the pointer value itself at this intermediate boundary. Boundary
contracts will supply admissible incoming-pointer facts in 04D/05.

Every evaluated expression result satisfies that same storage-value boundary,
including discarded known stream pointers and results joined by conditionals
or variable-index reads. Equivalent proved null representations retain their
common fact recursively through aggregates; unknown or expired pointers do not.
Target matching uses the authenticated AST ABI type relation (including
Int/I32 and U64/Size), ignoring only immediate pointee qualification. Array
bounds, nominal identities and qualifiers behind nested pointers remain exact.

Crossed lexical exits invalidate retained pointer copies recursively through
aggregates. A later activation of the same local declaration cannot revive
them. Returning or globally storing automatic addresses rejects, including
addresses nested inside aggregates. Function-entry globals are initialized
representations, not assumed copies of their startup initializer values.

The diagnostic boundary rejects calls and AllocationRestore until producing
call/lifecycle and body-derived effect evidence is composed. Adapter restoration
requires agreement with the retained original object type; casts do not grant
extent or alignment. These restrictions are explicit subset boundaries, not
claims that every rejected program is invalid C. No owning-handle or rendering
certificate is created.

The exact allocator, lifecycle and validation ladder are authoritative in
[callable ABI](callable-abi.md) and [ownership ABI](ownership-and-abi.md).
Storage checking derives local transitions and call-site obligations from
actual syntax. A matching prototype, registered contract or claimed pure
function cannot establish generated effects. Stage 05 authenticates summaries
from bodies; unresolved effects fail closed until that composition succeeds.
Do not weaken a local storage proof to break a circular summary dependency.

Every checkpoint has positive behavior, rejected mutations and private-boundary
tests. Later generated-native sanitizers and allocator fault injection remain
mandatory in stage 06; passing structural analysis does not replace those tests.
