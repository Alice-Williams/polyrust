# Rust finite binary64 constants in C17

- Status: target foundation and checked source integration complete
- Contract: [shared](../../rust-finite-f64-constants.md)

## Target foundation

Permit exactly finite CLiteral::F64 initializers for const Scalar(F64) objects
in the shared constant profile. Retain public-header identity, definition
linkage, exact declared type, certified platform requirements, import authority
and all source/stack/resource bounds. Extend the corresponding dependency
constant inventory and views without weakening other scalar checks.

Use the existing typed hexadecimal binary64 literal renderer. Ordinary public
header declarations and source definitions must compile separately; there is
no runtime file or dynamic initialization. C const objects are not promised to
be integer constant expressions or usable as array extents.

## Source lowering and proof

The compiler constant variant maps to CLiteral::F64 without host arithmetic.
Owned, local and imported reads preserve the existing storage/identity rules.
Original producer comparisons use exact finite bits, including zero sign.

Owned constants, constant imports and constant exports all select binary64
manifest schema 8, including function-free owners; system linkage selects 9.

Require native GCC14/Zig at O0/O2, strict warnings, standalone headers and UBSan;
external clients read actual declared objects/functions and compare bits.
Reject wrong type/initializer/linkage/provenance and forged import authority.
Retain previous bool/i32/i64 constant evidence and resource accounting. Source
admission occurs only after this foundation and the Java foundation are gated.
