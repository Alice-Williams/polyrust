# M31-03 — Add portable list first-index lookup

- Status: planned

## Goal

Add a reusable equality-based first-index operation without sentinel integers
or target-native coercion.

## Definition of done

- `ListIndexOf` checks as `List<T> × T -> Option<I64>` for every type eligible
  for portable structural equality.
- It returns the zero-based first matching index or `None`; duplicate values do
  not select a later match.
- Equality follows existing PolyRust value equality, including IEEE `F64`
  behavior, and never JavaScript coercion or object identity.
- The evaluator and all eight outputs implement identical behavior through
  dependency-complete fragments and helper closure.
- Focused tests cover empty, absent, first, middle, duplicate, nested value,
  NaN, negative zero, and invalid signature cases.

## Tests

- Focused IR, checker, evaluator, and eight-backend Bazel tests.
