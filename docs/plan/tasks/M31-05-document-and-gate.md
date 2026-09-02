# M31-05 — Document and release-gate M31

- Status: planned

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
