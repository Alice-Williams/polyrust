# M35-03A-02S-03 — Java wrapping-multiplication foundation

- Status: planned
- Parent: [02S](M35-03A-02S-wrapping-multiplication.md)
- Depends on: [C foundation](M35-03A-02S-02-c-multiplication.md)
- Specification: [Java21](../../specification/typed-generation/languages/java/rust-wrapping-multiplication.md)

## Contract

Extend dependency-body certification for exact primitive Int/Long Multiply with
Multiplicative precedence and equal operand/result types. No boxing, promotion,
Math.multiplyExact or custom helper. Preserve all child/dependency/budget checks.

## Definition of done and tests

Both widths certify and compile through separately built Java21 producer,
forwarder and external client. Oracle agreement and compiling wrong-operation,
saturation, narrowing and disconnected-operand controls pass. Mixed widths,
boxed/String/Boolean values, wrong result/precedence and unrelated operators
reject. Existing depth, call-height, arity, source-byte and both-child authority
gates stay active. Preserve existing bytes, full gate and clean independent
review; no compiler-source admission yet.
