# Boxes containing scalar records

- Status: in-progress
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

## Whole-body correspondence remains required

Before enabling a body certificate, inspect the pinned compiler's payload
aggregate construction, moves into Box, payload-pointer/field reads and
PostCleanup Drop. Specify the closed source grammar and correlate every
producer/field by actual nominal identity, typed place and source order.
No source binding may be invented for compiler staging storage. Native-layout
details observed in Rust are not instructions to copy Rust's Box representation
into C.

Structured source operations remain the future C lowering input. Use existing
C registry/AST/verifier/render-ready boundaries and derive dependencies from
types/operations; no raw bodies/imports or goto rendering is introduced.
Allocation failure, clone and native cleanup policy remain separate required
mapping/proof work. Java does not acquire heap support from this experiment.
