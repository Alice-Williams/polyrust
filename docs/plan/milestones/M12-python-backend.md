# M12 — Implement required Python backend

- Status: complete
- Phase: 3
- Depends on: M08, M09

## Outcome

Generate a typed, readable Python package for the full v0 IR while enforcing
fixed-width and immutable-value semantics absent from native Python defaults.

## Implementation checklist

- Python name/keyword handling, escaping, imports, type annotations, and lowering.
- Constant lowering, `Protocol` generation, explicit implementation checking,
  restricted contract dispatch, and native `pytest` generation.
- Reviewed representations for records, variants, `Option`, and `Result` without
  conflating `None` with a present null-like value.
- Checked/wrapping integer helpers, special-float canonicalization, Unicode-scalar
  validation, and immutable list representation/operations.
- Package metadata, supported Python version policy, type-checker/formatter
  configuration, generated tests/runner, and API design document.

## Required exit evidence

- Generated public APIs are fully annotated under the selected strict type-check
  profile.
- Integer helpers enforce exact `i32`/`i64` behavior.
- Variant dispatch is exhaustive by generated construction/checking rules.
- Contract declarations and implementations pass the selected static type checker.
- Every portable test is emitted as a discoverable native test.
- Generated functions do not mutate values reachable through another binding.
- Output imports and files are deterministic.

### Verification

- Unit/golden tests for indentation, identifiers, keywords, escaping, Unicode
  surrogates, integer boundaries, variants, protocols/implementations, portable
  tests, nested types, and every lowering case.
- Native generated-package checks using pinned tools:

```text
python -m compileall -q src tests
ruff format --check .
ruff check .
mypy --strict src tests
pytest
```

- Run on every supported Python version in CI.
- Runtime tests for width enforcement, special floats, scalar length, option
  distinction, and list non-aliasing.

### Completion gate

All v0 fixtures are deterministic, compile/format/lint/type/test checks pass on
the version matrix, the backend passes the contract suite, API snapshots are
reviewed, and at least 20 evaluator vectors agree with generated Python.

## Scope boundary

Supporting the currently installed Python 3.7 unless selected by the explicit
compatibility policy, runtime metaprogramming, async, and third-party data-model
frameworks.

## Exit evidence

- `org.polyrust.python` consumes only checked IR and emits a deterministic,
  dependency-free Python 3.13 package with fully annotated public functions,
  frozen records/variants, protocols, tagged option/result values, immutable
  bytes/tuples, and fixed-width integer helpers.
- The native generated-package gate passed compileall, Ruff 0.16.5 format/lint,
  mypy 2.3.1 strict checking, pytest 9.1.1, the expected-failure invalid-option
  type fixture, every portable test, and 20 semantic boundary vectors.
- Workspace Rustfmt and Clippy with warnings denied passed. The authoritative
  `bazel test //...` gate passed all 23 repository tests across 51 targets,
  including Buildifier/Starlark lint, dependency boundaries, and all earlier
  generated-language gates.
