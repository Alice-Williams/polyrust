# Nested owned record evidence

- Status: observations, operation identity and bounded body evidence implemented
- Plan: [M35-02B-03L](../../../../plan/tasks/M35-02B-03L-nested-owned-records.md)
- Existing flat boundary: [owned record fields](rust-owned-record-fields.md)

## First observation grammar

Two nongeneric local named-field record declarations have no custom Drop.
Inner has two standard Box<i32> fields. Outer has one Inner field followed by
one standard Box<i32> field. Safe free functions construct the three scalar
owners from distinct immutable i32 parameters, build Inner and then Outer,
move one or more leaves (or the whole Inner) into immutable bindings, and
return a dereferenced live leaf. Observe reversed initializer evaluation order
separately from declaration order. Compilation uses the pinned abort policy.

Use actual AdtDef/DefId, FieldIdx, Ty and Place/projection values. A nested leaf
is a typed nominal field path, not a dotted string or flattened numeric ID.
Whole-inner moves transfer all remaining leaf obligations; a later partial move
settles only that leaf's old position. Cleanup recursively follows record field
declaration order, skipping moved leaves, at the containing binding's exit.

## Staged architecture

L-01 asserts observed representation only. It produces no checked nested
capability, target package or ownership certificate. L-02 implements bounded
private [operation inputs](rust-nested-construction.md) and executable bindings;
L-03 implements complete [canonical source/normal-MIR correspondence](rust-nested-correspondence.md)
before returning private body evidence. Its separately frozen grammar covers
the two-level tree, whole-inner and leaf movement, and actual grouped or leaf
cleanup, with explicit statement/path/normal-flow budgets.

Retain source initializer order, staging operations, aggregate declaration
membership, complete nested move paths, lexical bindings and actual read/drop/
Return locations for ordinary typed consumers. Existing flat/single-owner
readers remain closed. Unsupported input diagnoses before target publication.

Body admission freezes depth/member/statement/path limits and requires
whole-body accounting, typed mismatch/forgery controls and coherent wrong-owner
mutations. Do not infer nested proof from equal record shapes or Drop counts.
Conditional partial initialization/moves remain the separate required 03M
checkpoint. C allocation/failure mapping and Java support are not enabled here.

## Observed pinned representation

The five initial Rust 1.98.0 fixtures retain two ADT aggregate assignments.
Each source initializer is moved through a distinct staging local; reversing
initializer order reverses staging order, not aggregate operand declaration
order. Moving `outer.nested.first` uses the full two-field path. Moving
`outer.nested` first transfers the Inner value into its own local, after which
partial extraction and remaining-field cleanup refer to that local.

After one nested leaf is extracted, its local drops first, then the remaining
inner leaf, then Outer's spare owner. Extracting both inner leaves drops their
locals in reverse declaration order and only the spare field in Outer. These
orders follow the actual normal CFG, not basic-block numbering. The selected
scalar read precedes all drops and the complete Return follows them.

Observation tests assert these patterns with compiler types and places and
reject changed field/read/staging/transfer cases. This is the input evidence
for the next capability design, not a safe general nested-body matcher.

L-01 is complete with exact constructor identity, one actual nominal pair,
local nongeneric named-field records and immutable binding assertions. Its
three invalid Rust and fourteen valid-source controls, full 488-test gate and
fresh clean review are documented in the task. L-02 is also complete, with
private bounded layouts/canonical constructors and a 501-test full gate plus
clean review after four proof/contract repairs. Complete body admission remains
the separate L-03 work; no C/Java heap output is enabled by these checkpoints.
