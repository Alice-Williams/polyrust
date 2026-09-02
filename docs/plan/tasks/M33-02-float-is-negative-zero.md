# M33-02 — Add portable negative-zero predicate

- Status: complete

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

## Completion evidence

- The IR round-trip and spelling test covers the 69th intrinsic.
- Checker tests prove the sole `F64 -> Bool` signature and reject zero, wrong
  typed, and two-argument invocations with `InvalidInvocation`.
- Evaluator cases cover both zeros, positive and negative normal/subnormal
  values, both infinities, and positive/negative NaN payloads.
- Rust, TypeScript/derived JavaScript, Python, Go, Java, C++, and C mappings
  have focused lowering and dependency-closure assertions. Minimal runtimes
  prove removal of the optional F64 closure where applicable; targets whose
  interpreter core already owns numeric dispatch prove the operation adds no
  import or helper requirement.
- Uncached core and all-backend Bazel run: 33/33 tests passed in the Linux
  development container.
