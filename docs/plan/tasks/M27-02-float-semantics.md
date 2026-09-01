# M27-02 — Portable truncation and exact test equality

- Status: complete

## Goal

Add the reusable binary64 operation and test semantics required to express and
prove parse-ms without changing language-visible IEEE equality.

## Definition of done

- `FloatTrunc` is a serialized intrinsic with checked signature `F64 -> F64`.
- The evaluator and all eight target mappings truncate toward zero and preserve
  signed zero, NaN, and infinities.
- `Equal`, `NotEqual`, and `ListContains` retain IEEE numeric equality.
- Portable expectations recursively distinguish signed zero and accept NaN
  expectations inside records and collections.
- F64 bit payloads remain lossless through every generated runtime/test table.

## Tests

- IR serialization and exhaustive checker signature tests.
- Evaluator vectors for finite truncation and explicit IEEE-vs-test equality.
- All backend generator unit tests and generated native fixture tests.
- parse-ms portable vectors for positive/negative zero, NaN, infinities,
  fractions, boundaries, and maximum finite binary64.
