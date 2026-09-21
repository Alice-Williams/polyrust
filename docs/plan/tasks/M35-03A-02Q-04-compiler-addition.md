# M35-03A-02Q-04 — Checked Rust wrapping-addition integration

- Status: planned
- Parent: [02Q](M35-03A-02Q-wrapping-addition.md)
- Depends on: [Java foundation](M35-03A-02Q-03-java-addition.md)
- Specification: [shared](../../specification/typed-generation/rust-wrapping-addition.md)

## Contract

Authenticate the actual primitive core operation in canonical HIR/TypeckResults.
Register a private-input executable WrappingAddition mapping per language through
the typed builder. Materialize original left then right exactly once.

## Definition of done and tests

Actual multi-crate Rust generates normal C/Java packages. Native modular results
and per-input producer traces agree with Rust and the independent oracle; exact
original APIs, imports, docs and external privacy controls pass. Missing,
duplicate/wrong bindings and private witness construction fail compilation.
Canonical/input/context and actual typed AST/dataflow probes detect faults,
including operand reversal despite commutative results. Probe/production bytes
match. Unsupported widths, casts, borrowed receivers, generic/trait lookalikes,
ordinary + and other wrapping operations reject atomically. Existing ordinary
functions named wrapping_add keep normal direct-call semantics.
Export actual examples, preserve older output/WIP hashes, pass the full isolated
release/lint gate and fresh review, then commit/push separately.
