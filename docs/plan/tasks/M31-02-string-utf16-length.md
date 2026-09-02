# M31-02 — Add portable UTF-16 code-unit length

- Status: complete

## Completion evidence

- IR serialization, checker signatures, evaluator semantics, conformance
  vectors, and all eight target implementations cover the operation.
- Boundary tests cover empty, ASCII, BMP, combining, supplementary, and mixed
  values; checker tests reject non-string and wrong-arity calls.
- `//examples/real-world/has-flag:all` proves the BMP, combining, and astral
  behavior in every generated language.

## Goal

Model JavaScript string length exactly with a reusable, explicitly named
operation rather than conflating code units, Unicode scalars, and UTF-8 bytes.

## Definition of done

- `StringUtf16Length` checks only as `String -> I64`.
- Its result is the number of code units in the string's well-formed UTF-16
  encoding; BMP scalars count as one and supplementary scalars count as two.
- The evaluator and all eight outputs implement identical semantics through
  dependency-complete target fragments and helper closure.
- Focused tests cover empty, ASCII, BMP, combining, supplementary, and mixed
  strings plus invalid signatures.

## Tests

- Focused IR, checker, evaluator, and eight-backend Bazel tests.
