# M29-02 — Add portable byte replacement

- Status: in-progress

## Goal

Add reusable literal byte-sequence replacement rather than hiding CRLF handling
in a project-specific helper.

## Definition of done

- `BytesReplaceAll` checks only as `Bytes × Bytes × Bytes -> Bytes`.
- Semantics are global, left-to-right, non-overlapping, and literal.
- Empty needles insert at every byte boundary, including both ends.
- The evaluator and all eight targets implement the same behavior through
  language IR.
- Focused conformance covers overlap, binary zero/high bytes, empty operands,
  growth, shrinkage, and no-match identity.

## Tests

- Focused IR, checker, evaluator, and eight-backend Bazel tests.
