# M14 — Build four-target differential conformance harness

- Status: complete
- Phase: 4
- Depends on: M05, M10, M11, M12, M13

## Outcome

Turn the reference semantics and user-authored portable tests into executable
evidence by generating and running them in Rust, TypeScript, Python, and Go and
comparing canonical outcomes.

## Implementation checklist

- Versioned conformance case format and corpus.
- Loader that derives cases from portable test declarations and verifies that
  every one is present in each target's native test manifest.
- Per-target generated runner adapters using canonical value encoding.
- Harness that runs the evaluator, builds/runs each target, normalizes
  process failures, and prints a useful mismatch report.
- At least 50 named vectors covering every v0 feature and difficult boundary.
- Determinism run that regenerates twice and compares full manifest bytes.
- Fault-injection fixtures proving the suite catches semantic divergence.

## Required exit evidence

- All five executions (evaluator plus four targets) agree byte-for-byte on
  canonical value or structured error for every vector.
- Every portable test passes in the evaluator and all four native test frameworks.
- Toolchain absence is a clear local skip only when explicitly allowed; it is a
  hard CI/release failure for every required target.
- Cases are target-neutral and contain no expected target source text.
- Mismatch output identifies case, function, input, oracle, target, and minimized
  structural difference.
- Corpus includes overflow, special floats, Unicode, evaluation order, every sum
  type, restricted contract dispatch, list aliasing, keywords, and empty/nested
  values.

### Verification

```text
cargo test -p polyrust-conformance
cargo run -p polyrust-conformance -- --all-targets
cargo run -p polyrust-conformance -- --determinism
```

Add tests that alter one arithmetic helper, one Unicode helper, and one enum tag
encoder in staged fixtures; each alteration must cause the expected case to fail.

### Completion gate

At least 50 vectors and all declared portable tests pass on the pinned
four-toolchain CI image, three independent faults are detected, repeated
generation is byte-identical, mismatch diagnostics are snapshot-tested, and every
Core capability maps to at least one case ID.

## Scope boundary

Performance equivalence, cross-language ABI calls, fuzzing arbitrary native
programs, and non-required targets.

## Exit evidence

- The versioned target-neutral corpus contains 50 stable case IDs spanning
  fixed-width boundaries, exact floats, Unicode, bytes, immutable/nested lists,
  option/result tags, empty values, and contract dispatch. Canonical values
  round-trip byte-stably through `polyrust.canonical.v0`.
- The evaluator passes every declared portable test. The aggregate Bazel target
  executes the generated Rust, TypeScript, Python, and Go native suites; both
  CLI modes report four-target deterministic manifests and complete coverage.
- Arithmetic, Unicode, and enum-tag fault injections are detected, and the
  mismatch snapshot contains case, function, input, oracle, target, and first
  difference.
- Cargo tests, Rustfmt, and warning-denied Clippy passed. The authoritative
  `bazel test //...` gate passed all 26 tests across 57 targets, including all
  four pinned native toolchains and repository linters.
