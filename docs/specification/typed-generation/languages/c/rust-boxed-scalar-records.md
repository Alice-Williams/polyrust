# Boxes containing scalar records

- Status: complete for the closed compiler-side construction/correspondence form
- Plan: [M35-02B-03H](../../../../plan/tasks/M35-02B-03H-boxed-scalar-records.md)
- Existing directions: [scalar Box construction](rust-owned-construction.md) and [records owning Boxes](rust-owned-record-fields.md)

## Distinct ownership direction

Box<Record> owns one allocation containing scalar fields. Record { Box<i32>, ... }
contains separate Box owners and supports partial owned-field moves. The two
shapes must have distinct typed inputs and ownership correspondence; the same
record spelling or equal field counts do not connect them.

The initial scalar-record payload is local, nongeneric, named-field, nonempty
and bounded to 128 fields, with default representation and no custom Drop.
Each field is exactly i32 or bool after compiler normalization. Preserve
AdtDef/DefId, GenericArgsRef, FieldIdx and Ty; use a closed scalar-kind enum to
select supported field mappings. Names remain presentation metadata only.

## Construction boundary

A separate OwnedScalarRecordBoxConstruction capability accepts only its private
compiler-session input. A canonical direct call must resolve to the compiler's
box_new diagnostic item; the instantiated safe Rust signature must take exactly
the actual payload type and return exactly the authentic standard Box result
including allocator identity. Neither method spelling, a wrapper function nor
a function-item local can select this capability. Keep Box<i32>'s existing
capability closed and preserve its historical rejection behavior.

The consuming owned-program builder stores a distinct executable mapping for
this capability. Supports is provided only by a correctly registered typed
slot; registration order must not change the input/context/output contract.
The input proves operation/type identity only. It must not claim payload
provenance or ownership cleanup merely because the Box constructor is genuine.

H-01 implements this boundary with a private shared standard-Box call reader,
the existing i32 input and a separate ScalarRecordBoxInput. Its payload metadata
is a private ScalarRecord containing ScalarField entries with ScalarKind::I32
or ScalarKind::Bool. The shared reader proves the actual signature/result type;
each capability wrapper independently checks its own payload policy. Neither
wrapper accepts caller-authored type or field metadata.

The builder's third scalar_record_box slot is independent of the existing Box
and record-construction slots. The required base Box slot is retained; absent,
duplicate and incorrectly typed registrations fail compilation. Tests invoke
all three stored mappings and preserve the prior Box-only and record consumers.
The new runtime proof also rejects adjusted Box results, direct local wrappers,
function-item variables, computed callees and same-spelled counterfeit methods.
Allocator-api source is excluded by the pinned stable compiler before operation
admission; exact standard allocator identity derives from the authenticated
constructor's full instantiated return type, not its first generic argument.

## Closed whole-body correspondence

H-02 admits only a root-scope, safe nongeneric function of immutable i32/bool
parameters, one complete scalar-record literal, one Box constructor consuming
that record binding, whole Box local moves, and a final explicit dereference
plus scalar field selection. Tail and explicit return retain distinct canonical
exit evidence. Implicit autoderef, record updates, arbitrary scalar expressions,
borrows, mutation, branches and other calls are diagnosed rather than approximated.

BoxedRecordBody exposes the existing SourceExit enum to consumers: Tail holds
the canonical value, while Return holds both canonical return expression and
value. Its returning location identifies the authenticated terminal MIR Return.
Private retention without a consumer-visible typed projection is insufficient.

The payload aggregate retains both source initializer order and declaration
FieldIdx order. Each operand traces to its exact parameter through a unique
typed scalar-copy staging assignment. A closed Move/Copy transfer records the
record-to-constructor-argument edge, checked against rustc's Copy predicate.
Neither transfer duplicates the owning Box. The final field read must use the
actual final Box's payload pointer and exact field type/index; one final Drop
must occur after that read and before return. All relevant locals, assignments,
calls and normal blocks must belong to this bounded correspondence.

The Copy distinction uses the compiler's
[type_is_copy_modulo_regions query](https://doc.rust-lang.org/stable/nightly-rustc/rustc_middle/ty/struct.TyCtxt.html#method.type_is_copy_modulo_regions)
only after successful analysis, with a fully monomorphized, lifetime-free
scalar-record payload. Pinned 1.98.0 compilation and fixture proofs are the
authoritative API/representation evidence; documentation is not a substitute.

The pinned compiler's payload aggregate, moves into Box, payload-pointer/field
reads and PostCleanup Drop have been inspected and correlated in H-02. Its
closed source grammar authenticates every producer/field by actual nominal
identity, typed place and source order before creating a body certificate.
No source binding may be invented for compiler staging storage. Native-layout
details observed in Rust are not instructions to copy Rust's Box representation
into C.

Structured source operations remain the future C lowering input. Use existing
C registry/AST/verifier/render-ready boundaries and derive dependencies from
types/operations; no raw bodies/imports or goto rendering is introduced.
Allocation failure, clone and native cleanup policy remain separate required
mapping/proof work. Java does not acquire heap support from this experiment.
