# M32-03 — Add portable scalar-range string slicing

- Status: pending

## Goal

Extract a reusable string range without leaking UTF-8 byte or UTF-16 code-unit
offsets into the portable model.

## Definition of done

- `StringSliceScalars` checks only as
  `String × I64 × I64 -> String`.
- Each endpoint clamps independently to the inclusive range from zero through
  scalar length; start greater than or equal to end returns an empty string.
- Slicing never splits a Unicode scalar and preserves the exact selected
  scalar sequence.
- The evaluator and all eight outputs implement identical behavior through
  dependency-complete target fragments and helper closure.
- Focused tests cover empty input, full/empty ranges, negative and oversized
  endpoints, reversed endpoints, combining sequences, BMP, astral scalars,
  NUL, and invalid arity/types.

## Tests

- Focused IR, checker, evaluator, conformance, and eight-backend Bazel tests.
