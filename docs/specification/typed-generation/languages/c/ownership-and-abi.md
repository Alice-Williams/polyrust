# C17 ownership, safety and public ABI

- Status: normative for M34A-11
- Replaces the historical M22B layout, not the portable value semantics

## Public values and compatibility

Preserve target ID `org.polyrust.c` and package entry paths
`src/generated.h`, `src/generated.c`, `src/runtime.h`, `src/runtime.c`
and native test entry points. The dialect/version is C17. The former
`org.polyrust.c17` specification spelling was not the registered target ID.

Scalars use exact fixed-width values, validated Unicode scalars and binary64.
Owning text, bytes, lists, records with owned data, options/results and interface
values use opaque handles with concrete monomorphized identities. Public headers
do not reveal mutable backing storage, union payloads, allocator fields or
vtable construction. Public operations expose checked factories, immutable
observations and typed clone/move/drop functions. Views carry a pointer and
length, are read-only and are valid only while the owner remains alive.

This intentionally changes the old public mutable-layout ABI. Update separate
consumer fixtures and ABI documentation at cutover with equivalent observations;
do not silently preserve a mutable escape to keep an old text snapshot passing.
Scalar-only value structs may remain by-value when all fields preserve the
specified immutable semantic boundary. The implementation owns their constructors.

C cannot stop an arbitrary foreign caller from copying raw pointers, forging
addresses, casting away const or violating a documented borrow lifetime.
Certification covers generated code and ordinary API use under the explicit
caller contract, not arbitrary hostile C. Boundary checks reject detectable
malformed views/tags/null arguments without dereferencing invalid storage.

## Ownership proof state

Each owned allocation/handle has a typed identity, allocator provenance,
concrete element type and initialization state. The verifier tracks
`Empty`, `Live`, `Moved` and `Dropped` states and joins branch states.
Borrowed views retain the owner and bounds; a borrow cannot outlive or mutate
its owner. Clone creates independent semantic ownership; move empties the
source; drop releases the live value once and resets the caller's slot.
A shallow C assignment never acts as clone. Cleanup visits only initialized
elements/fields and uses the allocator which created them.

Allocation goes through a typed allocator table. Every fallible construction
has an initially empty output, commits it only on success, and unwinds its
initialized prefix on failure. Failure at every allocation index must be
tested, including clone, nested aggregates, interface conversion and early exits.
Reallocation cannot discard the old pointer on failure. Element-count times
element-size and growth additions are checked before allocation.

Portable computation failures remain generated result values with the exact
portable error code/payload. A separate closed ABI transport status reports
allocation failure or invalid foreign input; it is not a new portable error,
an errno channel, a null option or process termination. An empty/moved handle
is a lifecycle state, not a constructed portable value.

## Sequencing and undefined behavior

Every effectful receiver/argument is materialized left-to-right once before a
call or parent expression. Short-circuit RHS plans remain conditional.
Generated locals own the intermediate values until consumed or cleaned up.

Signed overflow is never executed and then tested. Signed operators require
verified range facts/guards or exact checked-runtime plans. Wrapping operations
use unsigned fixed-width arithmetic and an explicit proved bit-preserving
conversion. Division checks zero and minimum/-1; shifts check width and signed
policy first. Bounds proof precedes indexing and pointer formation, not merely
the later dereference. Null+zero is not used as arbitrary pointer arithmetic.
No union punning or incompatible function-pointer casts implement interfaces
or F64: use matching active-member evidence and typed memcpy bit transfer.
No fast-math or host-sized integer approximation is permitted.

Proof metadata must be derived/revalidated from actual AST/control flow.
Caller-supplied `safe: true`, unchecked range IDs or a catalogue label without
a matching implementation cannot establish ownership or arithmetic safety.
