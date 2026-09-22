# M35-03A-02R-04 — Checked Rust wrapping-subtraction integration

- Status: planned
- Parent: [02R](M35-03A-02R-wrapping-subtraction.md)
- Depends on: [Java foundation](M35-03A-02R-03-java-subtraction.md)
- Specification: [shared](../../specification/typed-generation/rust-wrapping-subtraction.md)

## Contract

Authenticate actual core i32/i64 wrapping_sub in canonical HIR/TypeckResults.
Register a private-input WrappingSubtraction mapping per target through typed
builders. Fully materialize original left before right, once each.

## Definition of done and tests

Multi-crate native Rust/C/Java values and measured traces agree with independent
truth. Exact original API/import/doc inventories and external privacy controls
pass, including deliberate corruptions. Missing/duplicate/wrong mapping bindings
and private witness construction fail compilation. Typed AST/dataflow and hostile
canonical/context/child probes detect faults; production/probe bytes match.
Method/associated calls, nesting, grouping and same-spelled ordinary functions
have positive composition coverage. Unsupported widths, borrowed receivers,
casts, generics/traits, ordinary subtraction and unrelated operations reject
atomically. Export actual packages outside Docker. Preserve old outputs/WIP,
pass the full gate and clean fresh review, then commit/push separately.
