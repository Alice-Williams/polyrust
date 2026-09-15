# M35-02B-03G-02 — Authenticate aggregate and partial-move places

- Status: complete
- Parent: [M35-02B-03G](M35-02B-03G-partial-owned-records.md)
- Depends on: M35-02B-03G-01
- Specification: [owned record fields](../../specification/typed-generation/languages/c/rust-owned-record-fields.md)

## Contract

Use the checked record-construction capability to authenticate one flat
record's complete ownership path, including partial moves and remaining-field
cleanup. Begin with the parent's straight-line root-scope form: distinct
parameter-rooted Box<i32> constructions and whole-local moves, one complete
record initialized from those current local owners, then field-to-local moves
and optional subsequent whole-local moves. The scalar exit reads a current
moved-out field owner. Other constructed Boxes may remain outside the record.

Represent source owner steps with a typed local/record-field enum carrying
canonical binding and declaration identities; retain real MIR Place projections.
The compiler's AggregateKind identifies the nominal record, variant and type
arguments. Aggregate operands are paired by declared FieldIdx, not source field
spelling or source initializer position.

## Definition of done and tests

- Pinned typed inspection establishes the exact aggregate/partial-move/drop
  representation before the whole-body safe entry is enabled.
- Canonical HIR preserves source initializer order; typed aggregate evidence
  preserves declaration-indexed operands. Because admitted initializers are
  pure local move operands, retain their observed staging moves in source order
  and the single aggregate assignment in declaration order. Never invent extra
  aggregate events or identify staging temporaries from names/spans.
- Each parameter-rooted chain maps through exact local moves, one aggregate
  field edge where applicable, later field-to-local moves and one final drop.
  All Box locals, record storage, assignments, calls and normal blocks are
  accounted. No whole-record, nested, borrowed or ambiguous place is accepted.
- Final current owners drop in reverse local-binding order; remaining fields
  drop at the record binding's exit in declaration order, skipping moved fields.
  Preserve the exact drop place and source scope for every chain.
- Test reversed initializer order, first/middle/last and multiple field moves,
  local renaming/shadowing, extra live owners, distinct same-spelled record
  declarations and tail/explicit returns. Consume both executable capabilities.
- Negative source/typed-place/aggregate/cleanup substitutions cover wrong
  nominal identity, field index/type/operand, missing or duplicate membership,
  wrong move/read owner, moved-field cleanup, order and omitted cleanup.
- Exact private input/path boundaries, invalid-source and nonempty controls,
  all previous ownership/native/lint gates, fresh broad review and isolated
  full exact-tree testing precede documentation, commit and push.

Nested or conditional record ownership, record update/reassignment, function
transfers, custom Drop/unwind and scalar-record Box payloads remain separate
required contracts. This compiler-only step does not enable target heap output.

## Pinned observation before admission

Gate `7f0b5876-551f-4dde-a211-3513022eac78` passed the initial two observation
targets in 13.776 seconds. Four compiler-checked fixtures establish that the
pinned PostCleanup representation stages each local initializer through a
separate Box temporary, in source initializer order, before one record aggregate
whose operands follow declaration order. Partial moves use exact Field
projections; all-field moves leave no record-field Drop. The whole-body relation
must therefore authenticate both the staging move and aggregate field edge,
and account for those extra Box locals without pretending they are HIR bindings.
This initial observation is not yet a whole-body correspondence certificate.

## Current implementation evidence

- The separate query-only RecordOwnedBody now retains typed source local/field
  paths, actual MIR places, distinct initializer staging/aggregate events and
  root-scope cleanup. No arbitrary-MIR public constructor or local-only erasure
  is exposed. Source, aggregate, constructor and relation modules stay focused.
- Nine positive forms cover first/middle/last and multiple/all field moves,
  reversed initializers, extra/local-moved owners, shadowing, one-field records,
  distinct same-spelled module records and explicit returns. Fourteen exclusions
  and a historical tail compatibility case keep the exact inventory at 24.
- Thirty-one mutated compiler bodies reject incorrect nominal/variant/argument
  identity, operand membership, staging source/order, projections, moved-field
  cleanup, cleanup order/count, read origin and other complete-body obligations.
  Three private source-plan nominal/field substitutions reject independently.
- Source controls require E0382/E0063/E0062/E0502 before proof output; valid-Rust
  field/anchor/inventory substitutions and an empty source cannot claim success.
- Focused gate `4b90af19-4848-47cd-95ab-7245e40ba4aa` passed all six targets
  in 16.734 seconds, including runtime, Rustfmt and exact E0451/E0308/E0061
  privacy/non-erasure/no-arbitrary-MIR contracts. Clippy denies warnings.
- Initial isolated tree `846c6d435011e6f9f8ef993f311f7de1f2721ca2` passed
  all 432 tests across 557 targets in gate `a6261763-6e01-4414-a2d2-d83e36fdd158`
  (45.340 seconds). The archive was verified against 2,217 exact Git blobs and
  executable modes; all twenty preserved M34 files match their prior hashes.
- Fresh Sol Extra High read-only review found no concrete correctness, privacy
  or contract defects. It noted optional boundary hardening: G-02's independent
  128-statement budget allows at most 126 fields after construction/one extraction,
  unlike G-01's isolated 128-field operation limit. The specification now states
  both budgets explicitly. G-02 does not promise every G-01 operation can fit in
  a whole admitted body; budget rejection remains intentional, not approximation.
- The final closure tree and full-gate invocation are recorded in the checkpoint
  commit message. No target heap output is enabled; remaining boxed-record,
  conditional-initialization and function-boundary work stays open.
