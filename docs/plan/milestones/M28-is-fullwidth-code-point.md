# M28 — is-fullwidth-code-point equivalence port

- Status: complete
- Phase: 6
- Depends on: M24, M26, M27

## Outcome

Port the complete typed behavior of MIT-licensed
`is-fullwidth-code-point` 3.0.0 into one checked PolyRust model and generate
equivalent Rust, TypeScript, JavaScript, Python, Go, Java, C++, and C packages.

Version 3 is selected deliberately: its declared API is the exact portable
`number -> boolean` boundary and its Unicode range implementation is
self-contained. Later releases delegate classification to a separately
versioned Unicode-width package, so they are a distinct multi-upstream case.

## Implementation checklist

- Retain the immutable upstream implementation, declaration, type test, runtime
  test, package metadata, README, and MIT license at commit
  `80e5e314d86e5f76bd1b0573aa9d33e615a372db`.
- Verify every retained byte with its Git blob hash.
- Add the general `float_is_nan` intrinsic with checked `F64 -> Bool`
  semantics and native lowering in every target.
- Express the complete classifier from `float_is_nan`, ordered comparisons,
  equality, and short-circuit Boolean operations; do not add a
  project-specific Unicode-width intrinsic.
- Retain all six official assertions and add exhaustive range-boundary, hole,
  NaN, infinity, signed-zero, fractional, and out-of-Unicode-domain vectors.
- Differentially compare the pinned JavaScript implementation with generated
  TypeScript over every boundary neighborhood plus a deterministic broad
  numeric corpus.
- Generate and run all eight native packages, target linters/formatters,
  conformance tests, public consumers, and C/C++ sanitizers.

## Required exit evidence

- Provenance tests pass for every retained upstream blob.
- Checker and evaluator tests cover `float_is_nan`, including invalid
  signatures and IEEE edge classes.
- Every backend has focused `float_is_nan` lowering coverage.
- All portable vectors pass in the evaluator and eight generated targets.
- The differential oracle reports every comparison passing.
- Three complete generations are byte-identical.
- Uncached `//...` and `//:release_gate` tests pass in the Linux development
  container, including Buildifier, Rustfmt, and Clippy.

## Scope boundary

M28 reproduces version 3.0.0 exactly, including its pinned Unicode range table
and JavaScript-number behavior for fractional and non-code-point inputs. It
does not claim the newer Unicode database used by later releases.

## Local completion evidence

- The focused M28 gate passes 18/18 tests uncached, including Buildifier,
  Rustfmt, Clippy, provenance, evaluator, deterministic generation,
  differential comparison, eight native packages, and C/C++ sanitizers.
- The differential oracle agrees on 22,409 deterministic inputs.
- The complete uncached repository gate passes 183/183 tests.
- The complete uncached release gate passes 161/161 tests.
- Hosted GitHub CI run
  [33547403633](https://github.com/Alice-Williams/polyrust/actions/runs/33547403633)
  passes for implementation commit
  `2ce9f18cb759b83c19426d1d79bfee6b70a11ac4`, including cache-cold and
  cache-warm complete release gates.
