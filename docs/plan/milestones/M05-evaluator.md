# M05 — Implement reference evaluator, portable tests, and canonical values

- Status: planned
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

## Scope boundary

Optimization, effects, recursion, filesystem access, and target code execution.
