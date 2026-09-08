# C17 public value factories and observations

- Status: normative migration target; owners M34A-11-03/06
- Complements [callable and lifecycle ABI](callable-abi.md)

## Signature notation and output rules

S(T), H(T) and Borrow(T) have the callable ABI meanings. A is
const poly_allocator*. Every name below denotes a registered monomorphic symbol,
not a runtime name lookup or a C variadic prototype. The public ABI identity
map supplies its exact generated spelling. Repeated fields/elements below are
expanded into exact finite signatures.

OwnOut(T) is H(T)**. ObserveOut(T) is S(T)* for scalars and const H(T)** for
owned values. BorrowArray(T) is const S(T)* for scalar elements and
const H(T)* const* for owned elements. This is a borrowed array of immutable
handle slots, not a mutable representation of a list.

All functions below return poly_status. Factories/clone/coercion use OwnOut,
require a valid initially null output slot, and allocate with A. Invalid
nonempty owning slots return InvalidState unchanged. Observers and scalar-tag
constructors require writable output storage but no initial value; they write
only on Success and preserve every output byte on failure. ObserveOut is not
an owning slot and receives no transfer. Null required output addresses return
InvalidInput. Output storage must not overlap an input or its reachable storage.
All simultaneous detectable failures follow the callable ABI's ordered
validation ladder; the individual failure clauses below do not override it.

Raw input arrays may be null only when their length is zero; then no pointer
arithmetic, dereference or C library memory call is performed on the null
pointer. Nonzero arrays require valid storage for the exact element count;
null owned elements and malformed scalar domains are InvalidInput. Arbitrarily
forged storage cannot be made safe by inspecting a C pointer: validity, extent,
alignment and no-overlap are foreign caller preconditions. Detectable domain
errors are checked before access. Length arithmetic overflow returns Capacity.

Factories clone borrowed fields/elements in source order, roll back on every
failure and leave inputs unchanged. They return the new value directly, not
outcome_R: factory validation is a boundary transport failure. Portable checked
operations such as UTF-8 decoding explicitly map their semantic invalid-input
case to the specified computational error; they do not conflate every
InvalidInput/Capacity/AllocationFailure with that portable error.

## Closed view types

Two public non-owning view structs are admitted:

    poly_byte_view { const uint8_t *data; size_t length; }
    poly_utf8_view { const uint8_t *data; size_t length; }

The other public non-owning aggregate is the allocator protocol descriptor,
poly_allocator, with exactly context, allocate and release members and the
callback signatures in callable-abi.md. It is a distinct registered runtime
type, complete in the public runtime header so a foreign consumer can initialize
it. It is not a portable value, view, owning handle or freely extensible record.

The latter's length is UTF-8 bytes, not Unicode scalar or UTF-16 units. Both
have exactly these fields and no destructor. Empty views use null data and zero
length. Factories consume pointer/count arguments, not a supposedly trusted
foreign view certificate. A returned view is read-only and valid only while
its original owner remains live and unmoved. Copying it does not extend that
lifetime. Borrowed projections into any nested value share that lifetime.

## Factories and observers

