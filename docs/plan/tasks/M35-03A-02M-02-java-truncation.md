# M35-03A-02M-02 — Certified Java rounding-call foundation

- Status: planned
- Parent: [truncation](M35-03A-02M-floating-truncation.md)
- Depends on: [C target proof](M35-03A-02M-01-c-truncation.md)

## Contract

Admit only typed JavaKnownCallable::MathFloor and MathCeil with exact
(double)->double signatures, no receiver and Primary precedence. Retain normal
symbol resolution, call argument traversal and original dependency authority.
Extend the source reservation to account for the actual resolved standard
callable names; do not turn off source bounds or allow arbitrary known calls.

## Definition of done and tests

Certify producer/imported-owner packages and compile with Java21 strict lint.
Check floor, ceil and conditional truncation against independent integer-bit
oracles, including signed zeros, subnormals, fractions, infinities and NaNs.
Detect swapped rounding branches, zero-sign loss and dropped/duplicated receiver
calls. Reject malformed signatures, operands, results, precedence, receiver,
unadmitted known callables and forged dependencies. Actual rendered bytes fit
the computed reservation. Full Linux Bazel release/lint gates and independent
review pass; record exact tree and invocation before commit.
