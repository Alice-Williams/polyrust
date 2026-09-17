# M35-03A-02O-04 — Checked Rust binary64 remainder

- Status: planned
- Parent: [02O](M35-03A-02O-floating-remainder.md)
- Depends on: 02O-03

## Contract

Add a private canonical RemainderInput and an executable FloatingRemainder
capability slot. Require original HIR/TypeckResults, built-in Rem identity,
exact unadjusted f64 operands/result. Materialize left then right once.
C constructs the typed catalogue call; Java constructs the structural operator.

## Definition of done and tests

Three actual Rust crates generate ordinary C/Java packages. Native results and
traces agree with Rust and the independent oracle; exact manifests/visibility,
original imports and certificate-derived C library closure hold. AST probes
reject copied input/wrong context and detect actual operator/call/dataflow
mutations; probe bytes equal production. Missing/duplicate/wrong binding and
private-witness compile failures are exact. Integer %, f32, overloads, casts,
mutable assignment and rem_euclid reject atomically. Export real examples.
Full gate, old hashes, preserved WIP and two clean reviews precede completion.
