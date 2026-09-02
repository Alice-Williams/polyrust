# M34-02 — Add portable binary64 absolute value

- Status: complete

## Goal

Represent absolute value as one exact IEEE-754 operation without relying on
target-specific comparison, signed-zero, or NaN behavior.

## Definition of done

- `FloatAbs` checks only as `F64 -> F64`.
- Its result bits equal `input_bits & 0x7fff_ffff_ffff_ffff`.
- The evaluator and all eight outputs implement identical semantics through
  dependency-complete target fragments and helper closure.
- Focused tests cover serialization, valid and invalid signatures, both zeros,
  positive/negative normal and subnormal values, infinities, and signed NaNs.
- Exact dependency matrices prove that the feature adds only requirements
  owned by its mapping and removes them when the operation is absent.

## Tests

- Focused IR, checker, evaluator, conformance, and eight-backend Bazel tests.

## Completion evidence

- The IR round-trip and spelling test covers the 70th intrinsic,
  `float_abs`.
- Checker tests prove the sole `F64 -> F64` signature and reject zero,
  wrong-typed, and two-argument invocations with `InvalidInvocation`.
- Evaluator cases prove exact sign-bit clearing for signed zeros, normal and
  subnormal values, infinities, signaling NaNs, quiet NaNs, and positive
  values.
- Rust, TypeScript/derived JavaScript, Python, Go, Java, C++, and C mappings
  use representation-preserving lowerings. Focused dependency tests prove no
  unowned import or helper is added and optional F64 closures remain absent
  from minimal packages.
- Three canonical exact-bit vectors pass through the evaluator and every
  generated target. The focused core/all-backend suite passes 33/33 tests and
  the canonical all-target conformance run passes 31/31 tests in the Linux
  development container.
