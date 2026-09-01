# M28-04 — Document and release-gate M28

- Status: complete

## Goal

Record the version boundary and reusable language gap, then prove M28 does not
regress any earlier port.

## Definition of done

- The milestone, task set, compatibility ledger, port report, checker, and
  language map reflect the implementation.
- M28 participates in Rustfmt, Clippy, Buildifier, full-repository, and release
  gates.
- Every earlier native/differential port remains green.
- The M28 commit is pushed and hosted GitHub CI passes.

## Tests

- `bazel test //... --nocache_test_results --test_output=errors`
- `bazel test //:release_gate --nocache_test_results --test_output=errors`
