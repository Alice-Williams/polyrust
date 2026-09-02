# M34-02 — Add portable binary64 absolute value

- Status: planned

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
