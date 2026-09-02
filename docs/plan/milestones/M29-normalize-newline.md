# M29 — normalize-newline equivalence port

- Status: complete
- Phase: 6
- Depends on: M24, M26, M28

## Outcome

Port the complete typed value behavior of MIT-licensed `normalize-newline`
5.0.0 into one checked PolyRust model and generate equivalent Rust,
TypeScript, JavaScript, Python, Go, Java, C++, and C packages.

The upstream overloads `String -> String` and
`Uint8Array -> Uint8Array` become two explicit portable functions because
targets without overloads cannot expose one type-preserving union function.
Together they retain every valid typed input value, including arbitrary
non-UTF-8 bytes. Invalid JavaScript values in the runtime test are outside the
declaration and remain an explicit admission boundary.

## Implementation checklist

- Retain implementation, declaration, runtime test, package metadata, README,
  and MIT license at commit
  `bc6982d73ebd62de3729435d9baf8731ca274f7a`.
- Verify every retained byte with its Git blob hash.
- Add general literal, global, non-overlapping `BytesReplaceAll` semantics
  instead of a newline-specific primitive.
- Reuse `StringReplaceAll` for the text overload.
- Retain all 13 official valid assertions and add arbitrary binary, overlap,
  boundary, empty, large, and repeated-match vectors.
- Differentially compare both portable functions with the pinned JavaScript
  implementation over deterministic text and byte corpora.
- Generate and run all eight native packages, target linters/formatters,
  conformance tests, public consumers, and C/C++ sanitizers.

## Required exit evidence

- Provenance tests pass for every retained upstream blob.
- Checker and evaluator tests cover `BytesReplaceAll`, including empty needle
  semantics and invalid signatures.
- Every backend has focused `BytesReplaceAll` lowering coverage.
- All portable vectors pass in the evaluator and eight generated targets.
- The differential oracle reports every comparison passing.
- Three complete generations are byte-identical.
- Uncached `//...` and `//:release_gate` tests pass in the Linux development
  container, including Buildifier, Rustfmt, and Clippy.

## Scope boundary

M29 proves value equivalence. PolyRust `Bytes` is immutable, so generated APIs
do not reproduce JavaScript typed-array object identity or mutability. Version
5 always returns a fresh `Uint8Array`, and no admitted result value is lost.

## Local completion evidence

- The focused M29 suite plus repository lint gates pass 18/18 tests, including
  Buildifier, Rustfmt, Clippy, provenance, evaluator, deterministic generation,
  differential comparison, eight native packages, and C/C++ sanitizers.
- All 31 portable vectors pass in the evaluator and every generated target.
- The differential oracle agrees on 9,338 deterministic text inputs and 31,847
  deterministic byte inputs.
- Three complete eight-target generations are byte-identical.
- The complete uncached repository gate passes 198/198 tests.
- The complete uncached release gate passes 176/176 tests.
