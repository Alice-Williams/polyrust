# M27 — parse-ms equivalence port

- Status: complete
- Phase: 6
- Depends on: M23, M24, M25, M26

## Outcome

Port the complete representable typed behavior of MIT-licensed `parse-ms`
3.0.0 into one checked PolyRust model and generate equivalent Rust,
TypeScript, JavaScript, Python, Go, Java, C++, and C packages.

Version 3 is selected deliberately: its public API is `number ->
TimeComponents`, which maps exactly to PolyRust `F64` and a named record.
Version 4 adds JavaScript `bigint`; arbitrary-precision integer portability is a
future language feature and MUST remain an explicit version boundary.

## Implementation checklist

- Retain the immutable upstream source, declaration, tests, type test, package
  metadata, README, and MIT license at commit
  `49dab09236deeea5d2c082182e2c73e7a79763a8`.
- Verify every retained byte with Git blob hashes.
- Add a portable `float_trunc` intrinsic with IEEE binary64 semantics:
  truncation toward zero, signed-zero preservation, NaN propagation, and
  infinity preservation.
- Keep language `Equal` as IEEE equality while making portable-test comparison
  recursively exact for `F64`: NaNs compare as expected values and signed zero
  is distinguished inside records and collections.
- Model `TimeComponents` with seven `F64` fields and implement the upstream
  divisions, multiplications, truncations, and truncating remainders exactly.
- Retain all official vectors and add signed-zero, NaN, infinity, fractional,
  unit-boundary, negative, and large-magnitude coverage.
- Differentially compare the retained JavaScript implementation with generated
  TypeScript over a deterministic broad corpus using recursive `Object.is`/NaN
  semantics.
- Generate and run all eight native packages, target linters/formatters,
  conformance tests, public consumers, and C/C++ sanitizers.

## Required exit evidence

- Provenance test passes for every retained upstream blob.
- Checker and evaluator tests cover `float_trunc`, including invalid signatures
  and every IEEE edge class.
- Every backend has focused lowering coverage for `float_trunc` and nested-F64
  portable expectations.
- Reference-evaluator portable tests all pass.
- The differential oracle reports every corpus comparison passing.
- Three complete generations are byte-identical.
- `bazelisk test //... --test_output=errors` and
  `bazelisk test //:release_gate --test_output=errors` pass in the Linux
  development container, including Buildifier, Rustfmt, and Clippy.

## Scope boundary

M27 implements the complete typed `number -> TimeComponents` API of parse-ms
3.0.0. It does not approximate the untyped JavaScript call surface or parse-ms
4.x `bigint`; arbitrary-precision integers remain a future explicit IR feature.

## Completion evidence

- All seven retained upstream files pass immutable Git-blob provenance checks.
- The evaluator and all eight generated targets pass 30 portable vectors.
- The retained JavaScript oracle and generated TypeScript agree for 70,735
  exact field comparisons across 10,105 deterministic inputs.
- Three complete eight-target generations are byte-identical.
- The parse-ms `:all` gate passes 15/15 tests, including native toolchains,
  formatters, static analysis, public consumers, and C/C++ sanitizers.
- `bazel test //... --nocache_test_results --test_output=errors` passes all
  168 tests in the Linux development container.
- `bazel test //:release_gate --nocache_test_results --test_output=errors`
  passes all 146 release tests, including Buildifier, Rustfmt, and Clippy.
