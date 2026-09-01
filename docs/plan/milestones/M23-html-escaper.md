# M23 — Port html-escaper with simultaneous replacement proof

- Status: complete
- Phase: 6 compatibility continuation
- Depends on: M20, M21, M22A, M22B string-runtime checkpoint

## Outcome

Reimplement the complete typed behavior of the MIT-licensed
`WebReflection/html-escaper` 3.0.3 package at implementation revision
`c6e2b50d7b6f486afb3ddc92bfcfec89857b75d7` and declaration revision
`cd61c555bfc93e985b313263a42ed78074570d08` as checked PolyRust `escape`
and `unescape` functions, then generate equivalent packages in all eight
supported languages.

## Implementation checklist

- Pin the immutable MIT license, ESM source, official tests, and typed API.
- Specify a reusable ordered simultaneous literal-replacement intrinsic.
- Implement and test the intrinsic in the checker, evaluator, and all eight
  generated runtimes without rescanning replacement text.
- Carry every official assertion plus entity, boundary, Unicode, control,
  priority, and nested-encoding vectors into permanent portable evidence.
- Differentially compare both generated functions with the pinned upstream.
- Run native format, lint/static, compile, sanitizer, and test gates for every
  generated package and prove deterministic regeneration.

## Required exit evidence

- The complete `escape(str: string): string` and
  `unescape(str: string): string` typed domain is represented.
- All five escape mappings and all ten accepted unescape spellings pass in the
  evaluator and every generated target.
- Nested encodings prove replacements are simultaneous and non-recursive.
- A broad deterministic corpus and large boundary inputs match the pinned ESM
  oracle for both public functions.
- Three generations are byte-identical and the full release gate, including
  Rust and Bazel linters, passes.

## Scope boundary

The retained type declaration accepts only strings. JavaScript runtime
coercion of booleans and numbers, and exceptions for other dynamic values, are
outside that typed domain. This compatibility milestone uses the completed
M22B string ABI and does not claim completion of the remaining C aggregate,
control-flow, or callable lowering work.

## Completion evidence

- The retained implementation, official tests, and MIT license match their
  pinned Git blobs; the retained declaration matches its independent pinned
  DefinitelyTyped blob.
- Both complete string APIs are one checked program with 42 portable vectors
  and byte-identical manifests for all eight targets.
- The reusable 60th v0 intrinsic has valid and invalid checker coverage,
  evaluator priority/Unicode/empty-needle vectors, and implementation coverage
  in every generated runtime.
- The differential oracle passes 108,498 escape/unescape comparisons over
  54,249 unique strings, including exhaustive four-token entity fragments and
  two large repeated inputs.
- Fresh generated packages pass their native formatter/style, linter/static
  checker, compiler, tests, and C/C++ sanitizer gates.
- `bazelisk test //... --test_output=errors` passes 138/138 tests and
  `bazelisk test //:release_gate --test_output=errors` passes 116/116 tests,
  including Buildifier, Rustfmt, and Clippy.
- The full result is documented in
  [the compatibility-port report](../../ports/html-escaper.md).
