# M31-05 — Document and release-gate M31

- Status: complete

## Goal

Record the host/string admission boundaries and reusable language gaps, then
prove M31 does not regress any completed port.

## Definition of done

- The milestone, task set, compatibility ledger, port report, checker, and
  portable language map reflect the implemented semantics.
- M31 participates in Rustfmt, Clippy, Buildifier, full-repository, and release
  gates.
- Every earlier native/differential port remains green.
- M31 is committed and pushed, and hosted GitHub CI passes before another
  repository is selected.

## Tests

- `bazel test //... --nocache_test_results --test_output=errors`
- `bazel test //:release_gate --nocache_test_results --test_output=errors`

## Local evidence

- The uncached full repository gate passes 216/216 tests.
- The uncached release gate passes 193/193 tests.
- Buildifier, Rustfmt, Clippy, documentation, dependency-boundary,
  source-policy, and source-policy fault-injection targets pass.
- Every earlier real-world port's evaluator, deterministic generation,
  differential oracle, eight generated packages, Java/C/C++ consumers, and
  C/C++ sanitizer targets remain green.
- Implementation commit `45a5a701b1bd2b77f459ac1a5a0764815912f474`
  is pushed and hosted workflow
  [33584299238](https://github.com/Alice-Williams/polyrust/actions/runs/33584299238)
  completes successfully, including both determinism hosts and cold/warm
  complete release gates.
