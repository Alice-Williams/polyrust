# M32-02 — Add portable literal substring first-index lookup

- Status: complete

## Completion evidence

- The 67th serialized v0 operation has the sole checked signature
  `String × String -> Option<I64>`.
- Evaluator cases cover empty, absent, prefix, middle, suffix, overlap, NUL,
  combining, BMP, and astral behavior.
- Every target selects and executes its own dependency-complete lowering; all
  32 portable vectors and 58,274 upstream comparisons pass.

## Goal

Represent leftmost literal substring search with an explicit absence value and
one target-independent Unicode index unit.

## Definition of done

- `StringIndexOfLiteral` checks only as
  `String × String -> Option<I64>`.
- It returns the scalar offset of the leftmost case-sensitive, exact match,
  `Some(0)` for an empty needle, and `None` when absent.
- The evaluator and all eight outputs implement identical well-formed Unicode
  behavior through dependency-complete target fragments and helper closure.
- Focused tests cover empty values, absent/prefix/middle/suffix matches,
  overlapping matches, combining sequences, BMP, astral scalars, NUL, and
  invalid arity/types.

## Tests

- Focused IR, checker, evaluator, conformance, and eight-backend Bazel tests.
