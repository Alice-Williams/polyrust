# M28-02 — Add portable float-is-NaN semantics

- Status: complete

## Goal

Add a reusable IEEE-754 classification operation rather than hiding the
upstream predicate in a project-specific backend helper.

## Definition of done

- `FloatIsNaN` serializes in the IR and checks only as `F64 -> Bool`.
- The evaluator returns true for every NaN payload/sign and false for finite
  values, signed zeros, and infinities.
- All eight targets lower the operation natively through language IR.
- Capability, conformance, and backend-focused tests cover the operation.

## Tests

- Focused IR, checker, evaluator, and eight-backend Bazel tests.
- Generated native tests covering NaN and every non-NaN IEEE edge class.
