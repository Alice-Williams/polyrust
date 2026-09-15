# M35-02B-03G — Authenticate owned record fields and partial moves

- Status: in-progress
- Parent: [M35-02B-03](M35-02B-03-structured-owned-places.md)
- Depends on: M35-02B-03F
- Specification: [partial owned records](../../specification/typed-generation/languages/c/rust-owned-record-fields.md)

## Contract

Extend correspondence from local Box owners to compiler-typed record fields.
Start with one non-generic local named-field struct whose fields are Box<i32>,
no custom Drop, and a straight-line root body. Construct each Box from a distinct
immutable i32 parameter, move those owners into one complete record initializer,
then move one or more fields into local bindings. Return a scalar dereference of
one moved-out current owner. Remaining fields must receive exactly their actual
compiler cleanup in Rust field declaration order.

Keep source record/field identities, initializer evaluation order, local binding
scope and MIR places/projections distinct. Equal field types, field spellings or
matching counts do not authenticate correspondence.

## Definition of done and tests

Begin with [G-01 — Record construction identities](M35-02B-03G-01-record-construction-identities.md).
Individual source operation admission is not whole-body aggregate/cleanup
correspondence and cannot close this parent by itself.

- Inspect pinned HIR/typed aggregate operands/projected Drop before admission.
  Use compiler AdtDef/DefId, FieldIdx, Ty and Place projections, with a private
  source record-construction capability input and executable typed binding.
- Authenticate complete initializer field membership and source evaluation
  order independently of declaration order. Every field is initialized once
  from the correct parameter-rooted owner chain.
- Partial moves map the exact declared field into its new local; all remaining
  live fields are dropped once, in declaration order, after later local owners.
  No moved-out field is dropped again through the aggregate.
- Test reversed initializer order, moving first/middle/last fields, multiple
  partial moves, renamed fields/locals, same-type records, extra live owners,
  tail/explicit returns and source/private capability boundaries.
- Wrong record/field/type/order, missing/duplicate initializer operands,
  wrong projected moves/drops, moved-field cleanup and omitted cleanup must
  fail independent non-vacuous relation and native-compiler-source controls.
- Unsupported nested/conditional aggregates, custom Drop, update syntax,
  references, arrays and call-boundary transfers diagnose. Prior readers retain
  their closed grammars; no backend gains heap output from this proof.
- Fresh independent review, full isolated exact-tree historical/native/lint
  gates, documentation and a dedicated commit/push close this increment.

Boxes containing scalar records, nested owned records, conditional partial
initialization/moves and function-boundary transfers remain required additional
work before the parent ownership mapping can close.
