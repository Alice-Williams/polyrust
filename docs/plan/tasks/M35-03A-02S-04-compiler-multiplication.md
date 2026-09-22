# M35-03A-02S-04 — Checked Rust wrapping-multiplication integration

- Status: planned
- Parent: [02S](M35-03A-02S-wrapping-multiplication.md)
- Depends on: [Java foundation](M35-03A-02S-03-java-multiplication.md)
- Specification: [shared](../../specification/typed-generation/rust-wrapping-multiplication.md)

## Contract

Authenticate core primitive wrapping_mul in canonical HIR/TypeckResults using a
private operation-specific witness. Register required executable target mappings.
Fully materialize the original left operand before the right, exactly once each.

## Definition of done and tests

Three source crates and external consumers agree with independent truth/native
Rust at both widths; measured operand traces detect value-preserving dropped,
duplicated and reordered calls. Exact original APIs, imports, visibility and docs
have mutation-sensitive controls. Missing/duplicate/wrong binding and private
witness construction fail compilation. Actual typed dataflow and hostile witness
probes detect faults; probe/production bytes agree. Native composition covers
method/associated calls, nesting, grouping and same-spelled ordinary functions.
Unsupported widths, borrows, casts, trait/generic lookalikes, ordinary * and other
unsupported operations reject atomically. Update obsolete unsupported wrapping_mul
fixtures to a still-unsupported operation without disabling their gates. Export
actual packages, preserve old output/WIP, pass full gate and fresh review, then
commit/push. Wider runtime parity remains incomplete.
