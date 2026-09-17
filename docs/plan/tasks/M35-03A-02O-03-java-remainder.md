# M35-03A-02O-03 — Java binary64 remainder foundation

- Status: planned
- Parent: [02O](M35-03A-02O-floating-remainder.md)
- Depends on: 02O-02

## Contract

Admit the structural JavaBinaryOperator::Remainder with primitive Double
operands/result and Multiplicative precedence. Preserve recursive dependency
authority and byte/resource bounds. Do not use Math.IEEEremainder or a runtime.

## Definition of done and tests

Separate producer/importer/client compilation under Java21 strict lint agrees
with the independent remainder oracle. Compiling value/trace faults are detected.
Private-reader and public-certification tests reject wrong types, precedence,
arity, recursively unadmitted operands and unauthenticated imports. Existing
integer remainder rejection remains intact. Full gate and fresh review pass.
