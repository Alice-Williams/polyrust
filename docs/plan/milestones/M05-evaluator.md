# M05 — Implement reference evaluator, portable tests, and canonical values

- Status: complete
- Phase: 1
- Depends on: M02, M04

## Outcome

Create a simple executable oracle for checked Core semantics, including contract
dispatch and declared portable tests, plus a target-neutral canonical value/error
encoding used by differential tests.

## Implementation checklist

- Runtime `Value` types corresponding exactly to checked IR types.
- Evaluator for functions, expressions, matching, and bounded list iteration.
- Evaluator for concrete methods, restricted contract dispatch, and portable test
  declarations.
- Exact checked/wrapping `i32`/`i64` behavior and specified `f64` behavior.
- Unicode-scalar string operations and immutable list behavior.
- Fuel, call-depth, and collection-size limits.
- Canonical JSON-shaped encoding for values, NaN/infinities/negative zero, bytes,
  enums, options, results, and structured evaluation errors.

## Required exit evidence

- Interpreter accepts checked programs only.
- Evaluation order and short-circuit behavior match the spec.
- Resource exhaustion returns a deterministic structured error.
- Canonical encoding round-trips every representable runtime value.
- No evaluation path relies on Rust debug/release overflow defaults.

### Verification

- At least 20 initial semantic vectors and ten declared portable tests, including
  concrete/contract method dispatch, numeric boundaries, overflow,
  division errors, Unicode astral/combining text, every sum type, short circuit,
  nested values, and list non-aliasing.
- Release and debug test runs return identical canonical results.
- Property tests for wrapping operations against mathematical modulo definitions.
- Canonical encoding round-trip property tests, including special floats.
- Fuel/size/depth limit tests.

```text
cargo test -p polyrust-eval
cargo test -p polyrust-eval --release
```

### Completion gate

The initial corpus and declared tests pass identically in debug and release, all
limits are tested, canonical encoding is documented, and later target runners
can consume the same fixtures without importing evaluator internals.

### Completion evidence

Completed in the pinned Linux development image on 2026-08-31:

- The debug and release evaluator commands each passed the same 12 tests and
  canonical assertions, proving profile-independent checked and wrapping
  integer behavior.
- The semantic suite executes 26 operation and fault vectors, including integer
  boundaries, overflow and zero-division errors, special floats, astral and
  combining Unicode scalars, options, results, immutable lists, byte strings,
  conversions, and invalid UTF-8.
- conformance/v0/evaluator-vectors.json provides 20 unique target-neutral cases.
  Its arguments and expected outcomes are decoded by the test suite using only
  the public canonical protocol.
- Eleven checked portable test declarations pass through the public evaluator.
  They cover direct concrete method invocation, concrete dispatch, contract
  dispatch, short circuiting, option matching, bounded list iteration, astral
  text, and combining text.
- Explicit tests prove left-to-right error precedence, enum payload binding,
  list non-aliasing, mathematical modulo wrapping, canonical round trips for
  every runtime value and error, and deterministic public fuel, call-depth, and
  collection-size failures.
- The evaluator facade stores only a CheckedProgram reference. Checked IR is the
  only public program input, and concrete dispatch resolves exclusively through
  checked declaration IDs.
- docs/evaluator-v0.md specifies evaluation order, floating-point and Unicode
  behavior, portable test error expectations, resource accounting, and the
  lossless canonical JSON schema.
- The workspace formatting and Clippy gates passed. The authoritative
  bazel test //... gate passed all 13 repository tests across 33 analyzed
  targets, including Rustfmt, Clippy, Buildifier, dependency boundaries, and
  native generated Rust and Go tests.

## Scope boundary

Optimization, effects, recursion, filesystem access, and target code execution.
