# Rust finite binary64 constants in Java21

- Status: target foundation and checked source integration complete
- Contract: [shared](../../rust-finite-f64-constants.md)

## Target foundation

Permit exactly primitive Double fields initialized by a finite JavaLiteral::F64
under the existing public/static/final source-constant certificate. Preserve
original registration, source identity, resolved owner, exact type, visibility
and producer dependency authority. Do not admit boxed Double, Float, nonfinite
literals or arbitrary initializer expressions.

Use existing structural rendering and ordinary fields; no custom runtime,
helper catalogue, reflection or boxing. Certification and byte bounds continue
to account for the entire field and its finite literal. The shared constant
inventory now uses JavaScalarConstantValue::F64; its literal projection retains
exact bits. Infinity is a separate target-foundation variant and remains outside
this finite source capability.

## Source lowering and proof

Map the distinct compiler constant variant to TypePlan::F64 and the existing
finite Java literal. Preserve original declaration/alias bindings and compare
producer values by bits, not Java numeric equality.

Owned constants, constant imports and constant exports all select binary64
manifest schema 6, including function-free producers and alias-only owners.

Require separate Java21 compilation with all warnings treated as errors,
normal and interpreted execution, exact raw-bit observations in external test
clients and compiling mutation controls. Verify imports cannot swap in a
lookalike producer. Java may inline constant fields: actual Bazel dependency
invalidation must rebuild affected consumers after a producer-value change.
Source admission follows independently gated C and Java target foundations.
