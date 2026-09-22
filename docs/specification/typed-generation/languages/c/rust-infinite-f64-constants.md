# Rust signed-infinity constants in C17

- Status: planned
- Contract: [shared](../../rust-infinite-f64-constants.md)

## Representation and catalogue

Use the typed standard double constant HUGE_VAL for positive infinity and a
typed F64 unary-negation node over that exact constant for negative infinity.
The existing binary64 execution profile must establish infinity support; do
not infer it from sizeof(double) alone. The standard macro's double type and
IEEE infinity behavior are described by [math.h](https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/math.h.html).
Add its closed CKnownConstant identity, scalar type, constant-expression
classification, dependency/header and spelling to the existing catalogues.
Header selection remains symbol-derived; no unconditional math.h include or
invented -lm requirement. No generated runtime or support translation unit.

## Certification and constant authority

Introduce a closed C constant-value inventory that distinguishes existing
literal values from Infinity(Binary64Sign). Accept only the exact known
standard constant or its exact typed negation for the new branch. Reject
other macros, arbitrary expressions, wrong types/casts, mutable storage and
nonconstant initializers. Retain public-header/definition linkage, owner
registration, original producer references and output/resource bounds.
Imported numeric facts must accurately permit infinity; never label it finite.
Ordinary const double objects are not integer constant expressions.

## Required evidence

Compile real certified owned and imported declarations and generated readers
with GCC14/Zig at O0/O2, strict warnings and standalone headers; run GCC UBSan.
Recover bits via test-only memcpy. Check both signs, exact symbol/header
dependencies and no libm linkage when no math function is called. Reject
lookalike owners, same-type wrong values/signs, forged descriptors and missing
imports. Previously finite packages must retain identical bytes.
