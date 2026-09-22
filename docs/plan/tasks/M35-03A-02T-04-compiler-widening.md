# M35-03A-02T-04 — Checked Rust signed-widening integration

- Status: planned
- Parent: [02T](M35-03A-02T-signed-widening.md)
- Depends on: [Java foundation](M35-03A-02T-03-java-widening.md)
- Specification: [shared](../../specification/typed-generation/rust-signed-widening.md)

## Contract

Authenticate canonical HIR Cast and original TypeckResults with exact unadjusted
i32 operand and i64 result. Use a private operation witness and a required
executable SignedWidening mapping slot. Preserve one original operand evaluation,
declaration/call identities, crate/module ownership, public/private API and docs.

## Definition of done and tests

Three original Rust crates and separately built C/Java external consumers agree
with independent/native truth. Measure native source and target operand traces;
detect dropped/duplicated calls even when values agree, and compiling value
faults. Cover literals, original local/imported calls, records and composition
with existing scalar operators. Typed probes verify exact target shape, original
operand dataflow, hostile copied/context/width witnesses and identical production
bytes. Missing/duplicate/wrong bindings and private witness construction fail
compilation. Unsupported casts reject atomically without changing old outputs.
Update older rejection fixtures only if they reject the newly supported exact
widening operation; maintain each fixture's intended failure. Export actual
packages, update partial inventory, preserve WIP, pass full gate and fresh review,
then commit/push. Checked narrowing and arbitrary From/Into calls stay outside.