| Family | Exact signatures after symbol specialization |
| --- | --- |
| String | from_utf8(A, const uint8_t *data, size_t length, H(String)** out); utf8_view(const H(String)* value, poly_utf8_view* out) |
| Bytes | from_bytes(A, const uint8_t *data, size_t length, H(Bytes)** out); byte_view(const H(Bytes)* value, poly_byte_view* out) |
| List<T> | from_items(A, BorrowArray(T) items, size_t count, H(List<T>)** out); length(const H(List<T>)* value, size_t* out); get(const H(List<T>)* value, size_t index, ObserveOut(T) out) |
| Record R | create(A, Borrow(F0) field0, ..., H(R)** out); field_i(const H(R)* value, ObserveOut(Fi) out), one registered accessor per field |
| Option<T> | none(A, H(Option<T>)** out); some(A, Borrow(T) value, H(Option<T>)** out); tag(const H(Option<T>)* value, uint32_t* out); value(const H(Option<T>)* value, ObserveOut(T) out) |
| Result<T,E> | ok(A, Borrow(T) value, H(Result<T,E>)** out); err(A, Borrow(E) error, H(Result<T,E>)** out); tag(const H(Result<T,E>)* value, uint32_t* out); ok_value(const H(Result<T,E>)* value, ObserveOut(T) out); err_value(const H(Result<T,E>)* value, ObserveOut(E) out) |
| Legacy payload enum E | variant_i(A, Borrow(F0) field0, ..., H(E)** out), one exact constructor per variant; tag(const H(E)* value, uint32_t* out); variant_i_field_j(const H(E)* value, ObserveOut(Fj) out) |
| Payload-free enum E | from_tag(uint32_t tag, S(E)* out); tag(S(E) value, uint32_t* out) |
| Interface I implemented by R | from_R(A, const H(R)* source, H(I)** out), one constructor per exact registered witness; methods use the callable ABI |
| Computational outcome R | tag(const poly_outcome_R* value, uint32_t* out); value(const poly_outcome_R* value, ObserveOut(R) out); error_code(const poly_outcome_R* value, poly_utf8_view* out); error_message(const poly_outcome_R* value, poly_utf8_view* out) |

String construction validates all UTF-8 scalar rules, preserves embedded zero
and rejects malformed UTF-8 as InvalidInput. Bytes has no text validation.
List get with index >= length returns InvalidInput unchanged, not an unchecked
read or a computational outcome. Portable GetChecked uses its own specified
computational bounds error. A list exposes no mutable buffer or raw array of
owning handles. Cloning an owned element uses its existing clone function on
the borrowed result of get.

Option tags are None=0, Some=1; Result tags Ok=0, Err=1; computational outcome
tags Value=0, Error=1. Native named constants are registered int enumerators;
observer outputs retain the uint32_t ABI. The wrong branch of an accessor
returns InvalidState without modifying output. Invalid internal tags in valid
foreign storage return InvalidInput before any payload access. Ordinary
Result Err payloads are not computational errors.

Legacy enum tags and payload-free tags follow canonical variant order.
Out-of-domain payload-free enum inputs fail InvalidInput in both functions.
An empty enum has no successful from_tag and no native empty enum declaration.
Record factories with zero fields still have exactly allocator and out
parameters and produce a live owner with nonempty private bookkeeping layout.

Interface construction deeply clones the concrete record, then commits an
owning context paired with its exact private flat table. There is no public
context/table accessor, foreign table registration or downcast. An interface
with no implementations has no constructor; declarations and safe nesting
remain available. Neither observer nor factory invents an inhabitant.

Outcome construction is private to verified function/runtime lowering. Foreign
callers receive outcomes from portable functions/constants and use the listed
read-only observers plus the ordinary clone/move/drop family. Error code/message
views are valid UTF-8. No public take or mutable payload accessor is emitted.

## Proof obligations

Stage 03 registers these exact signatures, named tags, public view and allocator members and
reference-derived declarations; stage 06 implements their structural bodies.
Every family has a separate public-header consumer and native round trip,
including zero length, embedded zero, malformed scalar/UTF-8/enum inputs,
nested owned projections, wrong branch/index, and output-preservation controls.
Add pairwise simultaneous-failure cases across the applicable validation phases,
including null input plus nonempty output, malformed allocator plus nonempty
output, malformed UTF-8 plus nonempty output, and inactive branch plus null
observer output. Assert the exact status, untouched outputs and no later calls.
Tests copy borrowed inputs, prove clone independence, then drop every owner.
Each allocating factory/coercion/clone is fault-injected at every reached
allocation and checked for rollback, unchanged inputs and zero leaked blocks.
Borrow lifetime and no-overlap facts are verified for generated consumers;
foreign violation of those preconditions is not a promised runtime diagnostic.
All implementation files remain under the source-size policy.
