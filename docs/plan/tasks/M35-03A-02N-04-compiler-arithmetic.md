# M35-03A-02N-04 — Checked Rust binary64 arithmetic

- Status: planned
- Parent: [binary64 arithmetic](M35-03A-02N-floating-arithmetic.md)
- Depends on: 02N-02 and 02N-03

## Contract

Add a private canonical-HIR ArithmeticInput and closed FloatingArithmeticOperator
enum with Add, Subtract, Multiply and Divide. Prove original typeck, built-in
operator identity, exact f64 operands/result and absent adjustments.
FloatingArithmetic must be a real executable builder slot for both targets.
Lower/materialize left then right exactly once; preserve AST grouping.

## Definition of done and tests

Three original Rust crates generate ordinary C/Java packages with correct
visibility, original imports and deterministic metadata. Compare native outputs
to Rust and 02N-01; verify traces independently with compiled fault controls.
Exact typed AST/dataflow and canonical-witness probes pass, including copied
HIR/wrong context negatives. Missing/duplicate/wrong capability contracts fail
with exactly their intended compile diagnostic; backend-free positive probes
remain complete. Unsupported widths, overloads, assignments, casts, remainder
and fused methods reject atomically. Export real generated examples and retain
old bundle hashes. Full release/lint gates and fresh independent review pass.
