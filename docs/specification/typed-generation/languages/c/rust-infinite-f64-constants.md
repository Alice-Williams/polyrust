# Rust signed-infinity constants in C17

- Status: target foundation and checked source integration complete
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

## Concrete typed mapping

`CKnownConstant::DoubleInfinity` owns the F64 type and arithmetic-constant
category. The standard catalogue binds that identity to `HUGE_VAL` and
`CHeader::Math`; the dependency visitor requests the header from the node.
The renderer only spells that identity or recursively renders typed negation.
There is no caller-supplied macro text or separate import list in lowering.

`CScalarConstantValue` has Bool, I32, I64, finite F64 and Infinity sign variants.
Certificate-derived constant views and dependency descriptors carry this
inventory value, not an alleged nonfinite `CLiteral`. Its exact initializer
projection accepts no folding, casts, double negation or arbitrary macros.
The finite-literal projection returns None for infinity. Ownership numeric
analysis evaluates both owned syntax and imported evidence as exact signed
infinities; integer conversions remain rejected by numeric safety checking.

The supported execution profile remains pinned Linux LP64 GCC14/Zig with
binary64 semantics, not every C17 implementation whose double has eight bytes.
Existing generated radix/mantissa/exponent/subnormal/evaluation assertions are
retained, and native exact-bit tests establish both HUGE_VAL signs for the
pinned compilers. Floating expressions are not inserted into `_Static_assert`
as if they were integer constant expressions.

Checked source joins now compare the complete CScalarConstantValue, including
the infinity sign, against the original compiler definition and producer
certificate. Manifests serialize exact infinity bits without a finite-literal
projection. Local declarations remain compile-time declarations; ordinary
reads lower to typed values and public reads retain original object identity.
Scalar call-effect derivation recognizes only the DoubleInfinity known
constant as a storage-free leaf; missing callees and standard-stream pointers
still cannot obtain body-derived evidence.

## Required evidence

Compile real certified owned and imported declarations and generated readers
with GCC14/Zig at O0/O2, strict warnings and standalone headers; run GCC UBSan.
Recover bits via test-only memcpy. Check both signs, exact symbol/header
dependencies and no libm linkage when no math function is called. Reject
lookalike owners, same-type wrong values/signs, forged descriptors and missing
imports. Previously finite packages must retain identical bytes.
