# M33-02 — Add portable negative-zero predicate

- Status: pending

## Goal

Represent the IEEE-754 negative-zero classification explicitly without relying
on ordinary equality, reciprocal side effects, or target-specific bit access.

## Definition of done

- `FloatIsNegativeZero` checks only as `F64 -> Bool`.
- It returns true exactly for raw binary64 bits `0x8000000000000000`.
- Positive zero, finite nonzero values, subnormals, infinities, and every NaN
  payload/sign return false.
- The evaluator and all eight outputs implement identical semantics through
  dependency-complete target fragments and helper closure.
- Focused tests cover serialization, valid and invalid signatures, both zeros,
  positive/negative normal and subnormal values, infinities, and signed NaNs.
- Exact dependency matrices prove that the feature adds only requirements
  owned by its mapping and removes them when the operation is absent.

## Tests

- Focused IR, checker, evaluator, conformance, and eight-backend Bazel tests.
